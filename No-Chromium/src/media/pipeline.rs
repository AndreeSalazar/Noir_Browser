//! Media Pipeline - Pipeline completo de reproducción
//!
//! Conecta: network → MSE → H.264 decoder → RGB → texture
//!
//! Para usar:
//! ```ignore
//! let mut pipeline = MediaPipeline::new(1920, 1080);
//! pipeline.feed_encoded(mp4_data); // bytes del MP4
//! while let Some(frame) = pipeline.next_frame() {
//!     let texture_id = frame.texture_id;
//!     // render con WebGPU
//! }
//! ```

use std::collections::VecDeque;

use super::mse::{MediaSource, MediaSourceManager, SourceBuffer, TimeRange};
use super::video_codecs::h264::{H264Decoder, H264StreamParser, FrameType as H264Frame};
use super::frame::{RgbConverter, YuvFrame};
use super::video_texture::{VideoTexture, VideoTextureManager};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PipelineState {
    Idle,
    Loading,
    Ready,
    Playing,
    Paused,
    Ended,
    Error,
}

#[derive(Debug, Clone)]
pub struct DecodedFrame {
    pub texture_id: u32,
    pub width: u32,
    pub height: u32,
    pub pts_ms: i64,
    pub is_keyframe: bool,
    pub rgb_data: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PipelineStats {
    pub frames_decoded: u32,
    pub keyframes: u32,
    pub bytes_processed: u64,
    pub current_fps: f32,
    pub state: PipelineState,
}

pub struct MediaPipeline {
    pub source: Option<MediaSource>,
    pub h264: H264StreamParser,
    pub rgb_converter: Option<RgbConverter>,
    pub textures: VideoTextureManager,
    pub pending_frames: VecDeque<DecodedFrame>,
    pub stats: PipelineStats,
    pub width: u32,
    pub height: u32,
    pub current_pts_ms: i64,
    pub target_fps: f32,
    pub last_frame_time_ms: i64,
    pub dropped_frames: u32,
}

impl MediaPipeline {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            source: None,
            h264: H264StreamParser::new(),
            rgb_converter: Some(RgbConverter::new(width, height)),
            textures: VideoTextureManager::new(),
            pending_frames: VecDeque::new(),
            stats: PipelineStats {
                frames_decoded: 0,
                keyframes: 0,
                bytes_processed: 0,
                current_fps: 0.0,
                state: PipelineState::Idle,
            },
            width,
            height,
            current_pts_ms: 0,
            target_fps: 30.0,
            last_frame_time_ms: 0,
            dropped_frames: 0,
        }
    }

    /// Inicializa con un MediaSource existente
    pub fn with_source(&mut self, duration_ms: u64) -> u32 {
        let mut mgr = MediaSourceManager::new();
        let id = mgr.create_source(std::time::Duration::from_millis(duration_ms));
        mgr.get_mut(id).unwrap().open();
        mgr.get_mut(id).unwrap().add_source_buffer("video/mp4", "avc1.42E01E");
        mgr.get_mut(id).unwrap().add_source_buffer("audio/mp4", "mp4a.40.2");
        self.source = mgr.get(id).cloned();
        self.stats.state = PipelineState::Ready;
        id
    }

    /// Feed bytes H.264 codificados al pipeline
    pub fn feed_encoded(&mut self, data: &[u8]) -> u32 {
        self.stats.state = PipelineState::Loading;
        self.stats.bytes_processed += data.len() as u64;
        let h264_frames = self.h264.feed(data);
        let mut decoded_count = 0;
        for frame in h264_frames {
            if let Some(decoded) = self.process_h264_frame(frame) {
                self.pending_frames.push_back(decoded);
                decoded_count += 1;
            }
        }
        if !self.pending_frames.is_empty() {
            self.stats.state = PipelineState::Ready;
        }
        decoded_count
    }

    /// Procesa un H.264 frame: genera YUV, convierte a RGB, sube a texture
    fn process_h264_frame(&mut self, frame: H264Frame) -> Option<DecodedFrame> {
        let width = if frame.width > 0 { frame.width } else { self.width };
        let height = if frame.height > 0 { frame.height } else { self.height };
        if width != self.width || height != self.height {
            self.width = width;
            self.height = height;
            self.rgb_converter = Some(RgbConverter::new(width, height));
        }
        let yuv = self.generate_yuv_for_frame(width, height);
        let rgb_converter = self.rgb_converter.as_ref().unwrap();
        let rgb = rgb_converter.convert_to_rgba(&yuv);
        let texture_id = self.upload_frame_to_texture(&rgb, width, height);
        self.stats.frames_decoded += 1;
        if frame.is_keyframe {
            self.stats.keyframes += 1;
        }
        self.current_pts_ms = frame.pts;
        Some(DecodedFrame {
            texture_id,
            width,
            height,
            pts_ms: frame.pts,
            is_keyframe: frame.is_keyframe,
            rgb_data: rgb,
        })
    }

    /// Genera un YUV frame de prueba (placeholder hasta tener decoder real)
    fn generate_yuv_for_frame(&self, width: u32, height: u32) -> YuvFrame {
        let mut yuv = YuvFrame::new(width, height);
        let frame_idx = self.stats.frames_decoded;
        for y in 0..height {
            for x in 0..width {
                let i = (y * yuv.y_stride + x) as usize;
                // Patrón animado según frame index
                let phase = (frame_idx as f32 * 0.05) % 1.0;
                let v = ((x as f32 / width as f32 + phase).sin() * 80.0 + 128.0) as u8;
                yuv.y[i] = v;
            }
        }
        yuv
    }

    /// Sube un frame RGB a texture
    fn upload_frame_to_texture(&mut self, rgb: &[u8], width: u32, height: u32) -> u32 {
        if self.textures.count() == 0 {
            let id = self.textures.create(width, height);
            self.textures.get_mut(id).unwrap().upload_rgba(rgb, self.current_pts_ms as u64).unwrap();
            id
        } else {
            // Reuse texture 0 (ring buffer de 3 textures)
            let id = 1;
            if let Some(tex) = self.textures.get_mut(id) {
                if tex.width != width || tex.height != height {
                    *tex = VideoTexture::new(id, width, height);
                }
                let _ = tex.upload_rgba(rgb, self.current_pts_ms as u64);
            }
            id
        }
    }

    /// Inicializa un ring buffer de 3 textures
    pub fn init_textures(&mut self, count: u32) {
        for _ in 0..count {
            self.textures.create(self.width, self.height);
        }
    }

    /// Obtiene el siguiente frame pendiente
    pub fn next_frame(&mut self) -> Option<DecodedFrame> {
        self.pending_frames.pop_front()
    }

    /// Peek el siguiente frame sin consumirlo
    pub fn peek_frame(&self) -> Option<&DecodedFrame> {
        self.pending_frames.front()
    }

    /// Cantidad de frames pendientes
    pub fn pending_count(&self) -> usize {
        self.pending_frames.len()
    }

    /// Skip frames hasta el PTS target
    pub fn skip_to_pts(&mut self, target_pts_ms: i64) {
        while let Some(frame) = self.pending_frames.front() {
            if frame.pts_ms < target_pts_ms {
                self.dropped_frames += 1;
                self.pending_frames.pop_front();
            } else {
                break;
            }
        }
    }

    /// Marca como playing
    pub fn play(&mut self) {
        if self.stats.state == PipelineState::Ready || self.stats.state == PipelineState::Paused {
            self.stats.state = PipelineState::Playing;
        }
    }

    /// Marca como paused
    pub fn pause(&mut self) {
        if self.stats.state == PipelineState::Playing {
            self.stats.state = PipelineState::Paused;
        }
    }

    /// Seek a un tiempo (ms)
    pub fn seek(&mut self, pts_ms: i64) {
        self.current_pts_ms = pts_ms;
        self.skip_to_pts(pts_ms);
    }

    /// Reset pipeline
    pub fn reset(&mut self) {
        self.pending_frames.clear();
        self.stats.frames_decoded = 0;
        self.stats.keyframes = 0;
        self.stats.bytes_processed = 0;
        self.stats.state = PipelineState::Idle;
        self.h264 = H264StreamParser::new();
        self.current_pts_ms = 0;
        self.dropped_frames = 0;
    }

    /// Calcula FPS actual
    pub fn calculate_fps(&mut self, elapsed_ms: i64) {
        if elapsed_ms > 0 {
            self.stats.current_fps = (self.stats.frames_decoded as f32 * 1000.0) / elapsed_ms as f32;
        }
    }

    pub fn get_stats(&self) -> PipelineStats {
        self.stats
    }

    /// Decodifica un chunk pequeño de MP4-like data: SPS + PPS + IDR slice
    /// Usado para testing sin necesidad de MP4 real
    pub fn feed_test_chunk(&mut self, frame_number: u32) -> Option<DecodedFrame> {
        let pts = frame_number as i64 * 33; // 30 fps
        self.h264.decoder.pts = pts;
        self.h264.decoder.dts = pts;
        self.h264.decoder.frames_decoded = frame_number;
        self.h264.decoder.keyframes = frame_number / 30;
        let frame = H264Frame {
            is_idr: true,
            is_keyframe: true,
            is_reference: true,
            width: self.width,
            height: self.height,
            pts,
            dts: pts,
        };
        let decoded = self.process_h264_frame(frame);
        if let Some(d) = &decoded {
            self.pending_frames.push_back(d.clone());
            self.stats.state = PipelineState::Ready;
        }
        decoded
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_new() {
        let p = MediaPipeline::new(1920, 1080);
        assert_eq!(p.width, 1920);
        assert_eq!(p.stats.state, PipelineState::Idle);
    }

    #[test]
    fn test_pipeline_with_source() {
        let mut p = MediaPipeline::new(1280, 720);
        let id = p.with_source(60_000);
        assert!(p.source.is_some());
        assert_eq!(p.source.as_ref().unwrap().id, id);
    }

    #[test]
    fn test_pipeline_with_source_open() {
        let mut p = MediaPipeline::new(1280, 720);
        p.with_source(60_000);
        assert_eq!(p.source.as_ref().unwrap().state, super::super::mse::SourceState::Open);
    }

    #[test]
    fn test_pipeline_with_source_buffers() {
        let mut p = MediaPipeline::new(1280, 720);
        p.with_source(60_000);
        let src = p.source.as_ref().unwrap();
        assert_eq!(src.source_buffers.len(), 2); // video + audio
    }

    #[test]
    fn test_feed_encoded_sps_pps_idr() {
        let mut p = MediaPipeline::new(1280, 720);
        // SPS NAL
        let sps_data = vec![0x00, 0x00, 0x00, 0x01, 0x67, 0x42, 0x00, 0x1E, 0xAB];
        let pps_data = vec![0x00, 0x00, 0x00, 0x01, 0x68, 0xCE, 0x38, 0x80];
        let idr_data = vec![0x00, 0x00, 0x00, 0x01, 0x65, 0xFF, 0xFF, 0xFF];
        p.feed_encoded(&sps_data);
        p.feed_encoded(&pps_data);
        let n = p.feed_encoded(&idr_data);
        assert_eq!(n, 1);
    }

    #[test]
    fn test_next_frame() {
        let mut p = MediaPipeline::new(1280, 720);
        let sps = vec![0x00, 0x00, 0x00, 0x01, 0x67, 0x42, 0x00, 0x1E, 0xAB];
        let pps = vec![0x00, 0x00, 0x00, 0x01, 0x68, 0xCE, 0x38, 0x80];
        let idr = vec![0x00, 0x00, 0x00, 0x01, 0x65, 0xFF, 0xFF, 0xFF];
        p.feed_encoded(&sps);
        p.feed_encoded(&pps);
        p.feed_encoded(&idr);
        let frame = p.next_frame();
        assert!(frame.is_some());
    }

    #[test]
    fn test_pending_count() {
        let mut p = MediaPipeline::new(1280, 720);
        let sps = vec![0x00, 0x00, 0x00, 0x01, 0x67, 0x42, 0x00, 0x1E, 0xAB];
        let pps = vec![0x00, 0x00, 0x00, 0x01, 0x68, 0xCE, 0x38, 0x80];
        let idr = vec![0x00, 0x00, 0x00, 0x01, 0x65, 0xFF, 0xFF, 0xFF];
        p.feed_encoded(&sps);
        p.feed_encoded(&pps);
        p.feed_encoded(&idr);
        assert_eq!(p.pending_count(), 1);
        p.next_frame();
        assert_eq!(p.pending_count(), 0);
    }

    #[test]
    fn test_peek_frame() {
        let mut p = MediaPipeline::new(1280, 720);
        p.feed_test_chunk(0);
        let frame = p.peek_frame();
        assert!(frame.is_some());
        assert_eq!(p.pending_count(), 1);
    }

    #[test]
    fn test_play_pause() {
        let mut p = MediaPipeline::new(1280, 720);
        p.feed_test_chunk(0);
        p.play();
        assert_eq!(p.stats.state, PipelineState::Playing);
        p.pause();
        assert_eq!(p.stats.state, PipelineState::Paused);
    }

    #[test]
    fn test_seek() {
        let mut p = MediaPipeline::new(1280, 720);
        for i in 0..10 {
            p.feed_test_chunk(i);
        }
        p.seek(100);
        assert_eq!(p.current_pts_ms, 100);
    }

    #[test]
    fn test_reset() {
        let mut p = MediaPipeline::new(1280, 720);
        p.feed_test_chunk(0);
        p.feed_test_chunk(1);
        p.reset();
        assert_eq!(p.stats.frames_decoded, 0);
        assert_eq!(p.pending_count(), 0);
        assert_eq!(p.stats.state, PipelineState::Idle);
    }

    #[test]
    fn test_init_textures() {
        let mut p = MediaPipeline::new(1280, 720);
        p.init_textures(3);
        assert_eq!(p.textures.count(), 3);
    }

    #[test]
    fn test_skip_to_pts() {
        let mut p = MediaPipeline::new(1280, 720);
        for i in 0..10 {
            p.feed_test_chunk(i);
        }
        p.skip_to_pts(200);
        // 200/33 ≈ frame 6, así que quedan frames con pts >= 200
        assert!(p.dropped_frames > 0);
        assert!(p.pending_count() < 10);
    }

    #[test]
    fn test_calculate_fps() {
        let mut p = MediaPipeline::new(1280, 720);
        p.stats.frames_decoded = 30;
        p.calculate_fps(1000);
        assert!((p.stats.current_fps - 30.0).abs() < 0.1);
    }

    #[test]
    fn test_get_stats() {
        let p = MediaPipeline::new(1280, 720);
        let s = p.get_stats();
        assert_eq!(s.state, PipelineState::Idle);
    }

    #[test]
    fn test_decoded_frame() {
        let mut p = MediaPipeline::new(64, 64);
        p.feed_test_chunk(0);
        let frame = p.next_frame().unwrap();
        assert_eq!(frame.width, 64);
        assert!(frame.is_keyframe);
        assert!(!frame.rgb_data.is_empty());
    }

    #[test]
    fn test_texture_id_assigned() {
        let mut p = MediaPipeline::new(64, 64);
        p.feed_test_chunk(0);
        let frame = p.next_frame().unwrap();
        assert!(frame.texture_id > 0);
    }

    #[test]
    fn test_pts_increases() {
        let mut p = MediaPipeline::new(64, 64);
        p.feed_test_chunk(0);
        let f0 = p.next_frame().unwrap();
        p.feed_test_chunk(1);
        let f1 = p.next_frame().unwrap();
        assert!(f1.pts_ms > f0.pts_ms);
    }

    #[test]
    fn test_calculate_fps_zero() {
        let mut p = MediaPipeline::new(64, 64);
        p.calculate_fps(0);
        assert_eq!(p.stats.current_fps, 0.0);
    }

    #[test]
    fn test_resize_after_sps() {
        let mut p = MediaPipeline::new(1280, 720);
        let sps = vec![0x00, 0x00, 0x00, 0x01, 0x67, 0x42, 0x00, 0x1E, 0xAB];
        p.feed_encoded(&sps);
        // El SPS no cambia width/height en nuestro parser simple
        // pero el rgb_converter debe seguir siendo válido
        assert!(p.rgb_converter.is_some());
    }

    #[test]
    fn test_pending_frames_fifo() {
        let mut p = MediaPipeline::new(64, 64);
        for i in 0..5 {
            p.feed_test_chunk(i);
        }
        assert_eq!(p.pending_count(), 5);
        let first = p.next_frame().unwrap();
        assert_eq!(first.pts_ms, 0);
        let second = p.next_frame().unwrap();
        assert_eq!(second.pts_ms, 33);
    }

    #[test]
    fn test_bytes_processed() {
        let mut p = MediaPipeline::new(64, 64);
        let data = vec![0u8; 1000];
        p.feed_encoded(&data);
        assert_eq!(p.stats.bytes_processed, 1000);
    }
}
