//! HTML5 Video Element - Render y reproducción de video
//!
//! Maneja <video> con controles UI (play/pause, seek, volume).

use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VideoState {
    Empty,
    Loading,
    Ready,
    Playing,
    Paused,
    Ended,
    Error,
}

#[derive(Debug, Clone)]
pub struct VideoSource {
    pub src: String,
    pub mime_type: String,
    pub codec: String,
    pub width: u32,
    pub height: u32,
    pub bitrate: u32,
    pub duration: Duration,
}

impl VideoSource {
    pub fn new(src: &str) -> Self {
        let mime = guess_mime(src);
        let codec = guess_codec(src);
        Self {
            src: src.to_string(),
            mime_type: mime.to_string(),
            codec: codec.to_string(),
            width: 0,
            height: 0,
            bitrate: 0,
            duration: Duration::ZERO,
        }
    }

    pub fn with_dimensions(mut self, w: u32, h: u32) -> Self {
        self.width = w;
        self.height = h;
        self
    }

    pub fn with_duration(mut self, d: Duration) -> Self {
        self.duration = d;
        self
    }

    pub fn aspect_ratio(&self) -> f32 {
        if self.height == 0 { 16.0 / 9.0 } else { self.width as f32 / self.height as f32 }
    }
}

fn guess_mime(src: &str) -> &'static str {
    let lower = src.to_lowercase();
    if lower.ends_with(".mp4") { return "video/mp4"; }
    if lower.ends_with(".webm") { return "video/webm"; }
    if lower.ends_with(".ogg") || lower.ends_with(".ogv") { return "video/ogg"; }
    if lower.ends_with(".mov") { return "video/quicktime"; }
    if lower.ends_with(".avi") { return "video/x-msvideo"; }
    if lower.ends_with(".mkv") { return "video/x-matroska"; }
    "video/mp4"
}

fn guess_codec(src: &str) -> &'static str {
    let lower = src.to_lowercase();
    if lower.ends_with(".mp4") { return "avc1.42E01E"; }
    if lower.ends_with(".webm") { return "vp8"; }
    if lower.ends_with(".ogg") || lower.ends_with(".ogv") { return "theora"; }
    "unknown"
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VideoEvent {
    LoadStart,
    LoadedMetadata,
    LoadedData,
    CanPlay,
    Play,
    Pause,
    Seeking,
    Seeked,
    TimeUpdate,
    Ended,
    VolumeChange,
    Error,
}

#[derive(Debug, Clone)]
pub struct VideoControls {
    pub visible: bool,
    pub auto_hide: bool,
    pub height: u32,
}

impl Default for VideoControls {
    fn default() -> Self {
        Self {
            visible: true,
            auto_hide: true,
            height: 40,
        }
    }
}

pub struct VideoElement {
    pub id: u32,
    pub sources: Vec<VideoSource>,
    pub state: VideoState,
    pub current_time: Duration,
    pub playback_rate: f32,
    pub volume: f32,
    pub muted: bool,
    pub loop_playback: bool,
    pub autoplay: bool,
    pub preload: String,
    pub poster: Option<String>,
    pub width: u32,
    pub height: u32,
    pub controls: VideoControls,
    pub current_source: Option<usize>,
    pub buffered: Vec<(Duration, Duration)>,
    pub last_event: Option<(VideoEvent, Instant)>,
    pub error_message: Option<String>,
}

impl VideoElement {
    pub fn new(id: u32, src: &str) -> Self {
        let source = VideoSource::new(src);
        let w = source.width;
        let h = source.height;
        let mut sources = Vec::new();
        sources.push(source);
        Self {
            id,
            sources,
            state: VideoState::Empty,
            current_time: Duration::ZERO,
            playback_rate: 1.0,
            volume: 1.0,
            muted: false,
            loop_playback: false,
            autoplay: false,
            preload: "metadata".to_string(),
            poster: None,
            width: w,
            height: h,
            controls: VideoControls::default(),
            current_source: Some(0),
            buffered: Vec::new(),
            last_event: None,
            error_message: None,
        }
    }

    pub fn add_source(&mut self, src: &str) {
        self.sources.push(VideoSource::new(src));
    }

    pub fn load(&mut self) {
        self.state = VideoState::Loading;
        self.emit(VideoEvent::LoadStart);
        if let Some(idx) = self.current_source {
            if let Some(source) = self.sources.get(idx) {
                self.width = source.width;
                self.height = source.height;
            }
        }
        self.state = VideoState::Ready;
        self.emit(VideoEvent::LoadedMetadata);
        self.emit(VideoEvent::CanPlay);
    }

    pub fn play(&mut self) {
        if self.state == VideoState::Ready || self.state == VideoState::Paused {
            self.state = VideoState::Playing;
            self.emit(VideoEvent::Play);
        }
    }

    pub fn pause(&mut self) {
        if self.state == VideoState::Playing {
            self.state = VideoState::Paused;
            self.emit(VideoEvent::Pause);
        }
    }

    pub fn toggle(&mut self) {
        match self.state {
            VideoState::Playing => self.pause(),
            VideoState::Paused | VideoState::Ready => self.play(),
            _ => {}
        }
    }

    pub fn seek(&mut self, time: Duration) {
        self.emit(VideoEvent::Seeking);
        self.current_time = time;
        self.emit(VideoEvent::Seeked);
        self.emit(VideoEvent::TimeUpdate);
    }

    pub fn set_volume(&mut self, vol: f32) {
        self.volume = vol.clamp(0.0, 1.0);
        self.muted = self.volume == 0.0;
        self.emit(VideoEvent::VolumeChange);
    }

    pub fn toggle_mute(&mut self) {
        self.muted = !self.muted;
        self.emit(VideoEvent::VolumeChange);
    }

    pub fn advance(&mut self, delta: Duration) {
        if self.state == VideoState::Playing {
            let real_delta = delta.mul_f32(self.playback_rate);
            self.current_time += real_delta;
            self.emit(VideoEvent::TimeUpdate);
            if let Some(idx) = self.current_source {
                if let Some(source) = self.sources.get(idx) {
                    if self.current_time >= source.duration {
                        if self.loop_playback {
                            self.current_time = Duration::ZERO;
                        } else {
                            self.state = VideoState::Ended;
                            self.emit(VideoEvent::Ended);
                        }
                    }
                }
            }
        }
    }

    pub fn progress(&self) -> f32 {
        if let Some(idx) = self.current_source {
            if let Some(source) = self.sources.get(idx) {
                let total = source.duration.as_secs_f32();
                if total > 0.0 {
                    return (self.current_time.as_secs_f32() / total).clamp(0.0, 1.0);
                }
            }
        }
        0.0
    }

    pub fn select_source(&mut self, mime: &str) -> bool {
        for (i, s) in self.sources.iter().enumerate() {
            if s.mime_type == mime {
                self.current_source = Some(i);
                return true;
            }
        }
        false
    }

    pub fn duration(&self) -> Duration {
        self.current_source
            .and_then(|i| self.sources.get(i))
            .map(|s| s.duration)
            .unwrap_or(Duration::ZERO)
    }

    pub fn format_time(d: Duration) -> String {
        let total = d.as_secs();
        let h = total / 3600;
        let m = (total % 3600) / 60;
        let s = total % 60;
        if h > 0 {
            format!("{}:{:02}:{:02}", h, m, s)
        } else {
            format!("{}:{:02}", m, s)
        }
    }

    fn emit(&mut self, event: VideoEvent) {
        self.last_event = Some((event, Instant::now()));
    }

    pub fn is_playing(&self) -> bool {
        self.state == VideoState::Playing
    }
}

pub struct VideoManager {
    videos: Vec<VideoElement>,
    next_id: u32,
}

impl VideoManager {
    pub fn new() -> Self {
        Self {
            videos: Vec::new(),
            next_id: 1,
        }
    }

    pub fn create(&mut self, src: &str) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        let video = VideoElement::new(id, src);
        self.videos.push(video);
        id
    }

    pub fn get(&self, id: u32) -> Option<&VideoElement> {
        self.videos.iter().find(|v| v.id == id)
    }

    pub fn get_mut(&mut self, id: u32) -> Option<&mut VideoElement> {
        self.videos.iter_mut().find(|v| v.id == id)
    }

    pub fn all(&self) -> &Vec<VideoElement> {
        &self.videos
    }

    pub fn count(&self) -> usize {
        self.videos.len()
    }

    pub fn playing(&self) -> Vec<&VideoElement> {
        self.videos.iter().filter(|v| v.is_playing()).collect()
    }
}

impl Default for VideoManager {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_video_source_new() {
        let s = VideoSource::new("movie.mp4");
        assert_eq!(s.mime_type, "video/mp4");
        assert_eq!(s.codec, "avc1.42E01E");
    }

    #[test]
    fn test_video_source_webm() {
        let s = VideoSource::new("movie.webm");
        assert_eq!(s.mime_type, "video/webm");
        assert_eq!(s.codec, "vp8");
    }

    #[test]
    fn test_video_source_ogg() {
        let s = VideoSource::new("movie.ogv");
        assert_eq!(s.mime_type, "video/ogg");
        assert_eq!(s.codec, "theora");
    }

    #[test]
    fn test_video_source_aspect() {
        let s = VideoSource::new("a.mp4").with_dimensions(1920, 1080);
        assert_eq!(s.aspect_ratio(), 16.0/9.0);
    }

    #[test]
    fn test_video_source_aspect_zero() {
        let s = VideoSource::new("a.mp4");
        assert_eq!(s.aspect_ratio(), 16.0/9.0);
    }

    #[test]
    fn test_video_element_new() {
        let v = VideoElement::new(1, "a.mp4");
        assert_eq!(v.id, 1);
        assert_eq!(v.state, VideoState::Empty);
    }

    #[test]
    fn test_load() {
        let mut v = VideoElement::new(1, "a.mp4");
        v.load();
        assert_eq!(v.state, VideoState::Ready);
    }

    #[test]
    fn test_play() {
        let mut v = VideoElement::new(1, "a.mp4");
        v.load();
        v.play();
        assert!(v.is_playing());
    }

    #[test]
    fn test_pause() {
        let mut v = VideoElement::new(1, "a.mp4");
        v.load();
        v.play();
        v.pause();
        assert!(!v.is_playing());
        assert_eq!(v.state, VideoState::Paused);
    }

    #[test]
    fn test_toggle() {
        let mut v = VideoElement::new(1, "a.mp4");
        v.load();
        v.toggle();
        assert!(v.is_playing());
        v.toggle();
        assert!(!v.is_playing());
    }

    #[test]
    fn test_seek() {
        let mut v = VideoElement::new(1, "a.mp4");
        v.load();
        v.seek(Duration::from_secs(10));
        assert_eq!(v.current_time, Duration::from_secs(10));
    }

    #[test]
    fn test_volume() {
        let mut v = VideoElement::new(1, "a.mp4");
        v.set_volume(0.5);
        assert_eq!(v.volume, 0.5);
        v.set_volume(2.0);
        assert_eq!(v.volume, 1.0);
        v.set_volume(-1.0);
        assert_eq!(v.volume, 0.0);
    }

    #[test]
    fn test_mute_toggle() {
        let mut v = VideoElement::new(1, "a.mp4");
        assert!(!v.muted);
        v.toggle_mute();
        assert!(v.muted);
        v.toggle_mute();
        assert!(!v.muted);
    }

    #[test]
    fn test_advance() {
        let mut s = VideoSource::new("a.mp4");
        s.duration = Duration::from_secs(10);
        let mut v = VideoElement::new(1, "a.mp4");
        v.sources[0] = s;
        v.load();
        v.play();
        v.advance(Duration::from_secs(2));
        assert_eq!(v.current_time, Duration::from_secs(2));
    }

    #[test]
    fn test_advance_ended() {
        let mut s = VideoSource::new("a.mp4");
        s.duration = Duration::from_secs(10);
        let mut v = VideoElement::new(1, "a.mp4");
        v.sources[0] = s;
        v.load();
        v.play();
        v.advance(Duration::from_secs(15));
        assert_eq!(v.state, VideoState::Ended);
    }

    #[test]
    fn test_advance_loop() {
        let mut s = VideoSource::new("a.mp4");
        s.duration = Duration::from_secs(10);
        let mut v = VideoElement::new(1, "a.mp4");
        v.sources[0] = s;
        v.loop_playback = true;
        v.load();
        v.play();
        v.advance(Duration::from_secs(15));
        assert_eq!(v.current_time, Duration::ZERO);
    }

    #[test]
    fn test_advance_paused() {
        let mut s = VideoSource::new("a.mp4");
        s.duration = Duration::from_secs(10);
        let mut v = VideoElement::new(1, "a.mp4");
        v.sources[0] = s;
        v.load();
        v.advance(Duration::from_secs(5));
        assert_eq!(v.current_time, Duration::ZERO);
    }

    #[test]
    fn test_progress() {
        let mut s = VideoSource::new("a.mp4");
        s.duration = Duration::from_secs(10);
        let mut v = VideoElement::new(1, "a.mp4");
        v.sources[0] = s;
        v.load();
        v.play();
        v.advance(Duration::from_secs(5));
        assert_eq!(v.progress(), 0.5);
    }

    #[test]
    fn test_select_source() {
        let mut v = VideoElement::new(1, "a.mp4");
        v.add_source("b.webm");
        assert!(v.select_source("video/webm"));
        assert_eq!(v.current_source, Some(1));
    }

    #[test]
    fn test_format_time() {
        assert_eq!(VideoElement::format_time(Duration::from_secs(65)), "1:05");
        assert_eq!(VideoElement::format_time(Duration::from_secs(3661)), "1:01:01");
        assert_eq!(VideoElement::format_time(Duration::from_secs(30)), "0:30");
    }

    #[test]
    fn test_manager_create() {
        let mut m = VideoManager::new();
        let id = m.create("a.mp4");
        assert_eq!(m.count(), 1);
        assert!(m.get(id).is_some());
    }

    #[test]
    fn test_manager_playing() {
        let mut m = VideoManager::new();
        let id1 = m.create("a.mp4");
        let id2 = m.create("b.mp4");
        m.get_mut(id1).unwrap().load();
        m.get_mut(id1).unwrap().play();
        m.get_mut(id2).unwrap().load();
        assert_eq!(m.playing().len(), 1);
    }

    #[test]
    fn test_playback_rate() {
        let mut s = VideoSource::new("a.mp4");
        s.duration = Duration::from_secs(100);
        let mut v = VideoElement::new(1, "a.mp4");
        v.sources[0] = s;
        v.playback_rate = 2.0;
        v.load();
        v.play();
        v.advance(Duration::from_secs(10));
        assert_eq!(v.current_time, Duration::from_secs(20));
    }
}
