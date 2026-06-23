//! Media Source Extensions (MSE) - Streaming de video
//!
//! Permite streaming adaptativo bitrate para HLS/DASH.

use std::collections::VecDeque;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SourceState {
    Closed,
    Open,
    Ended,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppendBufferState {
    Waiting,
    Parsing,
    Parsed,
    Error,
}

#[derive(Debug, Clone)]
pub struct SourceBuffer {
    pub mime_type: String,
    pub codec: String,
    pub buffered: Vec<TimeRange>,
    pub state: AppendBufferState,
    pub append_window_start: Duration,
    pub append_window_end: Duration,
    pub timestamp_offset: f64,
    pub mode: AppendMode,
    pub total_bytes: u64,
    pub total_segments: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppendMode {
    Segments,
    Sequence,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimeRange {
    pub start: Duration,
    pub end: Duration,
}

impl TimeRange {
    pub fn contains(&self, time: Duration) -> bool {
        time >= self.start && time <= self.end
    }

    pub fn duration(&self) -> Duration {
        self.end - self.start
    }
}

impl SourceBuffer {
    pub fn new(mime: &str, codec: &str) -> Self {
        Self {
            mime_type: mime.to_string(),
            codec: codec.to_string(),
            buffered: Vec::new(),
            state: AppendBufferState::Waiting,
            append_window_start: Duration::ZERO,
            append_window_end: Duration::from_secs(3600 * 24), // 24h
            timestamp_offset: 0.0,
            mode: AppendMode::Segments,
            total_bytes: 0,
            total_segments: 0,
        }
    }

    /// Append bytes al buffer
    pub fn append_buffer(&mut self, data: &[u8], timestamp: Duration) -> Result<(), String> {
        if self.state == AppendBufferState::Parsing {
            return Err("SourceBuffer is parsing".to_string());
        }
        self.state = AppendBufferState::Parsing;
        self.total_bytes += data.len() as u64;
        self.total_segments += 1;
        // Calcula duración: 1 KB = 1 segundo (simplificado)
        let segment_duration = Duration::from_millis(data.len() as u64);
        let end = timestamp + segment_duration;
        self.buffered.push(TimeRange { start: timestamp, end });
        self.state = AppendBufferState::Parsed;
        Ok(())
    }

    /// Encuentra rango buffered que contiene el timestamp
    pub fn buffered_around(&self, time: Duration) -> Option<&TimeRange> {
        self.buffered.iter().find(|r| r.contains(time))
    }

    pub fn total_buffered(&self) -> Duration {
        self.buffered.iter().map(|r| r.duration()).sum()
    }

    pub fn end(&self) -> Option<Duration> {
        self.buffered.iter().map(|r| r.end).max()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QualityLevel {
    pub width: u32,
    pub height: u32,
    pub bitrate: u32,
    pub codecs: &'static str,
}

pub struct MediaSource {
    pub id: u32,
    pub state: SourceState,
    pub duration: Duration,
    pub source_buffers: Vec<SourceBuffer>,
    pub active_quality_levels: Vec<QualityLevel>,
    pub current_quality: usize,
    pub auto_switch_quality: bool,
    pub network_bandwidth: u32, // kbps
}

impl Clone for MediaSource {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            state: self.state,
            duration: self.duration,
            source_buffers: self.source_buffers.clone(),
            active_quality_levels: self.active_quality_levels.clone(),
            current_quality: self.current_quality,
            auto_switch_quality: self.auto_switch_quality,
            network_bandwidth: self.network_bandwidth,
        }
    }
}

impl MediaSource {
    pub fn new(id: u32, duration: Duration) -> Self {
        Self {
            id,
            state: SourceState::Closed,
            duration,
            source_buffers: Vec::new(),
            active_quality_levels: Vec::new(),
            current_quality: 0,
            auto_switch_quality: true,
            network_bandwidth: 5000, // 5 Mbps default
        }
    }

    pub fn open(&mut self) {
        self.state = SourceState::Open;
    }

    pub fn end_of_stream(&mut self) {
        if !self.source_buffers.is_empty() {
            self.state = SourceState::Ended;
        }
    }

    pub fn close(&mut self) {
        self.state = SourceState::Closed;
    }

    pub fn add_source_buffer(&mut self, mime: &str, codec: &str) -> usize {
        let sb = SourceBuffer::new(mime, codec);
        self.source_buffers.push(sb);
        self.source_buffers.len() - 1
    }

    pub fn add_quality_level(&mut self, q: QualityLevel) {
        self.active_quality_levels.push(q);
    }

    /// Adaptive bitrate: cambia calidad según bandwidth
    pub fn adapt_quality(&mut self) {
        if !self.auto_switch_quality || self.active_quality_levels.is_empty() {
            return;
        }
        let mut best_idx = 0;
        for (i, q) in self.active_quality_levels.iter().enumerate() {
            if q.bitrate <= self.network_bandwidth * 1000 {
                best_idx = i;
            } else {
                break;
            }
        }
        self.current_quality = best_idx;
    }

    pub fn current_quality(&self) -> Option<&QualityLevel> {
        self.active_quality_levels.get(self.current_quality)
    }

    pub fn is_open(&self) -> bool {
        self.state == SourceState::Open
    }
}

pub struct MediaSourceManager {
    sources: Vec<MediaSource>,
    pending_buffers: VecDeque<(u32, usize, Vec<u8>, Duration)>, // (source_id, sb_idx, data, timestamp)
    next_id: u32,
}

impl MediaSourceManager {
    pub fn new() -> Self {
        Self {
            sources: Vec::new(),
            pending_buffers: VecDeque::new(),
            next_id: 1,
        }
    }

    pub fn create_source(&mut self, duration: Duration) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.sources.push(MediaSource::new(id, duration));
        id
    }

    pub fn get(&self, id: u32) -> Option<&MediaSource> {
        self.sources.iter().find(|s| s.id == id)
    }

    pub fn get_mut(&mut self, id: u32) -> Option<&mut MediaSource> {
        self.sources.iter_mut().find(|s| s.id == id)
    }

    /// Queue data para append a un source buffer
    pub fn queue_append(&mut self, source_id: u32, sb_idx: usize, data: Vec<u8>, timestamp: Duration) {
        self.pending_buffers.push_back((source_id, sb_idx, data, timestamp));
    }

    /// Procesa pending buffers
    pub fn process_pending(&mut self) {
        while let Some((source_id, sb_idx, data, timestamp)) = self.pending_buffers.pop_front() {
            if let Some(source) = self.get_mut(source_id) {
                if let Some(sb) = source.source_buffers.get_mut(sb_idx) {
                    let _ = sb.append_buffer(&data, timestamp);
                }
            }
        }
    }

    pub fn count(&self) -> usize {
        self.sources.len()
    }

    pub fn total_buffered_bytes(&self) -> u64 {
        self.sources.iter()
            .flat_map(|s| s.source_buffers.iter())
            .map(|sb| sb.total_bytes)
            .sum()
    }
}

impl Default for MediaSourceManager {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_range_contains() {
        let r = TimeRange { start: Duration::from_secs(5), end: Duration::from_secs(10) };
        assert!(r.contains(Duration::from_secs(7)));
        assert!(!r.contains(Duration::from_secs(3)));
    }

    #[test]
    fn test_time_range_duration() {
        let r = TimeRange { start: Duration::from_secs(5), end: Duration::from_secs(15) };
        assert_eq!(r.duration(), Duration::from_secs(10));
    }

    #[test]
    fn test_source_buffer_new() {
        let sb = SourceBuffer::new("video/mp4", "avc1");
        assert_eq!(sb.mime_type, "video/mp4");
        assert_eq!(sb.state, AppendBufferState::Waiting);
    }

    #[test]
    fn test_source_buffer_append() {
        let mut sb = SourceBuffer::new("video/mp4", "avc1");
        let data = vec![0u8; 5000];
        sb.append_buffer(&data, Duration::from_secs(0)).unwrap();
        assert_eq!(sb.total_bytes, 5000);
        assert_eq!(sb.buffered.len(), 1);
    }

    #[test]
    fn test_source_buffer_total_buffered() {
        let mut sb = SourceBuffer::new("video/mp4", "avc1");
        sb.append_buffer(&vec![0u8; 5000], Duration::from_secs(0)).unwrap();
        sb.append_buffer(&vec![0u8; 5000], Duration::from_secs(5)).unwrap();
        assert_eq!(sb.total_buffered(), Duration::from_millis(10000));
    }

    #[test]
    fn test_source_buffer_end() {
        let mut sb = SourceBuffer::new("video/mp4", "avc1");
        sb.append_buffer(&vec![0u8; 5000], Duration::from_secs(0)).unwrap();
        sb.append_buffer(&vec![0u8; 5000], Duration::from_secs(10)).unwrap();
        assert_eq!(sb.end(), Some(Duration::from_millis(15000)));
    }

    #[test]
    fn test_media_source_new() {
        let s = MediaSource::new(1, Duration::from_secs(120));
        assert_eq!(s.state, SourceState::Closed);
    }

    #[test]
    fn test_media_source_open_close() {
        let mut s = MediaSource::new(1, Duration::from_secs(60));
        s.open();
        assert!(s.is_open());
        s.close();
        assert!(!s.is_open());
    }

    #[test]
    fn test_media_source_add_sb() {
        let mut s = MediaSource::new(1, Duration::from_secs(60));
        s.add_source_buffer("video/mp4", "avc1");
        s.add_source_buffer("audio/mp4", "aac");
        assert_eq!(s.source_buffers.len(), 2);
    }

    #[test]
    fn test_media_source_eos() {
        let mut s = MediaSource::new(1, Duration::from_secs(60));
        s.open();
        s.add_source_buffer("video/mp4", "avc1");
        s.end_of_stream();
        assert_eq!(s.state, SourceState::Ended);
    }

    #[test]
    fn test_media_source_eos_without_buffers() {
        let mut s = MediaSource::new(1, Duration::from_secs(60));
        s.open();
        s.end_of_stream();
        assert_eq!(s.state, SourceState::Open); // No cambia sin buffers
    }

    #[test]
    fn test_media_source_add_quality() {
        let mut s = MediaSource::new(1, Duration::from_secs(60));
        s.add_quality_level(QualityLevel { width: 1920, height: 1080, bitrate: 5_000_000, codecs: "avc1" });
        s.add_quality_level(QualityLevel { width: 1280, height: 720, bitrate: 2_500_000, codecs: "avc1" });
        s.add_quality_level(QualityLevel { width: 854, height: 480, bitrate: 1_000_000, codecs: "avc1" });
        assert_eq!(s.active_quality_levels.len(), 3);
    }

    #[test]
    fn test_media_source_adapt_quality_high_bw() {
        let mut s = MediaSource::new(1, Duration::from_secs(60));
        s.add_quality_level(QualityLevel { width: 1920, height: 1080, bitrate: 5_000_000, codecs: "avc1" });
        s.add_quality_level(QualityLevel { width: 1280, height: 720, bitrate: 2_500_000, codecs: "avc1" });
        s.network_bandwidth = 10000; // 10 Mbps
        s.adapt_quality();
        assert_eq!(s.current_quality(), Some(&s.active_quality_levels[1])); // 1080p
    }

    #[test]
    fn test_media_source_adapt_quality_low_bw() {
        let mut s = MediaSource::new(1, Duration::from_secs(60));
        s.add_quality_level(QualityLevel { width: 1920, height: 1080, bitrate: 5_000_000, codecs: "avc1" });
        s.add_quality_level(QualityLevel { width: 1280, height: 720, bitrate: 2_500_000, codecs: "avc1" });
        s.add_quality_level(QualityLevel { width: 854, height: 480, bitrate: 1_000_000, codecs: "avc1" });
        s.network_bandwidth = 500; // 500 kbps
        s.adapt_quality();
        assert_eq!(s.current_quality(), Some(&s.active_quality_levels[0])); // 480p
    }

    #[test]
    fn test_media_source_no_adapt() {
        let mut s = MediaSource::new(1, Duration::from_secs(60));
        s.add_quality_level(QualityLevel { width: 1920, height: 1080, bitrate: 5_000_000, codecs: "avc1" });
        s.auto_switch_quality = false;
        s.adapt_quality();
        assert_eq!(s.current_quality, 0);
    }

    #[test]
    fn test_manager_create() {
        let mut m = MediaSourceManager::new();
        let id = m.create_source(Duration::from_secs(60));
        assert!(m.get(id).is_some());
    }

    #[test]
    fn test_manager_queue_process() {
        let mut m = MediaSourceManager::new();
        let id = m.create_source(Duration::from_secs(60));
        m.get_mut(id).unwrap().open();
        let sb_idx = m.get_mut(id).unwrap().add_source_buffer("video/mp4", "avc1");
        m.queue_append(id, sb_idx, vec![0u8; 1000], Duration::from_secs(0));
        m.queue_append(id, sb_idx, vec![0u8; 1000], Duration::from_secs(2));
        m.process_pending();
        let s = m.get(id).unwrap();
        assert_eq!(s.source_buffers[0].total_bytes, 2000);
    }

    #[test]
    fn test_manager_total_bytes() {
        let mut m = MediaSourceManager::new();
        let id1 = m.create_source(Duration::from_secs(60));
        let id2 = m.create_source(Duration::from_secs(60));
        m.get_mut(id1).unwrap().add_source_buffer("video/mp4", "avc1");
        m.get_mut(id2).unwrap().add_source_buffer("video/mp4", "avc1");
        m.queue_append(id1, 0, vec![0u8; 1000], Duration::ZERO);
        m.queue_append(id2, 0, vec![0u8; 2000], Duration::ZERO);
        m.process_pending();
        assert_eq!(m.total_buffered_bytes(), 3000);
    }
}
