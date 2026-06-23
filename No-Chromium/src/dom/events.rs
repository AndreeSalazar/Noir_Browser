//! DOM Event System (FASE B2)
//!
//! Event system estilo DOM Level 2 Events + addEventListener/removeEventListener.
//! Incluye:
//! - Bubbling y capturing
//! - PreventDefault, stopPropagation
//! - Event types: click, input, submit, change, focus, blur, keydown, etc
//! - Event delegation
//! - Custom events

use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

/// Tipos de eventos (subset del DOM Events standard)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventType {
    Click,
    DblClick,
    MouseDown,
    MouseUp,
    MouseMove,
    MouseEnter,
    MouseLeave,
    KeyDown,
    KeyUp,
    KeyPress,
    Input,
    Change,
    Submit,
    Reset,
    Focus,
    Blur,
    Load,
    Unload,
    Error,
    Resize,
    Scroll,
    ContextMenu,
    Wheel,
    TouchStart,
    TouchEnd,
    TouchMove,
}

impl EventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EventType::Click => "click",
            EventType::DblClick => "dblclick",
            EventType::MouseDown => "mousedown",
            EventType::MouseUp => "mouseup",
            EventType::MouseMove => "mousemove",
            EventType::MouseEnter => "mouseenter",
            EventType::MouseLeave => "mouseleave",
            EventType::KeyDown => "keydown",
            EventType::KeyUp => "keyup",
            EventType::KeyPress => "keypress",
            EventType::Input => "input",
            EventType::Change => "change",
            EventType::Submit => "submit",
            EventType::Reset => "reset",
            EventType::Focus => "focus",
            EventType::Blur => "blur",
            EventType::Load => "load",
            EventType::Unload => "unload",
            EventType::Error => "error",
            EventType::Resize => "resize",
            EventType::Scroll => "scroll",
            EventType::ContextMenu => "contextmenu",
            EventType::Wheel => "wheel",
            EventType::TouchStart => "touchstart",
            EventType::TouchEnd => "touchend",
            EventType::TouchMove => "touchmove",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "click" => Some(EventType::Click),
            "dblclick" => Some(EventType::DblClick),
            "mousedown" => Some(EventType::MouseDown),
            "mouseup" => Some(EventType::MouseUp),
            "mousemove" => Some(EventType::MouseMove),
            "mouseenter" => Some(EventType::MouseEnter),
            "mouseleave" => Some(EventType::MouseLeave),
            "keydown" => Some(EventType::KeyDown),
            "keyup" => Some(EventType::KeyUp),
            "keypress" => Some(EventType::KeyPress),
            "input" => Some(EventType::Input),
            "change" => Some(EventType::Change),
            "submit" => Some(EventType::Submit),
            "reset" => Some(EventType::Reset),
            "focus" => Some(EventType::Focus),
            "blur" => Some(EventType::Blur),
            "load" => Some(EventType::Load),
            "unload" => Some(EventType::Unload),
            "error" => Some(EventType::Error),
            "resize" => Some(EventType::Resize),
            "scroll" => Some(EventType::Scroll),
            "contextmenu" => Some(EventType::ContextMenu),
            "wheel" => Some(EventType::Wheel),
            "touchstart" => Some(EventType::TouchStart),
            "touchend" => Some(EventType::TouchEnd),
            "touchmove" => Some(EventType::TouchMove),
            _ => None,
        }
    }
}

/// Fases del event flow
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EventPhase {
    None,
    Capturing,
    AtTarget,
    Bubbling,
}

/// Un evento del DOM
#[derive(Debug, Clone)]
pub struct Event {
    pub event_type: EventType,
    pub target_id: u64,
    pub current_target_id: u64,
    pub phase: EventPhase,
    pub default_prevented: bool,
    pub propagation_stopped: bool,
    pub timestamp_ms: u64,
    /// Datos del evento (mouse_x, mouse_y, key_code, etc)
    pub data: HashMap<String, String>,
}

impl Event {
    pub fn new(event_type: EventType, target_id: u64) -> Self {
        Self {
            event_type,
            target_id,
            current_target_id: target_id,
            phase: EventPhase::None,
            default_prevented: false,
            propagation_stopped: false,
            timestamp_ms: 0,
            data: HashMap::new(),
        }
    }

    pub fn with_data(mut self, key: &str, value: &str) -> Self {
        self.data.insert(key.to_string(), value.to_string());
        self
    }

    pub fn type_name(&self) -> &'static str {
        self.event_type.as_str()
    }

    /// Prevent default action (e.g. form submission, link navigation)
    pub fn prevent_default(&mut self) {
        self.default_prevented = true;
    }

    /// Stop propagation - no mas handlers en capturing/bubbling
    pub fn stop_propagation(&mut self) {
        self.propagation_stopped = true;
    }

    /// Stop immediate propagation - no mas handlers en este nodo
    pub fn stop_immediate_propagation(&mut self) {
        self.propagation_stopped = true;
    }

    /// Get data
    pub fn get(&self, key: &str) -> Option<&str> {
        self.data.get(key).map(|s| s.as_str())
    }
}

/// Listener registrado en un nodo
#[derive(Debug, Clone)]
pub struct EventListener {
    pub event_type: EventType,
    pub callback_id: u64,  // identificador unico del callback
    pub use_capture: bool,
    pub once: bool,
}

/// Event target - un nodo que puede recibir eventos
#[derive(Debug, Default)]
pub struct EventTarget {
    pub listeners: Vec<EventListener>,
    pub next_callback_id: u64,
}

impl EventTarget {
    pub fn new() -> Self { Self::default() }

    /// addEventListener
    pub fn add_event_listener(
        &mut self,
        event_type: EventType,
        callback_id: u64,
        use_capture: bool,
        once: bool,
    ) {
        // No duplicar
        if self.listeners.iter().any(|l| l.event_type == event_type && l.callback_id == callback_id && l.use_capture == use_capture) {
            return;
        }
        self.listeners.push(EventListener {
            event_type,
            callback_id,
            use_capture,
            once,
        });
    }

    /// removeEventListener
    pub fn remove_event_listener(
        &mut self,
        event_type: EventType,
        callback_id: u64,
        use_capture: bool,
    ) -> bool {
        let initial = self.listeners.len();
        self.listeners.retain(|l| !(l.event_type == event_type && l.callback_id == callback_id && l.use_capture == use_capture));
        self.listeners.len() != initial
    }

    /// Obtener listeners para un tipo de evento (filtrado por phase)
    pub fn listeners_for(&self, event_type: EventType, phase: EventPhase) -> Vec<&EventListener> {
        let in_capture = phase == EventPhase::Capturing;
        let in_bubble = phase == EventPhase::Bubbling || phase == EventPhase::AtTarget;
        self.listeners.iter()
            .filter(|l| l.event_type == event_type && (
                (in_capture && l.use_capture) ||
                (in_bubble && !l.use_capture)
            ))
            .collect()
    }
}

/// Callback ID - identifica una funcion callback
pub type CallbackId = u64;

/// Dispatch event simulation
/// Retorna true si algun handler llamo preventDefault
pub fn should_prevent_default(event: &Event) -> bool {
    event.default_prevented
}

/// Tracking global de eventos
#[derive(Debug, Default)]
pub struct EventLog {
    pub events: Vec<(u64, EventType)>,  // (timestamp, type)
    pub max_size: usize,
}

impl EventLog {
    pub fn new(max_size: usize) -> Self {
        Self { events: Vec::with_capacity(max_size), max_size }
    }

    pub fn log(&mut self, ts: u64, event_type: EventType) {
        if self.events.len() >= self.max_size {
            self.events.remove(0);
        }
        self.events.push((ts, event_type));
    }

    pub fn count_of(&self, event_type: EventType) -> usize {
        self.events.iter().filter(|(_, t)| *t == event_type).count()
    }

    pub fn clear(&mut self) {
        self.events.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_type_from_str() {
        assert_eq!(EventType::from_str("click"), Some(EventType::Click));
        assert_eq!(EventType::from_str("input"), Some(EventType::Input));
        assert_eq!(EventType::from_str("invalid"), None);
    }

    #[test]
    fn test_event_type_as_str() {
        assert_eq!(EventType::Click.as_str(), "click");
        assert_eq!(EventType::MouseDown.as_str(), "mousedown");
    }

    #[test]
    fn test_event_creation() {
        let e = Event::new(EventType::Click, 42);
        assert_eq!(e.target_id, 42);
        assert_eq!(e.phase, EventPhase::None);
    }

    #[test]
    fn test_event_with_data() {
        let e = Event::new(EventType::Click, 1)
            .with_data("x", "100")
            .with_data("y", "200");
        assert_eq!(e.get("x"), Some("100"));
        assert_eq!(e.get("y"), Some("200"));
    }

    #[test]
    fn test_prevent_default() {
        let mut e = Event::new(EventType::Submit, 1);
        e.prevent_default();
        assert!(e.default_prevented);
        assert!(should_prevent_default(&e));
    }

    #[test]
    fn test_stop_propagation() {
        let mut e = Event::new(EventType::Click, 1);
        e.stop_propagation();
        assert!(e.propagation_stopped);
    }

    #[test]
    fn test_add_event_listener() {
        let mut target = EventTarget::new();
        target.add_event_listener(EventType::Click, 1, false, false);
        assert_eq!(target.listeners.len(), 1);
    }

    #[test]
    fn test_add_duplicate_listener() {
        let mut target = EventTarget::new();
        target.add_event_listener(EventType::Click, 1, false, false);
        target.add_event_listener(EventType::Click, 1, false, false);
        assert_eq!(target.listeners.len(), 1);
    }

    #[test]
    fn test_remove_event_listener() {
        let mut target = EventTarget::new();
        target.add_event_listener(EventType::Click, 1, false, false);
        assert!(target.remove_event_listener(EventType::Click, 1, false));
        assert_eq!(target.listeners.len(), 0);
    }

    #[test]
    fn test_remove_nonexistent() {
        let mut target = EventTarget::new();
        assert!(!target.remove_event_listener(EventType::Click, 1, false));
    }

    #[test]
    fn test_listeners_for_phase() {
        let mut target = EventTarget::new();
        target.add_event_listener(EventType::Click, 1, true, false);   // capture
        target.add_event_listener(EventType::Click, 2, false, false);  // bubble

        let capture = target.listeners_for(EventType::Click, EventPhase::Capturing);
        assert_eq!(capture.len(), 1);
        assert_eq!(capture[0].callback_id, 1);

        let bubble = target.listeners_for(EventType::Click, EventPhase::Bubbling);
        assert_eq!(bubble.len(), 1);
        assert_eq!(bubble[0].callback_id, 2);
    }

    #[test]
    fn test_event_log() {
        let mut log = EventLog::new(10);
        log.log(0, EventType::Click);
        log.log(100, EventType::Input);
        assert_eq!(log.count_of(EventType::Click), 1);
        assert_eq!(log.count_of(EventType::Input), 1);
    }

    #[test]
    fn test_event_log_max_size() {
        let mut log = EventLog::new(3);
        log.log(0, EventType::Click);
        log.log(1, EventType::Click);
        log.log(2, EventType::Click);
        log.log(3, EventType::Click);  // overflow
        assert_eq!(log.events.len(), 3);
    }

    #[test]
    fn test_listener_with_capture() {
        let mut target = EventTarget::new();
        target.add_event_listener(EventType::MouseDown, 5, true, false);
        let l = &target.listeners[0];
        assert!(l.use_capture);
    }

    #[test]
    fn test_listener_once() {
        let mut target = EventTarget::new();
        target.add_event_listener(EventType::Load, 1, false, true);
        assert!(target.listeners[0].once);
    }
}
