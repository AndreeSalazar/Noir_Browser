//! HTML5 Audio Element - Reproducción de audio
//!
//! Maneja <audio> con controles UI (play/pause, seek, volume, mute).

use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AudioState {
    Empty,
    Loading,
    Ready,
    Playing,
    Paused,
    Ended,
    Error,
}

#[derive(Debug, Clone)]
pub struct AudioSource {
    pub src: String,
    pub mime_type: String,
    pub codec: String,
    pub channels: u8,
    pub sample_rate: u32,
    pub bitrate: u32,
    pub duration: Duration,
}

impl AudioSource {
    pub fn new(src: &str) -> Self {
        let mime = guess_audio_mime(src);
        let codec = guess_audio_codec(src);
        Self {
            src: src.to_string(),
            mime_type: mime.to_string(),
            codec: codec.to_string(),
            channels: 2,
            sample_rate: 44100,
            bitrate: 128_000,
            duration: Duration::ZERO,
        }
    }

    pub fn with_duration(mut self, d: Duration) -> Self {
        self.duration = d;
        self
    }
}

fn guess_audio_mime(src: &str) -> &'static str {
    let lower = src.to_lowercase();
    if lower.ends_with(".mp3") { return "audio/mpeg"; }
    if lower.ends_with(".wav") { return "audio/wav"; }
    if lower.ends_with(".ogg") || lower.ends_with(".oga") { return "audio/ogg"; }
    if lower.ends_with(".m4a") || lower.ends_with(".aac") { return "audio/aac"; }
    if lower.ends_with(".flac") { return "audio/flac"; }
    if lower.ends_with(".webm") { return "audio/webm"; }
    "audio/mpeg"
}

fn guess_audio_codec(src: &str) -> &'static str {
    let lower = src.to_lowercase();
    if lower.ends_with(".mp3") { return "mp3"; }
    if lower.ends_with(".wav") { return "pcm"; }
    if lower.ends_with(".ogg") { return "vorbis"; }
    if lower.ends_with(".m4a") { return "aac"; }
    if lower.ends_with(".flac") { return "flac"; }
    "unknown"
}

pub struct AudioElement {
    pub id: u32,
    pub sources: Vec<AudioSource>,
    pub state: AudioState,
    pub current_time: Duration,
    pub playback_rate: f32,
    pub volume: f32,
    pub muted: bool,
    pub loop_playback: bool,
    pub autoplay: bool,
    pub preload: String,
    pub current_source: Option<usize>,
    pub error_message: Option<String>,
}

impl AudioElement {
    pub fn new(id: u32, src: &str) -> Self {
        let source = AudioSource::new(src);
        let mut sources = Vec::new();
        sources.push(source);
        Self {
            id,
            sources,
            state: AudioState::Empty,
            current_time: Duration::ZERO,
            playback_rate: 1.0,
            volume: 1.0,
            muted: false,
            loop_playback: false,
            autoplay: false,
            preload: "metadata".to_string(),
            current_source: Some(0),
            error_message: None,
        }
    }

    pub fn add_source(&mut self, src: &str) {
        self.sources.push(AudioSource::new(src));
    }

    pub fn load(&mut self) {
        self.state = AudioState::Loading;
        self.state = AudioState::Ready;
    }

    pub fn play(&mut self) {
        if matches!(self.state, AudioState::Ready | AudioState::Paused) {
            self.state = AudioState::Playing;
        }
    }

    pub fn pause(&mut self) {
        if self.state == AudioState::Playing {
            self.state = AudioState::Paused;
        }
    }

    pub fn toggle(&mut self) {
        match self.state {
            AudioState::Playing => self.pause(),
            AudioState::Paused | AudioState::Ready => self.play(),
            _ => {}
        }
    }

    pub fn seek(&mut self, time: Duration) {
        self.current_time = time;
    }

    pub fn set_volume(&mut self, vol: f32) {
        self.volume = vol.clamp(0.0, 1.0);
        self.muted = self.volume == 0.0;
    }

    pub fn toggle_mute(&mut self) {
        self.muted = !self.muted;
    }

    pub fn advance(&mut self, delta: Duration) {
        if self.state == AudioState::Playing {
            let real_delta = delta.mul_f32(self.playback_rate);
            self.current_time += real_delta;
            if let Some(idx) = self.current_source {
                if let Some(source) = self.sources.get(idx) {
                    if self.current_time >= source.duration {
                        if self.loop_playback {
                            self.current_time = Duration::ZERO;
                        } else {
                            self.state = AudioState::Ended;
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

    pub fn is_playing(&self) -> bool {
        self.state == AudioState::Playing
    }
}

pub struct AudioManager {
    audios: Vec<AudioElement>,
    next_id: u32,
}

impl AudioManager {
    pub fn new() -> Self {
        Self {
            audios: Vec::new(),
            next_id: 1,
        }
    }

    pub fn create(&mut self, src: &str) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.audios.push(AudioElement::new(id, src));
        id
    }

    pub fn get(&self, id: u32) -> Option<&AudioElement> {
        self.audios.iter().find(|a| a.id == id)
    }

    pub fn get_mut(&mut self, id: u32) -> Option<&mut AudioElement> {
        self.audios.iter_mut().find(|a| a.id == id)
    }

    pub fn count(&self) -> usize {
        self.audios.len()
    }

    pub fn playing(&self) -> Vec<&AudioElement> {
        self.audios.iter().filter(|a| a.is_playing()).collect()
    }

    /// Pause todos los audios (cuando se inicia uno nuevo)
    pub fn pause_all_except(&mut self, except_id: u32) {
        for a in &mut self.audios {
            if a.id != except_id && a.state == AudioState::Playing {
                a.state = AudioState::Paused;
            }
        }
    }
}

impl Default for AudioManager {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_source_mp3() {
        let s = AudioSource::new("song.mp3");
        assert_eq!(s.mime_type, "audio/mpeg");
        assert_eq!(s.codec, "mp3");
    }

    #[test]
    fn test_audio_source_ogg() {
        let s = AudioSource::new("song.ogg");
        assert_eq!(s.mime_type, "audio/ogg");
        assert_eq!(s.codec, "vorbis");
    }

    #[test]
    fn test_audio_source_wav() {
        let s = AudioSource::new("song.wav");
        assert_eq!(s.mime_type, "audio/wav");
    }

    #[test]
    fn test_audio_source_m4a() {
        let s = AudioSource::new("song.m4a");
        assert_eq!(s.codec, "aac");
    }

    #[test]
    fn test_audio_element_new() {
        let a = AudioElement::new(1, "x.mp3");
        assert_eq!(a.id, 1);
        assert_eq!(a.state, AudioState::Empty);
    }

    #[test]
    fn test_audio_load() {
        let mut a = AudioElement::new(1, "x.mp3");
        a.load();
        assert_eq!(a.state, AudioState::Ready);
    }

    #[test]
    fn test_audio_play() {
        let mut a = AudioElement::new(1, "x.mp3");
        a.load();
        a.play();
        assert!(a.is_playing());
    }

    #[test]
    fn test_audio_pause() {
        let mut a = AudioElement::new(1, "x.mp3");
        a.load();
        a.play();
        a.pause();
        assert!(!a.is_playing());
    }

    #[test]
    fn test_audio_toggle() {
        let mut a = AudioElement::new(1, "x.mp3");
        a.load();
        a.toggle();
        assert!(a.is_playing());
        a.toggle();
        assert!(!a.is_playing());
    }

    #[test]
    fn test_audio_seek() {
        let mut a = AudioElement::new(1, "x.mp3");
        a.seek(Duration::from_secs(20));
        assert_eq!(a.current_time, Duration::from_secs(20));
    }

    #[test]
    fn test_audio_volume() {
        let mut a = AudioElement::new(1, "x.mp3");
        a.set_volume(0.3);
        assert_eq!(a.volume, 0.3);
        a.set_volume(1.5);
        assert_eq!(a.volume, 1.0);
    }

    #[test]
    fn test_audio_mute() {
        let mut a = AudioElement::new(1, "x.mp3");
        assert!(!a.muted);
        a.toggle_mute();
        assert!(a.muted);
    }

    #[test]
    fn test_audio_advance() {
        let mut s = AudioSource::new("x.mp3");
        s.duration = Duration::from_secs(30);
        let mut a = AudioElement::new(1, "x.mp3");
        a.sources[0] = s;
        a.load();
        a.play();
        a.advance(Duration::from_secs(5));
        assert_eq!(a.current_time, Duration::from_secs(5));
    }

    #[test]
    fn test_audio_advance_ended() {
        let mut s = AudioSource::new("x.mp3");
        s.duration = Duration::from_secs(10);
        let mut a = AudioElement::new(1, "x.mp3");
        a.sources[0] = s;
        a.load();
        a.play();
        a.advance(Duration::from_secs(15));
        assert_eq!(a.state, AudioState::Ended);
    }

    #[test]
    fn test_audio_advance_loop() {
        let mut s = AudioSource::new("x.mp3");
        s.duration = Duration::from_secs(10);
        let mut a = AudioElement::new(1, "x.mp3");
        a.sources[0] = s;
        a.loop_playback = true;
        a.load();
        a.play();
        a.advance(Duration::from_secs(15));
        assert_eq!(a.current_time, Duration::ZERO);
    }

    #[test]
    fn test_audio_select_source() {
        let mut a = AudioElement::new(1, "a.mp3");
        a.add_source("b.ogg");
        assert!(a.select_source("audio/ogg"));
        assert_eq!(a.current_source, Some(1));
    }

    #[test]
    fn test_audio_progress() {
        let mut s = AudioSource::new("x.mp3");
        s.duration = Duration::from_secs(10);
        let mut a = AudioElement::new(1, "x.mp3");
        a.sources[0] = s;
        a.load();
        a.play();
        a.advance(Duration::from_secs(2));
        assert_eq!(a.progress(), 0.2);
    }

    #[test]
    fn test_manager_create() {
        let mut m = AudioManager::new();
        let id = m.create("x.mp3");
        assert_eq!(m.count(), 1);
        assert!(m.get(id).is_some());
    }

    #[test]
    fn test_manager_pause_all_except() {
        let mut m = AudioManager::new();
        let id1 = m.create("a.mp3");
        let id2 = m.create("b.mp3");
        m.get_mut(id1).unwrap().load();
        m.get_mut(id1).unwrap().play();
        m.get_mut(id2).unwrap().load();
        m.get_mut(id2).unwrap().play();
        m.pause_all_except(id1);
        assert!(m.get(id1).unwrap().is_playing());
        assert!(!m.get(id2).unwrap().is_playing());
    }

    #[test]
    fn test_audio_playback_rate() {
        let mut s = AudioSource::new("x.mp3");
        s.duration = Duration::from_secs(100);
        let mut a = AudioElement::new(1, "x.mp3");
        a.sources[0] = s;
        a.playback_rate = 2.0;
        a.load();
        a.play();
        a.advance(Duration::from_secs(10));
        assert_eq!(a.current_time, Duration::from_secs(20));
    }
}
