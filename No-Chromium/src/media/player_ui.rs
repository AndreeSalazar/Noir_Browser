//! Player UI - Controles de video visibles
//!
//! play/pause, seek bar, volume, fullscreen, time display

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlayerControl {
    Play,
    Pause,
    Stop,
    SeekForward,
    SeekBackward,
    VolumeUp,
    VolumeDown,
    Mute,
    Fullscreen,
    Settings,
    Subtitles,
    PictureInPicture,
}

impl PlayerControl {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "play" | "p" => Some(Self::Play),
            "pause" => Some(Self::Pause),
            "stop" => Some(Self::Stop),
            "fwd" | "forward" | ">" => Some(Self::SeekForward),
            "bwd" | "backward" | "<" => Some(Self::SeekBackward),
            "vol+" | "volumeup" => Some(Self::VolumeUp),
            "vol-" | "volumedown" => Some(Self::VolumeDown),
            "mute" | "m" => Some(Self::Mute),
            "fullscreen" | "fs" | "f" => Some(Self::Fullscreen),
            "settings" => Some(Self::Settings),
            "subs" | "subtitles" | "cc" => Some(Self::Subtitles),
            "pip" => Some(Self::PictureInPicture),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayerUiConfig {
    pub show_controls: bool,
    pub auto_hide_ms: u32,
    pub control_height: u32,
    pub seek_step_seconds: u32,
    pub volume_step: f32,
    pub show_time: bool,
    pub show_volume: bool,
    pub show_settings: bool,
    pub show_subtitles_button: bool,
    pub show_fullscreen: bool,
    pub show_pip: bool,
}

impl Default for PlayerUiConfig {
    fn default() -> Self {
        Self {
            show_controls: true,
            auto_hide_ms: 3000,
            control_height: 50,
            seek_step_seconds: 10,
            volume_step: 0.1,
            show_time: true,
            show_volume: true,
            show_settings: true,
            show_subtitles_button: true,
            show_fullscreen: true,
            show_pip: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlayerLayout {
    Inline,
    Fullscreen,
    PictureInPicture,
    Minimized,
}

pub struct PlayerControls {
    pub config: PlayerUiConfig,
    pub layout: PlayerLayout,
    pub visible: bool,
    pub last_interaction_ms: u64,
    pub hovered: Option<PlayerControl>,
    pub focused: Option<PlayerControl>,
    pub is_dragging_seek: bool,
    pub drag_seek_pos: f32,
}

impl PlayerControls {
    pub fn new() -> Self {
        Self {
            config: PlayerUiConfig::default(),
            layout: PlayerLayout::Inline,
            visible: true,
            last_interaction_ms: 0,
            hovered: None,
            focused: None,
            is_dragging_seek: false,
            drag_seek_pos: 0.0,
        }
    }

    pub fn should_show(&self, now_ms: u64) -> bool {
        if !self.config.show_controls { return false; }
        if !self.is_dragging_seek && self.hovered.is_none() && self.focused.is_none() {
            now_ms.saturating_sub(self.last_interaction_ms) < self.config.auto_hide_ms as u64
        } else {
            true
        }
    }

    pub fn on_mouse_move(&mut self, control: PlayerControl, now_ms: u64) {
        self.hovered = Some(control);
        self.last_interaction_ms = now_ms;
        self.visible = true;
    }

    pub fn on_mouse_leave(&mut self) {
        self.hovered = None;
    }

    pub fn on_key_focus(&mut self, control: PlayerControl) {
        self.focused = Some(control);
    }

    pub fn on_key_blur(&mut self) {
        self.focused = None;
    }

    pub fn on_seek_start(&mut self, pos: f32) {
        self.is_dragging_seek = true;
        self.drag_seek_pos = pos;
    }

    pub fn on_seek_drag(&mut self, pos: f32) {
        self.drag_seek_pos = pos.clamp(0.0, 1.0);
    }

    pub fn on_seek_end(&mut self) -> Option<f32> {
        self.is_dragging_seek = false;
        let pos = self.drag_seek_pos;
        self.drag_seek_pos = 0.0;
        Some(pos)
    }

    pub fn set_layout(&mut self, layout: PlayerLayout) {
        self.layout = layout;
    }

    pub fn is_fullscreen(&self) -> bool {
        self.layout == PlayerLayout::Fullscreen
    }

    pub fn is_pip(&self) -> bool {
        self.layout == PlayerLayout::PictureInPicture
    }

    pub fn available_buttons(&self) -> Vec<&'static str> {
        let mut v = vec!["play", "seek"];
        if self.config.show_time { v.push("time"); }
        if self.config.show_volume { v.push("volume"); }
        if self.config.show_settings { v.push("settings"); }
        if self.config.show_subtitles_button { v.push("subtitles"); }
        if self.config.show_fullscreen { v.push("fullscreen"); }
        if self.config.show_pip { v.push("pip"); }
        v
    }
}

impl Default for PlayerControls {
    fn default() -> Self { Self::new() }
}

pub fn format_player_time(current_secs: f32, total_secs: f32) -> String {
    format!("{} / {}", format_seconds(current_secs), format_seconds(total_secs))
}

pub fn format_seconds(secs: f32) -> String {
    if secs < 0.0 || secs.is_nan() { return "0:00".to_string(); }
    let total = secs as u32;
    let h = total / 3600;
    let m = (total % 3600) / 60;
    let s = total % 60;
    if h > 0 {
        format!("{}:{:02}:{:02}", h, m, s)
    } else {
        format!("{}:{:02}", m, s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_control_from_str() {
        assert_eq!(PlayerControl::from_str("play"), Some(PlayerControl::Play));
        assert_eq!(PlayerControl::from_str("F"), Some(PlayerControl::Fullscreen));
        assert_eq!(PlayerControl::from_str("cc"), Some(PlayerControl::Subtitles));
        assert_eq!(PlayerControl::from_str("nope"), None);
    }

    #[test]
    fn test_config_default() {
        let c = PlayerUiConfig::default();
        assert!(c.show_controls);
        assert_eq!(c.seek_step_seconds, 10);
    }

    #[test]
    fn test_layout() {
        let mut p = PlayerControls::new();
        p.set_layout(PlayerLayout::Fullscreen);
        assert!(p.is_fullscreen());
        p.set_layout(PlayerLayout::PictureInPicture);
        assert!(p.is_pip());
    }

    #[test]
    fn test_should_show_initial() {
        let p = PlayerControls::new();
        assert!(p.should_show(0));
    }

    #[test]
    fn test_should_show_after_hover() {
        let mut p = PlayerControls::new();
        p.on_mouse_move(PlayerControl::Play, 1000);
        assert!(p.should_show(5000));
    }

    #[test]
    fn test_should_hide_after_timeout() {
        let mut p = PlayerControls::new();
        p.last_interaction_ms = 0;
        assert!(!p.should_show(10000));
    }

    #[test]
    fn test_seek_drag() {
        let mut p = PlayerControls::new();
        p.on_seek_start(0.0);
        p.on_seek_drag(0.5);
        p.on_seek_drag(0.7);
        p.on_seek_drag(1.5); // clamp
        assert_eq!(p.drag_seek_pos, 1.0);
        let pos = p.on_seek_end();
        assert_eq!(pos, Some(1.0));
        assert!(!p.is_dragging_seek);
    }

    #[test]
    fn test_seek_drag_clamp_low() {
        let mut p = PlayerControls::new();
        p.on_seek_start(0.0);
        p.on_seek_drag(-0.5);
        assert_eq!(p.drag_seek_pos, 0.0);
    }

    #[test]
    fn test_available_buttons() {
        let p = PlayerControls::new();
        let buttons = p.available_buttons();
        assert!(buttons.contains(&"play"));
        assert!(buttons.contains(&"fullscreen"));
        assert!(buttons.contains(&"subtitles"));
    }

    #[test]
    fn test_mouse_leave() {
        let mut p = PlayerControls::new();
        p.on_mouse_move(PlayerControl::Play, 1000);
        p.on_mouse_leave();
        assert!(p.hovered.is_none());
    }

    #[test]
    fn test_keyboard_focus() {
        let mut p = PlayerControls::new();
        p.on_key_focus(PlayerControl::Play);
        assert_eq!(p.focused, Some(PlayerControl::Play));
        p.on_key_blur();
        assert!(p.focused.is_none());
    }

    #[test]
    fn test_format_seconds_short() {
        assert_eq!(format_seconds(5.0), "0:05");
        assert_eq!(format_seconds(65.0), "1:05");
        assert_eq!(format_seconds(125.0), "2:05");
    }

    #[test]
    fn test_format_seconds_long() {
        assert_eq!(format_seconds(3661.0), "1:01:01");
    }

    #[test]
    fn test_format_seconds_zero() {
        assert_eq!(format_seconds(0.0), "0:00");
    }

    #[test]
    fn test_format_seconds_invalid() {
        assert_eq!(format_seconds(-1.0), "0:00");
        assert_eq!(format_seconds(f32::NAN), "0:00");
    }

    #[test]
    fn test_format_player_time() {
        let s = format_player_time(125.0, 3600.0);
        assert_eq!(s, "2:05 / 1:00:00");
    }

    #[test]
    fn test_player_controls_new() {
        let p = PlayerControls::new();
        assert_eq!(p.layout, PlayerLayout::Inline);
        assert!(p.visible);
    }
}
