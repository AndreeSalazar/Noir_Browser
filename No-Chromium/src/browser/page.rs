use crate::browser::LinkHitbox;
use crate::parsers::dom_tree::DomNode;
use crate::parsers::html_elements::HtmlTag;
use crate::render::text::{RasterizedAtlas, TextRasterizationOptions, TextRequest};
use crate::runtime::{collect_scripts, BrowserRuntime};
use url::Url;

const MAX_VISIBLE_TEXTS: usize = 48;
const MAX_VISIBLE_LINES: usize = 120;
const CONTENT_X: f32 = 40.0;
const CONTENT_TOP: f32 = 78.0;
const CONTENT_SIDE_PADDING: f32 = 80.0;

struct TextFragment {
    text: String,
    px_size: f32,
    is_bold: bool,
    line_height: f32,
    margin_after: f32,
    href: Option<String>,
}

pub fn load_page(
    target_url: &str,
    link_hitboxes: &mut Vec<LinkHitbox>,
    text_options: TextRasterizationOptions,
    viewport_width: f32,
) -> RasterizedAtlas {
    let html = crate::parsers::http_client::fetch_html(target_url)
        .unwrap_or_else(|_| "<h1>Network Error</h1>".to_string());
    let dom = crate::parsers::dom_tree::parse_html(&html);
    let base_url = Url::parse(target_url).ok();

    let mut fragments = Vec::new();
    extract_text_from_dom(&dom, &mut fragments, 24.0, false, 30.0, 4.0, None, base_url.as_ref());
    apply_runtime_scripts(&dom, &mut fragments, base_url.as_ref());
    normalize_fragments(&mut fragments);

    let mut text_requests = Vec::new();
    link_hitboxes.clear();

    text_requests.push(TextRequest {
        text: target_url.to_string(),
        px_size: 16.0,
        is_bold: false,
        pos_x: 20.0,
        pos_y: 48.0,
        color: [1.0, 1.0, 1.0, 1.0],
    });

    let content_width = (viewport_width - CONTENT_SIDE_PADDING).clamp(320.0, 1120.0);
    let mut current_y = CONTENT_TOP;
    let mut visible_lines = 0;

    for fragment in fragments {
        let color = if fragment.href.is_some() {
            [0.478, 0.635, 0.968, 1.0]
        } else {
            [1.0, 1.0, 1.0, 1.0]
        };

        let lines = wrap_text(&fragment.text, fragment.px_size, content_width);
        for line in lines {
            if visible_lines >= MAX_VISIBLE_LINES {
                break;
            }

            if let Some(href) = &fragment.href {
                link_hitboxes.push(LinkHitbox {
                    url: href.clone(),
                    y_min: current_y,
                    y_max: current_y + fragment.line_height,
                });
            }

            text_requests.push(TextRequest {
                text: line,
                px_size: fragment.px_size,
                is_bold: fragment.is_bold,
                pos_x: CONTENT_X,
                pos_y: current_y,
                color,
            });
            current_y += fragment.line_height;
            visible_lines += 1;
        }

        current_y += fragment.margin_after;
        if visible_lines >= MAX_VISIBLE_LINES {
            break;
        }
    }

    RasterizedAtlas::with_options(&text_requests, text_options)
}

fn extract_text_from_dom(
    nodes: &[DomNode],
    out: &mut Vec<TextFragment>,
    current_size: f32,
    current_bold: bool,
    current_line_height: f32,
    current_margin_after: f32,
    current_href: Option<String>,
    base_url: Option<&Url>,
) {
    for node in nodes {
        if out.len() >= MAX_VISIBLE_TEXTS {
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

                let mut new_size = current_size;
                let mut new_bold = current_bold;
                let mut line_height = current_line_height;
                let mut margin_after = current_margin_after;
                let mut new_href = current_href.clone();

                match tag {
                    HtmlTag::Custom(name) if name == "style" || name == "title" => continue,
                    HtmlTag::H1 => {
                        new_size = 32.0;
                        new_bold = true;
                        line_height = 40.0;
                        margin_after = 4.0;
                    }
                    HtmlTag::H2 => {
                        new_size = 24.0;
                        new_bold = true;
                        line_height = 32.0;
                        margin_after = 4.0;
                    }
                    HtmlTag::H3 => {
                        new_size = 20.0;
                        new_bold = true;
                        line_height = 28.0;
                        margin_after = 4.0;
                    }
                    HtmlTag::P => {
                        new_size = 16.0;
                        new_bold = false;
                        line_height = 22.0;
                        margin_after = 6.0;
                    }
                    HtmlTag::A => {
                        new_size = 14.0;
                        line_height = 20.0;
                        margin_after = 4.0;
                        if let Some(href) = attributes.get("href") {
                            new_href = resolve_url(base_url, href);
                        }
                    }
                    _ => {}
                }

                extract_text_from_dom(
                    children,
                    out,
                    new_size,
                    new_bold,
                    line_height,
                    margin_after,
                    new_href,
                    base_url,
                );
            }
            DomNode::Text(t) => {
                let trimmed = t.trim();
                if trimmed.len() > 2 {
                    out.push(TextFragment {
                        text: normalize_text(trimmed),
                        px_size: current_size,
                        is_bold: current_bold,
                        line_height: current_line_height,
                        margin_after: current_margin_after,
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

fn normalize_text(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn normalize_fragments(fragments: &mut Vec<TextFragment>) {
    let mut cleaned = Vec::new();
    let mut previous_key = String::new();

    for mut fragment in fragments.drain(..) {
        fragment.text = collapse_repeated_text(&normalize_text(&fragment.text));
        if fragment.text.len() <= 2 {
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

        let candidate_len = current.chars().count() + usize::from(!current.is_empty()) + word.chars().count();
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

fn should_skip_element(
    tag: &HtmlTag,
    attributes: &std::collections::HashMap<String, String>,
) -> bool {
    if matches!(
        tag,
        HtmlTag::Script
            | HtmlTag::Noscript
            | HtmlTag::Template
            | HtmlTag::Svg
            | HtmlTag::Canvas
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

    if let Some(style) = attributes.get("style").map(|value| value.to_ascii_lowercase()) {
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

fn apply_runtime_scripts(dom: &[DomNode], fragments: &mut Vec<TextFragment>, base_url: Option<&Url>) {
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
                href: None,
            },
        );
    }

    for text in report.dom.appended_text {
        if fragments.len() >= MAX_VISIBLE_TEXTS {
            break;
        }

        fragments.push(TextFragment {
            text: normalize_text(&text).chars().take(96).collect(),
            px_size: 16.0,
            is_bold: false,
            line_height: 22.0,
            margin_after: 6.0,
            href: None,
        });
    }
}
