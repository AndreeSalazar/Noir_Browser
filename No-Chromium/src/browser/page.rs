use crate::browser::LinkHitbox;
use crate::parsers::dom_tree::DomNode;
use crate::parsers::html_elements::HtmlTag;
use crate::render::text::{RasterizedAtlas, TextRequest};

struct TextFragment {
    text: String,
    px_size: f32,
    is_bold: bool,
    href: Option<String>,
}

pub fn load_page(target_url: &str, link_hitboxes: &mut Vec<LinkHitbox>) -> RasterizedAtlas {
    let html = crate::parsers::http_client::fetch_html(target_url)
        .unwrap_or_else(|_| "<h1>Network Error</h1>".to_string());
    let dom = crate::parsers::dom_tree::parse_html(&html);

    let mut fragments = Vec::new();
    extract_text_from_dom(&dom, &mut fragments, 24.0, false, None);

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

    let mut current_y = 80.0;
    for fragment in fragments {
        let color = if fragment.href.is_some() {
            [0.478, 0.635, 0.968, 1.0]
        } else {
            [1.0, 1.0, 1.0, 1.0]
        };

        if let Some(href) = fragment.href {
            link_hitboxes.push(LinkHitbox {
                url: href,
                y_min: current_y - fragment.px_size * 0.5,
                y_max: current_y + fragment.px_size * 1.5,
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
        current_y += 30.0;
    }

    RasterizedAtlas::new(&text_requests)
}

fn extract_text_from_dom(
    nodes: &[DomNode],
    out: &mut Vec<TextFragment>,
    current_size: f32,
    current_bold: bool,
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
                let mut new_href = current_href.clone();

                match tag {
                    HtmlTag::Custom(name) if name == "style" || name == "title" => continue,
                    HtmlTag::H1 => {
                        new_size = 32.0;
                        new_bold = true;
                    }
                    HtmlTag::H2 => {
                        new_size = 24.0;
                        new_bold = true;
                    }
                    HtmlTag::H3 => {
                        new_size = 20.0;
                        new_bold = true;
                    }
                    HtmlTag::P => {
                        new_size = 16.0;
                        new_bold = false;
                    }
                    HtmlTag::A => {
                        new_size = 14.0;
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

                extract_text_from_dom(children, out, new_size, new_bold, new_href);
            }
            DomNode::Text(t) => {
                let trimmed = t.trim();
                if trimmed.len() > 2 {
                    out.push(TextFragment {
                        text: trimmed.chars().take(40).collect(),
                        px_size: current_size,
                        is_bold: current_bold,
                        href: current_href.clone(),
                    });
                }
            }
        }
    }
}
