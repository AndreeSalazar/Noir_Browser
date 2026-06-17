use crate::parsers::dom_tree::{DomNode, parse_html};
use crate::parsers::html_elements::HtmlTag;

#[derive(Clone, Debug)]
pub struct TextBlock {
    pub text: String,
    pub tag: String,
    pub font_size: f32,
    pub bold: bool,
    pub link: Option<String>,
    pub indent_level: u32,
}

#[derive(Clone, Debug)]
pub struct ImageBlock {
    pub src: String,
    pub alt: String,
    pub width: Option<f32>,
    pub height: Option<f32>,
}

#[derive(Clone, Debug)]
pub struct LinkInfo {
    pub text: String,
    pub href: String,
}

#[derive(Clone, Debug)]
pub struct PageDocument {
    pub url: String,
    pub title: String,
    pub text_blocks: Vec<TextBlock>,
    pub image_blocks: Vec<ImageBlock>,
    pub links: Vec<LinkInfo>,
    pub style_blocks: Vec<String>,
}

impl PageDocument {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            title: String::new(),
            text_blocks: Vec::new(),
            image_blocks: Vec::new(),
            links: Vec::new(),
            style_blocks: Vec::new(),
        }
    }

    pub fn from_html(url: &str, html: &str) -> Self {
        let mut doc = PageDocument::new(url);
        doc.extract_style_blocks(html);
        let nodes = parse_html(html);
        doc.extract_from_nodes(&nodes, 0, &mut Vec::new(), None);
        doc
    }

    fn extract_style_blocks(&mut self, html: &str) {
        let mut remaining = html;
        while let Some(start) = remaining.find("<style") {
            let after_tag = &remaining[start..];
            if let Some(gt) = after_tag.find('>') {
                let content_start = gt + 1;
                if let Some(end) = remaining[start + content_start..].find("</style>") {
                    let css = &remaining[start + content_start..start + content_start + end];
                    if !css.trim().is_empty() {
                        self.style_blocks.push(css.to_string());
                    }
                    remaining = &remaining[start + content_start + end..];
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }

    fn extract_from_nodes(
        &mut self,
        nodes: &[DomNode],
        indent: u32,
        ancestors: &mut Vec<String>,
        current_href: Option<String>,
    ) {
        for node in nodes {
            match node {
                DomNode::Element {
                    tag,
                    attributes,
                    children,
                } => {
                    match tag {
                        HtmlTag::Title => {
                            self.title = self.collect_text(children);
                        }
                        HtmlTag::Style => {
                            // Style blocks are now extracted from raw HTML
                        }
                        HtmlTag::H1 => {
                            let text = self.collect_text(children);
                            if !text.is_empty() {
                                self.text_blocks.push(TextBlock {
                                    text,
                                    tag: "h1".into(),
                                    font_size: 28.0,
                                    bold: true,
                                    link: current_href.clone(),
                                    indent_level: indent,
                                });
                            }
                        }
                        HtmlTag::H2 => {
                            let text = self.collect_text(children);
                            if !text.is_empty() {
                                self.text_blocks.push(TextBlock {
                                    text,
                                    tag: "h2".into(),
                                    font_size: 22.0,
                                    bold: true,
                                    link: current_href.clone(),
                                    indent_level: indent,
                                });
                            }
                        }
                        HtmlTag::H3 => {
                            let text = self.collect_text(children);
                            if !text.is_empty() {
                                self.text_blocks.push(TextBlock {
                                    text,
                                    tag: "h3".into(),
                                    font_size: 18.0,
                                    bold: true,
                                    link: current_href.clone(),
                                    indent_level: indent,
                                });
                            }
                        }
                        HtmlTag::H4 | HtmlTag::H5 | HtmlTag::H6 => {
                            let text = self.collect_text(children);
                            if !text.is_empty() {
                                self.text_blocks.push(TextBlock {
                                    text,
                                    tag: "h4".into(),
                                    font_size: 16.0,
                                    bold: true,
                                    link: current_href.clone(),
                                    indent_level: indent,
                                });
                            }
                        }
                        HtmlTag::P => {
                            let text = self.collect_text(children);
                            if !text.is_empty() {
                                self.text_blocks.push(TextBlock {
                                    text,
                                    tag: "p".into(),
                                    font_size: 14.0,
                                    bold: false,
                                    link: current_href.clone(),
                                    indent_level: indent,
                                });
                            }
                        }
                        HtmlTag::A => {
                            let href = attributes
                                .get("href")
                                .cloned()
                                .unwrap_or_default();
                            let text = self.collect_text(children);
                            if !text.is_empty() {
                                let resolved = self.resolve_href(&href);
                                self.links.push(LinkInfo {
                                    text: text.clone(),
                                    href: resolved.clone(),
                                });
                                self.text_blocks.push(TextBlock {
                                    text,
                                    tag: "a".into(),
                                    font_size: 14.0,
                                    bold: false,
                                    link: Some(resolved),
                                    indent_level: indent,
                                });
                            }
                        }
                        HtmlTag::B | HtmlTag::Strong | HtmlTag::Em | HtmlTag::I | HtmlTag::Small | HtmlTag::U | HtmlTag::Cite | HtmlTag::Dfn | HtmlTag::Mark | HtmlTag::Q | HtmlTag::S | HtmlTag::Samp | HtmlTag::Var | HtmlTag::Kbd | HtmlTag::Abbr | HtmlTag::Time | HtmlTag::Data | HtmlTag::Del | HtmlTag::Ins | HtmlTag::Sub | HtmlTag::Sup => {
                            let text = self.collect_text(children);
                            if !text.is_empty() {
                                self.text_blocks.push(TextBlock {
                                    text,
                                    tag: "text".into(),
                                    font_size: 14.0,
                                    bold: matches!(tag, HtmlTag::B | HtmlTag::Strong),
                                    link: current_href.clone(),
                                    indent_level: indent,
                                });
                            }
                        }
                        HtmlTag::Li => {
                            let text = self.collect_text(children);
                            if !text.is_empty() {
                                self.text_blocks.push(TextBlock {
                                    text: format!("  * {}", text),
                                    tag: "li".into(),
                                    font_size: 14.0,
                                    bold: false,
                                    link: current_href.clone(),
                                    indent_level: indent + 1,
                                });
                            }
                        }
                        HtmlTag::Pre | HtmlTag::Code => {
                            let text = self.collect_text(children);
                            if !text.is_empty() {
                                self.text_blocks.push(TextBlock {
                                    text,
                                    tag: "code".into(),
                                    font_size: 12.0,
                                    bold: false,
                                    link: current_href.clone(),
                                    indent_level: indent,
                                });
                            }
                        }
                        HtmlTag::Img => {
                            if let Some(src) = attributes.get("src") {
                                let resolved = self.resolve_href(src);
                                let alt = attributes.get("alt").cloned().unwrap_or_default();
                                let width = attributes.get("width")
                                    .and_then(|w| w.trim().parse::<f32>().ok());
                                let height = attributes.get("height")
                                    .and_then(|h| h.trim().parse::<f32>().ok());
                                self.image_blocks.push(ImageBlock {
                                    src: resolved,
                                    alt,
                                    width,
                                    height,
                                });
                            }
                        }
                        HtmlTag::Blockquote => {
                            let text = self.collect_text(children);
                            if !text.is_empty() {
                                self.text_blocks.push(TextBlock {
                                    text: format!("> {}", text),
                                    tag: "blockquote".into(),
                                    font_size: 14.0,
                                    bold: false,
                                    link: current_href.clone(),
                                    indent_level: indent + 1,
                                });
                            }
                        }
                        HtmlTag::Hr => {
                            self.text_blocks.push(TextBlock {
                                text: "────────────────────────────────".into(),
                                tag: "hr".into(),
                                font_size: 14.0,
                                bold: false,
                                link: None,
                                indent_level: indent,
                            });
                        }
                        HtmlTag::Table | HtmlTag::Tbody | HtmlTag::Thead | HtmlTag::Tfoot | HtmlTag::Tr | HtmlTag::Td | HtmlTag::Th | HtmlTag::Caption | HtmlTag::Col | HtmlTag::Colgroup => {
                            self.extract_from_nodes(children, indent, ancestors, current_href.clone());
                        }
                        HtmlTag::Ol | HtmlTag::Ul | HtmlTag::Dl => {
                            self.extract_from_nodes(children, indent, ancestors, current_href.clone());
                        }
                        HtmlTag::Div | HtmlTag::Section | HtmlTag::Article | HtmlTag::Main | HtmlTag::Span | HtmlTag::Header | HtmlTag::Footer | HtmlTag::Nav | HtmlTag::Aside | HtmlTag::Address | HtmlTag::Figure | HtmlTag::Details | HtmlTag::Dialog | HtmlTag::Summary | HtmlTag::Slot => {
                            self.extract_from_nodes(
                                children,
                                indent,
                                ancestors,
                                current_href.clone(),
                            );
                        }
                        HtmlTag::Body | HtmlTag::Html => {
                            self.extract_from_nodes(children, indent, ancestors, current_href.clone());
                        }
                        HtmlTag::Custom(ref name) if name == "head" || name == "meta" || name == "link" => {
                            // Skip head metadata
                        }
                        HtmlTag::Custom(ref name) if name == "script" || name == "noscript" => {
                            // Skip scripts (already skipped in dom_tree but just in case)
                        }
                        _ => {
                            self.extract_from_nodes(children, indent, ancestors, current_href.clone());
                        }
                    }
                }
                DomNode::Text(text) => {
                    let trimmed = text.trim();
                    if !trimmed.is_empty() && !trimmed.starts_with("<!--") {
                        self.text_blocks.push(TextBlock {
                            text: trimmed.to_string(),
                            tag: "text".into(),
                            font_size: 14.0,
                            bold: false,
                            link: current_href.clone(),
                            indent_level: indent,
                        });
                    }
                }
            }
        }
    }

    fn collect_text(&self, nodes: &[DomNode]) -> String {
        let mut parts = Vec::new();
        for node in nodes {
            match node {
                DomNode::Text(text) => {
                    let trimmed = text.trim();
                    if !trimmed.is_empty() {
                        parts.push(trimmed.to_string());
                    }
                }
                DomNode::Element { children, .. } => {
                    let t = self.collect_text(children);
                    if !t.is_empty() {
                        parts.push(t);
                    }
                }
            }
        }
        parts.join(" ")
    }

    fn resolve_href(&self, href: &str) -> String {
        if href.starts_with("http://") || href.starts_with("https://") {
            href.to_string()
        } else if href.starts_with("//") {
            format!("https:{}", href)
        } else if href.starts_with('/') {
            if let Ok(parsed) = url::Url::parse(&self.url) {
                format!("{}://{}{}", parsed.scheme(), parsed.host_str().unwrap_or(""), href)
            } else {
                href.to_string()
            }
        } else {
            href.to_string()
        }
    }
}
