//! Browser Metrics (FASE A3)
//!
//! Rastrea rendimiento del navegador en tiempo real:
//! - RAM usage
//! - CPU usage estimado
//! - FPS
//! - Por-tab metrics
//! - HTTP request count, cache hit rate
//!
//! Util para DevTools y para mostrar al usuario.

use std::time::{Instant, Duration};

#[derive(Debug, Clone, Copy)]
pub struct FpsStats {
    pub current_fps: f32,
    pub avg_fps: f32,
    pub min_fps: f32,
    pub max_fps: f32,
    pub frame_count: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct MemoryStats {
    pub used_bytes: u64,
    pub total_bytes: u64,
    pub tab_count_bytes: u64,
    pub cache_bytes: u64,
    pub image_bytes: u64,
    pub css_bytes: u64,
    pub js_bytes: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct NetworkStats {
    pub requests_total: u64,
    pub requests_cached: u64,
    pub requests_failed: u64,
    pub bytes_downloaded: u64,
    pub avg_request_ms: f32,
}

#[derive(Debug)]
pub struct BrowserMetrics {
    pub start_time: Instant,
    pub last_frame: Instant,
    pub frame_times: Vec<Duration>,
    pub total_frames: u64,
    pub dropped_frames: u64,
    pub ram_bytes: u64,
    pub image_bytes: u64,
    pub css_bytes: u64,
    pub js_bytes: u64,
    pub cache_bytes: u64,
    pub requests_total: u64,
    pub requests_cached: u64,
    pub requests_failed: u64,
    pub bytes_downloaded: u64,
    pub request_times: Vec<Duration>,
    pub tabs_open: u32,
}

impl BrowserMetrics {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            last_frame: Instant::now(),
            frame_times: Vec::with_capacity(120),
            total_frames: 0,
            dropped_frames: 0,
            ram_bytes: 0,
            image_bytes: 0,
            css_bytes: 0,
            js_bytes: 0,
            cache_bytes: 0,
            requests_total: 0,
            requests_cached: 0,
            requests_failed: 0,
            bytes_downloaded: 0,
            request_times: Vec::with_capacity(100),
            tabs_open: 1,
        }
    }

    /// Llamar en cada frame de render
    pub fn record_frame(&mut self) {
        let now = Instant::now();
        let frame_time = now.duration_since(self.last_frame);
        self.last_frame = now;
        self.total_frames += 1;

        // Si un frame tomo mas de 33ms (30fps min), lo contamos como dropped
        if frame_time > Duration::from_millis(33) && self.total_frames > 1 {
            self.dropped_frames += 1;
        }

        // Mantener ultimos 120 frames
        if self.frame_times.len() >= 120 {
            self.frame_times.remove(0);
        }
        self.frame_times.push(frame_time);
    }

    /// Registra una request de red
    pub fn record_request(&mut self, time: Duration, size: u64, cached: bool, failed: bool) {
        self.requests_total += 1;
        if cached {
            self.requests_cached += 1;
        }
        if failed {
            self.requests_failed += 1;
        }
        self.bytes_downloaded += size;
        if self.request_times.len() >= 100 {
            self.request_times.remove(0);
        }
        self.request_times.push(time);
    }

    /// Actualiza uso de memoria
    pub fn update_memory(&mut self, ram: u64, images: u64, css: u64, js: u64, cache: u64) {
        self.ram_bytes = ram;
        self.image_bytes = images;
        self.css_bytes = css;
        self.js_bytes = js;
        self.cache_bytes = cache;
    }

    /// FPS actual (basado en los ultimos frames)
    pub fn fps_stats(&self) -> FpsStats {
        if self.frame_times.is_empty() {
            return FpsStats {
                current_fps: 0.0,
                avg_fps: 0.0,
                min_fps: 0.0,
                max_fps: 0.0,
                frame_count: self.total_frames,
            };
        }
        let current = self.frame_times.last().unwrap();
        let current_fps = 1.0 / current.as_secs_f32().max(0.001);

        let avg: f32 = self.frame_times.iter()
            .map(|d| 1.0 / d.as_secs_f32().max(0.001))
            .sum::<f32>() / self.frame_times.len() as f32;

        let min_fps = self.frame_times.iter()
            .map(|d| 1.0 / d.as_secs_f32().max(0.001))
            .fold(f32::MAX, f32::min);

        let max_fps = self.frame_times.iter()
            .map(|d| 1.0 / d.as_secs_f32().max(0.001))
            .fold(0.0_f32, f32::max);

        FpsStats {
            current_fps,
            avg_fps: avg,
            min_fps,
            max_fps,
            frame_count: self.total_frames,
        }
    }

    /// Memory stats
    pub fn memory_stats(&self) -> MemoryStats {
        MemoryStats {
            used_bytes: self.ram_bytes,
            total_bytes: self.ram_bytes,
            tab_count_bytes: self.image_bytes + self.css_bytes + self.js_bytes,
            cache_bytes: self.cache_bytes,
            image_bytes: self.image_bytes,
            css_bytes: self.css_bytes,
            js_bytes: self.js_bytes,
        }
    }

    /// Network stats
    pub fn network_stats(&self) -> NetworkStats {
        let avg_ms = if self.request_times.is_empty() {
            0.0
        } else {
            self.request_times.iter()
                .map(|d| d.as_millis() as f32)
                .sum::<f32>() / self.request_times.len() as f32
        };
        NetworkStats {
            requests_total: self.requests_total,
            requests_cached: self.requests_cached,
            requests_failed: self.requests_failed,
            bytes_downloaded: self.bytes_downloaded,
            avg_request_ms: avg_ms,
        }
    }

    /// Uptime
    pub fn uptime_secs(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }

    /// Drop rate
    pub fn drop_rate(&self) -> f32 {
        if self.total_frames == 0 {
            0.0
        } else {
            self.dropped_frames as f32 / self.total_frames as f32
        }
    }

    /// Cache hit rate
    pub fn cache_hit_rate(&self) -> f32 {
        if self.requests_total == 0 {
            0.0
        } else {
            self.requests_cached as f32 / self.requests_total as f32
        }
    }

    /// Format bytes to human readable
    pub fn format_bytes(bytes: u64) -> String {
        if bytes < 1024 {
            format!("{} B", bytes)
        } else if bytes < 1024 * 1024 {
            format!("{:.1} KB", bytes as f64 / 1024.0)
        } else if bytes < 1024 * 1024 * 1024 {
            format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
        } else {
            format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
        }
    }
}

impl Default for BrowserMetrics {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_creation() {
        let m = BrowserMetrics::new();
        assert_eq!(m.total_frames, 0);
    }

    #[test]
    fn test_record_frame() {
        let mut m = BrowserMetrics::new();
        m.record_frame();
        assert_eq!(m.total_frames, 1);
    }

    #[test]
    fn test_dropped_frames() {
        let mut m = BrowserMetrics::new();
        m.record_frame();
        std::thread::sleep(Duration::from_millis(50));  // > 33ms
        m.record_frame();
        assert!(m.dropped_frames >= 1);
    }

    #[test]
    fn test_record_request() {
        let mut m = BrowserMetrics::new();
        m.record_request(Duration::from_millis(100), 1024, false, false);
        m.record_request(Duration::from_millis(50), 512, true, false);
        m.record_request(Duration::from_millis(200), 0, false, true);
        let net = m.network_stats();
        assert_eq!(net.requests_total, 3);
        assert_eq!(net.requests_cached, 1);
        assert_eq!(net.requests_failed, 1);
        assert_eq!(net.bytes_downloaded, 1536);
    }

    #[test]
    fn test_update_memory() {
        let mut m = BrowserMetrics::new();
        m.update_memory(10_000_000, 1_000_000, 500_000, 2_000_000, 3_000_000);
        let mem = m.memory_stats();
        assert_eq!(mem.image_bytes, 1_000_000);
    }

    #[test]
    fn test_fps_stats() {
        let mut m = BrowserMetrics::new();
        m.record_frame();
        m.record_frame();
        let fps = m.fps_stats();
        assert!(fps.frame_count == 2);
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(BrowserMetrics::format_bytes(500), "500 B");
        assert_eq!(BrowserMetrics::format_bytes(2048), "2.0 KB");
        assert_eq!(BrowserMetrics::format_bytes(2 * 1024 * 1024), "2.0 MB");
    }

    #[test]
    fn test_cache_hit_rate() {
        let mut m = BrowserMetrics::new();
        m.record_request(Duration::from_millis(10), 100, true, false);
        m.record_request(Duration::from_millis(10), 100, true, false);
        m.record_request(Duration::from_millis(10), 100, false, false);
        let rate = m.cache_hit_rate();
        assert!((rate - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_drop_rate() {
        let mut m = BrowserMetrics::new();
        m.record_frame();
        m.dropped_frames = 1;
        m.total_frames = 4;
        assert!((m.drop_rate() - 0.25).abs() < 0.01);
    }

    #[test]
    fn test_uptime() {
        let m = BrowserMetrics::new();
        std::thread::sleep(Duration::from_millis(50));
        assert!(m.uptime_secs() < 1);
    }
}
