use crate::browser::LinkHitbox;
use crate::parsers::dom_tree::DomNode;
use crate::parsers::html_elements::HtmlTag;
use crate::render::text::{RasterizedAtlas, TextRasterizationOptions, TextRequest};
use crate::runtime::{collect_scripts, BrowserRuntime};
use url::Url;

const MAX_VISIBLE_TEXTS: usize = 48;

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
) -> RasterizedAtlas {
    let html = crate::parsers::http_client::fetch_html(target_url)
        .unwrap_or_else(|_| "<h1>Network Error</h1>".to_string());
    let dom = crate::parsers::dom_tree::parse_html(&html);
    let base_url = Url::parse(target_url).ok();

    let mut fragments = Vec::new();
    extract_text_from_dom(&dom, &mut fragments, 24.0, false, 30.0, 4.0, None, base_url.as_ref());
    apply_runtime_scripts(&dom, &mut fragments, base_url.as_ref());

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

    let mut current_y = 78.0;
    for fragment in fragments {
        let color = if fragment.href.is_some() {
            [0.478, 0.635, 0.968, 1.0]
        } else {
            [1.0, 1.0, 1.0, 1.0]
        };

        if let Some(href) = fragment.href {
            link_hitboxes.push(LinkHitbox {
                url: href,
                y_min: current_y,
                y_max: current_y + fragment.line_height,
            });
        }

        text_requests.push(TextRequest {
            text: fragment.text,
            px_size: fragment.px_size,
            is_bold: fragment.is_bold,
            pos_x: 40.0,
            pos_y: current_y,
            color,
        });
        current_y += fragment.line_height + fragment.margin_after;
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
                if matches!(tag, HtmlTag::Script | HtmlTag::Noscript) {
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
                        text: normalize_text(trimmed).chars().take(96).collect(),
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
