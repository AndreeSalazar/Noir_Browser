use crate::browser::LinkHitbox;
use crate::media::discovery::{discover_media, MediaReport};
use crate::parsers::css_engine::ComputedStyle;
use crate::parsers::css_simple::{parse_color, parse_px, CssCascade, CssElementContext};
use crate::parsers::dom_tree::DomNode;
use crate::parsers::html_elements::HtmlTag;
use crate::parsers::resource_loader::{
    fetch_document, CacheStatus, ResourceResponse, ResourceType,
};
use crate::parsers::style_collector::{
    collect_inline_styles, collect_stylesheets, StylesheetBundle,
};
use crate::render::text::{RasterizedAtlas, TextRasterizationOptions, TextRequest};
use crate::runtime::{collect_scripts, BrowserRuntime, ScriptSource};
use serde_json::Value;
use std::collections::HashMap;
use url::Url;

const MAX_TEXT_FRAGMENTS: usize = 2048;
const MAX_VISIBLE_LINES: usize = 180;
const CONTENT_X: f32 = 40.0;
const CONTENT_TOP: f32 = 78.0;
const CONTENT_SIDE_PADDING: f32 = 80.0;
const VIEWPORT_BOTTOM_PADDING: f32 = 48.0;
const URL_TEXT_X: f32 = 202.0;

#[derive(Clone, Debug)]
struct TextFragment {
    text: String,
    px_size: f32,
    is_bold: bool,
    line_height: f32,
    margin_after: f32,
    line_break_after: bool,
    layout: FragmentLayout,
    color: [f32; 4],
    href: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct FragmentLayout {
    width: Option<String>,
    max_width: Option<String>,
    margin_left: Option<String>,
    margin_right: Option<String>,
    padding_left: Option<String>,
    padding_right: Option<String>,
    text_align: Option<String>,
}

#[derive(Clone, Debug)]
struct TextStyleState {
    px_size: f32,
    is_bold: bool,
    line_height: f32,
    margin_after: f32,
    color: [f32; 4],
    hidden: bool,
    display: Option<String>,
    text_transform: Option<String>,
    layout: FragmentLayout,
    in_navigation: bool,
}

impl Default for TextStyleState {
    fn default() -> Self {
        Self::default_with_color([1.0, 1.0, 1.0, 1.0])
    }
}

impl TextStyleState {
    fn default_with_color(color: [f32; 4]) -> Self {
        Self {
            px_size: 16.0,
            is_bold: false,
            line_height: 22.0,
            margin_after: 6.0,
            color,
            hidden: false,
            display: None,
            text_transform: None,
            layout: FragmentLayout::default(),
            in_navigation: false,
        }
    }
}

pub struct PageDocument {
    fragments: Vec<TextFragment>,
    media: MediaReport,
    page_style: PageStyle,
}

impl PageDocument {
    pub fn media_summary(&self) -> Option<String> {
        self.media.summary()
    }

    pub fn computed_style(&self) -> ComputedStyle {
        let mut style = ComputedStyle::default();
        style.background_color = Some(self.page_style.background_hex.clone());
        style.width = Some("100%".to_string());
        style.height = Some("100%".to_string());
        style
    }
}

#[derive(Clone, Debug)]
struct PageStyle {
    background_hex: String,
    default_text_color: [f32; 4],
}

pub struct PageRender {
    pub atlas: RasterizedAtlas,
    pub content_height: f32,
}

pub fn load_page_document(target_url: &str) -> PageDocument {
    let response = fetch_document(target_url).unwrap_or_else(|error| {
        let body = format!("<h1>Network Error</h1><p>{error}</p>");
        ResourceResponse {
            requested_url: target_url.to_string(),
            final_url: target_url.to_string(),
            status: 0,
            resource_type: ResourceType::Document,
            content_type: Some("text/html; charset=utf-8".to_string()),
            body_bytes: body.len(),
            body,
            cache_status: CacheStatus::Network,
        }
    });
    let dom = crate::parsers::dom_tree::parse_html(&response.body);
    let base_url = Url::parse(&response.final_url)
        .or_else(|_| Url::parse(target_url))
        .ok();

    let mut fragments = Vec::new();
    let stylesheet_bundle = load_stylesheet_bundle(&dom, base_url.as_ref());
    let css = CssCascade::from_blocks(&stylesheet_bundle.blocks);
    let page_style = derive_page_style(&css);
    let mut ancestors = Vec::new();
    extract_text_from_dom(
        &dom,
        &mut fragments,
        &css,
        TextStyleState::default_with_color(page_style.default_text_color),
        None,
        base_url.as_ref(),
        &mut ancestors,
    );
    append_stylesheet_summary(&mut fragments, &stylesheet_bundle);
    apply_runtime_scripts(&dom, &mut fragments, base_url.as_ref());
    append_app_shell_fallback(
        &dom,
        &response.body,
        &mut fragments,
        page_style.default_text_color,
    );
    append_response_summary(&mut fragments, &response);
    let media = discover_media(&dom, &response.final_url);
    append_media_summary(&mut fragments, &media);
    normalize_fragments(&mut fragments);

    PageDocument {
        fragments,
        media,
        page_style,
    }
}

fn append_response_summary(fragments: &mut Vec<TextFragment>, response: &ResourceResponse) {
    if response.is_success()
        && response.requested_url == response.final_url
        && response.is_html_like()
        && response.cache_status == CacheStatus::Network
    {
        return;
    }

    let content_type = response
        .content_type
        .as_deref()
        .unwrap_or("content-type desconocido");
    let mut summary = format!(
        "HTTP {} / {:?} / {} bytes / {}",
        response.status, response.resource_type, response.body_bytes, content_type
    );
    if response.requested_url != response.final_url {
        summary.push_str(&format!(" / redirigido a {}", response.final_url));
    }
    if !response.is_html_like() {
        summary.push_str(" / no parece HTML");
    }
    match response.cache_status {
        CacheStatus::Network => {}
        CacheStatus::Revalidated => summary.push_str(" / cache revalidado"),
        CacheStatus::Fallback => summary.push_str(" / cache offline"),
    }

    fragments.insert(
        0,
        TextFragment {
            text: summary,
            px_size: 13.0,
            is_bold: true,
            line_height: 19.0,
            margin_after: 6.0,
            line_break_after: true,
            layout: FragmentLayout::default(),
            color: [0.725, 0.790, 0.980, 1.0],
            href: None,
        },
    );
}

fn load_stylesheet_bundle(dom: &[DomNode], base_url: Option<&Url>) -> StylesheetBundle {
    let mut bundle = StylesheetBundle::default();
    let stylesheets = collect_stylesheets(dom, base_url);
    bundle.external_count = stylesheets.len();
    bundle.blocks.extend(collect_inline_styles(dom));
    bundle.inline_count = bundle.blocks.len();

    let mut workers = Vec::new();
    for stylesheet in stylesheets.iter().take(32) {
        let url = stylesheet.url.clone();
        workers.push(std::thread::spawn(move || {
            crate::parsers::resource_loader::fetch_style(&url).ok()
        }));
    }

    for worker in workers {
        if let Ok(Some(response)) = worker.join() {
            bundle.loaded_external += 1;
            bundle.blocks.push(response.body);
        }
    }

    bundle
}

fn append_stylesheet_summary(fragments: &mut Vec<TextFragment>, bundle: &StylesheetBundle) {
    if bundle.external_count == 0 && bundle.inline_count == 0 {
        return;
    }

    fragments.insert(
        0,
        TextFragment {
            text: format!(
                "CSS detectado: {} inline / {} externas / {} precargadas",
                bundle.inline_count, bundle.external_count, bundle.loaded_external
            ),
            px_size: 13.0,
            is_bold: true,
            line_height: 19.0,
            margin_after: 6.0,
            line_break_after: true,
            layout: FragmentLayout::default(),
            color: [0.725, 0.790, 0.980, 1.0],
            href: None,
        },
    );
}

fn derive_page_style(css: &CssCascade) -> PageStyle {
    let empty = HashMap::new();
    let html = css.declarations_for(&HtmlTag::Custom("html".to_string()), &empty);
    let body = css.declarations_for(&HtmlTag::Custom("body".to_string()), &empty);

    let background = body
        .background_color
        .as_deref()
        .or(body.background.as_deref())
        .or(html.background_color.as_deref())
        .or(html.background.as_deref())
        .and_then(first_css_color)
        .unwrap_or([0.102, 0.102, 0.180, 1.0]);

    let mut text = body
        .color
        .as_deref()
        .or(html.color.as_deref())
        .and_then(parse_color)
        .unwrap_or_else(|| readable_text_color(background));
    text = ensure_contrast(text, background);

    PageStyle {
        background_hex: rgba_to_hex(background),
        default_text_color: text,
    }
}

fn first_css_color(value: &str) -> Option<[f32; 4]> {
    parse_color(value).or_else(|| {
        value
            .split_whitespace()
            .find_map(|part| parse_color(part.trim_matches(',')))
    })
}

fn readable_text_color(background: [f32; 4]) -> [f32; 4] {
    let luminance = 0.2126 * background[0] + 0.7152 * background[1] + 0.0722 * background[2];
    if luminance > 0.55 {
        [0.08, 0.08, 0.10, 1.0]
    } else {
        [1.0, 1.0, 1.0, 1.0]
    }
}

fn ensure_contrast(text: [f32; 4], background: [f32; 4]) -> [f32; 4] {
    let text_lum = 0.2126 * text[0] + 0.7152 * text[1] + 0.0722 * text[2];
    let bg_lum = 0.2126 * background[0] + 0.7152 * background[1] + 0.0722 * background[2];
    if (text_lum - bg_lum).abs() >= 0.32 {
        return text;
    }

    readable_text_color(background)
}

fn rgba_to_hex(color: [f32; 4]) -> String {
    let r = (color[0].clamp(0.0, 1.0) * 255.0).round() as u8;
    let g = (color[1].clamp(0.0, 1.0) * 255.0).round() as u8;
    let b = (color[2].clamp(0.0, 1.0) * 255.0).round() as u8;
    format!("#{r:02x}{g:02x}{b:02x}")
}

pub fn render_page(
    target_url: &str,
    document: &PageDocument,
    link_hitboxes: &mut Vec<LinkHitbox>,
    text_options: TextRasterizationOptions,
    viewport_width: f32,
    viewport_height: f32,
    scroll_offset: f32,
) -> PageRender {
    let mut text_requests = Vec::new();
    link_hitboxes.clear();

    text_requests.push(TextRequest {
        text: compact_url_text(target_url, viewport_width),
        px_size: 16.0,
        is_bold: false,
        pos_x: URL_TEXT_X,
        pos_y: 48.0,
        color: [1.0, 1.0, 1.0, 1.0],
    });

    let default_content_width = (viewport_width - CONTENT_SIDE_PADDING).clamp(320.0, 1120.0);
    let default_content_x = CONTENT_X;
    let visible_bottom = (viewport_height - VIEWPORT_BOTTOM_PADDING).max(CONTENT_TOP);
    let mut document_y = CONTENT_TOP;
    let mut cursor_x = default_content_x;
    let mut line_height = 0.0_f32;
    let mut line_started = false;
    let mut line_index = 0;
    let mut active_box = ResolvedLayoutBox {
        x: default_content_x,
        width: default_content_width,
    };

    for fragment in &document.fragments {
        let layout_box = resolve_fragment_layout(
            &fragment.layout,
            viewport_width,
            default_content_x,
            default_content_width,
        );
        if line_started && layout_box != active_box {
            document_y += line_height.max(fragment.line_height);
            cursor_x = layout_box.x;
            line_height = 0.0;
            line_started = false;
            line_index += 1;
        }
        active_box = layout_box;

        let color = if fragment.href.is_some() && fragment.color == TextStyleState::default().color
        {
            [0.478, 0.635, 0.968, 1.0]
        } else {
            fragment.color
        };

        let space_width = estimated_text_width(" ", fragment.px_size);
        for word in fragment.text.split_whitespace() {
            let word_width = estimated_text_width(word, fragment.px_size);
            let mut leading_space = if line_started { space_width } else { 0.0 };
            if line_started
                && cursor_x + leading_space + word_width > layout_box.x + layout_box.width
            {
                document_y += line_height.max(fragment.line_height);
                cursor_x = layout_box.x;
                line_height = 0.0;
                leading_space = 0.0;
                line_index += 1;
            }

            let x = cursor_x + leading_space;
            let active_line_height = line_height.max(fragment.line_height);
            let screen_y = document_y - scroll_offset;
            let line_bottom = screen_y + active_line_height;

            if line_bottom >= CONTENT_TOP
                && screen_y <= visible_bottom
                && line_index < MAX_VISIBLE_LINES
            {
                if let Some(href) = &fragment.href {
                    link_hitboxes.push(LinkHitbox {
                        url: href.clone(),
                        y_min: screen_y,
                        y_max: line_bottom,
                    });
                }

                text_requests.push(TextRequest {
                    text: word.to_string(),
                    px_size: fragment.px_size,
                    is_bold: fragment.is_bold,
                    pos_x: x,
                    pos_y: screen_y,
                    color,
                });
            }

            cursor_x = x + word_width;
            line_height = active_line_height;
            line_started = true;
        }

        if fragment.line_break_after && line_started {
            document_y += line_height.max(fragment.line_height);
            cursor_x = active_box.x;
            line_height = 0.0;
            line_started = false;
            line_index += 1;
        }

        if fragment.line_break_after {
            document_y += fragment.margin_after;
        }
    }

    if line_started {
        document_y += line_height;
    }

    let content_height = document_y + VIEWPORT_BOTTOM_PADDING;
    PageRender {
        atlas: RasterizedAtlas::with_options(&text_requests, text_options),
        content_height,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct ResolvedLayoutBox {
    x: f32,
    width: f32,
}

fn resolve_fragment_layout(
    layout: &FragmentLayout,
    viewport_width: f32,
    default_x: f32,
    default_width: f32,
) -> ResolvedLayoutBox {
    let available = (viewport_width - CONTENT_SIDE_PADDING).max(320.0);
    let mut width = layout
        .width
        .as_deref()
        .and_then(|value| parse_layout_length(value, available))
        .unwrap_or(default_width);
    if let Some(max_width) = layout
        .max_width
        .as_deref()
        .and_then(|value| parse_layout_length(value, available))
    {
        width = width.min(max_width);
    }
    width = width.clamp(260.0, available);

    let padding_left = layout
        .padding_left
        .as_deref()
        .and_then(|value| parse_layout_length(value, width))
        .unwrap_or(0.0)
        .clamp(0.0, 96.0);
    let padding_right = layout
        .padding_right
        .as_deref()
        .and_then(|value| parse_layout_length(value, width))
        .unwrap_or(0.0)
        .clamp(0.0, 96.0);

    let auto_left = layout
        .margin_left
        .as_deref()
        .is_some_and(|value| value.eq_ignore_ascii_case("auto"));
    let auto_right = layout
        .margin_right
        .as_deref()
        .is_some_and(|value| value.eq_ignore_ascii_case("auto"));
    let centered = auto_left && auto_right;
    let x = if centered {
        ((viewport_width - width) * 0.5).max(CONTENT_X)
    } else {
        default_x
            + layout
                .margin_left
                .as_deref()
                .and_then(|value| parse_layout_length(value, available))
                .unwrap_or(0.0)
    };

    ResolvedLayoutBox {
        x: x + padding_left,
        width: (width - padding_left - padding_right).max(240.0),
    }
}

fn parse_layout_length(value: &str, basis: f32) -> Option<f32> {
    let value = value.trim().to_ascii_lowercase();
    if value == "auto" {
        return None;
    }
    if value.starts_with("calc(") && value.ends_with(')') {
        return parse_calc_length(&value[5..value.len() - 1], basis);
    }
    if let Some(px) = value.strip_suffix("px") {
        return px.trim().parse::<f32>().ok();
    }
    if let Some(percent) = value.strip_suffix('%') {
        return percent
            .trim()
            .parse::<f32>()
            .ok()
            .map(|value| basis * value / 100.0);
    }
    if let Some(rem) = value
        .strip_suffix("rem")
        .or_else(|| value.strip_suffix("em"))
    {
        return rem.trim().parse::<f32>().ok().map(|value| value * 16.0);
    }
    value.parse::<f32>().ok()
}

fn parse_calc_length(value: &str, basis: f32) -> Option<f32> {
    let normalized = value.replace('+', " + ").replace('-', " - ");
    let mut total = 0.0;
    let mut sign = 1.0;
    for token in normalized.split_whitespace() {
        match token {
            "+" => sign = 1.0,
            "-" => sign = -1.0,
            value => {
                total += sign * parse_layout_length(value, basis)?;
                sign = 1.0;
            }
        }
    }
    Some(total)
}

fn compact_url_text(url: &str, viewport_width: f32) -> String {
    let max_chars = ((viewport_width - 250.0).max(160.0) / 8.5) as usize;
    if url.chars().count() <= max_chars {
        return url.to_string();
    }

    let keep_start = (max_chars / 2).saturating_sub(2).max(12);
    let keep_end = max_chars.saturating_sub(keep_start + 3).max(8);
    let start: String = url.chars().take(keep_start).collect();
    let end: String = url
        .chars()
        .rev()
        .take(keep_end)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    format!("{start}...{end}")
}

fn extract_text_from_dom(
    nodes: &[DomNode],
    out: &mut Vec<TextFragment>,
    css: &CssCascade,
    current_style: TextStyleState,
    current_href: Option<String>,
    base_url: Option<&Url>,
    ancestors: &mut Vec<CssElementContext>,
) {
    for node in nodes {
        if out.len() >= MAX_TEXT_FRAGMENTS {
            break;
        }

        match node {
            DomNode::Element {
                tag,
                attributes,
                children,
            } => {
                if should_skip_element(tag, attributes) {
                    continue;
                }

                let mut next_style = apply_css_declarations(
                    apply_tag_defaults(tag, &current_style),
                    &css.declarations_for_with_ancestors(tag, attributes, ancestors),
                );
                let mut new_href = current_href.clone();
                if is_navigation_context(tag, attributes) {
                    next_style.in_navigation = true;
                }
                if matches!(tag, HtmlTag::Main) && ancestor_has_class(ancestors, "sidenav") {
                    next_style.layout.max_width = Some("820px".to_string());
                }

                match tag {
                    HtmlTag::A => {
                        next_style.margin_after = 0.0;
                        if next_style.color == current_style.color {
                            next_style.color = [0.0, 0.33, 0.70, 1.0];
                        }
                        if let Some(href) = attributes.get("href") {
                            new_href = resolve_url(base_url, href);
                        }
                    }
                    HtmlTag::Li if current_style.in_navigation => {
                        next_style.display = Some("inline".to_string());
                        next_style.margin_after = 0.0;
                    }
                    _ => {}
                }
                if !element_breaks_line(tag, &next_style) {
                    clear_inline_box_layout(&mut next_style.layout);
                }

                if next_style.hidden {
                    continue;
                }

                let fragments_before = out.len();
                let should_break = element_breaks_line(tag, &next_style);
                let element_margin_after = next_style.margin_after.max(block_margin_after(tag));
                ancestors.push(CssElementContext::from_element(tag, attributes));
                extract_text_from_dom(
                    children, out, css, next_style, new_href, base_url, ancestors,
                );
                ancestors.pop();
                if should_break && out.len() > fragments_before {
                    if let Some(last) = out.last_mut() {
                        last.line_break_after = true;
                        last.margin_after = last.margin_after.max(element_margin_after);
                    }
                }
            }
            DomNode::Text(t) => {
                let text = normalize_text(t.trim());
                if text.len() > 2 {
                    let text = apply_text_transform(text, current_style.text_transform.as_deref());
                    out.push(TextFragment {
                        text,
                        px_size: current_style.px_size,
                        is_bold: current_style.is_bold,
                        line_height: current_style.line_height,
                        margin_after: 0.0,
                        line_break_after: false,
                        layout: current_style.layout.clone(),
                        color: current_style.color,
                        href: current_href.clone(),
                    });
                }
            }
        }
    }
}

fn resolve_url(base_url: Option<&Url>, href: &str) -> Option<String> {
    if href.starts_with('#') || href.starts_with("javascript:") || href.starts_with("mailto:") {
        return None;
    }

    match base_url {
        Some(base) => base.join(href).ok().map(|url| url.to_string()),
        None => Some(href.to_string()),
    }
}

fn apply_tag_defaults(tag: &HtmlTag, current: &TextStyleState) -> TextStyleState {
    let mut next = current.clone();
    match tag {
        HtmlTag::H1 => {
            next.px_size = 32.0;
            next.is_bold = true;
            next.line_height = 40.0;
            next.margin_after = 6.0;
        }
        HtmlTag::H2 => {
            next.px_size = 24.0;
            next.is_bold = true;
            next.line_height = 32.0;
            next.margin_after = 5.0;
        }
        HtmlTag::H3 => {
            next.px_size = 20.0;
            next.is_bold = true;
            next.line_height = 28.0;
            next.margin_after = 4.0;
        }
        HtmlTag::P | HtmlTag::Li | HtmlTag::Dd | HtmlTag::Dt => {
            next.px_size = 16.0;
            next.is_bold = false;
            next.line_height = 22.0;
            next.margin_after = 6.0;
        }
        HtmlTag::Pre | HtmlTag::Code | HtmlTag::Samp | HtmlTag::Kbd => {
            next.px_size = 14.0;
            next.line_height = 20.0;
        }
        HtmlTag::Strong | HtmlTag::B => {
            next.is_bold = true;
        }
        HtmlTag::Small => {
            next.px_size = 13.0;
            next.line_height = 18.0;
        }
        _ => {}
    }
    next
}

fn apply_css_declarations(
    mut style: TextStyleState,
    declarations: &crate::parsers::css_simple::CssDeclarations,
) -> TextStyleState {
    if declarations
        .display
        .as_deref()
        .is_some_and(|value| value.eq_ignore_ascii_case("none"))
        || declarations
            .visibility
            .as_deref()
            .is_some_and(|value| value.eq_ignore_ascii_case("hidden"))
        || declarations.opacity.as_deref() == Some("0")
    {
        style.hidden = true;
    }

    if let Some(color) = declarations.color.as_deref().and_then(parse_color) {
        style.color = color;
    }
    if let Some(px_size) = declarations
        .font_size
        .as_deref()
        .and_then(|value| parse_px(value, style.px_size))
    {
        style.px_size = px_size.clamp(8.0, 72.0);
    }
    if let Some(line_height) = declarations
        .line_height
        .as_deref()
        .and_then(|value| parse_px(value, style.px_size))
    {
        style.line_height = line_height.clamp(style.px_size, 96.0);
    } else {
        style.line_height = style.line_height.max(style.px_size * 1.2);
    }
    if let Some(margin_after) = declarations
        .margin_bottom
        .as_deref()
        .and_then(|value| parse_px(value, style.px_size))
    {
        style.margin_after = margin_after.clamp(0.0, 48.0);
    }
    if let Some(weight) = &declarations.font_weight {
        let weight = weight.trim().to_ascii_lowercase();
        style.is_bold = weight == "bold"
            || weight == "bolder"
            || weight.parse::<u16>().is_ok_and(|value| value >= 600);
    }
    if let Some(transform) = &declarations.text_transform {
        style.text_transform = Some(transform.to_ascii_lowercase());
    }
    if let Some(display) = &declarations.display {
        style.display = Some(display.to_ascii_lowercase());
    }
    assign_layout_property(&mut style.layout.width, &declarations.width);
    assign_layout_property(&mut style.layout.max_width, &declarations.max_width);
    assign_layout_property(&mut style.layout.margin_left, &declarations.margin_left);
    assign_layout_property(&mut style.layout.margin_right, &declarations.margin_right);
    assign_layout_property(&mut style.layout.padding_left, &declarations.padding_left);
    assign_layout_property(&mut style.layout.padding_right, &declarations.padding_right);
    assign_layout_property(&mut style.layout.text_align, &declarations.text_align);

    style
}

fn assign_layout_property(target: &mut Option<String>, source: &Option<String>) {
    if let Some(value) = source {
        *target = Some(value.clone());
    }
}

fn clear_inline_box_layout(layout: &mut FragmentLayout) {
    layout.width = None;
    layout.max_width = None;
    layout.margin_left = None;
    layout.margin_right = None;
    layout.padding_left = None;
    layout.padding_right = None;
}

fn is_navigation_context(tag: &HtmlTag, attributes: &HashMap<String, String>) -> bool {
    if matches!(tag, HtmlTag::Nav | HtmlTag::Menu) {
        return true;
    }

    attributes
        .get("id")
        .or_else(|| attributes.get("class"))
        .is_some_and(|value| {
            let value = value.to_ascii_lowercase();
            value.contains("nav") || value.contains("menu")
        })
}

fn ancestor_has_class(ancestors: &[CssElementContext], class_name: &str) -> bool {
    ancestors
        .iter()
        .any(|ancestor| ancestor.classes.iter().any(|class| class == class_name))
}

fn element_breaks_line(tag: &HtmlTag, style: &TextStyleState) -> bool {
    if let Some(display) = style.display.as_deref() {
        if matches!(
            display,
            "inline" | "inline-block" | "inline-flex" | "contents"
        ) {
            return false;
        }
    }

    matches!(
        tag,
        HtmlTag::Address
            | HtmlTag::Article
            | HtmlTag::Aside
            | HtmlTag::Blockquote
            | HtmlTag::Dd
            | HtmlTag::Div
            | HtmlTag::Dl
            | HtmlTag::Dt
            | HtmlTag::Figcaption
            | HtmlTag::Figure
            | HtmlTag::Footer
            | HtmlTag::Form
            | HtmlTag::H1
            | HtmlTag::H2
            | HtmlTag::H3
            | HtmlTag::H4
            | HtmlTag::H5
            | HtmlTag::H6
            | HtmlTag::Header
            | HtmlTag::Hr
            | HtmlTag::Li
            | HtmlTag::Main
            | HtmlTag::Nav
            | HtmlTag::Ol
            | HtmlTag::P
            | HtmlTag::Pre
            | HtmlTag::Section
            | HtmlTag::Table
            | HtmlTag::Ul
    )
}

fn block_margin_after(tag: &HtmlTag) -> f32 {
    match tag {
        HtmlTag::H1 => 10.0,
        HtmlTag::H2 => 8.0,
        HtmlTag::H3 | HtmlTag::H4 | HtmlTag::H5 | HtmlTag::H6 => 6.0,
        HtmlTag::P | HtmlTag::Pre | HtmlTag::Blockquote => 10.0,
        HtmlTag::Li => 4.0,
        HtmlTag::Nav | HtmlTag::Header | HtmlTag::Footer => 8.0,
        _ => 6.0,
    }
}

fn apply_text_transform(text: String, transform: Option<&str>) -> String {
    match transform {
        Some("uppercase") => text.to_uppercase(),
        Some("lowercase") => text.to_lowercase(),
        Some("capitalize") => text
            .split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                    None => String::new(),
                }
            })
            .collect::<Vec<_>>()
            .join(" "),
        _ => text,
    }
}

fn normalize_text(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn normalize_fragments(fragments: &mut Vec<TextFragment>) {
    let mut cleaned = Vec::new();
    let mut previous_key = String::new();

    for mut fragment in fragments.drain(..) {
        fragment.text = collapse_repeated_text(&normalize_text(&fragment.text));
        if fragment.text.len() <= 2 || is_low_value_text(&fragment.text) {
            continue;
        }

        let key = fragment.text.to_lowercase();
        if key == previous_key {
            continue;
        }

        previous_key = key;
        cleaned.push(fragment);
    }

    *fragments = cleaned;
}

#[derive(Default)]
struct PageMetadata {
    title: Option<String>,
    description: Option<String>,
    site_name: Option<String>,
    canonical_url: Option<String>,
}

#[derive(Clone, Debug)]
struct VideoCard {
    title: String,
    url: String,
    subtitle: Option<String>,
    duration: Option<String>,
}

fn append_app_shell_fallback(
    dom: &[DomNode],
    raw_html: &str,
    fragments: &mut Vec<TextFragment>,
    text_color: [f32; 4],
) {
    let visible_fragments = fragments
        .iter()
        .filter(|fragment| fragment.px_size >= 15.0 && fragment.text.len() > 3)
        .count();
    if visible_fragments >= 3 {
        return;
    }

    let metadata = collect_page_metadata(dom);
    let mut added = 0;

    if let Some(title) = metadata.title.as_deref().filter(|title| !title.is_empty()) {
        push_fallback_fragment(fragments, title, 30.0, true, 38.0, 8.0, text_color, true);
        added += 1;
    }

    if let Some(description) = metadata
        .description
        .as_deref()
        .filter(|description| !description.is_empty())
    {
        push_fallback_fragment(
            fragments,
            description,
            16.0,
            false,
            23.0,
            10.0,
            text_color,
            true,
        );
        added += 1;
    }

    let video_cards = extract_embedded_video_cards(raw_html, 12);
    if !video_cards.is_empty() {
        push_fallback_fragment(
            fragments,
            "Videos detectados",
            20.0,
            true,
            28.0,
            8.0,
            text_color,
            true,
        );
        for video in video_cards {
            push_video_card_fragment(fragments, video);
        }
        added += 1;
    }

    let app_texts = extract_embedded_app_text(raw_html, 10, &metadata);
    if !app_texts.is_empty() {
        let source = metadata.site_name.as_deref().unwrap_or("aplicacion web");
        push_fallback_fragment(
            fragments,
            &format!("Vista ligera de {source}"),
            18.0,
            true,
            26.0,
            6.0,
            text_color,
            true,
        );
        for text in app_texts {
            push_fallback_fragment(fragments, &text, 15.0, false, 22.0, 4.0, text_color, true);
        }
        added += 1;
    }

    if added == 0 {
        push_fallback_fragment(
            fragments,
            "Aplicacion web detectada: el HTML inicial no trae contenido visible suficiente.",
            16.0,
            false,
            23.0,
            8.0,
            text_color,
            true,
        );
    }

    if let Some(canonical_url) = metadata.canonical_url {
        push_fallback_fragment(
            fragments,
            &canonical_url,
            13.0,
            false,
            19.0,
            4.0,
            [0.478, 0.635, 0.968, 1.0],
            true,
        );
    }
}

fn push_video_card_fragment(fragments: &mut Vec<TextFragment>, video: VideoCard) {
    let mut text = video.title;
    let mut details = Vec::new();
    if let Some(duration) = video.duration.filter(|value| !value.is_empty()) {
        details.push(duration);
    }
    if let Some(subtitle) = video.subtitle.filter(|value| !value.is_empty()) {
        details.push(subtitle);
    }
    if !details.is_empty() {
        text.push_str(" - ");
        text.push_str(&details.join(" / "));
    }

    fragments.push(TextFragment {
        text: normalize_text(&text),
        px_size: 15.0,
        is_bold: false,
        line_height: 22.0,
        margin_after: 5.0,
        line_break_after: true,
        layout: FragmentLayout {
            max_width: Some("920px".to_string()),
            ..FragmentLayout::default()
        },
        color: [0.478, 0.635, 0.968, 1.0],
        href: Some(video.url),
    });
}

fn push_fallback_fragment(
    fragments: &mut Vec<TextFragment>,
    text: &str,
    px_size: f32,
    is_bold: bool,
    line_height: f32,
    margin_after: f32,
    color: [f32; 4],
    line_break_after: bool,
) {
    let text = normalize_text(text);
    if text.len() <= 2 {
        return;
    }

    fragments.push(TextFragment {
        text,
        px_size,
        is_bold,
        line_height,
        margin_after,
        line_break_after,
        layout: FragmentLayout {
            max_width: Some("860px".to_string()),
            ..FragmentLayout::default()
        },
        color,
        href: None,
    });
}

fn collect_page_metadata(nodes: &[DomNode]) -> PageMetadata {
    let mut metadata = PageMetadata::default();
    collect_page_metadata_inner(nodes, &mut metadata);
    metadata
}

fn collect_page_metadata_inner(nodes: &[DomNode], metadata: &mut PageMetadata) {
    for node in nodes {
        let DomNode::Element {
            tag,
            attributes,
            children,
        } = node
        else {
            continue;
        };

        if matches!(tag, HtmlTag::Custom(name) if name == "title") {
            let title = collect_node_text(children);
            if !title.trim().is_empty() {
                metadata.title = Some(normalize_text(&title));
            }
        }

        if matches!(tag, HtmlTag::Custom(name) if name == "meta") {
            let key = attributes
                .get("name")
                .or_else(|| attributes.get("property"))
                .map(|value| value.to_ascii_lowercase());
            if let (Some(key), Some(content)) = (key, attributes.get("content")) {
                let content = normalize_text(content);
                match key.as_str() {
                    "description" | "og:description" | "twitter:description"
                        if metadata.description.is_none() =>
                    {
                        metadata.description = Some(content)
                    }
                    "og:title" | "twitter:title" | "title" if metadata.title.is_none() => {
                        metadata.title = Some(content)
                    }
                    "og:site_name" | "application-name" if metadata.site_name.is_none() => {
                        metadata.site_name = Some(content)
                    }
                    _ => {}
                }
            }
        }

        if matches!(tag, HtmlTag::Custom(name) if name == "link")
            && attributes
                .get("rel")
                .is_some_and(|rel| rel.to_ascii_lowercase().contains("canonical"))
            && metadata.canonical_url.is_none()
        {
            metadata.canonical_url = attributes.get("href").cloned();
        }

        collect_page_metadata_inner(children, metadata);
    }
}

fn collect_node_text(nodes: &[DomNode]) -> String {
    let mut out = String::new();
    for node in nodes {
        match node {
            DomNode::Text(text) => {
                out.push_str(text);
                out.push(' ');
            }
            DomNode::Element { children, .. } => out.push_str(&collect_node_text(children)),
        }
    }
    out
}

fn extract_embedded_app_text(raw_html: &str, limit: usize, metadata: &PageMetadata) -> Vec<String> {
    let mut out = Vec::new();
    for marker in [
        "\"label\":\"",
        "\"simpleText\":\"",
        "\"text\":\"",
        "\"title\":\"",
        "\"ariaLabel\":\"",
    ] {
        collect_json_string_values(raw_html, marker, limit, metadata, &mut out);
        if out.len() >= limit {
            break;
        }
    }
    out
}

fn extract_embedded_video_cards(raw_html: &str, limit: usize) -> Vec<VideoCard> {
    let Some(json) = extract_assigned_json(raw_html, "ytInitialData") else {
        return Vec::new();
    };
    let Ok(value) = serde_json::from_str::<Value>(&json) else {
        return Vec::new();
    };

    let mut videos = Vec::new();
    collect_video_cards(&value, limit, &mut videos);
    videos
}

fn collect_video_cards(value: &Value, limit: usize, out: &mut Vec<VideoCard>) {
    if out.len() >= limit {
        return;
    }

    match value {
        Value::Object(map) => {
            for key in [
                "videoRenderer",
                "compactVideoRenderer",
                "gridVideoRenderer",
                "playlistVideoRenderer",
                "reelItemRenderer",
            ] {
                if let Some(renderer) = map.get(key) {
                    if let Some(video) = video_card_from_renderer(renderer) {
                        if !out.iter().any(|existing| existing.url == video.url) {
                            out.push(video);
                            if out.len() >= limit {
                                return;
                            }
                        }
                    }
                }
            }

            for child in map.values() {
                collect_video_cards(child, limit, out);
                if out.len() >= limit {
                    return;
                }
            }
        }
        Value::Array(items) => {
            for child in items {
                collect_video_cards(child, limit, out);
                if out.len() >= limit {
                    return;
                }
            }
        }
        _ => {}
    }
}

fn video_card_from_renderer(renderer: &Value) -> Option<VideoCard> {
    let video_id = renderer.get("videoId")?.as_str()?;
    if video_id.len() < 6 {
        return None;
    }

    let title = text_from_json_text(renderer.get("title")?)
        .or_else(|| renderer.get("headline").and_then(text_from_json_text))
        .filter(|title| is_useful_video_title(title))?;
    let subtitle = renderer
        .get("ownerText")
        .or_else(|| renderer.get("longBylineText"))
        .or_else(|| renderer.get("shortBylineText"))
        .and_then(text_from_json_text);
    let duration = renderer
        .get("lengthText")
        .or_else(|| renderer.get("thumbnailOverlayTimeStatusRenderer"))
        .and_then(text_from_json_text);

    Some(VideoCard {
        title,
        url: format!("https://www.youtube.com/watch?v={video_id}"),
        subtitle,
        duration,
    })
}

fn text_from_json_text(value: &Value) -> Option<String> {
    if let Some(text) = value.get("simpleText").and_then(Value::as_str) {
        return Some(normalize_text(text));
    }

    let runs = value.get("runs")?.as_array()?;
    let text = runs
        .iter()
        .filter_map(|run| run.get("text").and_then(Value::as_str))
        .collect::<Vec<_>>()
        .join("");
    if text.trim().is_empty() {
        None
    } else {
        Some(normalize_text(&text))
    }
}

fn is_useful_video_title(title: &str) -> bool {
    let lower = title.to_ascii_lowercase();
    title.len() >= 3
        && !is_noisy_app_text(&lower)
        && !lower.contains("youtube")
        && !lower.contains("busca algo")
}

fn extract_assigned_json(raw_html: &str, variable: &str) -> Option<String> {
    let marker = format!("{variable} = ");
    let start = raw_html.find(&marker)? + marker.len();
    let json_start = raw_html[start..].find('{')? + start;
    extract_balanced_json_object(&raw_html[json_start..])
}

fn extract_balanced_json_object(text: &str) -> Option<String> {
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    for (idx, ch) in text.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }

        match ch {
            '"' => in_string = true,
            '{' => depth += 1,
            '}' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(text[..=idx].to_string());
                }
            }
            _ => {}
        }
    }
    None
}

fn collect_json_string_values(
    raw_html: &str,
    marker: &str,
    limit: usize,
    metadata: &PageMetadata,
    out: &mut Vec<String>,
) {
    let mut start = 0;
    while out.len() < limit {
        let Some(pos) = raw_html[start..].find(marker) else {
            break;
        };
        let value_start = start + pos + marker.len();
        let Some((value, consumed)) = read_json_string_fragment(&raw_html[value_start..]) else {
            start = value_start;
            continue;
        };
        start = value_start + consumed;

        let value = normalize_text(&decode_json_text(&value));
        if is_useful_app_text(&value)
            && !matches_metadata_text(&value, metadata)
            && !out
                .iter()
                .any(|existing| existing.eq_ignore_ascii_case(&value))
        {
            out.push(value);
        }
    }
}

fn matches_metadata_text(text: &str, metadata: &PageMetadata) -> bool {
    let text = text.trim();
    [metadata.title.as_deref(), metadata.description.as_deref()]
        .into_iter()
        .flatten()
        .any(|metadata_text| {
            metadata_text.eq_ignore_ascii_case(text)
                || metadata_text
                    .to_ascii_lowercase()
                    .contains(&text.to_ascii_lowercase())
        })
}

fn read_json_string_fragment(text: &str) -> Option<(String, usize)> {
    let mut value = String::new();
    let mut escaped = false;
    for (idx, ch) in text.char_indices() {
        if escaped {
            value.push('\\');
            value.push(ch);
            escaped = false;
            continue;
        }
        match ch {
            '\\' => escaped = true,
            '"' => return Some((value, idx + 1)),
            _ => value.push(ch),
        }
    }
    None
}

fn decode_json_text(text: &str) -> String {
    let mut out = String::new();
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            out.push(ch);
            continue;
        }

        match chars.next() {
            Some('n') | Some('r') | Some('t') => out.push(' '),
            Some('"') => out.push('"'),
            Some('\\') => out.push('\\'),
            Some('/') => out.push('/'),
            Some('u') => {
                let hex = chars.by_ref().take(4).collect::<String>();
                if let Ok(code) = u32::from_str_radix(&hex, 16) {
                    if let Some(decoded) = char::from_u32(code) {
                        out.push(decoded);
                    }
                }
            }
            Some(other) => out.push(other),
            None => {}
        }
    }
    out
}

fn is_useful_app_text(text: &str) -> bool {
    let trimmed = text.trim();
    if trimmed.len() < 3 || trimmed.len() > 120 {
        return false;
    }
    let lower = trimmed.to_ascii_lowercase();
    if lower.starts_with("http")
        || lower.contains(".js")
        || lower.contains(".css")
        || lower.contains("sprite")
        || lower.contains("endpoint")
        || is_noisy_app_text(&lower)
    {
        return false;
    }
    let words = trimmed.split_whitespace().count();
    if words < 2 && trimmed.chars().count() < 14 {
        return false;
    }
    let letters = trimmed.chars().filter(|ch| ch.is_alphabetic()).count();
    letters >= 2
}

fn is_noisy_app_text(lower: &str) -> bool {
    const NOISE_PARTS: &[&str] = &[
        "acceder",
        "activar o desactivar",
        "adelantar",
        "aria",
        "atajo",
        "aumentar velocidad",
        "avanzar",
        "borrar busqueda",
        "borrar búsqueda",
        "cancelar",
        "capitulo",
        "capítulo",
        "combinaciones de teclas",
        "configuracion",
        "configuración",
        "cuadro anterior",
        "disminuir velocidad",
        "pantalla completa",
        "pausa",
        "principal",
        "realiza busquedas con la voz",
        "realiza búsquedas con la voz",
        "reproduccion",
        "reproducción",
        "retroceder",
        "saltar al",
        "siguiente cuadro",
        "siguiente video",
        "tecla",
        "video anterior",
    ];

    if NOISE_PARTS.iter().any(|part| lower.contains(part)) {
        return true;
    }

    matches!(
        lower.trim(),
        "buscar" | "coma" | "general" | "menos" | "punto" | "visitar la fuente"
    )
}

fn collapse_repeated_text(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() < 4 || chars.len() % 2 != 0 {
        return text.to_string();
    }

    let half = chars.len() / 2;
    if chars[..half] == chars[half..] {
        chars[..half].iter().collect()
    } else {
        text.to_string()
    }
}

fn is_low_value_text(text: &str) -> bool {
    matches!(text.trim(), "." | "," | "|" | "-" | "•")
}

fn estimated_text_width(text: &str, px_size: f32) -> f32 {
    text.chars().count() as f32 * (px_size * 0.54).max(7.0)
}

fn should_skip_element(tag: &HtmlTag, attributes: &HashMap<String, String>) -> bool {
    if matches!(
        tag,
        HtmlTag::Script | HtmlTag::Noscript | HtmlTag::Template | HtmlTag::Svg | HtmlTag::Canvas
    ) {
        return true;
    }

    if matches!(tag, HtmlTag::Custom(name) if name == "style" || name == "title") {
        return true;
    }

    if attributes.contains_key("hidden") {
        return true;
    }

    if attributes
        .get("aria-hidden")
        .is_some_and(|value| value.eq_ignore_ascii_case("true"))
    {
        return true;
    }

    if let Some(style) = attributes
        .get("style")
        .map(|value| value.to_ascii_lowercase())
    {
        if style.contains("display:none")
            || style.contains("display: none")
            || style.contains("visibility:hidden")
            || style.contains("visibility: hidden")
            || style.contains("opacity:0")
            || style.contains("opacity: 0")
        {
            return true;
        }
    }

    let Some(class) = attributes.get("class") else {
        return false;
    };

    class
        .split_whitespace()
        .map(|token| token.to_ascii_lowercase())
        .any(|token| {
            matches!(
                token.as_str(),
                "hidden"
                    | "d-none"
                    | "sr-only"
                    | "visually-hidden"
                    | "modal"
                    | "modal-dialog"
                    | "modal-content"
                    | "offcanvas"
                    | "dropdown-menu"
                    | "collapse"
            )
        })
}

fn apply_runtime_scripts(
    dom: &[DomNode],
    fragments: &mut Vec<TextFragment>,
    base_url: Option<&Url>,
) {
    let scripts = collect_scripts(dom, base_url);
    if scripts.is_empty() {
        return;
    }

    prefetch_external_scripts(&scripts);
    let report = BrowserRuntime::new().execute_scripts(&scripts);
    if report.inline_scripts_executed > 0 || !report.external_scripts_seen.is_empty() {
        println!(
            "[Runtime] inline={} external={} console={} unsupported={}",
            report.inline_scripts_executed,
            report.external_scripts_seen.len(),
            report.console_messages.len(),
            report.unsupported_statements.len()
        );
    }

    if let Some(title) = report.dom.title {
        fragments.insert(
            0,
            TextFragment {
                text: normalize_text(&title),
                px_size: 24.0,
                is_bold: true,
                line_height: 32.0,
                margin_after: 4.0,
                line_break_after: true,
                layout: FragmentLayout::default(),
                color: [1.0, 1.0, 1.0, 1.0],
                href: None,
            },
        );
    }

    for text in report.dom.appended_text {
        if fragments.len() >= MAX_TEXT_FRAGMENTS {
            break;
        }

        fragments.push(TextFragment {
            text: normalize_text(&text),
            px_size: 16.0,
            is_bold: false,
            line_height: 22.0,
            margin_after: 6.0,
            line_break_after: true,
            layout: FragmentLayout::default(),
            color: [1.0, 1.0, 1.0, 1.0],
            href: None,
        });
    }
}

fn prefetch_external_scripts(scripts: &[ScriptSource]) {
    let urls = scripts
        .iter()
        .filter_map(|script| match script {
            ScriptSource::External(url) => Some(url.clone()),
            ScriptSource::Inline(_) => None,
        })
        .take(24)
        .collect::<Vec<_>>();

    if urls.is_empty() {
        return;
    }

    std::thread::spawn(move || {
        let workers = urls
            .into_iter()
            .map(|url| {
                std::thread::spawn(move || {
                    let _ = crate::parsers::resource_loader::fetch_script(&url);
                })
            })
            .collect::<Vec<_>>();
        for worker in workers {
            let _ = worker.join();
        }
    });
}

fn append_media_summary(fragments: &mut Vec<TextFragment>, media: &MediaReport) {
    let Some(summary) = media.summary() else {
        return;
    };

    fragments.insert(
        0,
        TextFragment {
            text: summary,
            px_size: 14.0,
            is_bold: true,
            line_height: 20.0,
            margin_after: 8.0,
            line_break_after: true,
            layout: FragmentLayout::default(),
            color: [0.725, 0.790, 0.980, 1.0],
            href: None,
        },
    );
}
