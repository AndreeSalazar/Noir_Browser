//! GPU Window Surface (FASE D1)
//!
//! Conecta wgpu a la ventana real (vía winit + wgpu::Surface).
//! Encapsula:
//! - Crear surface desde winit::Window
//! - Configurar swapchain
//! - Resize handling
//! - Frame present
//!
//! Inspirado en el pipeline de Chrome compositor + Firefox WebRender.

use std::sync::Arc;

use crate::webgpu::gpu_renderer::{AdapterInfo, GpuState};

/// Configuracion de la surface GPU
#[derive(Debug, Clone)]
pub struct SurfaceConfig {
    pub width: u32,
    pub height: u32,
    pub format: wgpu::TextureFormat,
    pub present_mode: wgpu::PresentMode,
    pub alpha_mode: wgpu::CompositeAlphaMode,
    pub usage: wgpu::TextureUsages,
}

impl Default for SurfaceConfig {
    fn default() -> Self {
        Self {
            width: 800,
            height: 600,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            present_mode: wgpu::PresentMode::Fifo,  // VSync
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        }
    }
}

impl SurfaceConfig {
    pub fn from_size(width: u32, height: u32) -> Self {
        Self {
            width, height,
            ..Default::default()
        }
    }
}

/// Surface GPU (wrapper tipado para no exponer wgpu directamente)
pub struct WindowSurface {
    pub surface: Arc<wgpu::Surface<'static>>,
    pub config: SurfaceConfig,
    pub gpu_state: Arc<GpuState>,
}

impl WindowSurface {
    /// Configurar la surface con un tamano
    pub fn configure_surface(
        surface: &wgpu::Surface,
        gpu_state: &GpuState,
        width: u32,
        height: u32,
    ) -> SurfaceConfig {
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: width.max(1),
            height: height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&gpu_state.device, &config);
        SurfaceConfig {
            width,
            height,
            format: config.format,
            present_mode: config.present_mode,
            alpha_mode: config.alpha_mode,
            usage: config.usage,
        }
    }

    /// Reconfigurar (resize)
    pub fn resize(&mut self, width: u32, height: u32) {
        self.config = Self::configure_surface(&self.surface, &self.gpu_state, width, height);
    }

    /// Tamano actual
    pub fn size(&self) -> (u32, u32) {
        (self.config.width, self.config.height)
    }
}

/// Swapchain info para monitoring
#[derive(Debug, Clone, Copy)]
pub struct SwapchainInfo {
    pub width: u32,
    pub height: u32,
    pub format: wgpu::TextureFormat,
    pub present_mode: wgpu::PresentMode,
}

impl SwapchainInfo {
    pub fn from_config(config: &SurfaceConfig) -> Self {
        Self {
            width: config.width,
            height: config.height,
            format: config.format,
            present_mode: config.present_mode,
        }
    }
}

/// Adapter info helper
pub fn get_adapter_info(adapter: &wgpu::Adapter) -> AdapterInfo {
    let info = adapter.get_info();
    AdapterInfo {
        name: info.name,
        vendor: format!("{:?}", info.vendor),
        backend: format!("{:?}", info.backend),
    }
}

/// Frame info - describe el frame en curso
#[derive(Debug, Clone, Copy)]
pub struct FrameInfo {
    pub frame_number: u64,
    pub width: u32,
    pub height: u32,
    pub time_ms: u64,
}

/// Manager de frames (para tracking de FPS y frameskip)
#[derive(Debug, Default)]
pub struct FrameManager {
    pub frame_number: u64,
    pub last_frame_ms: u64,
    pub total_present_time_us: u64,
    pub present_count: u64,
}

impl FrameManager {
    pub fn new() -> Self { Self::default() }

    /// Llamar al inicio de un nuevo frame
    pub fn begin_frame(&mut self, time_ms: u64) -> FrameInfo {
        self.frame_number += 1;
        let info = FrameInfo {
            frame_number: self.frame_number,
            width: 0,
            height: 0,
            time_ms,
        };
        info
    }

    /// Llamar al final del frame
    pub fn end_frame(&mut self, duration_us: u64) {
        self.total_present_time_us += duration_us;
        self.present_count += 1;
    }

    pub fn avg_frame_time_us(&self) -> u64 {
        if self.present_count == 0 {
            0
        } else {
            self.total_present_time_us / self.present_count
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_surface_config_default() {
        let c = SurfaceConfig::default();
        assert_eq!(c.width, 800);
        assert_eq!(c.height, 600);
        assert_eq!(c.present_mode, wgpu::PresentMode::Fifo);
    }

    #[test]
    fn test_surface_config_from_size() {
        let c = SurfaceConfig::from_size(1920, 1080);
        assert_eq!(c.width, 1920);
        assert_eq!(c.height, 1080);
    }

    #[test]
    fn test_swapchain_info() {
        let c = SurfaceConfig::from_size(800, 600);
        let info = SwapchainInfo::from_config(&c);
        assert_eq!(info.width, 800);
        assert_eq!(info.height, 600);
        assert_eq!(info.format, wgpu::TextureFormat::Bgra8UnormSrgb);
    }

    #[test]
    fn test_swapchain_present_mode() {
        let mut c = SurfaceConfig::default();
        c.present_mode = wgpu::PresentMode::Mailbox;
        let info = SwapchainInfo::from_config(&c);
        assert_eq!(info.present_mode, wgpu::PresentMode::Mailbox);
    }

    #[test]
    fn test_config_min_size() {
        let c = SurfaceConfig::from_size(0, 0);
        assert_eq!(c.width, 0);
    }

    #[test]
    fn test_alpha_modes() {
        let c = SurfaceConfig::default();
        assert_eq!(c.alpha_mode, wgpu::CompositeAlphaMode::Auto);
    }

    #[test]
    fn test_frame_manager_creation() {
        let m = FrameManager::new();
        assert_eq!(m.frame_number, 0);
        assert_eq!(m.present_count, 0);
    }

    #[test]
    fn test_frame_manager_begin() {
        let mut m = FrameManager::new();
        let info = m.begin_frame(100);
        assert_eq!(info.frame_number, 1);
        assert_eq!(info.time_ms, 100);
    }

    #[test]
    fn test_frame_manager_end() {
        let mut m = FrameManager::new();
        m.begin_frame(0);
        m.end_frame(1000);
        m.begin_frame(16);
        m.end_frame(2000);
        assert_eq!(m.present_count, 2);
        assert_eq!(m.avg_frame_time_us(), 1500);
    }

    #[test]
    fn test_frame_manager_avg_zero() {
        let m = FrameManager::new();
        assert_eq!(m.avg_frame_time_us(), 0);
    }
}
