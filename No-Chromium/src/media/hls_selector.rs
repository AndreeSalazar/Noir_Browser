//! HLS Variant Selector (FASE E3)
//!
//! Selecciona automaticamente la mejor variant de HLS segun:
//! - Bandwidth estimado de la red
//! - Resolucion del viewport
//! - Capacidad del device
//!
//! Inspirado en:
//! - HLS Authoring Specification
//! - ABR (Adaptive Bitrate) algorithms de Netflix/YouTube

use crate::media::hls::StreamVariant;

/// Nivel de calidad seleccionado
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QualityLevel {
    Low,        // < 500 kbps
    Medium,     // 500-1500 kbps
    High,       // 1500-3000 kbps
    VeryHigh,   // 3000-6000 kbps
    Ultra,      // > 6000 kbps
}

impl QualityLevel {
    pub fn from_bandwidth(bw: u32) -> Self {
        match bw {
            0..=500_000 => QualityLevel::Low,
            500_001..=1_500_000 => QualityLevel::Medium,
            1_500_001..=3_000_000 => QualityLevel::High,
            3_000_001..=6_000_000 => QualityLevel::VeryHigh,
            _ => QualityLevel::Ultra,
        }
    }
    pub fn as_str(&self) -> &'static str {
        match self {
            QualityLevel::Low => "low",
            QualityLevel::Medium => "medium",
            QualityLevel::High => "high",
            QualityLevel::VeryHigh => "very-high",
            QualityLevel::Ultra => "ultra",
        }
    }
}

/// Estimador de bandwidth
#[derive(Debug, Default)]
pub struct BandwidthEstimator {
    samples: Vec<(u64, u64)>,  // (timestamp_ms, bytes)
    window_ms: u64,
    pub estimated_kbps: u32,
}

impl BandwidthEstimator {
    pub fn new() -> Self { Self::default() }

    /// Agregar muestra de download
    pub fn add_sample(&mut self, timestamp_ms: u64, bytes: u64) {
        self.samples.push((timestamp_ms, bytes));
        // Eliminar muestras fuera de la ventana
        self.samples.retain(|(ts, _)| timestamp_ms - *ts <= self.window_ms);
        self.recalculate();
    }

    fn recalculate(&mut self) {
        if self.samples.len() < 2 {
            return;
        }
        let first = self.samples.first().unwrap().0;
        let last = self.samples.last().unwrap().0;
        let total_bytes: u64 = self.samples.iter().map(|(_, b)| *b).sum();
        let duration_ms = last - first;
        if duration_ms == 0 {
            return;
        }
        let bps = (total_bytes * 8 * 1000) / duration_ms;
        self.estimated_kbps = (bps / 1000) as u32;
    }

    /// Set ventana de tiempo
    pub fn set_window(&mut self, ms: u64) {
        self.window_ms = ms;
    }
}

/// Estimador de capacidad del device
#[derive(Debug, Clone, Copy)]
pub struct DeviceCapabilities {
    pub max_resolution_w: u32,
    pub max_resolution_h: u32,
    pub hw_decoding: bool,
    pub hdr_support: bool,
}

impl Default for DeviceCapabilities {
    fn default() -> Self {
        Self {
            max_resolution_w: 3840,
            max_resolution_h: 2160,
            hw_decoding: true,
            hdr_support: false,
        }
    }
}

/// Selector de variant HLS
pub struct HlsVariantSelector {
    pub bandwidth_estimator: BandwidthEstimator,
    pub device_caps: DeviceCapabilities,
    pub viewport_w: u32,
    pub viewport_h: u32,
    pub target_quality: QualityLevel,
    pub safety_factor: f32,  // Multiplicar bandwidth por <1 para safety
    pub last_selected_idx: Option<usize>,
}

impl HlsVariantSelector {
    pub fn new(viewport_w: u32, viewport_h: u32) -> Self {
        Self {
            bandwidth_estimator: BandwidthEstimator::new(),
            device_caps: DeviceCapabilities::default(),
            viewport_w,
            viewport_h,
            target_quality: QualityLevel::Medium,
            safety_factor: 0.9,
            last_selected_idx: None,
        }
    }

    /// Calcular bandwidth objetivo segun calidad
    fn target_bandwidth_for(&self, level: QualityLevel) -> u32 {
        match level {
            QualityLevel::Low => 300_000,
            QualityLevel::Medium => 1_000_000,
            QualityLevel::High => 2_500_000,
            QualityLevel::VeryHigh => 5_000_000,
            QualityLevel::Ultra => 10_000_000,
        }
    }

    /// Seleccionar la mejor variant
    pub fn select(&mut self, variants: &[StreamVariant]) -> Option<usize> {
        if variants.is_empty() {
            return None;
        }

        // Si no hay estimate, usar la del medio
        let estimated_kbps = if self.bandwidth_estimator.estimated_kbps > 0 {
            (self.bandwidth_estimator.estimated_kbps as f32 * self.safety_factor) as u32
        } else {
            1_500_000  // Default
        };

        // Encontrar la mejor variant que:
        // 1. Estime por debajo de nuestro bandwidth
        // 2. Estime por debajo de la capacidad del device
        // 3. Resolucion <= viewport

        let max_pixels = (self.viewport_w * self.viewport_h) as u64;
        let device_pixels = (self.device_caps.max_resolution_w * self.device_caps.max_resolution_h) as u64;
        let effective_max = max_pixels.min(device_pixels);

        let mut best: Option<(usize, u32)> = None;
        for (i, v) in variants.iter().enumerate() {
            // Skip si excede capacidad del device
            let variant_pixels = (v.width as u64) * (v.height as u64);
            if variant_pixels > effective_max {
                continue;
            }
            // Skip si excede bandwidth
            if v.bandwidth > estimated_kbps {
                continue;
            }
            // Mejor = mayor bandwidth dentro del limite
            if best.is_none() || v.bandwidth > best.unwrap().1 {
                best = Some((i, v.bandwidth));
            }
        }

        // Si no encontro nada, usar la mas baja
        if best.is_none() {
            let min = variants.iter().enumerate()
                .min_by_key(|(_, v)| v.bandwidth);
            best = min.map(|(i, v)| (i, v.bandwidth));
        }

        let idx = best.map(|(i, _)| i);
        if let Some(i) = idx {
            self.last_selected_idx = Some(i);
            self.target_quality = QualityLevel::from_bandwidth(variants[i].bandwidth);
        }
        idx
    }

    /// Cambiar manualmente a una calidad especifica
    pub fn set_quality(&mut self, level: QualityLevel) {
        self.target_quality = level;
    }

    /// Auto-quality segun bandwidth
    pub fn auto_quality(&mut self) {
        self.target_quality = QualityLevel::from_bandwidth(self.bandwidth_estimator.estimated_kbps);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_variants() -> Vec<StreamVariant> {
        vec![
            StreamVariant {
                bandwidth: 300_000,
                avg_bandwidth: 0,
                codecs: "avc1.42c00d".into(),
                resolution: "320x180".into(),
                width: 320, height: 180,
                frame_rate: 30.0,
                uri: "low.m3u8".into(),
                audio: "audio.m3u8".into(),
                subtitles: String::new(),
            },
            StreamVariant {
                bandwidth: 1_000_000,
                avg_bandwidth: 0,
                codecs: "avc1.42c01e".into(),
                resolution: "640x360".into(),
                width: 640, height: 360,
                frame_rate: 30.0,
                uri: "med.m3u8".into(),
                audio: "audio.m3u8".into(),
                subtitles: String::new(),
            },
            StreamVariant {
                bandwidth: 3_000_000,
                avg_bandwidth: 0,
                codecs: "avc1.42c01f".into(),
                resolution: "1280x720".into(),
                width: 1280, height: 720,
                frame_rate: 30.0,
                uri: "high.m3u8".into(),
                audio: "audio.m3u8".into(),
                subtitles: String::new(),
            },
            StreamVariant {
                bandwidth: 6_000_000,
                avg_bandwidth: 0,
                codecs: "avc1.42c028".into(),
                resolution: "1920x1080".into(),
                width: 1920, height: 1080,
                frame_rate: 30.0,
                uri: "uhd.m3u8".into(),
                audio: "audio.m3u8".into(),
                subtitles: String::new(),
            },
        ]
    }

    #[test]
    fn test_quality_level_from_bandwidth() {
        assert_eq!(QualityLevel::from_bandwidth(100_000), QualityLevel::Low);
        assert_eq!(QualityLevel::from_bandwidth(1_000_000), QualityLevel::Medium);
        assert_eq!(QualityLevel::from_bandwidth(2_000_000), QualityLevel::High);
        assert_eq!(QualityLevel::from_bandwidth(5_000_000), QualityLevel::VeryHigh);
        assert_eq!(QualityLevel::from_bandwidth(10_000_000), QualityLevel::Ultra);
    }

    #[test]
    fn test_quality_level_str() {
        assert_eq!(QualityLevel::Low.as_str(), "low");
        assert_eq!(QualityLevel::Ultra.as_str(), "ultra");
    }

    #[test]
    fn test_bandwidth_estimator_creation() {
        let e = BandwidthEstimator::new();
        assert_eq!(e.estimated_kbps, 0);
    }

    #[test]
    fn test_bandwidth_estimator_add_sample() {
        let mut e = BandwidthEstimator::new();
        e.set_window(1000);
        e.add_sample(0, 100_000);  // 100KB en t=0
        e.add_sample(1000, 100_000);  // otro 100KB en t=1000ms
        // Total 200KB en 1000ms = 200KB/s = 1600 kbps
        assert!(e.estimated_kbps > 1000);
    }

    #[test]
    fn test_bandwidth_estimator_window() {
        let mut e = BandwidthEstimator::new();
        e.set_window(500);
        e.add_sample(0, 1000);
        e.add_sample(1000, 1000);  // fuera de ventana
        // Solo 1 muestra valida, no calcula
        assert_eq!(e.estimated_kbps, 0);
    }

    #[test]
    fn test_device_capabilities_default() {
        let c = DeviceCapabilities::default();
        assert_eq!(c.max_resolution_w, 3840);
        assert!(c.hw_decoding);
    }

    #[test]
    fn test_selector_creation() {
        let s = HlsVariantSelector::new(1920, 1080);
        assert_eq!(s.viewport_w, 1920);
    }

    #[test]
    fn test_selector_picks_low_for_low_bw() {
        let mut s = HlsVariantSelector::new(1920, 1080);
        // 300kbps bandwidth
        s.bandwidth_estimator.add_sample(0, 30_000);
        s.bandwidth_estimator.add_sample(1000, 30_000);
        let idx = s.select(&make_variants());
        assert!(idx.is_some());
        // Debe elegir low (300kbps) o medium (1Mbps)
    }

    #[test]
    fn test_selector_picks_high_for_high_bw() {
        let mut s = HlsVariantSelector::new(1920, 1080);
        // 3Mbps bandwidth
        for i in 0..5 {
            s.bandwidth_estimator.add_sample(i * 100, 375_000);  // 3Mbps
        }
        let idx = s.select(&make_variants()).unwrap();
        let v = &make_variants()[idx];
        // Debe elegir high (3Mbps)
        assert!(v.bandwidth >= 1_000_000);
    }

    #[test]
    fn test_selector_no_variants() {
        let mut s = HlsVariantSelector::new(1920, 1080);
        let empty: Vec<StreamVariant> = vec![];
        assert!(s.select(&empty).is_none());
    }

    #[test]
    fn test_selector_respects_viewport() {
        let mut s = HlsVariantSelector::new(320, 240);  // viewport pequeno
        for i in 0..5 {
            s.bandwidth_estimator.add_sample(i * 100, 750_000);  // 6Mbps
        }
        let idx = s.select(&make_variants()).unwrap();
        let v = &make_variants()[idx];
        // Debe elegir variant que cabe en 320x240
        assert!(v.width <= 640);
    }

    #[test]
    fn test_selector_respects_device_caps() {
        let mut s = HlsVariantSelector::new(3840, 2160);
        s.device_caps.max_resolution_w = 1280;
        s.device_caps.max_resolution_h = 720;
        for i in 0..5 {
            s.bandwidth_estimator.add_sample(i * 100, 750_000);
        }
        let idx = s.select(&make_variants()).unwrap();
        let v = &make_variants()[idx];
        // No debe elegir 1920x1080
        assert!(v.width <= 1280);
    }

    #[test]
    fn test_set_quality() {
        let mut s = HlsVariantSelector::new(1920, 1080);
        s.set_quality(QualityLevel::Ultra);
        assert_eq!(s.target_quality, QualityLevel::Ultra);
    }

    #[test]
    fn test_auto_quality() {
        let mut s = HlsVariantSelector::new(1920, 1080);
        s.bandwidth_estimator.estimated_kbps = 4_000_000;
        s.auto_quality();
        assert_eq!(s.target_quality, QualityLevel::VeryHigh);
    }
}
