//! Text Shaping - Word breaking, hyphenation, kerning (Chrome-style)
//!
//! Chrome/Edge tienen ICU-based text shaping. Implementamos:
//! - word-break: break-all, keep-all, normal
//! - overflow-wrap: anywhere, break-word, normal
//! - hyphens: auto, manual, none
//! - white-space: normal, nowrap, pre, pre-wrap, pre-line, break-spaces

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WordBreak {
    Normal,
    BreakAll,    // break anywhere
    KeepAll,     // CJK only
    BreakWord,    // legacy alias for overflow-wrap
}

impl WordBreak {
    pub fn from_str(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "break-all" => Self::BreakAll,
            "keep-all" => Self::KeepAll,
            "break-word" => Self::BreakWord,
            _ => Self::Normal,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OverflowWrap {
    Normal,
    BreakWord,  // legacy
    Anywhere,   // new spec
}

impl OverflowWrap {
    pub fn from_str(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "break-word" => Self::BreakWord,
            "anywhere" => Self::Anywhere,
            _ => Self::Normal,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Hyphens {
    None,
    Manual,    // solo con -
    Auto,      // browser añade -
}

impl Hyphens {
    pub fn from_str(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "manual" => Self::Manual,
            "auto" => Self::Auto,
            _ => Self::None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WhiteSpace {
    Normal,
    Nowrap,
    Pre,
    PreWrap,
    PreLine,
    BreakSpaces,
}

impl WhiteSpace {
    pub fn from_str(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "nowrap" => Self::Nowrap,
            "pre" => Self::Pre,
            "pre-wrap" => Self::PreWrap,
            "pre-line" => Self::PreLine,
            "break-spaces" => Self::BreakSpaces,
            _ => Self::Normal,
        }
    }

    pub fn preserves_newlines(&self) -> bool {
        matches!(self, Self::Pre | Self::PreWrap | Self::PreLine)
    }

    pub fn preserves_spaces(&self) -> bool {
        matches!(self, Self::Pre | Self::PreWrap | Self::BreakSpaces | Self::Nowrap)
    }

    pub fn collapses_spaces(&self) -> bool {
        matches!(self, Self::Normal | Self::PreLine | Self::BreakSpaces)
    }
}

pub struct TextShaper {
    pub word_break: WordBreak,
    pub overflow_wrap: OverflowWrap,
    pub hyphens: Hyphens,
    pub white_space: WhiteSpace,
    pub tab_size: u32,
    pub word_spacing: f32,
    pub letter_spacing: f32,
}

impl TextShaper {
    pub fn new() -> Self {
        Self {
            word_break: WordBreak::Normal,
            overflow_wrap: OverflowWrap::Normal,
            hyphens: Hyphens::None,
            white_space: WhiteSpace::Normal,
            tab_size: 8,
            word_spacing: 0.0,
            letter_spacing: 0.0,
        }
    }

    pub fn with_word_break(mut self, wb: WordBreak) -> Self {
        self.word_break = wb;
        self
    }

    pub fn with_overflow_wrap(mut self, ow: OverflowWrap) -> Self {
        self.overflow_wrap = ow;
        self
    }

    pub fn with_hyphens(mut self, h: Hyphens) -> Self {
        self.hyphens = h;
        self
    }

    pub fn with_white_space(mut self, ws: WhiteSpace) -> Self {
        self.white_space = ws;
        self
    }

    /// Cuenta cuántas líneas toma un texto en un ancho dado
    pub fn count_lines(&self, text: &str, max_chars_per_line: usize) -> usize {
        if self.white_space == WhiteSpace::Nowrap {
            return 1;
        }
        let lines: Vec<&str> = text.split('\n').collect();
        let mut total = 0;
        for line in lines {
            let chars = line.chars().count();
            if chars == 0 {
                total += 1;
            } else {
                total += (chars + max_chars_per_line - 1) / max_chars_per_line;
            }
        }
        total
    }

    /// Encuentra puntos de quiebre posibles en una palabra
    pub fn break_points(&self, word: &str) -> Vec<usize> {
        let mut points = Vec::new();
        if word.is_empty() {
            return points;
        }
        let chars: Vec<(usize, char)> = word.char_indices().collect();
        match self.word_break {
            WordBreak::BreakAll => {
                // Puede romper entre cualquier par de caracteres
                for i in 0..chars.len() {
                    points.push(chars[i].0);
                }
            }
            WordBreak::Normal | WordBreak::KeepAll | WordBreak::BreakWord => {
                // Solo en límites de palabras (no aplica dentro)
            }
        }
        points
    }

    /// Aplica hyphenation (insertar -)
    pub fn hyphenate(&self, word: &str) -> Vec<String> {
        if self.hyphens == Hyphens::None {
            return vec![word.to_string()];
        }
        if self.hyphens == Hyphens::Manual {
            // Split en '-' manual
            return word.split('-').map(|s| s.to_string()).collect();
        }
        // Auto: heurística simple (cada 5+ chars)
        if word.len() < 6 {
            return vec![word.to_string()];
        }
        let chars: Vec<char> = word.chars().collect();
        let mut result = Vec::new();
        let mut i = 0;
        while i < chars.len() {
            let end = (i + 5).min(chars.len());
            let piece: String = chars[i..end].iter().collect();
            result.push(piece);
            i = end;
        }
        result
    }

    /// Procesa un texto con white-space rules
    pub fn preprocess(&self, text: &str) -> String {
        match self.white_space {
            WhiteSpace::Normal | WhiteSpace::PreLine => {
                // Colapsa espacios múltiples a uno
                let mut out = String::new();
                let mut last_was_space = false;
                for c in text.chars() {
                    if c == ' ' || c == '\t' {
                        if !last_was_space && !out.is_empty() {
                            out.push(' ');
                            last_was_space = true;
                        }
                    } else if c == '\n' {
                        if self.white_space == WhiteSpace::PreLine {
                            out.push('\n');
                            last_was_space = false;
                        } else if !out.is_empty() {
                            out.push(' ');
                            last_was_space = true;
                        }
                    } else {
                        out.push(c);
                        last_was_space = false;
                    }
                }
                out
            }
            WhiteSpace::Pre | WhiteSpace::PreWrap | WhiteSpace::Nowrap => text.to_string(),
            WhiteSpace::BreakSpaces => {
                text.split('\n').map(|l| l.trim_end().to_string()).collect::<Vec<_>>().join("\n")
            }
        }
    }

    /// Render texto con word wrap y posiblemente hyphenation
    pub fn wrap(&self, text: &str, max_chars: usize) -> Vec<String> {
        let preprocessed = self.preprocess(text);
        if self.white_space == WhiteSpace::Nowrap || self.white_space == WhiteSpace::Pre {
            return preprocessed.split('\n').map(|s| s.to_string()).collect();
        }
        let mut lines = Vec::new();
        for paragraph in preprocessed.split('\n') {
            if paragraph.is_empty() {
                lines.push(String::new());
                continue;
            }
            let words: Vec<&str> = paragraph.split_whitespace().collect();
            let mut current_line = String::new();
            for word in words {
                // Si la palabra sola es más larga que max_chars, dividirla
                if word.chars().count() > max_chars && self.overflow_wrap == OverflowWrap::Anywhere {
                    let chars: Vec<char> = word.chars().collect();
                    for chunk in chars.chunks(max_chars) {
                        let s: String = chunk.iter().collect();
                        if !current_line.is_empty() {
                            lines.push(current_line.clone());
                            current_line.clear();
                        }
                        lines.push(s);
                    }
                    continue;
                }
                if word.chars().count() > max_chars && self.word_break == WordBreak::BreakAll {
                    // Break all
                    let chars: Vec<char> = word.chars().collect();
                    for chunk in chars.chunks(max_chars) {
                        let s: String = chunk.iter().collect();
                        if !current_line.is_empty() {
                            lines.push(current_line.clone());
                            current_line.clear();
                        }
                        lines.push(s);
                    }
                    continue;
                }
                let test = if current_line.is_empty() {
                    word.to_string()
                } else {
                    format!("{} {}", current_line, word)
                };
                if test.chars().count() <= max_chars {
                    current_line = test;
                } else {
                    if !current_line.is_empty() {
                        lines.push(current_line);
                    }
                    current_line = word.to_string();
                }
            }
            if !current_line.is_empty() {
                lines.push(current_line);
            }
        }
        lines
    }
}

impl Default for TextShaper {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_word_break_from_str() {
        assert_eq!(WordBreak::from_str("normal"), WordBreak::Normal);
        assert_eq!(WordBreak::from_str("break-all"), WordBreak::BreakAll);
        assert_eq!(WordBreak::from_str("keep-all"), WordBreak::KeepAll);
    }

    #[test]
    fn test_overflow_wrap_from_str() {
        assert_eq!(OverflowWrap::from_str("anywhere"), OverflowWrap::Anywhere);
        assert_eq!(OverflowWrap::from_str("break-word"), OverflowWrap::BreakWord);
    }

    #[test]
    fn test_hyphens_from_str() {
        assert_eq!(Hyphens::from_str("auto"), Hyphens::Auto);
        assert_eq!(Hyphens::from_str("manual"), Hyphens::Manual);
        assert_eq!(Hyphens::from_str("none"), Hyphens::None);
    }

    #[test]
    fn test_white_space_from_str() {
        assert_eq!(WhiteSpace::from_str("nowrap"), WhiteSpace::Nowrap);
        assert_eq!(WhiteSpace::from_str("pre"), WhiteSpace::Pre);
        assert_eq!(WhiteSpace::from_str("pre-wrap"), WhiteSpace::PreWrap);
    }

    #[test]
    fn test_white_space_flags() {
        assert!(WhiteSpace::Pre.preserves_newlines());
        assert!(WhiteSpace::PreLine.preserves_newlines());
        assert!(!WhiteSpace::Normal.preserves_newlines());
        assert!(WhiteSpace::Pre.preserves_spaces());
        assert!(WhiteSpace::Normal.collapses_spaces());
    }

    #[test]
    fn test_shaper_new() {
        let s = TextShaper::new();
        assert_eq!(s.word_break, WordBreak::Normal);
    }

    #[test]
    fn test_shaper_with() {
        let s = TextShaper::new()
            .with_word_break(WordBreak::BreakAll)
            .with_hyphens(Hyphens::Auto);
        assert_eq!(s.word_break, WordBreak::BreakAll);
        assert_eq!(s.hyphens, Hyphens::Auto);
    }

    #[test]
    fn test_count_lines() {
        let s = TextShaper::new();
        assert_eq!(s.count_lines("hello", 80), 1);
        assert!(s.count_lines("a".repeat(100).as_str(), 10) >= 10);
    }

    #[test]
    fn test_count_lines_nowrap() {
        let s = TextShaper::new().with_white_space(WhiteSpace::Nowrap);
        assert_eq!(s.count_lines("a".repeat(100).as_str(), 10), 1);
    }

    #[test]
    fn test_break_points_normal() {
        let s = TextShaper::new();
        assert!(s.break_points("hello").is_empty());
    }

    #[test]
    fn test_break_points_break_all() {
        let s = TextShaper::new().with_word_break(WordBreak::BreakAll);
        let points = s.break_points("hello");
        assert!(!points.is_empty());
    }

    #[test]
    fn test_hyphenate_none() {
        let s = TextShaper::new();
        assert_eq!(s.hyphenate("hello"), vec!["hello".to_string()]);
    }

    #[test]
    fn test_hyphenate_manual() {
        let s = TextShaper::new().with_hyphens(Hyphens::Manual);
        assert_eq!(s.hyphenate("hello-world"), vec!["hello", "world"]);
    }

    #[test]
    fn test_hyphenate_auto() {
        let s = TextShaper::new().with_hyphens(Hyphens::Auto);
        let parts = s.hyphenate("hello world");
        assert!(parts.len() > 1);
    }

    #[test]
    fn test_hyphenate_auto_short() {
        let s = TextShaper::new().with_hyphens(Hyphens::Auto);
        assert_eq!(s.hyphenate("hi"), vec!["hi".to_string()]);
    }

    #[test]
    fn test_preprocess_normal() {
        let s = TextShaper::new();
        let out = s.preprocess("hello    world");
        assert_eq!(out, "hello world");
    }

    #[test]
    fn test_preprocess_pre() {
        let s = TextShaper::new().with_white_space(WhiteSpace::Pre);
        let out = s.preprocess("hello    world\nfoo");
        assert_eq!(out, "hello    world\nfoo");
    }

    #[test]
    fn test_wrap_simple() {
        let s = TextShaper::new();
        let lines = s.wrap("hello world foo bar", 10);
        assert!(lines.len() >= 2);
    }

    #[test]
    fn test_wrap_nowrap() {
        let s = TextShaper::new().with_white_space(WhiteSpace::Nowrap);
        let lines = s.wrap("a b c d e f g h i j k l m n o p", 5);
        assert_eq!(lines.len(), 1);
    }

    #[test]
    fn test_wrap_with_overflow_anywhere() {
        let s = TextShaper::new().with_overflow_wrap(OverflowWrap::Anywhere);
        let lines = s.wrap("verylongwordwithoutspaces", 5);
        // "verylongwordwithoutspaces" tiene 26 chars
        // con anywhere, divide en chunks de 5
        assert!(lines.len() >= 5);
    }

    #[test]
    fn test_wrap_with_break_all() {
        let s = TextShaper::new().with_word_break(WordBreak::BreakAll);
        let lines = s.wrap("verylongword", 5);
        assert!(lines.len() >= 2);
    }

    #[test]
    fn test_wrap_empty_paragraphs() {
        let s = TextShaper::new().with_white_space(WhiteSpace::PreWrap);
        let lines = s.wrap("hello\n\nworld", 80);
        assert!(lines.len() >= 2);
    }

    #[test]
    fn test_preprocess_newline_normal() {
        let s = TextShaper::new();
        let out = s.preprocess("hello\nworld");
        assert_eq!(out, "hello world");
    }

    #[test]
    fn test_preprocess_newline_pre_line() {
        let s = TextShaper::new().with_white_space(WhiteSpace::PreLine);
        let out = s.preprocess("hello\nworld");
        assert_eq!(out, "hello\nworld");
    }
}
