//! Click feedback visual - Highlights, cursor types, focus rings
//!
//! Para hacer más claro qué elementos se pueden clickear y qué se clickeó.

use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CursorType {
    Default,
    Pointer,
    Text,
    Wait,
    Help,
    NotAllowed,
    Grab,
    Grabbing,
    Crosshair,
    Move,
    ResizeNs,
    ResizeEw,
    ResizeNesw,
    ResizeNwse,
    ZoomIn,
    ZoomOut,
}

impl CursorType {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "pointer" | "hand" => Self::Pointer,
            "text" | "i-beam" => Self::Text,
            "wait" | "progress" => Self::Wait,
            "help" | "question" => Self::Help,
            "not-allowed" | "no-drop" => Self::NotAllowed,
            "grab" => Self::Grab,
            "grabbing" => Self::Grabbing,
            "crosshair" => Self::Crosshair,
            "move" => Self::Move,
            "ns-resize" | "row-resize" => Self::ResizeNs,
            "ew-resize" | "col-resize" => Self::ResizeEw,
            "nesw-resize" => Self::ResizeNesw,
            "nwse-resize" => Self::ResizeNwse,
            "zoom-in" => Self::ZoomIn,
            "zoom-out" => Self::ZoomOut,
            _ => Self::Default,
        }
    }

    pub fn to_css(&self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Pointer => "pointer",
            Self::Text => "text",
            Self::Wait => "wait",
            Self::Help => "help",
            Self::NotAllowed => "not-allowed",
            Self::Grab => "grab",
            Self::Grabbing => "grabbing",
            Self::Crosshair => "crosshair",
            Self::Move => "move",
            Self::ResizeNs => "ns-resize",
            Self::ResizeEw => "ew-resize",
            Self::ResizeNesw => "nesw-resize",
            Self::ResizeNwse => "nwse-resize",
            Self::ZoomIn => "zoom-in",
            Self::ZoomOut => "zoom-out",
        }
    }

    pub fn is_interactive(&self) -> bool {
        matches!(self, Self::Pointer | Self::Grab | Self::Text | Self::Crosshair | Self::ZoomIn | Self::ZoomOut)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InteractiveRole {
    Button,
    Link,
    Input,
    Checkbox,
    Radio,
    Select,
    Textarea,
    Tab,
    MenuItem,
    Slider,
    Other,
}

impl InteractiveRole {
    pub fn from_tag(tag: &str) -> Self {
        match tag.to_lowercase().as_str() {
            "button" => Self::Button,
            "a" | "area" => Self::Link,
            "input" => Self::Input,
            "select" => Self::Select,
            "textarea" => Self::Textarea,
            "tab" => Self::Tab,
            "option" => Self::MenuItem,
            _ => Self::Other,
        }
    }

    pub fn default_cursor(&self) -> CursorType {
        match self {
            Self::Button | Self::Link | Self::Tab | Self::MenuItem => CursorType::Pointer,
            Self::Input | Self::Textarea => CursorType::Text,
            Self::Select => CursorType::Pointer,
            Self::Checkbox | Self::Radio => CursorType::Pointer,
            Self::Slider => CursorType::Grab,
            Self::Other => CursorType::Default,
        }
    }

    pub fn is_focusable(&self) -> bool {
        !matches!(self, Self::Other)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InteractiveBox {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub role: InteractiveRole,
    pub enabled: bool,
    pub focused: bool,
    pub pressed: bool,
}

impl InteractiveBox {
    pub fn new(x: i32, y: i32, width: u32, height: u32, role: InteractiveRole) -> Self {
        Self {
            x, y, width, height, role,
            enabled: true,
            focused: false,
            pressed: false,
        }
    }

    pub fn contains(&self, px: i32, py: i32) -> bool {
        px >= self.x && px < self.x + self.width as i32 &&
        py >= self.y && py < self.y + self.height as i32
    }

    pub fn center(&self) -> (i32, i32) {
        (self.x + self.width as i32 / 2, self.y + self.height as i32 / 2)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FocusRing {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub color: u32,
    pub thickness: u32,
    pub visible: bool,
    pub animated: bool,
}

impl FocusRing {
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            x, y, width, height,
            color: 0x5599FF, // blue
            thickness: 2,
            visible: false,
            animated: true,
        }
    }

    pub fn with_color(mut self, color: u32) -> Self {
        self.color = color;
        self
    }

    pub fn with_thickness(mut self, t: u32) -> Self {
        self.thickness = t;
        self
    }

    pub fn show(&mut self) {
        self.visible = true;
    }

    pub fn hide(&mut self) {
        self.visible = false;
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ClickEffect {
    None,
    Ripple,
    Highlight,
    Pulse,
    Border,
}

impl ClickEffect {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "ripple" => Self::Ripple,
            "highlight" => Self::Highlight,
            "pulse" => Self::Pulse,
            "border" => Self::Border,
            _ => Self::None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ClickFeedback {
    pub x: i32,
    pub y: i32,
    pub effect: ClickEffect,
    pub started: Instant,
    pub duration_ms: u32,
    pub color: u32,
}

impl ClickFeedback {
    pub fn new(x: i32, y: i32, effect: ClickEffect) -> Self {
        Self {
            x, y, effect,
            started: Instant::now(),
            duration_ms: 300,
            color: 0x5599FF,
        }
    }

    pub fn with_duration(mut self, ms: u32) -> Self {
        self.duration_ms = ms;
        self
    }

    pub fn with_color(mut self, c: u32) -> Self {
        self.color = c;
        self
    }

    pub fn progress(&self) -> f32 {
        let elapsed = self.started.elapsed().as_millis() as u32;
        if self.duration_ms == 0 { return 1.0; }
        (elapsed as f32 / self.duration_ms as f32).clamp(0.0, 1.0)
    }

    pub fn is_finished(&self) -> bool {
        self.started.elapsed().as_millis() as u32 >= self.duration_ms
    }

    pub fn radius(&self) -> i32 {
        let p = self.progress();
        (p * 50.0) as i32
    }

    pub fn alpha(&self) -> u8 {
        let p = self.progress();
        ((1.0 - p) * 255.0) as u8
    }
}

pub struct InteractionState {
    pub hover_boxes: Vec<InteractiveBox>,
    pub focused_box: Option<usize>,
    pub focus_ring: FocusRing,
    pub active_cursor: CursorType,
    pub active_click: Option<ClickFeedback>,
    pub click_history: Vec<ClickFeedback>,
    pub last_click_pos: Option<(i32, i32)>,
    pub last_click_time: Option<Instant>,
}

impl InteractionState {
    pub fn new() -> Self {
        Self {
            hover_boxes: Vec::new(),
            focused_box: None,
            focus_ring: FocusRing::new(0, 0, 0, 0),
            active_cursor: CursorType::Default,
            active_click: None,
            click_history: Vec::new(),
            last_click_pos: None,
            last_click_time: None,
        }
    }

    pub fn register_box(&mut self, b: InteractiveBox) -> usize {
        self.hover_boxes.push(b);
        self.hover_boxes.len() - 1
    }

    pub fn update_hover(&mut self, x: i32, y: i32) {
        let mut new_cursor = CursorType::Default;
        for b in &self.hover_boxes {
            if b.contains(x, y) && b.enabled {
                new_cursor = b.role.default_cursor();
                break;
            }
        }
        self.active_cursor = new_cursor;
    }

    pub fn click(&mut self, x: i32, y: i32, effect: ClickEffect) -> bool {
        self.last_click_pos = Some((x, y));
        self.last_click_time = Some(Instant::now());
        for b in &self.hover_boxes {
            if b.contains(x, y) && b.enabled {
                let feedback = ClickFeedback::new(x, y, effect);
                self.active_click = Some(feedback);
                self.click_history.push(feedback);
                return true;
            }
        }
        false
    }

    pub fn focus(&mut self, index: usize) {
        if let Some(b) = self.hover_boxes.get(index) {
            self.focused_box = Some(index);
            self.focus_ring = FocusRing::new(b.x - 2, b.y - 2, b.width + 4, b.height + 4);
            self.focus_ring.show();
        }
    }

    pub fn blur(&mut self) {
        self.focused_box = None;
        self.focus_ring.hide();
    }

    pub fn tick(&mut self) {
        if let Some(click) = self.active_click {
            if click.is_finished() {
                self.active_click = None;
            }
        }
        // Limpiar history viejo
        self.click_history.retain(|c| !c.is_finished());
    }

    pub fn has_focus(&self) -> bool {
        self.focused_box.is_some() && self.focus_ring.visible
    }

    pub fn active_box(&self) -> Option<&InteractiveBox> {
        self.focused_box.and_then(|i| self.hover_boxes.get(i))
    }
}

impl Default for InteractionState {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_cursor_from_str() {
        assert_eq!(CursorType::from_str("pointer"), CursorType::Pointer);
        assert_eq!(CursorType::from_str("grab"), CursorType::Grab);
        assert_eq!(CursorType::from_str("unknown"), CursorType::Default);
    }

    #[test]
    fn test_cursor_to_css() {
        assert_eq!(CursorType::Pointer.to_css(), "pointer");
        assert_eq!(CursorType::Wait.to_css(), "wait");
    }

    #[test]
    fn test_cursor_is_interactive() {
        assert!(CursorType::Pointer.is_interactive());
        assert!(!CursorType::Default.is_interactive());
        assert!(!CursorType::Wait.is_interactive());
    }

    #[test]
    fn test_role_from_tag() {
        assert_eq!(InteractiveRole::from_tag("button"), InteractiveRole::Button);
        assert_eq!(InteractiveRole::from_tag("a"), InteractiveRole::Link);
        assert_eq!(InteractiveRole::from_tag("input"), InteractiveRole::Input);
        assert_eq!(InteractiveRole::from_tag("textarea"), InteractiveRole::Textarea);
    }

    #[test]
    fn test_role_default_cursor() {
        assert_eq!(InteractiveRole::Button.default_cursor(), CursorType::Pointer);
        assert_eq!(InteractiveRole::Input.default_cursor(), CursorType::Text);
    }

    #[test]
    fn test_role_focusable() {
        assert!(InteractiveRole::Button.is_focusable());
        assert!(!InteractiveRole::Other.is_focusable());
    }

    #[test]
    fn test_box_new() {
        let b = InteractiveBox::new(10, 20, 100, 30, InteractiveRole::Button);
        assert_eq!(b.x, 10);
        assert!(b.enabled);
    }

    #[test]
    fn test_box_contains() {
        let b = InteractiveBox::new(10, 20, 100, 30, InteractiveRole::Button);
        assert!(b.contains(50, 30));
        assert!(!b.contains(5, 30));
        assert!(!b.contains(50, 60));
        assert!(b.contains(10, 20)); // top-left
        assert!(!b.contains(110, 30)); // right edge exclusive
    }

    #[test]
    fn test_box_center() {
        let b = InteractiveBox::new(0, 0, 100, 50, InteractiveRole::Button);
        assert_eq!(b.center(), (50, 25));
    }

    #[test]
    fn test_focus_ring_new() {
        let r = FocusRing::new(0, 0, 100, 50);
        assert!(!r.visible);
        assert_eq!(r.thickness, 2);
    }

    #[test]
    fn test_focus_ring_show_hide() {
        let mut r = FocusRing::new(0, 0, 100, 50);
        r.show();
        assert!(r.visible);
        r.hide();
        assert!(!r.visible);
    }

    #[test]
    fn test_focus_ring_with_color() {
        let r = FocusRing::new(0, 0, 100, 50).with_color(0xFF0000);
        assert_eq!(r.color, 0xFF0000);
    }

    #[test]
    fn test_focus_ring_thickness() {
        let r = FocusRing::new(0, 0, 100, 50).with_thickness(4);
        assert_eq!(r.thickness, 4);
    }

    #[test]
    fn test_click_effect_from_str() {
        assert_eq!(ClickEffect::from_str("ripple"), ClickEffect::Ripple);
        assert_eq!(ClickEffect::from_str("highlight"), ClickEffect::Highlight);
    }

    #[test]
    fn test_click_feedback_new() {
        let c = ClickFeedback::new(50, 50, ClickEffect::Ripple);
        assert_eq!(c.x, 50);
        assert!(!c.is_finished());
    }

    #[test]
    fn test_click_feedback_progress() {
        let c = ClickFeedback::new(0, 0, ClickEffect::Ripple).with_duration(1000);
        let p = c.progress();
        assert!(p >= 0.0 && p <= 1.0);
    }

    #[test]
    fn test_click_feedback_radius() {
        let c = ClickFeedback::new(0, 0, ClickEffect::Ripple);
        assert_eq!(c.radius(), 0);
    }

    #[test]
    fn test_click_feedback_alpha() {
        let c = ClickFeedback::new(0, 0, ClickEffect::Ripple);
        assert_eq!(c.alpha(), 255);
    }

    #[test]
    fn test_click_feedback_with_color() {
        let c = ClickFeedback::new(0, 0, ClickEffect::Ripple).with_color(0x00FF00);
        assert_eq!(c.color, 0x00FF00);
    }

    #[test]
    fn test_interaction_state_new() {
        let s = InteractionState::new();
        assert_eq!(s.active_cursor, CursorType::Default);
        assert!(!s.has_focus());
    }

    #[test]
    fn test_register_box() {
        let mut s = InteractionState::new();
        let b = InteractiveBox::new(0, 0, 100, 50, InteractiveRole::Button);
        let idx = s.register_box(b);
        assert_eq!(idx, 0);
        assert_eq!(s.hover_boxes.len(), 1);
    }

    #[test]
    fn test_update_hover_default() {
        let mut s = InteractionState::new();
        s.update_hover(50, 50);
        assert_eq!(s.active_cursor, CursorType::Default);
    }

    #[test]
    fn test_update_hover_button() {
        let mut s = InteractionState::new();
        s.register_box(InteractiveBox::new(0, 0, 100, 50, InteractiveRole::Button));
        s.update_hover(50, 25);
        assert_eq!(s.active_cursor, CursorType::Pointer);
    }

    #[test]
    fn test_update_hover_text() {
        let mut s = InteractionState::new();
        s.register_box(InteractiveBox::new(0, 0, 100, 50, InteractiveRole::Input));
        s.update_hover(50, 25);
        assert_eq!(s.active_cursor, CursorType::Text);
    }

    #[test]
    fn test_click_on_box() {
        let mut s = InteractionState::new();
        s.register_box(InteractiveBox::new(0, 0, 100, 50, InteractiveRole::Button));
        let clicked = s.click(50, 25, ClickEffect::Ripple);
        assert!(clicked);
        assert!(s.active_click.is_some());
    }

    #[test]
    fn test_click_outside() {
        let mut s = InteractionState::new();
        s.register_box(InteractiveBox::new(0, 0, 100, 50, InteractiveRole::Button));
        let clicked = s.click(200, 200, ClickEffect::Ripple);
        assert!(!clicked);
    }

    #[test]
    fn test_click_disabled() {
        let mut s = InteractionState::new();
        let mut b = InteractiveBox::new(0, 0, 100, 50, InteractiveRole::Button);
        b.enabled = false;
        s.register_box(b);
        let clicked = s.click(50, 25, ClickEffect::Ripple);
        assert!(!clicked);
    }

    #[test]
    fn test_focus_blur() {
        let mut s = InteractionState::new();
        s.register_box(InteractiveBox::new(0, 0, 100, 50, InteractiveRole::Button));
        s.focus(0);
        assert!(s.has_focus());
        s.blur();
        assert!(!s.has_focus());
    }

    #[test]
    fn test_focus_invalid_index() {
        let mut s = InteractionState::new();
        s.focus(99);
        assert!(!s.has_focus());
    }

    #[test]
    fn test_active_box() {
        let mut s = InteractionState::new();
        s.register_box(InteractiveBox::new(0, 0, 100, 50, InteractiveRole::Button));
        s.focus(0);
        let b = s.active_box();
        assert!(b.is_some());
    }

    #[test]
    fn test_tick_clears_finished() {
        let mut s = InteractionState::new();
        s.register_box(InteractiveBox::new(0, 0, 100, 50, InteractiveRole::Button));
        s.click(50, 25, ClickEffect::Ripple);
        thread::sleep(Duration::from_millis(10));
        s.tick();
        // No debería limpiar inmediatamente porque el default es 300ms
        assert!(s.active_click.is_some());
    }

    #[test]
    fn test_last_click_pos() {
        let mut s = InteractionState::new();
        s.register_box(InteractiveBox::new(0, 0, 100, 50, InteractiveRole::Button));
        s.click(30, 20, ClickEffect::Ripple);
        assert_eq!(s.last_click_pos, Some((30, 20)));
    }

    #[test]
    fn test_cursor_grabbing() {
        assert_eq!(CursorType::Grabbing.to_css(), "grabbing");
    }

    #[test]
    fn test_cursor_resize() {
        assert_eq!(CursorType::ResizeNs.to_css(), "ns-resize");
        assert_eq!(CursorType::ResizeEw.to_css(), "ew-resize");
    }
}
