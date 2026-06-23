//! Video Texture - GPU texture para video frames
//!
//! Convierte frames RGB a texture WebGPU para rendering en pantalla.

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VideoTextureFormat {
    Rgba8,
    Bgra8,
    Rgb8,
    R8,
}

impl VideoTextureFormat {
    pub fn bytes_per_pixel(&self) -> u32 {
        match self {
            Self::Rgba8 | Self::Bgra8 => 4,
            Self::Rgb8 => 3,
            Self::R8 => 1,
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "rgba" | "rgba8" => Self::Rgba8,
            "bgra" | "bgra8" => Self::Bgra8,
            "rgb" | "rgb8" => Self::Rgb8,
            "r" | "r8" => Self::R8,
            _ => Self::Rgba8,
        }
    }
}

#[derive(Debug, Clone)]
pub struct VideoTextureDescriptor {
    pub width: u32,
    pub height: u32,
    pub format: VideoTextureFormat,
    pub usage: TextureUsage,
    pub mipmaps: bool,
    pub srgb: bool,
}

impl Default for VideoTextureDescriptor {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            format: VideoTextureFormat::Rgba8,
            usage: TextureUsage::sampled(),
            mipmaps: false,
            srgb: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextureUsage {
    pub sampled: bool,
    pub storage: bool,
    pub render_target: bool,
    pub copy_src: bool,
    pub copy_dst: bool,
}

impl TextureUsage {
    pub fn sampled() -> Self { Self { sampled: true, storage: false, render_target: false, copy_src: false, copy_dst: false } }
    pub fn render_target() -> Self { Self { sampled: false, storage: false, render_target: true, copy_src: false, copy_dst: false } }
}

#[derive(Debug, Clone)]
pub struct VideoTexture {
    pub id: u32,
    pub descriptor: VideoTextureDescriptor,
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub last_updated: u64,
    pub frame_count: u64,
    pub dirty: bool,
}

impl VideoTexture {
    pub fn new(id: u32, width: u32, height: u32) -> Self {
        let descriptor = VideoTextureDescriptor {
            width,
            height,
            ..Default::default()
        };
        let bpp = descriptor.format.bytes_per_pixel();
        let data = vec![0u8; (width * height * bpp) as usize];
        Self {
            id,
            descriptor,
            data,
            width,
            height,
            last_updated: 0,
            frame_count: 0,
            dirty: true,
        }
    }

    pub fn with_format(mut self, format: VideoTextureFormat) -> Self {
        self.descriptor.format = format;
        let bpp = format.bytes_per_pixel();
        self.data = vec![0u8; (self.width * self.height * bpp) as usize];
        self
    }

    /// Upload RGB data (3 bytes/pixel) al texture
    pub fn upload_rgb(&mut self, rgb: &[u8], timestamp: u64) -> Result<(), String> {
        if rgb.len() != (self.width * self.height * 3) as usize {
            return Err(format!("RGB size mismatch: {} != {}", rgb.len(), self.width * self.height * 3));
        }
        if self.descriptor.format == VideoTextureFormat::Rgb8 {
            self.data.copy_from_slice(rgb);
        } else {
            // Convert RGB to RGBA
            self.data = vec![0u8; (self.width * self.height * 4) as usize];
            for i in 0..(self.width * self.height) as usize {
                self.data[i*4] = rgb[i*3];
                self.data[i*4 + 1] = rgb[i*3 + 1];
                self.data[i*4 + 2] = rgb[i*3 + 2];
                self.data[i*4 + 3] = 255;
            }
        }
        self.last_updated = timestamp;
        self.frame_count += 1;
        self.dirty = true;
        Ok(())
    }

    /// Upload RGBA data (4 bytes/pixel)
    pub fn upload_rgba(&mut self, rgba: &[u8], timestamp: u64) -> Result<(), String> {
        let expected = (self.width * self.height * 4) as usize;
        if rgba.len() != expected {
            return Err(format!("RGBA size mismatch: {} != {}", rgba.len(), expected));
        }
        self.data = rgba.to_vec();
        self.last_updated = timestamp;
        self.frame_count += 1;
        self.dirty = true;
        Ok(())
    }

    pub fn size_bytes(&self) -> usize {
        (self.width * self.height * self.descriptor.format.bytes_per_pixel()) as usize
    }

    pub fn aspect_ratio(&self) -> f32 {
        if self.height == 0 { 16.0 / 9.0 } else { self.width as f32 / self.height as f32 }
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }
}

pub struct VideoTextureManager {
    textures: Vec<VideoTexture>,
    next_id: u32,
}

impl VideoTextureManager {
    pub fn new() -> Self {
        Self {
            textures: Vec::new(),
            next_id: 1,
        }
    }

    pub fn create(&mut self, width: u32, height: u32) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.textures.push(VideoTexture::new(id, width, height));
        id
    }

    pub fn get(&self, id: u32) -> Option<&VideoTexture> {
        self.textures.iter().find(|t| t.id == id)
    }

    pub fn get_mut(&mut self, id: u32) -> Option<&mut VideoTexture> {
        self.textures.iter_mut().find(|t| t.id == id)
    }

    pub fn count(&self) -> usize {
        self.textures.len()
    }

    pub fn total_memory(&self) -> usize {
        self.textures.iter().map(|t| t.size_bytes()).sum()
    }

    pub fn dirty_count(&self) -> usize {
        self.textures.iter().filter(|t| t.is_dirty()).count()
    }
}

impl Default for VideoTextureManager {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bpp() {
        assert_eq!(VideoTextureFormat::Rgba8.bytes_per_pixel(), 4);
        assert_eq!(VideoTextureFormat::Rgb8.bytes_per_pixel(), 3);
        assert_eq!(VideoTextureFormat::R8.bytes_per_pixel(), 1);
    }

    #[test]
    fn test_format_from_str() {
        assert_eq!(VideoTextureFormat::from_str("rgba"), VideoTextureFormat::Rgba8);
        assert_eq!(VideoTextureFormat::from_str("RGB"), VideoTextureFormat::Rgb8);
    }

    #[test]
    fn test_descriptor_default() {
        let d = VideoTextureDescriptor::default();
        assert_eq!(d.width, 1920);
        assert_eq!(d.height, 1080);
        assert!(d.srgb);
    }

    #[test]
    fn test_texture_new() {
        let t = VideoTexture::new(1, 320, 240);
        assert_eq!(t.id, 1);
        assert_eq!(t.width, 320);
        assert!(t.dirty);
    }

    #[test]
    fn test_texture_with_format() {
        let t = VideoTexture::new(1, 320, 240).with_format(VideoTextureFormat::Rgb8);
        assert_eq!(t.data.len(), 320 * 240 * 3);
    }

    #[test]
    fn test_texture_upload_rgb_rgb8() {
        let mut t = VideoTexture::new(1, 4, 4).with_format(VideoTextureFormat::Rgb8);
        let rgb = vec![128u8; 4 * 4 * 3];
        t.upload_rgb(&rgb, 1).unwrap();
        assert_eq!(t.data.len(), 4 * 4 * 3);
        assert_eq!(t.frame_count, 1);
    }

    #[test]
    fn test_texture_upload_rgb_to_rgba() {
        let mut t = VideoTexture::new(1, 4, 4);
        let rgb = vec![100u8; 4 * 4 * 3];
        t.upload_rgb(&rgb, 1).unwrap();
        assert_eq!(t.data.len(), 4 * 4 * 4);
        assert_eq!(t.data[3], 255); // alpha
    }

    #[test]
    fn test_texture_upload_size_mismatch() {
        let mut t = VideoTexture::new(1, 4, 4).with_format(VideoTextureFormat::Rgb8);
        let rgb = vec![0u8; 100];
        assert!(t.upload_rgb(&rgb, 1).is_err());
    }

    #[test]
    fn test_texture_upload_rgba() {
        let mut t = VideoTexture::new(1, 4, 4);
        let rgba = vec![100u8; 4 * 4 * 4];
        t.upload_rgba(&rgba, 1).unwrap();
        assert_eq!(t.frame_count, 1);
    }

    #[test]
    fn test_texture_size_bytes() {
        let t = VideoTexture::new(1, 320, 240);
        assert_eq!(t.size_bytes(), 320 * 240 * 4);
    }

    #[test]
    fn test_texture_aspect_ratio() {
        let t = VideoTexture::new(1, 1920, 1080);
        assert_eq!(t.aspect_ratio(), 16.0/9.0);
    }

    #[test]
    fn test_texture_dirty() {
        let mut t = VideoTexture::new(1, 4, 4);
        assert!(t.is_dirty());
        t.mark_clean();
        assert!(!t.is_dirty());
    }

    #[test]
    fn test_manager_create() {
        let mut m = VideoTextureManager::new();
        let id = m.create(320, 240);
        assert!(m.get(id).is_some());
    }

    #[test]
    fn test_manager_total_memory() {
        let mut m = VideoTextureManager::new();
        m.create(320, 240);
        m.create(640, 480);
        assert_eq!(m.total_memory(), 320*240*4 + 640*480*4);
    }

    #[test]
    fn test_manager_dirty_count() {
        let mut m = VideoTextureManager::new();
        let id1 = m.create(4, 4);
        let id2 = m.create(4, 4);
        m.get_mut(id1).unwrap().mark_clean();
        assert_eq!(m.dirty_count(), 1);
    }

    #[test]
    fn test_texture_count() {
        let mut m = VideoTextureManager::new();
        m.create(100, 100);
        m.create(100, 100);
        m.create(100, 100);
        assert_eq!(m.count(), 3);
    }

    #[test]
    fn test_texture_upload_increments_count() {
        let mut t = VideoTexture::new(1, 4, 4);
        let data = vec![0u8; 4*4*4];
        t.upload_rgba(&data, 1).unwrap();
        t.upload_rgba(&data, 2).unwrap();
        t.upload_rgba(&data, 3).unwrap();
        assert_eq!(t.frame_count, 3);
    }
}
