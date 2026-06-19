#![allow(dead_code)]

#[derive(Debug, Clone, Copy)]
pub enum TextFiltering {
    Linear,
    SubpixelLinear,
}

#[derive(Debug, Clone, Copy)]
pub enum PixelSnap {
    None,
    LogicalPixels,
    PhysicalPixels,
}

#[derive(Debug, Clone, Copy)]
pub struct QualityProfile {
    pub device_pixel_ratio: f32,
    pub text_filtering: TextFiltering,
    pub pixel_snap: PixelSnap,
    pub msaa_samples: u32,
}

impl QualityProfile {
    pub fn ultra_native(device_pixel_ratio: f32) -> Self {
        Self {
            device_pixel_ratio: device_pixel_ratio.max(1.0),
            text_filtering: TextFiltering::SubpixelLinear,
            pixel_snap: PixelSnap::PhysicalPixels,
            msaa_samples: 1,
        }
    }

    pub fn text_rasterization_options(self) -> crate::render::text::TextRasterizationOptions {
        use crate::render::text::TextBitmapMode;

        let mut options = crate::render::text::TextRasterizationOptions::sharp_lcd();
        options.oversample = (self.device_pixel_ratio * 3.0).clamp(3.0, 6.0);
        options.bitmap_mode = match self.text_filtering {
            TextFiltering::Linear => TextBitmapMode::AlphaMask,
            TextFiltering::SubpixelLinear => TextBitmapMode::SubpixelMask,
        };
        options
    }
}
