//! Renderer Trait - Abstracción de rendering
//! Permite cambiar entre CPU (softbuffer) y GPU (Vulkan) sin cambiar app/

pub trait Renderer {
    type Buffer;
    type Error;

    fn begin_frame(&mut self, width: u32, height: u32) -> Result<(), Self::Error>;
    fn end_frame(&mut self) -> Result<(), Self::Error>;
    fn clear(&mut self, color: u32) -> Result<(), Self::Error>;
    fn draw_rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: u32) -> Result<(), Self::Error>;
    fn draw_text(&mut self, x: i32, y: i32, text: &str, color: u32, scale: f32) -> Result<(), Self::Error>;
    fn draw_image(&mut self, x: i32, y: i32, w: i32, h: i32, data: &[u8], img_w: u32, img_h: u32) -> Result<(), Self::Error>;
    fn get_buffer(&mut self) -> &mut Self::Buffer;
}

/// Backend trait - abstrae el hardware (CPU vs GPU)
pub trait Backend {
    type Buffer;
    type Error;

    fn create_buffer(&mut self, width: u32, height: u32) -> Result<Self::Buffer, Self::Error>;
    fn present(&mut self, buffer: &Self::Buffer) -> Result<(), Self::Error>;
    fn resize(&mut self, width: u32, height: u32) -> Result<(), Self::Error>;
}

/// CPU Backend using softbuffer
pub struct CpuBackend {
    width: u32,
    height: u32,
}

impl CpuBackend {
    pub fn new() -> Self {
        Self { width: 0, height: 0 }
    }
}

impl Default for CpuBackend {
    fn default() -> Self {
        Self::new()
    }
}

/// CPU Renderer implementation
pub struct CpuRenderer {
    pub buffer: Vec<u32>,
    pub width: u32,
    pub height: u32,
}

impl CpuRenderer {
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width * height) as usize;
        Self {
            buffer: vec![0; size],
            width,
            height,
        }
    }
}

impl Renderer for CpuRenderer {
    type Buffer = Vec<u32>;
    type Error = String;

    fn begin_frame(&mut self, width: u32, height: u32) -> Result<(), Self::Error> {
        if self.width != width || self.height != height {
            self.width = width;
            self.height = height;
            self.buffer = vec![0; (width * height) as usize];
        }
        Ok(())
    }

    fn end_frame(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn clear(&mut self, color: u32) -> Result<(), Self::Error> {
        for pixel in &mut self.buffer {
            *pixel = color;
        }
        Ok(())
    }

    fn draw_rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: u32) -> Result<(), Self::Error> {
        for dy in 0..h {
            for dx in 0..w {
                let px = x + dx;
                let py = y + dy;
                if px >= 0 && py >= 0 && (px as u32) < self.width && (py as u32) < self.height {
                    self.buffer[(py as u32 * self.width + px as u32) as usize] = color;
                }
            }
        }
        Ok(())
    }

    fn draw_text(&mut self, x: i32, y: i32, text: &str, color: u32, scale: f32) -> Result<(), Self::Error> {
        // Placeholder - would use actual text rendering
        let _ = (x, y, text, color, scale);
        Ok(())
    }

    fn draw_image(&mut self, x: i32, y: i32, w: i32, h: i32, data: &[u8], img_w: u32, img_h: u32) -> Result<(), Self::Error> {
        let _ = (x, y, w, h, data, img_w, img_h);
        Ok(())
    }

    fn get_buffer(&mut self) -> &mut Self::Buffer {
        &mut self.buffer
    }
}

/// GPU Backend placeholder (Vulkan)
pub struct GpuBackend {
    pub initialized: bool,
}

impl GpuBackend {
    pub fn new() -> Self {
        Self { initialized: false }
    }
}

impl Default for GpuBackend {
    fn default() -> Self {
        Self::new()
    }
}

/// Factory function to create the appropriate backend
pub fn create_backend(use_gpu: bool) -> Box<dyn Backend<Buffer = Vec<u32>, Error = String>> {
    if use_gpu {
        // Would return Vulkan backend
        Box::new(CpuBackendWrapper::new())
    } else {
        Box::new(CpuBackendWrapper::new())
    }
}

pub struct CpuBackendWrapper {
    width: u32,
    height: u32,
}

impl CpuBackendWrapper {
    pub fn new() -> Self {
        Self { width: 0, height: 0 }
    }
}

impl Default for CpuBackendWrapper {
    fn default() -> Self {
        Self::new()
    }
}

impl Backend for CpuBackendWrapper {
    type Buffer = Vec<u32>;
    type Error = String;

    fn create_buffer(&mut self, width: u32, height: u32) -> Result<Self::Buffer, Self::Error> {
        self.width = width;
        self.height = height;
        Ok(vec![0; (width * height) as usize])
    }

    fn present(&mut self, _buffer: &Self::Buffer) -> Result<(), Self::Error> {
        Ok(())
    }

    fn resize(&mut self, width: u32, height: u32) -> Result<(), Self::Error> {
        self.width = width;
        self.height = height;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_renderer_creation() {
        let renderer = CpuRenderer::new(100, 100);
        assert_eq!(renderer.width, 100);
        assert_eq!(renderer.height, 100);
        assert_eq!(renderer.buffer.len(), 10000);
    }

    #[test]
    fn test_cpu_renderer_clear() {
        let mut renderer = CpuRenderer::new(10, 10);
        renderer.clear(0xFF000000).unwrap();
        assert!(renderer.buffer.iter().all(|&p| p == 0xFF000000));
    }

    #[test]
    fn test_cpu_backend_creation() {
        let backend = CpuBackendWrapper::new();
        assert_eq!(backend.width, 0);
    }

    #[test]
    fn test_create_backend() {
        let _backend = create_backend(false);
        let _backend = create_backend(true);
    }
}
