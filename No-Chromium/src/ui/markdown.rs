//! Markdown Rendering - Procesa markdown antes de mostrar texto
//!
//! Soporta: **bold**, _italic_, # headers, [links](url), `code`

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MarkdownStyle {
    Plain,
    Bold,
    Italic,
    BoldItalic,
    Code,
    Header1,
    Header2,
    Header3,
    Header4,
    Header5,
    Header6,
    Link,
}

impl MarkdownStyle {
    pub fn is_bold(&self) -> bool {
        matches!(self, Self::Bold | Self::BoldItalic | Self::Header1 | Self::Header2 | Self::Header3 | Self::Header4 | Self::Header5 | Self::Header6)
    }

    pub fn is_italic(&self) -> bool {
        matches!(self, Self::Italic | Self::BoldItalic)
    }

    pub fn is_header(&self) -> bool {
        matches!(self, Self::Header1 | Self::Header2 | Self::Header3 | Self::Header4 | Self::Header5 | Self::Header6)
    }

    pub fn header_level(&self) -> u32 {
        match self {
            Self::Header1 => 1,
            Self::Header2 => 2,
            Self::Header3 => 3,
            Self::Header4 => 4,
            Self::Header5 => 5,
            Self::Header6 => 6,
            _ => 0,
        }
    }

    pub fn prefix(&self) -> &'static str {
        match self {
            Self::Header1 => "# ",
            Self::Header2 => "## ",
            Self::Header3 => "### ",
            Self::Header4 => "#### ",
            Self::Header5 => "##### ",
            Self::Header6 => "###### ",
            _ => "",
        }
    }
}

#[derive(Debug, Clone)]
pub struct MarkdownSegment {
    pub text: String,
    pub style: MarkdownStyle,
    pub link_url: Option<String>,
}

impl MarkdownSegment {
    pub fn new(text: &str, style: MarkdownStyle) -> Self {
        Self {
            text: text.to_string(),
            style,
            link_url: None,
        }
    }

    pub fn with_link(mut self, url: &str) -> Self {
        self.link_url = Some(url.to_string());
        self
    }
}

#[derive(Debug, Clone)]
pub struct MarkdownLine {
    pub segments: Vec<MarkdownSegment>,
    pub is_header: bool,
    pub header_level: u32,
    pub is_list_item: bool,
    pub is_code_block: bool,
    pub is_blockquote: bool,
}

pub struct MarkdownRenderer {
    pub max_line_length: usize,
    pub strip_markdown_chars: bool,
    pub detect_links: bool,
    pub detect_headers: bool,
    pub detect_lists: bool,
    pub detect_code: bool,
    pub detect_bold_italic: bool,
}

impl MarkdownRenderer {
    pub fn new() -> Self {
        Self {
            max_line_length: 200,
            strip_markdown_chars: true,
            detect_links: true,
            detect_headers: true,
            detect_lists: true,
            detect_code: true,
            detect_bold_italic: true,
        }
    }

    /// Parsea texto markdown a líneas con segmentos
    pub fn parse(&self, text: &str) -> Vec<MarkdownLine> {
        let mut lines = Vec::new();
        let mut in_code_block = false;
        for raw_line in text.lines() {
            let line = raw_line.to_string();
            // Code blocks
            if line.trim_start().starts_with("```") {
                in_code_block = !in_code_block;
                let mut ml = MarkdownLine {
                    segments: vec![MarkdownSegment::new("", MarkdownStyle::Code)],
                    is_header: false,
                    header_level: 0,
                    is_list_item: false,
                    is_code_block: true,
                    is_blockquote: false,
                };
                lines.push(ml);
                continue;
            }
            if in_code_block {
                let mut ml = MarkdownLine {
                    segments: vec![MarkdownSegment::new(&line, MarkdownStyle::Code)],
                    is_header: false,
                    header_level: 0,
                    is_list_item: false,
                    is_code_block: true,
                    is_blockquote: false,
                };
                lines.push(ml);
                continue;
            }
            lines.push(self.parse_line(&line));
        }
        lines
    }

    fn parse_line(&self, line: &str) -> MarkdownLine {
        let mut ml = MarkdownLine {
            segments: Vec::new(),
            is_header: false,
            header_level: 0,
            is_list_item: false,
            is_code_block: false,
            is_blockquote: false,
        };
        let trimmed = line.trim_start();
        // Headers
        if self.detect_headers {
            if let Some(header_text) = Self::parse_header(trimmed) {
                let (level, text) = header_text;
                ml.is_header = true;
                ml.header_level = level;
                ml.segments = self.parse_inline(&text);
                // Marcar todos los segmentos como header
                for seg in &mut ml.segments {
                    seg.style = match level {
                        1 => MarkdownStyle::Header1,
                        2 => MarkdownStyle::Header2,
                        3 => MarkdownStyle::Header3,
                        4 => MarkdownStyle::Header4,
                        5 => MarkdownStyle::Header5,
                        _ => MarkdownStyle::Header6,
                    };
                }
                return ml;
            }
        }
        // Blockquote
        if trimmed.starts_with("> ") {
            ml.is_blockquote = true;
            let text = &trimmed[2..];
            ml.segments = self.parse_inline(text);
            return ml;
        }
        // List items
        if self.detect_lists {
            if let Some(item_text) = Self::parse_list_item(trimmed) {
                ml.is_list_item = true;
                ml.segments = self.parse_inline(&item_text);
                return ml;
            }
        }
        // Plain line
        ml.segments = self.parse_inline(line);
        ml
    }

    fn parse_header(line: &str) -> Option<(u32, String)> {
        let level = line.chars().take_while(|&c| c == '#').count();
        if level == 0 || level > 6 { return None; }
        let rest = &line[level..];
        if !rest.starts_with(' ') { return None; }
        Some((level as u32, rest[1..].to_string()))
    }

    fn parse_list_item(line: &str) -> Option<String> {
        if let Some(rest) = line.strip_prefix("- ") {
            return Some(rest.to_string());
        }
        if let Some(rest) = line.strip_prefix("* ") {
            return Some(rest.to_string());
        }
        let mut chars = line.chars().peekable();
        let mut num = String::new();
        while let Some(&c) = chars.peek() {
            if c.is_ascii_digit() { num.push(c); chars.next(); } else { break; }
        }
        if !num.is_empty() {
            let rest: String = chars.collect();
            if let Some(stripped) = rest.strip_prefix(". ") {
                return Some(stripped.to_string());
            }
        }
        None
    }

    fn parse_inline(&self, text: &str) -> Vec<MarkdownSegment> {
        let mut segments = Vec::new();
        let mut current_text = String::new();
        let mut current_style = MarkdownStyle::Plain;
        let mut chars = text.chars().peekable();
        while let Some(c) = chars.next() {
            if self.detect_code && c == '`' {
                // Inline code
                if !current_text.is_empty() {
                    segments.push(MarkdownSegment::new(&current_text, current_style));
                    current_text.clear();
                }
                let mut code = String::new();
                while let Some(&nc) = chars.peek() {
                    if nc == '`' { chars.next(); break; }
                    code.push(nc);
                    chars.next();
                }
                segments.push(MarkdownSegment::new(&code, MarkdownStyle::Code));
                current_style = MarkdownStyle::Plain;
                continue;
            }
            if self.detect_bold_italic && c == '*' {
                if chars.peek() == Some(&'*') {
                    chars.next();
                    if !current_text.is_empty() {
                        segments.push(MarkdownSegment::new(&current_text, current_style));
                        current_text.clear();
                    }
                    let is_triple = chars.peek() == Some(&'*');
                    if is_triple { chars.next(); }
                    let mut content = String::new();
                    let mut closed = false;
                    while let Some(&nc) = chars.peek() {
                        if nc == '*' {
                            if is_triple {
                                let p1 = chars.clone().nth(1);
                                let p2 = chars.clone().nth(2);
                                if p1 == Some('*') && p2 == Some('*') {
                                    chars.next(); chars.next(); chars.next();
                                    closed = true;
                                    break;
                                }
                            } else {
                                if chars.clone().nth(1) == Some('*') {
                                    chars.next(); chars.next();
                                    closed = true;
                                    break;
                                }
                                chars.next();
                                closed = true;
                                break;
                            }
                        }
                        content.push(nc);
                        chars.next();
                    }
                    if closed {
                        let new_style = if is_triple { MarkdownStyle::BoldItalic } else { MarkdownStyle::Bold };
                        // Parsear el contenido recursivamente para nested styles
                        let inner = self.parse_inline(&content);
                        for mut seg in inner {
                            if seg.style == MarkdownStyle::Plain {
                                seg.style = new_style;
                            } else if seg.style == MarkdownStyle::Italic {
                                seg.style = if new_style == MarkdownStyle::Bold { MarkdownStyle::BoldItalic } else { seg.style };
                            } else if seg.style == MarkdownStyle::Bold {
                                seg.style = if new_style == MarkdownStyle::Italic { MarkdownStyle::BoldItalic } else { seg.style };
                            }
                            segments.push(seg);
                        }
                        current_style = MarkdownStyle::Plain;
                    } else {
                        current_text.push_str("**");
                        if is_triple { current_text.push('*'); }
                        current_text.push_str(&content);
                    }
                    continue;
                } else {
                    if !current_text.is_empty() {
                        segments.push(MarkdownSegment::new(&current_text, current_style));
                        current_text.clear();
                    }
                    let mut italic = String::new();
                    let mut closed = false;
                    while let Some(&nc) = chars.peek() {
                        if nc == '*' { chars.next(); closed = true; break; }
                        italic.push(nc);
                        chars.next();
                    }
                    if closed {
                        let new_style = if current_style.is_bold() { MarkdownStyle::BoldItalic } else { MarkdownStyle::Italic };
                        segments.push(MarkdownSegment::new(&italic, new_style));
                        current_style = MarkdownStyle::Plain;
                    } else {
                        current_text.push('*');
                        current_text.push_str(&italic);
                    }
                    continue;
                }
            }
            if self.detect_links && c == '[' {
                // [text](url)
                if !current_text.is_empty() {
                    segments.push(MarkdownSegment::new(&current_text, current_style));
                    current_text.clear();
                }
                let mut link_text = String::new();
                let mut found_close = false;
                while let Some(&nc) = chars.peek() {
                    if nc == ']' { chars.next(); found_close = true; break; }
                    link_text.push(nc);
                    chars.next();
                }
                if found_close && chars.peek() == Some(&'(') {
                    chars.next();
                    let mut url = String::new();
                    while let Some(&nc) = chars.peek() {
                        if nc == ')' { chars.next(); break; }
                        url.push(nc);
                        chars.next();
                    }
                    let mut seg = MarkdownSegment::new(&link_text, MarkdownStyle::Link);
                    seg.link_url = Some(url);
                    segments.push(seg);
                } else {
                    segments.push(MarkdownSegment::new(&link_text, current_style));
                }
                current_style = MarkdownStyle::Plain;
                continue;
            }
            if self.detect_bold_italic && c == '_' {
                // Underscore italic: solo si está después de whitespace o inicio
                let prev_is_alnum = current_text.chars().last()
                    .map(|c| c.is_alphanumeric())
                    .unwrap_or(false);
                // Si está al inicio de línea o después de whitespace, es italic
                // Si está después de alfanum (mid-word), es plain
                if !prev_is_alnum {
                    if !current_text.is_empty() {
                        segments.push(MarkdownSegment::new(&current_text, current_style));
                        current_text.clear();
                    }
                    let mut italic = String::new();
                    let mut closed = false;
                    while let Some(&nc) = chars.peek() {
                        if nc == '_' { chars.next(); closed = true; break; }
                        italic.push(nc);
                        chars.next();
                    }
                    if closed {
                        let new_style = if current_style.is_bold() { MarkdownStyle::BoldItalic } else { MarkdownStyle::Italic };
                        segments.push(MarkdownSegment::new(&italic, new_style));
                        current_style = MarkdownStyle::Plain;
                    } else {
                        // No se cerró, tratarlo como plain
                        current_text.push('_');
                        current_text.push_str(&italic);
                    }
                    continue;
                }
            }
            current_text.push(c);
        }
        if !current_text.is_empty() {
            segments.push(MarkdownSegment::new(&current_text, current_style));
        }
        if segments.is_empty() {
            segments.push(MarkdownSegment::new("", MarkdownStyle::Plain));
        }
        segments
    }

    /// Convierte markdown a plain text (sin chars especiales)
    pub fn to_plain(&self, text: &str) -> String {
        let mut out = String::new();
        for line in self.parse(text) {
            for seg in &line.segments {
                out.push_str(&seg.text);
            }
            out.push('\n');
        }
        out
    }

    /// Obtiene líneas con metadata de estilo
    pub fn render_lines(&self, text: &str) -> Vec<(String, MarkdownStyle)> {
        let mut out = Vec::new();
        for line in self.parse(text) {
            for seg in &line.segments {
                out.push((seg.text.clone(), seg.style));
            }
        }
        out
    }

    /// Verifica si el texto contiene markdown
    pub fn has_markdown(&self, text: &str) -> bool {
        text.contains("**") || text.contains("__") ||
        text.contains('*') || text.contains('_') ||
        text.contains('`') || text.contains("```") ||
        text.contains("# ") || text.contains("## ") || text.contains("### ") ||
        text.contains("[") && text.contains("](")
    }
}

impl Default for MarkdownRenderer {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_style_is_bold() {
        assert!(MarkdownStyle::Bold.is_bold());
        assert!(MarkdownStyle::BoldItalic.is_bold());
        assert!(!MarkdownStyle::Italic.is_bold());
    }

    #[test]
    fn test_style_is_italic() {
        assert!(MarkdownStyle::Italic.is_italic());
        assert!(MarkdownStyle::BoldItalic.is_italic());
        assert!(!MarkdownStyle::Bold.is_italic());
    }

    #[test]
    fn test_style_is_header() {
        assert!(MarkdownStyle::Header1.is_header());
        assert!(!MarkdownStyle::Bold.is_header());
    }

    #[test]
    fn test_style_header_level() {
        assert_eq!(MarkdownStyle::Header1.header_level(), 1);
        assert_eq!(MarkdownStyle::Header3.header_level(), 3);
        assert_eq!(MarkdownStyle::Bold.header_level(), 0);
    }

    #[test]
    fn test_segment_new() {
        let s = MarkdownSegment::new("hello", MarkdownStyle::Plain);
        assert_eq!(s.text, "hello");
    }

    #[test]
    fn test_segment_with_link() {
        let s = MarkdownSegment::new("click", MarkdownStyle::Link).with_link("https://x.com");
        assert_eq!(s.link_url, Some("https://x.com".to_string()));
    }

    #[test]
    fn test_renderer_new() {
        let r = MarkdownRenderer::new();
        assert!(r.detect_links);
        assert_eq!(r.max_line_length, 200);
    }

    #[test]
    fn test_parse_plain() {
        let r = MarkdownRenderer::new();
        let lines = r.parse("Hello world");
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].segments[0].text, "Hello world");
    }

    #[test]
    fn test_parse_header1() {
        let r = MarkdownRenderer::new();
        let lines = r.parse("# Title");
        assert_eq!(lines.len(), 1);
        assert!(lines[0].is_header);
        assert_eq!(lines[0].header_level, 1);
        assert_eq!(lines[0].segments[0].style, MarkdownStyle::Header1);
    }

    #[test]
    fn test_parse_header2() {
        let r = MarkdownRenderer::new();
        let lines = r.parse("## Subtitle");
        assert_eq!(lines[0].header_level, 2);
        assert_eq!(lines[0].segments[0].style, MarkdownStyle::Header2);
    }

    #[test]
    fn test_parse_header6() {
        let r = MarkdownRenderer::new();
        let lines = r.parse("###### Smallest");
        assert_eq!(lines[0].header_level, 6);
    }

    #[test]
    fn test_parse_invalid_header() {
        let r = MarkdownRenderer::new();
        let lines = r.parse("####### 7-hash");
        assert!(!lines[0].is_header);
    }

    #[test]
    fn test_parse_bold() {
        let r = MarkdownRenderer::new();
        let lines = r.parse("**bold text**");
        assert_eq!(lines[0].segments[0].style, MarkdownStyle::Bold);
        assert_eq!(lines[0].segments[0].text, "bold text");
    }

    #[test]
    fn test_parse_italic_star() {
        let r = MarkdownRenderer::new();
        let lines = r.parse("*italic text*");
        assert_eq!(lines[0].segments[0].style, MarkdownStyle::Italic);
    }

    #[test]
    fn test_parse_italic_underscore() {
        let r = MarkdownRenderer::new();
        let lines = r.parse("_italic text_");
        assert_eq!(lines[0].segments[0].style, MarkdownStyle::Italic);
    }

    #[test]
    fn test_parse_bold_italic() {
        let r = MarkdownRenderer::new();
        let lines = r.parse("***bold italic***");
        assert_eq!(lines[0].segments[0].style, MarkdownStyle::BoldItalic);
    }

    #[test]
    fn test_parse_link() {
        let r = MarkdownRenderer::new();
        let lines = r.parse("[click here](https://example.com)");
        let seg = &lines[0].segments[0];
        assert_eq!(seg.text, "click here");
        assert_eq!(seg.style, MarkdownStyle::Link);
        assert_eq!(seg.link_url, Some("https://example.com".to_string()));
    }

    #[test]
    fn test_parse_inline_code() {
        let r = MarkdownRenderer::new();
        let lines = r.parse("Use `code` here");
        let code_seg = lines[0].segments.iter().find(|s| s.style == MarkdownStyle::Code).unwrap();
        assert_eq!(code_seg.text, "code");
    }

    #[test]
    fn test_parse_code_block() {
        let r = MarkdownRenderer::new();
        let lines = r.parse("```\ncode line\n```");
        assert!(lines[0].is_code_block);
        assert!(lines[1].is_code_block);
        assert!(lines[2].is_code_block);
    }

    #[test]
    fn test_parse_list_dash() {
        let r = MarkdownRenderer::new();
        let lines = r.parse("- item one");
        assert!(lines[0].is_list_item);
    }

    #[test]
    fn test_parse_list_star() {
        let r = MarkdownRenderer::new();
        let lines = r.parse("* item one");
        assert!(lines[0].is_list_item);
    }

    #[test]
    fn test_parse_list_numbered() {
        let r = MarkdownRenderer::new();
        let lines = r.parse("1. first");
        assert!(lines[0].is_list_item);
    }

    #[test]
    fn test_parse_blockquote() {
        let r = MarkdownRenderer::new();
        let lines = r.parse("> quoted text");
        assert!(lines[0].is_blockquote);
    }

    #[test]
    fn test_parse_mixed() {
        let r = MarkdownRenderer::new();
        let lines = r.parse("This is **bold** and *italic*");
        assert!(lines[0].segments.iter().any(|s| s.style == MarkdownStyle::Bold));
        assert!(lines[0].segments.iter().any(|s| s.style == MarkdownStyle::Italic));
    }

    #[test]
    fn test_to_plain() {
        let r = MarkdownRenderer::new();
        let plain = r.to_plain("**Hello** _world_");
        assert!(!plain.contains("**"));
        assert!(!plain.contains("_"));
    }

    #[test]
    fn test_has_markdown() {
        let r = MarkdownRenderer::new();
        assert!(r.has_markdown("**bold**"));
        assert!(r.has_markdown("# Header"));
        assert!(!r.has_markdown("plain text"));
    }

    #[test]
    fn test_render_lines() {
        let r = MarkdownRenderer::new();
        let lines = r.render_lines("**bold**");
        assert!(!lines.is_empty());
        assert_eq!(lines[0].0, "bold");
    }

    #[test]
    fn test_parse_header_no_space() {
        let r = MarkdownRenderer::new();
        let lines = r.parse("#Header");
        // Sin espacio no es header válido
        assert!(!lines[0].is_header);
    }

    #[test]
    fn test_strip_markdown_chars() {
        let r = MarkdownRenderer::new();
        let plain = r.to_plain("**hello**");
        assert_eq!(plain.trim(), "hello");
    }

    #[test]
    fn test_multiple_segments() {
        let r = MarkdownRenderer::new();
        let lines = r.parse("a **b** c");
        assert!(lines[0].segments.len() >= 3);
    }

    #[test]
    fn test_nested_styles() {
        let r = MarkdownRenderer::new();
        let lines = r.parse("**bold _italic_ inside**");
        // El italic dentro de bold debería ser bold_italic
        let italic_seg = lines[0].segments.iter().find(|s| s.style == MarkdownStyle::Italic || s.style == MarkdownStyle::BoldItalic);
        assert!(italic_seg.is_some());
    }
}
