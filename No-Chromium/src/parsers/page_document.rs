use crate::parsers::dom_tree::{DomNode, parse_html};
use crate::parsers::html_elements::HtmlTag;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct TextBlock {
    pub text: String,
    pub tag: String,
    pub font_size: f32,
    pub bold: bool,
    pub link: Option<String>,
    pub indent_level: u32,
    pub attributes: HashMap<String, String>,
}

#[derive(Clone, Debug)]
pub struct ImageBlock {
    pub src: String,
    pub alt: String,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub lazy: bool,
}

#[derive(Clone, Debug)]
pub struct VideoBlock {
    pub src: String,
    pub poster: Option<String>,
    pub controls: bool,
    pub autoplay: bool,
    pub loop_video: bool,
    pub muted: bool,
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
    pub video_blocks: Vec<VideoBlock>,
    pub links: Vec<LinkInfo>,
    pub style_blocks: Vec<String>,
    pub css_urls: Vec<String>,
    pub viewport_width: Option<f32>,
    pub dom_nodes: Vec<DomNode>,
}

impl PageDocument {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            title: String::new(),
            text_blocks: Vec::new(),
            image_blocks: Vec::new(),
            video_blocks: Vec::new(),
            links: Vec::new(),
            style_blocks: Vec::new(),
            css_urls: Vec::new(),
            viewport_width: None,
            dom_nodes: Vec::new(),
        }
    }

    pub fn from_html(url: &str, html: &str) -> Self {
        let mut doc = PageDocument::new(url);
        doc.extract_style_blocks(html);
        doc.extract_css_links(html);
        doc.extract_viewport(html);
        let nodes = parse_html(html);
        doc.dom_nodes = nodes.clone();
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

    fn extract_viewport(&mut self, html: &str) {
        if let Some(meta_pos) = html.to_lowercase().find("<meta") {
            let tag_area = &html[meta_pos..];
            if let Some(gt) = tag_area.find('>') {
                let tag = &tag_area[..gt + 1].to_lowercase();
                if tag.contains("viewport") {
                    if let Some(content_start) = tag.find("content=\"") {
                        let val_start = content_start + 9;
                        if let Some(val_end) = tag[val_start..].find('"') {
                            let content = &tag[val_start..val_start + val_end];
                            for part in content.split(',') {
                                let part = part.trim();
                                if part.starts_with("width=") {
                                    if let Some(w) = part.strip_prefix("width=") {
                                        if let Ok(v) = w.trim().parse::<f32>() {
                                            self.viewport_width = Some(v);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn extract_css_links(&mut self, html: &str) {
        let mut remaining = html;
        while let Some(pos) = remaining.find("<link") {
            let tag_start = pos;
            if let Some(gt) = remaining[tag_start..].find('>') {
                let tag_content = &remaining[tag_start..tag_start + gt + 1];
                let lower = tag_content.to_lowercase();
                if lower.contains("rel=\"stylesheet\"") || lower.contains("rel='stylesheet'") || lower.contains("rel=stylesheet") {
                    if let Some(href_start) = lower.find("href=\"") {
                        let href_val_start = href_start + 6;
                        if let Some(href_end) = remaining[tag_start + href_val_start..].find('"') {
                            let href = &remaining[tag_start + href_val_start..tag_start + href_val_start + href_end];
                            if !href.is_empty() {
                                if let Ok(resolved) = self.resolve_href_url(href) {
                                    self.css_urls.push(resolved);
                                }
                            }
                        }
                    }
                }
                remaining = &remaining[tag_start + gt + 1..];
            } else {
                break;
            }
        }
    }

    fn resolve_href_url(&self, href: &str) -> Result<String, ()> {
        if href.starts_with("http://") || href.starts_with("https://") {
            Ok(href.to_string())
        } else if href.starts_with("//") {
            Ok(format!("https:{}", href))
        } else if href.starts_with('/') {
            if let Ok(parsed) = url::Url::parse(&self.url) {
                Ok(format!("{}://{}{}", parsed.scheme(), parsed.host_str().unwrap_or(""), href))
            } else {
                Err(())
            }
        } else {
            if let Ok(parsed) = url::Url::parse(&self.url) {
                if let Ok(base) = parsed.join(href) {
                    Ok(base.to_string())
                } else {
                    Err(())
                }
            } else {
                Err(())
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
        let parent_is_block = !ancestors.is_empty() && matches!(
            ancestors.last().map(|s| s.as_str()),
            Some("p") | Some("h1") | Some("h2") | Some("h3") | Some("h4") | Some("h5") | Some("h6")
                | Some("li") | Some("blockquote") | Some("pre") | Some("div")
        );
        for node in nodes {
            match node {
                DomNode::Element {
                    tag,
                    attributes,
                    children,
                } => {
                    // Skip content of script, style, noscript, template tags
                    if matches!(tag, HtmlTag::Script | HtmlTag::Style | HtmlTag::Noscript)
                        || matches!(tag, HtmlTag::Custom(name) if name == "template") {
                        continue;
                    }
                    // Skip inline tags dentro de bloque padre (ya se procesó el texto)
                    let tag_name = Self::tag_name(tag);
                    if parent_is_block && matches!(
                        tag,
                        HtmlTag::B | HtmlTag::Strong | HtmlTag::Em | HtmlTag::I
                            | HtmlTag::Small | HtmlTag::U | HtmlTag::Cite | HtmlTag::Dfn
                            | HtmlTag::Mark | HtmlTag::Q | HtmlTag::S | HtmlTag::Samp
                            | HtmlTag::Var | HtmlTag::Kbd | HtmlTag::Abbr | HtmlTag::Time
                            | HtmlTag::Data | HtmlTag::Del | HtmlTag::Ins | HtmlTag::Sub
                            | HtmlTag::Sup
                    ) {
                        ancestors.push(tag_name);
                        self.extract_from_nodes(children, indent, ancestors, current_href.clone());
                        ancestors.pop();
                        continue;
                    }
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
                                    attributes: attributes.clone(),
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
                                    attributes: attributes.clone(),
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
                                    attributes: attributes.clone(),
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
                                    attributes: attributes.clone(),
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
                                    attributes: attributes.clone(),
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
                                    attributes: attributes.clone(),
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
                                    attributes: attributes.clone(),
                                });
                            }
                        }
                        HtmlTag::Li => {
                            let text = self.collect_text(children);
                            if !text.is_empty() {
                                // Detect ordered list parent
                                let marker = if attributes.get("value").is_some() {
                                    format!("{}.", attributes.get("value").unwrap())
                                } else {
                                    "•".to_string()
                                };
                                self.text_blocks.push(TextBlock {
                                    text: format!("  {} {}", marker, text),
                                    tag: "li".into(),
                                    font_size: 14.0,
                                    bold: false,
                                    link: current_href.clone(),
                                    indent_level: indent + 1,
                                    attributes: attributes.clone(),
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
                                    attributes: attributes.clone(),
                                });
                            }
                        }
                        HtmlTag::Img => {
                            // Try multiple sources: src, data-src, srcset
                            let src = attributes.get("src")
                                .or_else(|| attributes.get("data-src"))
                                .or_else(|| attributes.get("data-original"))
                                .cloned()
                                .or_else(|| {
                                    attributes.get("srcset").and_then(|s| {
                                        // Parse srcset: "url1 1x, url2 2x"
                                        s.split(',').next().and_then(|first| {
                                            first.trim().split_whitespace().next().map(String::from)
                                        })
                                    })
                                });

                            if let Some(src) = src {
                                if !src.starts_with("data:") {
                                    let resolved = self.resolve_href(&src);
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
                                        lazy: attributes.get("loading").map(|v| v == "lazy").unwrap_or(false),
                                    });
                                }
                            }
                            // Still recurse to find nested images
                            self.extract_from_nodes(children, indent, ancestors, current_href.clone());
                        }
                        HtmlTag::Video => {
                            if let Some(src) = attributes.get("src") {
                                let resolved = self.resolve_href(src);
                                let poster = attributes.get("poster").map(|s| self.resolve_href(s));
                                let width = attributes.get("width")
                                    .and_then(|w| w.trim().parse::<f32>().ok());
                                let height = attributes.get("height")
                                    .and_then(|h| h.trim().parse::<f32>().ok());
                                self.video_blocks.push(VideoBlock {
                                    src: resolved,
                                    poster,
                                    controls: attributes.get("controls").is_some(),
                                    autoplay: attributes.get("autoplay").is_some(),
                                    loop_video: attributes.get("loop").is_some(),
                                    muted: attributes.get("muted").is_some(),
                                    width,
                                    height,
                                });
                            }
                        }
                        HtmlTag::Audio => {
                            if let Some(src) = attributes.get("src") {
                                tracing::info!("Audio source: {}", src);
                            }
                        }
                        HtmlTag::Iframe => {
                            // Iframe often used for embedded videos
                            if let Some(src) = attributes.get("src") {
                                let resolved = self.resolve_href(src);
                                let width = attributes.get("width")
                                    .and_then(|w| w.trim().parse::<f32>().ok())
                                    .unwrap_or(560.0);
                                let height = attributes.get("height")
                                    .and_then(|h| h.trim().parse::<f32>().ok())
                                    .unwrap_or(315.0);
                                // Detect video iframes
                                let is_video = resolved.contains("youtube")
                                    || resolved.contains("vimeo")
                                    || resolved.contains("player")
                                    || resolved.contains("embed");
                                if is_video {
                                    self.video_blocks.push(VideoBlock {
                                        src: resolved,
                                        poster: None,
                                        controls: true,
                                        autoplay: false,
                                        loop_video: false,
                                        muted: false,
                                        width: Some(width),
                                        height: Some(height),
                                    });
                                } else {
                                    // Treat as content block
                                    self.text_blocks.push(TextBlock {
                                        text: format!("[iframe: {}]", resolved),
                                        tag: "iframe".into(),
                                        font_size: 12.0,
                                        bold: false,
                                        link: None,
                                        indent_level: indent,
                                        attributes: attributes.clone(),
                                    });
                                }
                            }
                            self.extract_from_nodes(children, indent, ancestors, current_href.clone());
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
                                    attributes: attributes.clone(),
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
                                attributes: HashMap::new(),
                            });
                        }
                        HtmlTag::Input => {
                            let input_type = attributes.get("type").cloned().unwrap_or_else(|| "text".into());
                            let placeholder = attributes.get("placeholder").cloned().unwrap_or_default();
                            let value = attributes.get("value").cloned().unwrap_or_default();
                            let name = attributes.get("name").cloned().unwrap_or_default();
                            let checked = attributes.contains_key("checked");
                            // Render different input types with appropriate symbols
                            let label_text = match input_type.as_str() {
                                "checkbox" => {
                                    let mark = if checked { "[X]" } else { "[ ]" };
                                    let lbl = if !name.is_empty() { format!(" {} ", name) } else { String::new() };
                                    format!("{}{}", mark, lbl)
                                }
                                "radio" => {
                                    let mark = if checked { "(O)" } else { "( )" };
                                    let lbl = if !name.is_empty() { format!(" {} ", name) } else { String::new() };
                                    format!("{}{}", mark, lbl)
                                }
                                "submit" | "button" => {
                                    format!("[ {} ]", if value.is_empty() { "Submit" } else { &value })
                                }
                                "hidden" => String::new(),
                                "file" => format!("[ Choose File ] (no file)"),
                                _ => {
                                    let disp = if !value.is_empty() {
                                        value.as_str()
                                    } else if !placeholder.is_empty() {
                                        placeholder.as_str()
                                    } else {
                                        ""
                                    };
                                    if let Some(_for_id) = attributes.get("id") {
                                        if !name.is_empty() {
                                            format!("{}: [{}]", name, disp)
                                        } else {
                                            format!("[{}]", disp)
                                        }
                                    } else {
                                        format!("[{}]", disp)
                                    }
                                }
                            };
                            if label_text.is_empty() {
                                continue;
                            }
                            self.text_blocks.push(TextBlock {
                                text: label_text,
                                tag: "input".into(),
                                font_size: 14.0,
                                bold: false,
                                link: None,
                                indent_level: indent,
                                attributes: {
                                    let mut a = attributes.clone();
                                    a.insert("__input_type".into(), input_type);
                                    a.insert("__input_value".into(), value);
                                    a.insert("__placeholder".into(), placeholder);
                                    a
                                },
                            });
                        }
                        HtmlTag::Button => {
                            let text = self.collect_text(children);
                            // Use aria-label if no text content
                            let btn_text = if text.is_empty() {
                                attributes.get("aria-label")
                                    .or_else(|| attributes.get("title"))
                                    .cloned()
                                    .unwrap_or_else(|| "Button".into())
                            } else {
                                text
                            };
                            self.text_blocks.push(TextBlock {
                                text: format!("[ {} ]", btn_text),
                                tag: "button".into(),
                                font_size: 14.0,
                                bold: true,
                                link: None,
                                indent_level: indent,
                                attributes: attributes.clone(),
                            });
                        }
                        HtmlTag::Form => {
                            let action = attributes.get("action").cloned().unwrap_or_default();
                            let method = attributes.get("method").cloned().unwrap_or_else(|| "GET".into());
                            let mut form_attrs = attributes.clone();
                            form_attrs.insert("__form_action".into(), action);
                            form_attrs.insert("__form_method".into(), method);
                            self.extract_from_nodes(children, indent, ancestors, current_href.clone());
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
                        HtmlTag::Custom(ref name) if name == "head" => {
                            // Recurse into head to find <title> but skip meta/link
                            self.extract_from_nodes(children, indent, ancestors, current_href.clone());
                        }
                        HtmlTag::Custom(ref name) if name == "meta" || name == "link" => {
                            // Skip meta/link metadata
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
                            attributes: HashMap::new(),
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

    pub fn resolve_href(&self, href: &str) -> String {
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

    pub fn resolve_href_simple(&self, href: &str) -> String {
        self.resolve_href(href)
    }

    fn tag_name(tag: &HtmlTag) -> String {
        match tag {
            HtmlTag::H1 => "h1".to_string(),
            HtmlTag::H2 => "h2".to_string(),
            HtmlTag::H3 => "h3".to_string(),
            HtmlTag::H4 => "h4".to_string(),
            HtmlTag::H5 => "h5".to_string(),
            HtmlTag::H6 => "h6".to_string(),
            HtmlTag::P => "p".to_string(),
            HtmlTag::Li => "li".to_string(),
            HtmlTag::Blockquote => "blockquote".to_string(),
            HtmlTag::Pre => "pre".to_string(),
            HtmlTag::Div => "div".to_string(),
            HtmlTag::B | HtmlTag::Strong => "strong".to_string(),
            HtmlTag::A => "a".to_string(),
            HtmlTag::Em | HtmlTag::I => "em".to_string(),
            _ => "span".to_string(),
        }
    }
}
