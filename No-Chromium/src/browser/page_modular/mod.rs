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
use std::collections::HashMap;
use url::Url;

mod app_shell;

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
    append_direct_resource_notice(&mut fragments, &response, page_style.default_text_color);
    append_stylesheet_summary(&mut fragments, &stylesheet_bundle);
    apply_runtime_scripts(&dom, &mut fragments, base_url.as_ref());
    app_shell::append_app_shell_fallback(
        &dom,
        &response.body,
        &response.final_url,
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

fn append_direct_resource_notice(
    fragments: &mut Vec<TextFragment>,
    response: &ResourceResponse,
    text_color: [f32; 4],
) {
    let Some(kind) = direct_resource_kind(response) else {
        return;
    };

    let visible_fragments = fragments
        .iter()
        .filter(|fragment| fragment.px_size >= 15.0 && fragment.text.len() > 3)
        .count();
    if visible_fragments >= 3 {
        return;
    }

    push_notice_fragment(
        fragments,
        &format!("{kind} detectado"),
        22.0,
        true,
        text_color,
    );

    if response.status == 403 {
        push_notice_fragment(
            fragments,
            "HTTP 403: el enlace directo expiro o requiere firma, cookies o cabeceras del reproductor original.",
            16.0,
            false,
            text_color,
        );
        push_notice_fragment(
            fragments,
            "Vuelve al enlace /watch de YouTube para reconstruir la vista ligera; el probe Python puede diagnosticar formatos cifrados.",
            15.0,
            false,
            text_color,
        );
    } else if response.is_success() {
        push_notice_fragment(
            fragments,
            "Es un recurso directo, no una pagina HTML completa. El navegador lo reconoce y evita mostrar una pagina vacia.",
            16.0,
            false,
            text_color,
        );
    } else {
        push_notice_fragment(
            fragments,
            &format!(
                "La red respondio HTTP {} con {} bytes; no hay HTML visible que renderizar.",
                response.status, response.body_bytes
            ),
            16.0,
            false,
            text_color,
        );
    }

    push_notice_fragment(
        fragments,
        &response.final_url,
        13.0,
        false,
        [0.478, 0.635, 0.968, 1.0],
    );
}

fn direct_resource_kind(response: &ResourceResponse) -> Option<&'static str> {
    let requested = response.requested_url.to_ascii_lowercase();
    let final_url = response.final_url.to_ascii_lowercase();
    let content_type = response
        .content_type
        .as_deref()
        .unwrap_or_default()
        .to_ascii_lowercase();

    if requested.contains("googlevideo.com/videoplayback")
        || final_url.contains("googlevideo.com/videoplayback")
    {
        return Some("Stream directo de YouTube/Googlevideo");
    }

    if content_type.starts_with("video/")
        || content_type.starts_with("audio/")
        || content_type.contains("application/dash+xml")
        || content_type.contains("mpegurl")
    {
        return Some("Recurso multimedia directo");
    }

    if final_url.ends_with(".mp4")
        || final_url.ends_with(".webm")
        || final_url.ends_with(".m4a")
        || final_url.ends_with(".mp3")
        || final_url.ends_with(".m3u8")
        || final_url.ends_with(".mpd")
    {
        return Some("Recurso multimedia directo");
    }

    if !response.is_html_like() {
        return Some("Recurso no HTML");
    }

    None
}

fn push_notice_fragment(
    fragments: &mut Vec<TextFragment>,
    text: &str,
    px_size: f32,
    is_bold: bool,
    color: [f32; 4],
) {
    fragments.push(TextFragment {
        text: text.to_string(),
        px_size,
        is_bold,
        line_height: (px_size + 7.0).max(20.0),
        margin_after: 6.0,
        line_break_after: true,
        layout: FragmentLayout::default(),
        color,
        href: None,
    });
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

                if let Some(fragment) = intrinsic_element_fragment(
                    tag,
                    attributes,
                    &next_style,
                    new_href.as_deref(),
                    base_url,
                ) {
                    out.push(fragment);
                    if is_void_or_external_element(tag) {
                        continue;
                    }
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

fn intrinsic_element_fragment(
    tag: &HtmlTag,
    attributes: &HashMap<String, String>,
    style: &TextStyleState,
    inherited_href: Option<&str>,
    base_url: Option<&Url>,
) -> Option<TextFragment> {
    let (text, href, line_break_after) = match tag {
        HtmlTag::Img => {
            let label =
                first_attribute(attributes, &["alt", "title", "aria-label"]).or_else(|| {
                    source_label(attributes, base_url, &["src", "data-src", "data-original"])
                });
            let label = label.unwrap_or_else(|| "imagen".to_string());
            let href = inherited_href
                .map(str::to_string)
                .or_else(|| first_resolved_attribute(attributes, base_url, &["src", "data-src"]));
            (format!("Imagen: {label}"), href, true)
        }
        HtmlTag::Video | HtmlTag::Audio => {
            let kind = if matches!(tag, HtmlTag::Video) {
                "Video"
            } else {
                "Audio"
            };
            let label = first_attribute(attributes, &["title", "aria-label"])
                .or_else(|| source_label(attributes, base_url, &["src", "poster"]))
                .unwrap_or_else(|| "medio incrustado".to_string());
            let href = first_resolved_attribute(attributes, base_url, &["src", "poster"]);
            (format!("{kind}: {label}"), href, true)
        }
        HtmlTag::Iframe | HtmlTag::Embed | HtmlTag::Object => {
            let label = first_attribute(attributes, &["title", "aria-label", "name"])
                .or_else(|| source_label(attributes, base_url, &["src", "data"]))
                .unwrap_or_else(|| "contenido embebido".to_string());
            let href = first_resolved_attribute(attributes, base_url, &["src", "data"]);
            (format!("Embebido: {label}"), href, true)
        }
        HtmlTag::Input => {
            let input_type = attributes
                .get("type")
                .map(|value| value.to_ascii_lowercase())
                .unwrap_or_else(|| "text".to_string());
            if input_type == "hidden" {
                return None;
            }
            let label = first_attribute(
                attributes,
                &["aria-label", "placeholder", "value", "name", "id"],
            )
            .unwrap_or_else(|| input_type.clone());
            (
                format!("Campo {input_type}: {label}"),
                inherited_href.map(str::to_string),
                false,
            )
        }
        HtmlTag::Textarea | HtmlTag::Select => {
            let kind = if matches!(tag, HtmlTag::Textarea) {
                "Area de texto"
            } else {
                "Selector"
            };
            let label = first_attribute(attributes, &["aria-label", "placeholder", "name", "id"])
                .unwrap_or_else(|| "control".to_string());
            (
                format!("{kind}: {label}"),
                inherited_href.map(str::to_string),
                false,
            )
        }
        HtmlTag::Progress | HtmlTag::Meter => {
            let value = first_attribute(attributes, &["value", "aria-valuenow"])
                .unwrap_or_else(|| "sin valor".to_string());
            (
                format!("Indicador: {value}"),
                inherited_href.map(str::to_string),
                false,
            )
        }
        _ => return None,
    };

    Some(TextFragment {
        text,
        px_size: style.px_size.max(13.0),
        is_bold: false,
        line_height: style.line_height.max(18.0),
        margin_after: 4.0,
        line_break_after,
        layout: style.layout.clone(),
        color: soften_auxiliary_color(style.color),
        href,
    })
}

fn is_void_or_external_element(tag: &HtmlTag) -> bool {
    matches!(
        tag,
        HtmlTag::Img
            | HtmlTag::Input
            | HtmlTag::Iframe
            | HtmlTag::Embed
            | HtmlTag::Object
            | HtmlTag::Audio
            | HtmlTag::Video
    )
}

fn first_attribute(attributes: &HashMap<String, String>, names: &[&str]) -> Option<String> {
    names.iter().find_map(|name| {
        attributes
            .get(*name)
            .map(|value| normalize_text(value))
            .filter(|value| !value.is_empty())
    })
}

fn first_resolved_attribute(
    attributes: &HashMap<String, String>,
    base_url: Option<&Url>,
    names: &[&str],
) -> Option<String> {
    names.iter().find_map(|name| {
        attributes
            .get(*name)
            .and_then(|value| resolve_url(base_url, value))
    })
}

fn source_label(
    attributes: &HashMap<String, String>,
    base_url: Option<&Url>,
    names: &[&str],
) -> Option<String> {
    let url = first_resolved_attribute(attributes, base_url, names)?;
    Some(compact_resource_label(&url))
}

fn compact_resource_label(url: &str) -> String {
    Url::parse(url)
        .ok()
        .and_then(|parsed| {
            parsed
                .path_segments()
                .and_then(|mut segments| segments.next_back())
                .filter(|segment| !segment.is_empty())
                .map(str::to_string)
        })
        .unwrap_or_else(|| url.chars().take(72).collect())
}

fn soften_auxiliary_color(color: [f32; 4]) -> [f32; 4] {
    [
        (color[0] * 0.82 + 0.18).min(1.0),
        (color[1] * 0.82 + 0.18).min(1.0),
        (color[2] * 0.82 + 0.18).min(1.0),
        color[3],
    ]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_googlevideo_direct_resource() {
        let response = ResourceResponse {
            requested_url: "https://rr1---sn.googlevideo.com/videoplayback?id=abc".to_string(),
            final_url: "https://rr1---sn.googlevideo.com/videoplayback?id=abc".to_string(),
            resource_type: ResourceType::Document,
            status: 403,
            content_type: Some("text/plain".to_string()),
            body: String::new(),
            body_bytes: 0,
            cache_status: CacheStatus::Network,
        };

        assert_eq!(
            direct_resource_kind(&response),
            Some("Stream directo de YouTube/Googlevideo")
        );
    }

    #[test]
    fn renders_image_placeholder_from_alt_text() {
        let mut attributes = HashMap::new();
        attributes.insert("alt".to_string(), "Miniatura del video".to_string());
        attributes.insert("src".to_string(), "/thumb.jpg".to_string());
        let base_url = Url::parse("https://example.com/watch").ok();

        let fragment = intrinsic_element_fragment(
            &HtmlTag::Img,
            &attributes,
            &TextStyleState::default(),
            None,
            base_url.as_ref(),
        )
        .expect("image should produce a visible fragment");

        assert_eq!(fragment.text, "Imagen: Miniatura del video");
        assert_eq!(
            fragment.href.as_deref(),
            Some("https://example.com/thumb.jpg")
        );
    }
}
