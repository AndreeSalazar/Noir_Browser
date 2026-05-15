use crate::browser::LinkHitbox;
use crate::media::discovery::{discover_media, MediaReport};
use crate::parsers::css_simple::{parse_color, parse_px, CssCascade};
use crate::parsers::dom_tree::DomNode;
use crate::parsers::html_elements::HtmlTag;
use crate::parsers::resource_loader::{
    fetch_document, CacheStatus, ResourceResponse, ResourceType,
};
use crate::parsers::style_collector::{
    collect_inline_styles, collect_stylesheets, StylesheetBundle,
};
use crate::render::text::{RasterizedAtlas, TextRasterizationOptions, TextRequest};
use crate::runtime::{collect_scripts, BrowserRuntime};
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
    color: [f32; 4],
    href: Option<String>,
}

#[derive(Clone, Debug)]
struct TextStyleState {
    px_size: f32,
    is_bold: bool,
    line_height: f32,
    margin_after: f32,
    color: [f32; 4],
    hidden: bool,
    text_transform: Option<String>,
}

impl Default for TextStyleState {
    fn default() -> Self {
        Self {
            px_size: 16.0,
            is_bold: false,
            line_height: 22.0,
            margin_after: 6.0,
            color: [1.0, 1.0, 1.0, 1.0],
            hidden: false,
            text_transform: None,
        }
    }
}

pub struct PageDocument {
    fragments: Vec<TextFragment>,
    media: MediaReport,
}

impl PageDocument {
    pub fn media_summary(&self) -> Option<String> {
        self.media.summary()
    }
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
    extract_text_from_dom(
        &dom,
        &mut fragments,
        &css,
        TextStyleState::default(),
        None,
        base_url.as_ref(),
    );
    append_stylesheet_summary(&mut fragments, &stylesheet_bundle);
    apply_runtime_scripts(&dom, &mut fragments, base_url.as_ref());
    append_response_summary(&mut fragments, &response);
    let media = discover_media(&dom, &response.final_url);
    append_media_summary(&mut fragments, &media);
    normalize_fragments(&mut fragments);

    PageDocument { fragments, media }
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

    for stylesheet in stylesheets.iter().take(16) {
        if let Ok(response) = crate::parsers::resource_loader::fetch_style(&stylesheet.url) {
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
            color: [0.725, 0.790, 0.980, 1.0],
            href: None,
        },
    );
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

    let content_width = (viewport_width - CONTENT_SIDE_PADDING).clamp(320.0, 1120.0);
    let visible_bottom = (viewport_height - VIEWPORT_BOTTOM_PADDING).max(CONTENT_TOP);
    let mut document_y = CONTENT_TOP;
    let mut visible_lines = 0;

    for fragment in &document.fragments {
        let color = if fragment.href.is_some() && fragment.color == TextStyleState::default().color
        {
            [0.478, 0.635, 0.968, 1.0]
        } else {
            fragment.color
        };

        for line in wrap_text(&fragment.text, fragment.px_size, content_width) {
            let screen_y = document_y - scroll_offset;
            let line_bottom = screen_y + fragment.line_height;

            if line_bottom >= CONTENT_TOP
                && screen_y <= visible_bottom
                && visible_lines < MAX_VISIBLE_LINES
            {
                if let Some(href) = &fragment.href {
                    link_hitboxes.push(LinkHitbox {
                        url: href.clone(),
                        y_min: screen_y,
                        y_max: line_bottom,
                    });
                }

                text_requests.push(TextRequest {
                    text: line,
                    px_size: fragment.px_size,
                    is_bold: fragment.is_bold,
                    pos_x: CONTENT_X,
                    pos_y: screen_y,
                    color,
                });
                visible_lines += 1;
            }

            document_y += fragment.line_height;
        }

        document_y += fragment.margin_after;
    }

    let content_height = document_y + VIEWPORT_BOTTOM_PADDING;
    PageRender {
        atlas: RasterizedAtlas::with_options(&text_requests, text_options),
        content_height,
    }
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
                    &css.declarations_for(tag, attributes),
                );
                let mut new_href = current_href.clone();

                match tag {
                    HtmlTag::A => {
                        if current_style.px_size <= 16.0 {
                            next_style.px_size = 14.0;
                            next_style.line_height = 20.0;
                        }
                        next_style.margin_after = 4.0;
                        if let Some(href) = attributes.get("href") {
                            new_href = resolve_url(base_url, href);
                        }
                    }
                    _ => {}
                }

                if next_style.hidden {
                    continue;
                }

                extract_text_from_dom(children, out, css, next_style, new_href, base_url);
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
                        margin_after: current_style.margin_after,
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
    if declarations.display.as_deref() == Some("none")
        || declarations.visibility.as_deref() == Some("hidden")
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

    style
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

fn wrap_text(text: &str, px_size: f32, max_width: f32) -> Vec<String> {
    let avg_char_width = (px_size * 0.54).max(7.0);
    let max_chars = (max_width / avg_char_width).floor().max(12.0) as usize;
    let mut lines = Vec::new();
    let mut current = String::new();

    for word in text.split_whitespace() {
        if word.chars().count() > max_chars {
            flush_line(&mut lines, &mut current);
            split_long_word(word, max_chars, &mut lines);
            continue;
        }

        let candidate_len =
            current.chars().count() + usize::from(!current.is_empty()) + word.chars().count();
        if candidate_len > max_chars {
            flush_line(&mut lines, &mut current);
        }

        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(word);
    }

    flush_line(&mut lines, &mut current);

    if lines.is_empty() {
        vec![text.to_string()]
    } else {
        lines
    }
}

fn flush_line(lines: &mut Vec<String>, current: &mut String) {
    if !current.trim().is_empty() {
        lines.push(current.trim().to_string());
        current.clear();
    }
}

fn split_long_word(word: &str, max_chars: usize, lines: &mut Vec<String>) {
    let mut chunk = String::new();
    for ch in word.chars() {
        if chunk.chars().count() >= max_chars {
            lines.push(chunk);
            chunk = String::new();
        }
        chunk.push(ch);
    }
    if !chunk.is_empty() {
        lines.push(chunk);
    }
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
            color: [1.0, 1.0, 1.0, 1.0],
            href: None,
        });
    }
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
            color: [0.725, 0.790, 0.980, 1.0],
            href: None,
        },
    );
}
