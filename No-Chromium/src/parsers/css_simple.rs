use crate::parsers::html_elements::HtmlTag;
use std::collections::HashMap;

#[derive(Clone, Debug, Default)]
pub struct CssCascade {
    rules: Vec<CssRule>,
}

#[derive(Clone, Debug)]
struct CssRule {
    selector: CssSelector,
    declarations: CssDeclarations,
    specificity: u32,
    order: usize,
}

#[derive(Clone, Debug, Default)]
pub struct CssDeclarations {
    pub background: Option<String>,
    pub background_color: Option<String>,
    pub display: Option<String>,
    pub visibility: Option<String>,
    pub opacity: Option<String>,
    pub color: Option<String>,
    pub font_size: Option<String>,
    pub font_weight: Option<String>,
    pub line_height: Option<String>,
    pub width: Option<String>,
    pub max_width: Option<String>,
    pub margin_bottom: Option<String>,
    pub margin_top: Option<String>,
    pub margin_left: Option<String>,
    pub margin_right: Option<String>,
    pub padding_top: Option<String>,
    pub padding_right: Option<String>,
    pub padding_bottom: Option<String>,
    pub padding_left: Option<String>,
    pub text_align: Option<String>,
    pub text_transform: Option<String>,
    pub border_radius: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct CssElementContext {
    pub tag_name: String,
    pub id: Option<String>,
    pub classes: Vec<String>,
}

impl CssElementContext {
    pub fn from_element(tag: &HtmlTag, attributes: &HashMap<String, String>) -> Self {
        Self {
            tag_name: tag_name(tag),
            id: attributes.get("id").map(|value| value.to_ascii_lowercase()),
            classes: attributes
                .get("class")
                .map(|class_attr| {
                    class_attr
                        .split_whitespace()
                        .map(|class| class.to_ascii_lowercase())
                        .collect()
                })
                .unwrap_or_default(),
        }
    }
}

#[derive(Clone, Debug, Default)]
struct CssSelector {
    ancestors: Vec<SimpleSelector>,
    target: SimpleSelector,
}

#[derive(Clone, Debug, Default)]
struct SimpleSelector {
    tag: Option<String>,
    id: Option<String>,
    classes: Vec<String>,
}

impl CssCascade {
    pub fn from_blocks(blocks: &[String]) -> Self {
        let mut rules = Vec::new();

        for css in blocks {
            parse_css_block(css, &mut rules);
        }

        Self { rules }
    }

    pub fn declarations_for(
        &self,
        tag: &HtmlTag,
        attributes: &HashMap<String, String>,
    ) -> CssDeclarations {
        self.declarations_for_with_ancestors(tag, attributes, &[])
    }

    pub fn declarations_for_with_ancestors(
        &self,
        tag: &HtmlTag,
        attributes: &HashMap<String, String>,
        ancestors: &[CssElementContext],
    ) -> CssDeclarations {
        let mut matched = self
            .rules
            .iter()
            .filter(|rule| rule.selector.matches(tag, attributes, ancestors))
            .collect::<Vec<_>>();

        matched.sort_by_key(|rule| (rule.specificity, rule.order));

        let mut declarations = CssDeclarations::default();
        for rule in matched {
            declarations.merge(&rule.declarations);
        }

        if let Some(style) = attributes.get("style") {
            declarations.merge(&parse_declarations(style));
        }

        declarations
    }
}

impl CssDeclarations {
    fn merge(&mut self, other: &CssDeclarations) {
        assign_if_some(&mut self.background, &other.background);
        assign_if_some(&mut self.background_color, &other.background_color);
        assign_if_some(&mut self.display, &other.display);
        assign_if_some(&mut self.visibility, &other.visibility);
        assign_if_some(&mut self.opacity, &other.opacity);
        assign_if_some(&mut self.color, &other.color);
        assign_if_some(&mut self.font_size, &other.font_size);
        assign_if_some(&mut self.font_weight, &other.font_weight);
        assign_if_some(&mut self.line_height, &other.line_height);
        assign_if_some(&mut self.width, &other.width);
        assign_if_some(&mut self.max_width, &other.max_width);
        assign_if_some(&mut self.margin_bottom, &other.margin_bottom);
        assign_if_some(&mut self.margin_top, &other.margin_top);
        assign_if_some(&mut self.margin_left, &other.margin_left);
        assign_if_some(&mut self.margin_right, &other.margin_right);
        assign_if_some(&mut self.padding_top, &other.padding_top);
        assign_if_some(&mut self.padding_right, &other.padding_right);
        assign_if_some(&mut self.padding_bottom, &other.padding_bottom);
        assign_if_some(&mut self.padding_left, &other.padding_left);
        assign_if_some(&mut self.text_align, &other.text_align);
        assign_if_some(&mut self.text_transform, &other.text_transform);
        assign_if_some(&mut self.border_radius, &other.border_radius);
    }
}

impl CssSelector {
    fn matches(
        &self,
        tag: &HtmlTag,
        attributes: &HashMap<String, String>,
        ancestors: &[CssElementContext],
    ) -> bool {
        if !self.target.matches(tag, attributes) {
            return false;
        }

        let mut search_until = ancestors.len();
        for selector in self.ancestors.iter().rev() {
            let Some(index) = ancestors[..search_until]
                .iter()
                .rposition(|ancestor| selector.matches_context(ancestor))
            else {
                return false;
            };
            search_until = index;
        }

        true
    }

    fn specificity(&self) -> u32 {
        self.target.specificity()
            + self
                .ancestors
                .iter()
                .map(SimpleSelector::specificity)
                .sum::<u32>()
    }
}

impl SimpleSelector {
    fn matches(&self, tag: &HtmlTag, attributes: &HashMap<String, String>) -> bool {
        if let Some(expected_tag) = &self.tag {
            if tag_name(tag) != *expected_tag {
                return false;
            }
        }

        if let Some(expected_id) = &self.id {
            if attributes.get("id") != Some(expected_id) {
                return false;
            }
        }

        if !self.classes.is_empty() {
            let Some(class_attr) = attributes.get("class") else {
                return false;
            };
            let classes = class_attr
                .split_whitespace()
                .map(|class| class.to_ascii_lowercase())
                .collect::<Vec<_>>();
            if !self.classes.iter().all(|class| classes.contains(class)) {
                return false;
            }
        }

        true
    }

    fn matches_context(&self, context: &CssElementContext) -> bool {
        if let Some(expected_tag) = &self.tag {
            if context.tag_name != *expected_tag {
                return false;
            }
        }

        if let Some(expected_id) = &self.id {
            if context.id.as_deref() != Some(expected_id.as_str()) {
                return false;
            }
        }

        self.classes
            .iter()
            .all(|class| context.classes.contains(class))
    }

    fn specificity(&self) -> u32 {
        u32::from(self.id.is_some()) * 100
            + self.classes.len() as u32 * 10
            + u32::from(self.tag.is_some())
    }
}

pub fn parse_px(value: &str, inherited: f32) -> Option<f32> {
    let value = value.trim().to_ascii_lowercase();
    if let Some(px) = value.strip_suffix("px") {
        return px.trim().parse::<f32>().ok();
    }
    if let Some(rem) = value
        .strip_suffix("rem")
        .or_else(|| value.strip_suffix("em"))
    {
        return rem.trim().parse::<f32>().ok().map(|v| v * inherited);
    }
    if let Some(percent) = value.strip_suffix('%') {
        return percent
            .trim()
            .parse::<f32>()
            .ok()
            .map(|v| inherited * v / 100.0);
    }

    match value.as_str() {
        "xx-small" => Some(9.0),
        "x-small" => Some(10.0),
        "small" => Some(13.0),
        "medium" => Some(16.0),
        "large" => Some(18.0),
        "x-large" => Some(24.0),
        "xx-large" => Some(32.0),
        _ => value.parse::<f32>().ok(),
    }
}

pub fn parse_color(value: &str) -> Option<[f32; 4]> {
    let value = value.trim().to_ascii_lowercase();
    if value == "transparent" {
        return Some([1.0, 1.0, 1.0, 0.0]);
    }

    if let Some(hex) = value.strip_prefix('#') {
        return parse_hex_color(hex);
    }

    if value.starts_with("rgb(") || value.starts_with("rgba(") {
        return parse_rgb_color(&value);
    }

    let (r, g, b) = match value.as_str() {
        "black" => (0, 0, 0),
        "white" => (255, 255, 255),
        "red" => (255, 0, 0),
        "green" => (0, 128, 0),
        "blue" => (0, 0, 255),
        "gray" | "grey" => (128, 128, 128),
        "darkgray" | "darkgrey" => (169, 169, 169),
        "lightgray" | "lightgrey" => (211, 211, 211),
        "silver" => (192, 192, 192),
        "navy" => (0, 0, 128),
        "teal" => (0, 128, 128),
        "purple" => (128, 0, 128),
        "orange" => (255, 165, 0),
        "yellow" => (255, 255, 0),
        "maroon" => (128, 0, 0),
        "olive" => (128, 128, 0),
        "lime" => (0, 255, 0),
        "aqua" | "cyan" => (0, 255, 255),
        "fuchsia" | "magenta" => (255, 0, 255),
        _ => return None,
    };

    Some([r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0])
}

fn parse_css_block(css: &str, rules: &mut Vec<CssRule>) {
    let css = strip_comments(css);
    for chunk in css.split('}') {
        let Some((selector_text, declarations_text)) = chunk.split_once('{') else {
            continue;
        };

        if selector_text.trim_start().starts_with('@') {
            continue;
        }

        let declarations = parse_declarations(declarations_text);
        for selector_text in selector_text.split(',') {
            if let Some(selector) = parse_selector(selector_text) {
                let order = rules.len();
                rules.push(CssRule {
                    specificity: selector.specificity(),
                    selector,
                    declarations: declarations.clone(),
                    order,
                });
            }
        }
    }
}

fn parse_selector(selector: &str) -> Option<CssSelector> {
    let selector = selector
        .replace('>', " ")
        .replace('+', " ")
        .replace('~', " ");
    let mut parts = Vec::new();
    for part in selector.split_whitespace() {
        let part = part.trim().trim_matches('*');
        if part.is_empty() || part.contains('[') {
            continue;
        }

        let part = part.split(':').next().unwrap_or(part).trim();
        if let Some(simple) = parse_simple_selector(part) {
            parts.push(simple);
        }
    }

    let target = parts.pop()?;
    Some(CssSelector {
        ancestors: parts,
        target,
    })
}

fn parse_simple_selector(selector: &str) -> Option<SimpleSelector> {
    if selector.is_empty() {
        return None;
    }

    let selector = selector.trim_matches(|ch: char| matches!(ch, '>' | '+' | '~' | '*'));
    if selector.is_empty() {
        return None;
    }

    let mut parsed = SimpleSelector::default();
    let mut buffer = String::new();
    let mut mode = 't';

    for ch in selector.chars().chain(std::iter::once('.')) {
        if ch == '.' || ch == '#' {
            flush_selector_part(&mut parsed, mode, &buffer);
            buffer.clear();
            mode = ch;
        } else {
            buffer.push(ch);
        }
    }

    if parsed.tag.is_none() && parsed.id.is_none() && parsed.classes.is_empty() {
        None
    } else {
        Some(parsed)
    }
}

fn flush_selector_part(parsed: &mut SimpleSelector, mode: char, value: &str) {
    let value = value.trim();
    if value.is_empty() {
        return;
    }

    match mode {
        '#' => parsed.id = Some(value.to_string()),
        '.' => parsed.classes.push(value.to_ascii_lowercase()),
        _ => parsed.tag = Some(value.to_ascii_lowercase()),
    }
}

fn parse_declarations(text: &str) -> CssDeclarations {
    let mut declarations = CssDeclarations::default();
    for declaration in text.split(';') {
        let Some((name, value)) = declaration.split_once(':') else {
            continue;
        };
        let name = name.trim().to_ascii_lowercase();
        let value = value
            .trim()
            .trim_matches('"')
            .trim_matches('\'')
            .trim_end_matches("!important")
            .trim()
            .to_string();

        match name.as_str() {
            "display" => declarations.display = Some(value),
            "background" => declarations.background = Some(value),
            "background-color" => declarations.background_color = Some(value),
            "visibility" => declarations.visibility = Some(value),
            "opacity" => declarations.opacity = Some(value),
            "color" => declarations.color = Some(value),
            "font-size" => declarations.font_size = Some(value),
            "font-weight" => declarations.font_weight = Some(value),
            "line-height" => declarations.line_height = Some(value),
            "width" => declarations.width = Some(value),
            "max-width" => declarations.max_width = Some(value),
            "margin-bottom" => declarations.margin_bottom = Some(value),
            "margin-top" => declarations.margin_top = Some(value),
            "margin-left" => declarations.margin_left = Some(value),
            "margin-right" => declarations.margin_right = Some(value),
            "margin" => {
                declarations.margin_top = box_edge_value(&value, BoxEdge::Top);
                declarations.margin_bottom = box_edge_value(&value, BoxEdge::Bottom);
                declarations.margin_left = box_edge_value(&value, BoxEdge::Left);
                declarations.margin_right = box_edge_value(&value, BoxEdge::Right);
            }
            "padding-top" => declarations.padding_top = Some(value),
            "padding-right" => declarations.padding_right = Some(value),
            "padding-bottom" => declarations.padding_bottom = Some(value),
            "padding-left" => declarations.padding_left = Some(value),
            "padding" => {
                declarations.padding_top = box_edge_value(&value, BoxEdge::Top);
                declarations.padding_right = box_edge_value(&value, BoxEdge::Right);
                declarations.padding_bottom = box_edge_value(&value, BoxEdge::Bottom);
                declarations.padding_left = box_edge_value(&value, BoxEdge::Left);
            }
            "text-align" => declarations.text_align = Some(value),
            "text-transform" => declarations.text_transform = Some(value),
            "border-radius" => declarations.border_radius = Some(value),
            _ => {}
        }
    }
    declarations
}

#[derive(Clone, Copy)]
enum BoxEdge {
    Top,
    Right,
    Bottom,
    Left,
}

fn box_edge_value(value: &str, edge: BoxEdge) -> Option<String> {
    let parts = value.split_whitespace().collect::<Vec<_>>();
    if parts.is_empty() {
        return None;
    }

    let index = match (parts.len(), edge) {
        (1, _) => 0,
        (2, BoxEdge::Top | BoxEdge::Bottom) => 0,
        (2, BoxEdge::Right | BoxEdge::Left) => 1,
        (3, BoxEdge::Top) => 0,
        (3, BoxEdge::Right | BoxEdge::Left) => 1,
        (3, BoxEdge::Bottom) => 2,
        (_, BoxEdge::Top) => 0,
        (_, BoxEdge::Right) => 1,
        (_, BoxEdge::Bottom) => 2,
        (_, BoxEdge::Left) => 3,
    };

    parts.get(index).map(|value| (*value).to_string())
}

fn strip_comments(css: &str) -> String {
    let mut out = String::with_capacity(css.len());
    let mut chars = css.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '/' && chars.peek() == Some(&'*') {
            chars.next();
            while let Some(inner) = chars.next() {
                if inner == '*' && chars.peek() == Some(&'/') {
                    chars.next();
                    break;
                }
            }
        } else {
            out.push(ch);
        }
    }
    out
}

fn parse_hex_color(hex: &str) -> Option<[f32; 4]> {
    let expanded;
    let hex = if hex.len() == 3 || hex.len() == 4 {
        expanded = hex.chars().flat_map(|ch| [ch, ch]).collect::<String>();
        expanded.as_str()
    } else {
        hex
    };

    if hex.len() != 6 && hex.len() != 8 {
        return None;
    }

    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    let a = if hex.len() == 8 {
        u8::from_str_radix(&hex[6..8], 16).ok()? as f32 / 255.0
    } else {
        1.0
    };
    Some([r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a])
}

fn parse_rgb_color(value: &str) -> Option<[f32; 4]> {
    let inner = value
        .trim_start_matches("rgba(")
        .trim_start_matches("rgb(")
        .trim_end_matches(')');
    let normalized = inner.replace('/', " ").replace(',', " ");
    let parts = normalized
        .split_whitespace()
        .map(parse_rgb_component)
        .collect::<Option<Vec<_>>>()?;
    if parts.len() < 3 {
        return None;
    }

    Some([
        parts[0].clamp(0.0, 1.0),
        parts[1].clamp(0.0, 1.0),
        parts[2].clamp(0.0, 1.0),
        parts.get(3).copied().unwrap_or(1.0).clamp(0.0, 1.0),
    ])
}

fn parse_rgb_component(part: &str) -> Option<f32> {
    let part = part.trim();
    if let Some(percent) = part.strip_suffix('%') {
        return percent
            .trim()
            .parse::<f32>()
            .ok()
            .map(|value| value / 100.0);
    }

    let value = part.parse::<f32>().ok()?;
    if value <= 1.0 {
        Some(value)
    } else {
        Some(value / 255.0)
    }
}

fn assign_if_some(target: &mut Option<String>, source: &Option<String>) {
    if let Some(value) = source {
        *target = Some(value.clone());
    }
}

fn tag_name(tag: &HtmlTag) -> String {
    match tag {
        HtmlTag::Custom(name) => name.to_ascii_lowercase(),
        other => format!("{:?}", other).to_ascii_lowercase(),
    }
}
