//! WebGPU Renderer Integration
//!
//! Integrates WebGPU as the optional GPU renderer for Noir Browser.
//! Falls back to softbuffer (CPU) if WebGPU is not available.

use crate::webgpu::{Device, GpuBackend, JsBridge, Renderer as WgpuRenderer};

/// Integrated renderer that can use WebGPU or fall back to CPU
pub struct IntegratedRenderer {
    pub webgpu_available: bool,
    pub webgpu_renderer: Option<WgpuRenderer>,
    pub webgpu_bridge: Option<JsBridge>,
    pub frame_count: u64,
}

impl IntegratedRenderer {
    /// Create a new integrated renderer
    pub fn new() -> Self {
        let bridge = JsBridge::new();
        let device = Device::new(
            "Primary GPU",
            "Auto-detected",
            GpuBackend::WebGPU,
        );

        let renderer = WgpuRenderer::new(device);

        // Check if WebGPU is actually available (not fallback)
        let webgpu_available = bridge.is_available() && !bridge.renderer.lock().unwrap().device.is_fallback;

        Self {
            webgpu_available,
            webgpu_renderer: Some(renderer),
            webgpu_bridge: Some(bridge),
            frame_count: 0,
        }
    }

    /// Initialize WebGPU rendering
    pub fn init_webgpu(&mut self) -> Result<(), String> {
        if let Some(renderer) = &mut self.webgpu_renderer {
            renderer.init();
        }
        Ok(())
    }

    /// Begin a new frame
    pub fn begin_frame(&mut self) {
        if let Some(renderer) = &mut self.webgpu_renderer {
            renderer.begin_frame();
        }
        self.frame_count += 1;
    }

    /// End frame and present
    pub fn end_frame(&mut self) {
        if let Some(renderer) = &mut self.webgpu_renderer {
            renderer.end_frame();
        }
    }

    /// Draw a colored rectangle
    pub fn draw_rect(&mut self, x: f32, y: f32, w: f32, h: f32, r: f32, g: f32, b: f32, a: f32) {
        if let Some(renderer) = &mut self.webgpu_renderer {
            renderer.draw_rect(x, y, w, h, r, g, b, a);
        }
    }

    /// Draw text
    pub fn draw_text(&mut self, x: f32, y: f32, text: &str, color: [f32; 4], size: f32) {
        if let Some(renderer) = &mut self.webgpu_renderer {
            renderer.draw_text(x, y, text, color, size);
        }
    }

    /// Get GPU info
    pub fn get_gpu_info(&self) -> String {
        if let Some(bridge) = &self.webgpu_bridge {
            bridge.get_info()
        } else {
            "WebGPU not available".to_string()
        }
    }

    /// Get statistics
    pub fn get_stats(&self) -> RendererStats {
        let (draw_calls, triangles) = if let Some(renderer) = &self.webgpu_renderer {
            let stats = renderer.stats();
            (stats.draw_calls, stats.triangles_rendered)
        } else {
            (0, 0)
        };

        RendererStats {
            webgpu_available: self.webgpu_available,
            frame_count: self.frame_count,
            draw_calls,
            triangles_rendered: triangles,
            gpu_info: self.get_gpu_info(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RendererStats {
    pub webgpu_available: bool,
    pub frame_count: u64,
    pub draw_calls: u64,
    pub triangles_rendered: u64,
    pub gpu_info: String,
}

impl Default for IntegratedRenderer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integrated_renderer_creation() {
        let renderer = IntegratedRenderer::new();
        // Should always be available (with fallback if no GPU)
        assert!(renderer.webgpu_renderer.is_some());
        assert!(renderer.webgpu_bridge.is_some());
    }

    #[test]
    fn test_renderer_init() {
        let mut renderer = IntegratedRenderer::new();
        assert!(renderer.init_webgpu().is_ok());
    }

    #[test]
    fn test_renderer_frame() {
        let mut renderer = IntegratedRenderer::new();
        renderer.init_webgpu().unwrap();
        renderer.begin_frame();
        renderer.draw_rect(0.0, 0.0, 100.0, 100.0, 1.0, 0.0, 0.0, 1.0);
        renderer.end_frame();
        let stats = renderer.get_stats();
        assert_eq!(stats.frame_count, 1);
        assert!(stats.draw_calls > 0);
    }

    #[test]
    fn test_renderer_stats() {
        let renderer = IntegratedRenderer::new();
        let stats = renderer.get_stats();
        assert!(!stats.gpu_info.is_empty());
    }
}
