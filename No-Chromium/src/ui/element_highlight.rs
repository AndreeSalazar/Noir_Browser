//! Element Highlighter - Resalta elementos interactivos en la página
//!
//! Hace visible qué se puede clickear con overlays de colores.

use std::collections::HashMap;
use std::time::Instant;

use super::click_feedback::{CursorType, InteractiveRole};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HighlightCategory {
    Primary,    // Buttons principales - azul
    Link,       // Links - cyan
    Input,      // Inputs - amarillo
    Media,      // Video/Audio - verde
    Navigation, // Nav links - púrpura
    Form,       // Form controls - naranja
    Disabled,   // Disabled elements - gris
    Other,      // Otros - blanco translúcido
}

impl HighlightCategory {
    pub fn from_role(role: InteractiveRole) -> Self {
        match role {
            InteractiveRole::Button => Self::Primary,
            InteractiveRole::Link => Self::Navigation,
            InteractiveRole::Input | InteractiveRole::Textarea | InteractiveRole::Select => Self::Input,
            InteractiveRole::Checkbox | InteractiveRole::Radio => Self::Form,
            InteractiveRole::Tab | InteractiveRole::MenuItem => Self::Navigation,
            InteractiveRole::Slider => Self::Form,
            InteractiveRole::Other => Self::Other,
        }
    }

    pub fn color(&self) -> u32 {
        match self {
            Self::Primary => 0x5599FF,    // blue
            Self::Link => 0x00CCCC,       // cyan
            Self::Input => 0xFFDD55,      // yellow
            Self::Media => 0x55DD55,      // green
            Self::Navigation => 0xCC55FF, // purple
            Self::Form => 0xFF8855,       // orange
            Self::Disabled => 0x888888,   // gray
            Self::Other => 0xCCCCCC,      // light gray
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Primary => "button",
            Self::Link => "link",
            Self::Input => "input",
            Self::Media => "media",
            Self::Navigation => "nav",
            Self::Form => "form",
            Self::Disabled => "disabled",
            Self::Other => "other",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Highlight {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub category: HighlightCategory,
    pub label: String,
    pub visible: bool,
    pub pressed_at: Option<Instant>,
    pub hover_intensity: f32,
}

impl Highlight {
    pub fn new(x: i32, y: i32, width: u32, height: u32, category: HighlightCategory, label: &str) -> Self {
        Self {
            x, y, width, height,
            category,
            label: label.to_string(),
            visible: true,
            pressed_at: None,
            hover_intensity: 0.0,
        }
    }

    pub fn contains(&self, px: i32, py: i32) -> bool {
        px >= self.x && px < self.x + self.width as i32 &&
        py >= self.y && py < self.y + self.height as i32
    }

    pub fn is_pressed(&self) -> bool {
        self.pressed_at.is_some()
    }

    pub fn press(&mut self) {
        self.pressed_at = Some(Instant::now());
    }

    pub fn release(&mut self) {
        self.pressed_at = None;
    }

    pub fn set_hover(&mut self, hovering: bool) {
        self.hover_intensity = if hovering { 1.0 } else { 0.0 };
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HighlightMode {
    Off,        // No highlights
    On,         // Todos los elementos
    Interactive, // Solo interactivos
    Hover,      // Solo el hovered
    Selection,  // Selection mode (como devtools)
}

impl HighlightMode {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "on" | "all" => Self::On,
            "interactive" => Self::Interactive,
            "hover" => Self::Hover,
            "selection" | "select" => Self::Selection,
            _ => Self::Off,
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::On => "on",
            Self::Interactive => "interactive",
            Self::Hover => "hover",
            Self::Selection => "selection",
        }
    }
}

pub struct ElementHighlighter {
    pub mode: HighlightMode,
    pub highlights: Vec<Highlight>,
    pub by_id: HashMap<String, usize>,
    pub hovered: Option<usize>,
    pub pressed: Option<usize>,
    pub show_labels: bool,
    pub outline_thickness: u32,
    pub outline_alpha: u8,
    pub fill_alpha: u8,
    pub pulse_enabled: bool,
    pub pulse_speed_ms: u32,
}

impl ElementHighlighter {
    pub fn new() -> Self {
        Self {
            mode: HighlightMode::Off,
            highlights: Vec::new(),
            by_id: HashMap::new(),
            hovered: None,
            pressed: None,
            show_labels: true,
            outline_thickness: 2,
            outline_alpha: 200,
            fill_alpha: 30,
            pulse_enabled: true,
            pulse_speed_ms: 1500,
        }
    }

    pub fn set_mode(&mut self, mode: HighlightMode) {
        self.mode = mode;
        if matches!(mode, HighlightMode::Off) {
            self.highlights.clear();
            self.by_id.clear();
            self.hovered = None;
        }
    }

    pub fn add(&mut self, h: Highlight) -> usize {
        let id = h.label.clone();
        let idx = self.highlights.len();
        self.highlights.push(h);
        self.by_id.insert(id, idx);
        idx
    }

    pub fn remove(&mut self, idx: usize) {
        if idx < self.highlights.len() {
            self.highlights.remove(idx);
            self.by_id.clear();
            for (i, h) in self.highlights.iter().enumerate() {
                self.by_id.insert(h.label.clone(), i);
            }
        }
    }

    pub fn clear(&mut self) {
        self.highlights.clear();
        self.by_id.clear();
        self.hovered = None;
        self.pressed = None;
    }

    pub fn count(&self) -> usize {
        self.highlights.len()
    }

    pub fn get(&self, idx: usize) -> Option<&Highlight> {
        self.highlights.get(idx)
    }

    pub fn get_mut(&mut self, idx: usize) -> Option<&mut Highlight> {
        self.highlights.get_mut(idx)
    }

    pub fn find_at(&self, x: i32, y: i32) -> Option<usize> {
        for (i, h) in self.highlights.iter().enumerate() {
            if h.contains(x, y) {
                return Some(i);
            }
        }
        None
    }

    pub fn update_hover(&mut self, x: i32, y: i32) {
        // Limpiar hover previo
        if let Some(prev) = self.hovered {
            if let Some(h) = self.highlights.get_mut(prev) {
                h.set_hover(false);
            }
        }
        let new_hover = if matches!(self.mode, HighlightMode::On | HighlightMode::Interactive | HighlightMode::Hover) {
            self.find_at(x, y)
        } else {
            None
        };
        if let Some(idx) = new_hover {
            if let Some(h) = self.highlights.get_mut(idx) {
                h.set_hover(true);
            }
        }
        self.hovered = new_hover;
    }

    pub fn press(&mut self, x: i32, y: i32) -> bool {
        if let Some(idx) = self.find_at(x, y) {
            if let Some(h) = self.highlights.get_mut(idx) {
                h.press();
            }
            self.pressed = Some(idx);
            true
        } else {
            false
        }
    }

    pub fn release(&mut self) {
        if let Some(idx) = self.pressed {
            if let Some(h) = self.highlights.get_mut(idx) {
                h.release();
            }
        }
        self.pressed = None;
    }

    pub fn visible_highlights(&self) -> impl Iterator<Item = &Highlight> {
        self.highlights.iter().filter(|h| h.visible)
    }

    pub fn count_by_category(&self, cat: HighlightCategory) -> usize {
        self.highlights.iter().filter(|h| h.category == cat).count()
    }

    pub fn categories_present(&self) -> Vec<HighlightCategory> {
        let mut cats: Vec<HighlightCategory> = self.highlights.iter().map(|h| h.category).collect();
        cats.sort_by_key(|c| *c as u32);
        cats.dedup();
        cats
    }
}

impl Default for ElementHighlighter {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_from_role() {
        assert_eq!(HighlightCategory::from_role(InteractiveRole::Button), HighlightCategory::Primary);
        assert_eq!(HighlightCategory::from_role(InteractiveRole::Link), HighlightCategory::Navigation);
        assert_eq!(HighlightCategory::from_role(InteractiveRole::Input), HighlightCategory::Input);
        assert_eq!(HighlightCategory::from_role(InteractiveRole::Other), HighlightCategory::Other);
    }

    #[test]
    fn test_category_color() {
        assert_eq!(HighlightCategory::Primary.color(), 0x5599FF);
        assert_eq!(HighlightCategory::Link.color(), 0x00CCCC);
    }

    #[test]
    fn test_category_label() {
        assert_eq!(HighlightCategory::Primary.label(), "button");
        assert_eq!(HighlightCategory::Link.label(), "link");
    }

    #[test]
    fn test_highlight_new() {
        let h = Highlight::new(0, 0, 100, 50, HighlightCategory::Primary, "btn");
        assert_eq!(h.x, 0);
        assert!(h.visible);
    }

    #[test]
    fn test_highlight_contains() {
        let h = Highlight::new(10, 20, 100, 30, HighlightCategory::Primary, "x");
        assert!(h.contains(50, 30));
        assert!(!h.contains(5, 30));
    }

    #[test]
    fn test_highlight_press_release() {
        let mut h = Highlight::new(0, 0, 100, 50, HighlightCategory::Primary, "x");
        assert!(!h.is_pressed());
        h.press();
        assert!(h.is_pressed());
        h.release();
        assert!(!h.is_pressed());
    }

    #[test]
    fn test_highlight_hover() {
        let mut h = Highlight::new(0, 0, 100, 50, HighlightCategory::Primary, "x");
        h.set_hover(true);
        assert_eq!(h.hover_intensity, 1.0);
        h.set_hover(false);
        assert_eq!(h.hover_intensity, 0.0);
    }

    #[test]
    fn test_mode_from_str() {
        assert_eq!(HighlightMode::from_str("on"), HighlightMode::On);
        assert_eq!(HighlightMode::from_str("off"), HighlightMode::Off);
        assert_eq!(HighlightMode::from_str("interactive"), HighlightMode::Interactive);
        assert_eq!(HighlightMode::from_str("selection"), HighlightMode::Selection);
    }

    #[test]
    fn test_mode_to_str() {
        assert_eq!(HighlightMode::On.to_str(), "on");
        assert_eq!(HighlightMode::Selection.to_str(), "selection");
    }

    #[test]
    fn test_highlighter_new() {
        let h = ElementHighlighter::new();
        assert_eq!(h.mode, HighlightMode::Off);
        assert_eq!(h.count(), 0);
    }

    #[test]
    fn test_highlighter_add() {
        let mut h = ElementHighlighter::new();
        let idx = h.add(Highlight::new(0, 0, 100, 50, HighlightCategory::Primary, "btn1"));
        assert_eq!(idx, 0);
        assert_eq!(h.count(), 1);
    }

    #[test]
    fn test_highlighter_set_mode_off() {
        let mut h = ElementHighlighter::new();
        h.add(Highlight::new(0, 0, 100, 50, HighlightCategory::Primary, "btn"));
        h.set_mode(HighlightMode::Off);
        assert_eq!(h.count(), 0);
    }

    #[test]
    fn test_highlighter_remove() {
        let mut h = ElementHighlighter::new();
        h.add(Highlight::new(0, 0, 100, 50, HighlightCategory::Primary, "a"));
        h.add(Highlight::new(0, 0, 100, 50, HighlightCategory::Primary, "b"));
        h.remove(0);
        assert_eq!(h.count(), 1);
    }

    #[test]
    fn test_highlighter_clear() {
        let mut h = ElementHighlighter::new();
        h.add(Highlight::new(0, 0, 100, 50, HighlightCategory::Primary, "a"));
        h.clear();
        assert_eq!(h.count(), 0);
    }

    #[test]
    fn test_highlighter_find_at() {
        let mut h = ElementHighlighter::new();
        h.add(Highlight::new(0, 0, 100, 50, HighlightCategory::Primary, "a"));
        h.add(Highlight::new(200, 200, 50, 30, HighlightCategory::Primary, "b"));
        assert_eq!(h.find_at(50, 25), Some(0));
        assert_eq!(h.find_at(220, 210), Some(1));
        assert_eq!(h.find_at(500, 500), None);
    }

    #[test]
    fn test_highlighter_update_hover() {
        let mut h = ElementHighlighter::new();
        h.set_mode(HighlightMode::On);
        h.add(Highlight::new(0, 0, 100, 50, HighlightCategory::Primary, "a"));
        h.update_hover(50, 25);
        assert_eq!(h.hovered, Some(0));
    }

    #[test]
    fn test_highlighter_press() {
        let mut h = ElementHighlighter::new();
        h.add(Highlight::new(0, 0, 100, 50, HighlightCategory::Primary, "a"));
        assert!(h.press(50, 25));
        assert!(h.get(0).unwrap().is_pressed());
        h.release();
        assert!(!h.get(0).unwrap().is_pressed());
    }

    #[test]
    fn test_highlighter_press_outside() {
        let mut h = ElementHighlighter::new();
        h.add(Highlight::new(0, 0, 100, 50, HighlightCategory::Primary, "a"));
        assert!(!h.press(500, 500));
    }

    #[test]
    fn test_count_by_category() {
        let mut h = ElementHighlighter::new();
        h.add(Highlight::new(0, 0, 50, 30, HighlightCategory::Primary, "a"));
        h.add(Highlight::new(0, 0, 50, 30, HighlightCategory::Primary, "b"));
        h.add(Highlight::new(0, 0, 50, 30, HighlightCategory::Link, "c"));
        assert_eq!(h.count_by_category(HighlightCategory::Primary), 2);
        assert_eq!(h.count_by_category(HighlightCategory::Link), 1);
    }

    #[test]
    fn test_categories_present() {
        let mut h = ElementHighlighter::new();
        h.add(Highlight::new(0, 0, 50, 30, HighlightCategory::Primary, "a"));
        h.add(Highlight::new(0, 0, 50, 30, HighlightCategory::Link, "b"));
        let cats = h.categories_present();
        assert_eq!(cats.len(), 2);
    }

    #[test]
    fn test_visible_highlights() {
        let mut h = ElementHighlighter::new();
        h.add(Highlight::new(0, 0, 50, 30, HighlightCategory::Primary, "a"));
        let v: Vec<_> = h.visible_highlights().collect();
        assert_eq!(v.len(), 1);
    }

    #[test]
    fn test_highlight_pressed_after_press() {
        let mut h = ElementHighlighter::new();
        let idx = h.add(Highlight::new(0, 0, 100, 50, HighlightCategory::Primary, "a"));
        h.press(50, 25);
        assert_eq!(h.pressed, Some(idx));
    }
}
