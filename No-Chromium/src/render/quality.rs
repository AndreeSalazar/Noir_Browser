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
    pub fn ultra_native() -> Self {
        Self {
            device_pixel_ratio: 1.0,
            text_filtering: TextFiltering::SubpixelLinear,
            pixel_snap: PixelSnap::PhysicalPixels,
            msaa_samples: 1,
        }
    }
}
