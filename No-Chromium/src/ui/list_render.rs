//! List Rendering - Listas ordenadas y no ordenadas con bullets/numbers
//!
//! Tipos: ul, ol, dl (description lists), con indent y markers visuales

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ListType {
    Unordered, // ul
    Ordered,   // ol
    Description, // dl
    Custom(char), // con marker custom
}

impl ListType {
    pub fn from_tag(tag: &str) -> Self {
        match tag.to_lowercase().as_str() {
            "ul" => Self::Unordered,
            "ol" => Self::Ordered,
            "dl" => Self::Description,
            _ => Self::Unordered,
        }
    }

    pub fn default_marker(&self) -> &'static str {
        match self {
            Self::Unordered => "•",
            Self::Ordered => "1.",
            Self::Description => "",
            Self::Custom(_) => "",
        }
    }

    pub fn is_ordered(&self) -> bool {
        matches!(self, Self::Ordered)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BulletStyle {
    Disc,      // •
    Circle,    // ○
    Square,    // ▪
    Decimal,   // 1.
    UpperRoman, // I.
    LowerRoman, // i.
    UpperAlpha, // A.
    LowerAlpha, // a.
    None,
}

impl BulletStyle {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "disc" | "bullet" => Self::Disc,
            "circle" => Self::Circle,
            "square" => Self::Square,
            "decimal" | "number" => Self::Decimal,
            "upper-roman" => Self::UpperRoman,
            "lower-roman" => Self::LowerRoman,
            "upper-alpha" | "upper-latin" => Self::UpperAlpha,
            "lower-alpha" | "lower-latin" => Self::LowerAlpha,
            "none" => Self::None,
            _ => Self::Disc,
        }
    }

    pub fn render(&self, index: usize) -> String {
        match self {
            Self::Disc => "•".to_string(),
            Self::Circle => "○".to_string(),
            Self::Square => "▪".to_string(),
            Self::Decimal => format!("{}.", index + 1),
            Self::UpperRoman => format!("{}.", Self::to_roman(index + 1, true)),
            Self::LowerRoman => format!("{}.", Self::to_roman(index + 1, false)),
            Self::UpperAlpha => format!("{}.", Self::to_alpha(index + 1, true)),
            Self::LowerAlpha => format!("{}.", Self::to_alpha(index + 1, false)),
            Self::None => String::new(),
        }
    }

    fn to_roman(mut n: usize, upper: bool) -> String {
        let vals = [(1000, "M"), (900, "CM"), (500, "D"), (400, "CD"),
                    (100, "C"), (90, "XC"), (50, "L"), (40, "XL"),
                    (10, "X"), (9, "IX"), (5, "V"), (4, "IV"), (1, "I")];
        let mut result = String::new();
        for (v, s) in vals {
            while n >= v {
                result.push_str(s);
                n -= v;
            }
        }
        if !upper { result = result.to_lowercase(); }
        result
    }

    fn to_alpha(n: usize, upper: bool) -> String {
        if n == 0 { return String::new(); }
        let mut result = String::new();
        let mut n = n;
        while n > 0 {
            n -= 1;
            let c = ((n % 26) as u8 + b'A') as char;
            result.insert(0, c);
            n /= 26;
        }
        if !upper { result = result.to_lowercase(); }
        result
    }
}

#[derive(Debug, Clone)]
pub struct ListItem {
    pub text: String,
    pub marker: String,
    pub indent_level: u32,
    pub children: Vec<ListItem>,
}

impl ListItem {
    pub fn new(text: &str, marker: &str) -> Self {
        Self {
            text: text.to_string(),
            marker: marker.to_string(),
            indent_level: 0,
            children: Vec::new(),
        }
    }

    pub fn with_indent(mut self, level: u32) -> Self {
        self.indent_level = level;
        self
    }

    pub fn indent_string(&self, char_per_level: &str) -> String {
        char_per_level.repeat(self.indent_level as usize)
    }
}

#[derive(Debug, Clone)]
pub struct List {
    pub list_type: ListType,
    pub bullet_style: BulletStyle,
    pub items: Vec<ListItem>,
    pub start_index: u32,
}

impl List {
    pub fn new(list_type: ListType) -> Self {
        Self {
            list_type,
            bullet_style: match list_type {
                ListType::Ordered => BulletStyle::Decimal,
                _ => BulletStyle::Disc,
            },
            items: Vec::new(),
            start_index: 1,
        }
    }

    pub fn with_bullet_style(mut self, style: BulletStyle) -> Self {
        self.bullet_style = style;
        self
    }

    pub fn with_start(mut self, start: u32) -> Self {
        self.start_index = start;
        self
    }

    pub fn add(&mut self, item: ListItem) {
        self.items.push(item);
    }

    pub fn count(&self) -> usize {
        self.items.len()
    }

    /// Render a plain text con markers
    pub fn render_text(&self) -> String {
        let mut out = String::new();
        for (i, item) in self.items.iter().enumerate() {
            let marker = if self.list_type.is_ordered() {
                self.bullet_style.render(i + self.start_index as usize - 1)
            } else {
                self.marker_for_item(i, item)
            };
            let indent = "  ".repeat(item.indent_level as usize);
            out.push_str(&format!("{}{} {}\n", indent, marker, item.text));
            for child in &item.children {
                let child_indent = "  ".repeat(child.indent_level as usize);
                out.push_str(&format!("{}  {}\n", child_indent, child.text));
            }
        }
        out
    }

    fn marker_for_item(&self, index: usize, item: &ListItem) -> String {
        if !item.marker.is_empty() {
            item.marker.clone()
        } else {
            self.bullet_style.render(index)
        }
    }
}

pub struct ListRenderer {
    pub indent_char: String,
    pub marker_color: u32,
    pub text_color: u32,
    pub line_height: f32,
    pub max_line_length: usize,
}

impl ListRenderer {
    pub fn new() -> Self {
        Self {
            indent_char: "  ".to_string(),
            marker_color: 0xFF5599FF,
            text_color: 0xFFE0E0E8,
            line_height: 1.4,
            max_line_length: 80,
        }
    }

    pub fn with_marker_color(mut self, color: u32) -> Self {
        self.marker_color = color;
        self
    }

    pub fn with_text_color(mut self, color: u32) -> Self {
        self.text_color = color;
        self
    }

    /// Calcula dimensiones de render
    pub fn calculate_dimensions(&self, list: &List, char_w: f32, char_h: f32) -> (f32, f32) {
        let total_chars = list.render_text().chars().count();
        let max_chars_per_line = self.max_line_length;
        let lines = (total_chars as f32 / max_chars_per_line as f32).ceil().max(list.count() as f32);
        let width = max_chars_per_line as f32 * char_w;
        let height = lines * char_h * self.line_height;
        (width, height)
    }
}

impl Default for ListRenderer {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_type_from_tag() {
        assert_eq!(ListType::from_tag("ul"), ListType::Unordered);
        assert_eq!(ListType::from_tag("ol"), ListType::Ordered);
        assert_eq!(ListType::from_tag("dl"), ListType::Description);
    }

    #[test]
    fn test_list_type_default_marker() {
        assert_eq!(ListType::Unordered.default_marker(), "•");
        assert_eq!(ListType::Ordered.default_marker(), "1.");
        assert_eq!(ListType::Description.default_marker(), "");
    }

    #[test]
    fn test_list_type_is_ordered() {
        assert!(ListType::Ordered.is_ordered());
        assert!(!ListType::Unordered.is_ordered());
    }

    #[test]
    fn test_bullet_style_from_str() {
        assert_eq!(BulletStyle::from_str("disc"), BulletStyle::Disc);
        assert_eq!(BulletStyle::from_str("decimal"), BulletStyle::Decimal);
        assert_eq!(BulletStyle::from_str("circle"), BulletStyle::Circle);
    }

    #[test]
    fn test_bullet_render_disc() {
        assert_eq!(BulletStyle::Disc.render(0), "•");
    }

    #[test]
    fn test_bullet_render_decimal() {
        assert_eq!(BulletStyle::Decimal.render(0), "1.");
        assert_eq!(BulletStyle::Decimal.render(4), "5.");
    }

    #[test]
    fn test_bullet_render_roman() {
        assert_eq!(BulletStyle::UpperRoman.render(0), "I.");
        assert_eq!(BulletStyle::UpperRoman.render(3), "IV.");
        assert_eq!(BulletStyle::UpperRoman.render(9), "X.");
        assert_eq!(BulletStyle::LowerRoman.render(0), "i.");
    }

    #[test]
    fn test_bullet_render_alpha() {
        assert_eq!(BulletStyle::UpperAlpha.render(0), "A.");
        assert_eq!(BulletStyle::UpperAlpha.render(25), "Z.");
        assert_eq!(BulletStyle::LowerAlpha.render(0), "a.");
    }

    #[test]
    fn test_bullet_render_none() {
        assert_eq!(BulletStyle::None.render(0), "");
    }

    #[test]
    fn test_list_item_new() {
        let item = ListItem::new("test", "•");
        assert_eq!(item.text, "test");
    }

    #[test]
    fn test_list_item_with_indent() {
        let item = ListItem::new("test", "•").with_indent(2);
        assert_eq!(item.indent_level, 2);
    }

    #[test]
    fn test_list_item_indent_string() {
        let item = ListItem::new("t", "•").with_indent(2);
        assert_eq!(item.indent_string("  "), "    ");
    }

    #[test]
    fn test_list_new() {
        let l = List::new(ListType::Unordered);
        assert_eq!(l.list_type, ListType::Unordered);
    }

    #[test]
    fn test_list_with_bullet_style() {
        let l = List::new(ListType::Unordered).with_bullet_style(BulletStyle::Square);
        assert_eq!(l.bullet_style, BulletStyle::Square);
    }

    #[test]
    fn test_list_with_start() {
        let l = List::new(ListType::Ordered).with_start(5);
        assert_eq!(l.start_index, 5);
    }

    #[test]
    fn test_list_add() {
        let mut l = List::new(ListType::Unordered);
        l.add(ListItem::new("a", "•"));
        l.add(ListItem::new("b", "•"));
        assert_eq!(l.count(), 2);
    }

    #[test]
    fn test_list_render_unordered() {
        let mut l = List::new(ListType::Unordered);
        l.add(ListItem::new("first", "•"));
        l.add(ListItem::new("second", "•"));
        let out = l.render_text();
        assert!(out.contains("• first"));
        assert!(out.contains("• second"));
    }

    #[test]
    fn test_list_render_ordered() {
        let mut l = List::new(ListType::Ordered);
        l.add(ListItem::new("first", ""));
        l.add(ListItem::new("second", ""));
        let out = l.render_text();
        assert!(out.contains("1. first"));
        assert!(out.contains("2. second"));
    }

    #[test]
    fn test_list_render_start_offset() {
        let mut l = List::new(ListType::Ordered).with_start(5);
        l.add(ListItem::new("a", ""));
        let out = l.render_text();
        assert!(out.contains("5. a"));
    }

    #[test]
    fn test_list_render_roman() {
        let mut l = List::new(ListType::Ordered).with_bullet_style(BulletStyle::UpperRoman);
        l.add(ListItem::new("a", ""));
        l.add(ListItem::new("b", ""));
        l.add(ListItem::new("c", ""));
        let out = l.render_text();
        assert!(out.contains("I. a"));
        assert!(out.contains("II. b"));
        assert!(out.contains("III. c"));
    }

    #[test]
    fn test_renderer_new() {
        let r = ListRenderer::new();
        assert_eq!(r.indent_char, "  ");
    }

    #[test]
    fn test_renderer_with_colors() {
        let r = ListRenderer::new().with_marker_color(0xFF0000).with_text_color(0xFFFFFF);
        assert_eq!(r.marker_color, 0xFF0000);
    }

    #[test]
    fn test_renderer_calculate_dimensions() {
        let r = ListRenderer::new();
        let mut l = List::new(ListType::Unordered);
        l.add(ListItem::new("item", "•"));
        let (w, h) = r.calculate_dimensions(&l, 7.0, 12.0);
        assert!(w > 0.0);
        assert!(h > 0.0);
    }

    #[test]
    fn test_renderer_dimensions_empty() {
        let r = ListRenderer::new();
        let l = List::new(ListType::Unordered);
        let (_w, h) = r.calculate_dimensions(&l, 7.0, 12.0);
        assert_eq!(h, 0.0);
    }

    #[test]
    fn test_list_item_custom_marker() {
        let item = ListItem::new("a", "→");
        assert_eq!(item.marker, "→");
    }

    #[test]
    fn test_list_with_alpha() {
        let mut l = List::new(ListType::Ordered).with_bullet_style(BulletStyle::UpperAlpha);
        l.add(ListItem::new("a", ""));
        l.add(ListItem::new("b", ""));
        let out = l.render_text();
        assert!(out.contains("A. a"));
        assert!(out.contains("B. b"));
    }
}
