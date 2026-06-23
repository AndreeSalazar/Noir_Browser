//! WebGPU Texture - GPU image storage
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureFormat {
    R8Unorm,
    Rg8Unorm,
    Rgba8Unorm,
    Rgba8UnormSrgb,
    Bgra8Unorm,
    Bgra8UnormSrgb,
    R16Float,
    Rg16Float,
    Rgba16Float,
    R32Float,
    Rg32Float,
    Rgba32Float,
    Depth32Float,
    Depth24PlusStencil8,
    Bc1RgbaUnorm,
    Bc7RgbaUnorm,
    Etc2Rgb8Unorm,
    Etc2Rgba8Unorm,
    Astc4x4Unorm,
}

impl TextureFormat {
    pub fn bytes_per_pixel(&self) -> u32 {
        match self {
            TextureFormat::R8Unorm => 1,
            TextureFormat::Rg8Unorm => 2,
            TextureFormat::Rgba8Unorm | TextureFormat::Rgba8UnormSrgb => 4,
            TextureFormat::Bgra8Unorm | TextureFormat::Bgra8UnormSrgb => 4,
            TextureFormat::R16Float => 2,
            TextureFormat::Rg16Float => 4,
            TextureFormat::Rgba16Float => 8,
            TextureFormat::R32Float => 4,
            TextureFormat::Rg32Float => 8,
            TextureFormat::Rgba32Float => 16,
            TextureFormat::Depth32Float => 4,
            TextureFormat::Depth24PlusStencil8 => 4,
            TextureFormat::Bc1RgbaUnorm => 8, // 4 bits per pixel, block compressed
            TextureFormat::Bc7RgbaUnorm => 16, // 8 bits per pixel
            TextureFormat::Etc2Rgb8Unorm => 8,
            TextureFormat::Etc2Rgba8Unorm => 16,
            TextureFormat::Astc4x4Unorm => 16,
        }
    }

    pub fn is_srgb(&self) -> bool {
        matches!(self, TextureFormat::Rgba8UnormSrgb | TextureFormat::Bgra8UnormSrgb)
    }

    pub fn is_compressed(&self) -> bool {
        matches!(self,
            TextureFormat::Bc1RgbaUnorm |
            TextureFormat::Bc7RgbaUnorm |
            TextureFormat::Etc2Rgb8Unorm |
            TextureFormat::Etc2Rgba8Unorm |
            TextureFormat::Astc4x4Unorm
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextureUsage {
    pub copy_src: bool,
    pub copy_dst: bool,
    pub texture_binding: bool,
    pub storage_binding: bool,
    pub render_attachment: bool,
}

impl TextureUsage {
    pub fn sampled() -> Self {
        Self { texture_binding: true, copy_dst: true, ..Default::default() }
    }

    pub fn storage() -> Self {
        Self { storage_binding: true, copy_src: true, copy_dst: true, ..Default::default() }
    }

    pub fn render_target() -> Self {
        Self { render_attachment: true, ..Default::default() }
    }
}

impl Default for TextureUsage {
    fn default() -> Self {
        Self {
            copy_src: false,
            copy_dst: false,
            texture_binding: false,
            storage_binding: false,
            render_attachment: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Texture {
    pub id: u64,
    pub width: u32,
    pub height: u32,
    pub format: TextureFormat,
    pub usage: TextureUsage,
    pub label: Option<String>,
}

impl Texture {
    pub fn new(id: u64, width: u32, height: u32, format: TextureFormat, usage: TextureUsage) -> Self {
        Self {
            id,
            width,
            height,
            format,
            usage,
            label: None,
        }
    }

    pub fn with_label(mut self, label: &str) -> Self {
        self.label = Some(label.to_string());
        self
    }

    pub fn pixel_count(&self) -> u32 {
        self.width * self.height
    }

    pub fn size_in_bytes(&self) -> u64 {
        if self.format.is_compressed() {
            (self.pixel_count() as u64 * self.format.bytes_per_pixel() as u64) / 4 // Block compression
        } else {
            self.pixel_count() as u64 * self.format.bytes_per_pixel() as u64
        }
    }
}
