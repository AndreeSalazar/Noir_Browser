use crate::browser::LinkHitbox;
use crate::parsers::dom_tree::DomNode;
use crate::parsers::html_elements::HtmlTag;
use crate::render::text::{RasterizedAtlas, TextRasterizationOptions, TextRequest};

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

    let mut fragments = Vec::new();
    extract_text_from_dom(&dom, &mut fragments, 24.0, false, 30.0, 4.0, None);

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
) {
    for node in nodes {
        if out.len() >= 4 {
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
                            let absolute_url = if href.starts_with('/') {
                                format!("https://example.com{}", href)
                            } else {
                                href.clone()
                            };
                            new_href = Some(absolute_url);
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
                );
            }
            DomNode::Text(t) => {
                let trimmed = t.trim();
                if trimmed.len() > 2 {
                    out.push(TextFragment {
                        text: trimmed.chars().take(40).collect(),
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
