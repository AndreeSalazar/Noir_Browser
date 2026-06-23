//! Frame Decoder - YUV a RGB conversion
//!
//! Implementa conversión BT.601/BT.709 para mostrar frames H.264.

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorSpace {
    Bt601,
    Bt709,
    Bt2020,
}

impl ColorSpace {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "bt709" => Self::Bt709,
            "bt2020" => Self::Bt2020,
            _ => Self::Bt601,
        }
    }
}

#[derive(Debug, Clone)]
pub struct YuvFrame {
    pub y: Vec<u8>,
    pub u: Vec<u8>,
    pub v: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub y_stride: u32,
    pub u_stride: u32,
    pub v_stride: u32,
    pub color_space: ColorSpace,
}

impl YuvFrame {
    pub fn new(width: u32, height: u32) -> Self {
        let y_stride = width;
        let u_stride = width / 2;
        let v_stride = width / 2;
        let y = vec![0u8; (y_stride * height) as usize];
        let u = vec![128u8; (u_stride * height / 2) as usize];
        let v = vec![128u8; (v_stride * height / 2) as usize];
        Self {
            y, u, v,
            width, height,
            y_stride, u_stride, v_stride,
            color_space: ColorSpace::Bt601,
        }
    }

    pub fn with_color_space(mut self, cs: ColorSpace) -> Self {
        self.color_space = cs;
        self
    }

    pub fn fill_test_pattern(&mut self) {
        // Genera patrón: Y=128, U=128+sin(x), V=128+cos(y)
        for y in 0..self.height {
            for x in 0..self.width {
                let i = (y * self.y_stride + x) as usize;
                self.y[i] = ((x + y) % 256) as u8;
                if x % 2 == 0 && y % 2 == 0 {
                    let u_idx = (y / 2 * self.u_stride + x / 2) as usize;
                    let v_idx = (y / 2 * self.v_stride + x / 2) as usize;
                    self.u[u_idx] = 128;
                    self.v[v_idx] = 128;
                }
            }
        }
    }
}

pub struct RgbConverter {
    pub width: u32,
    pub height: u32,
}

impl RgbConverter {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    /// Convierte un YUV frame a RGB
    pub fn convert(&self, frame: &YuvFrame) -> Vec<u8> {
        let mut rgb = vec![0u8; (self.width * self.height * 3) as usize];
        self.convert_into(frame, &mut rgb);
        rgb
    }

    pub fn convert_into(&self, frame: &YuvFrame, output: &mut [u8]) {
        let kr_kg_kb = match frame.color_space {
            ColorSpace::Bt601 => (0.299, 0.587, 0.114),
            ColorSpace::Bt709 => (0.2126, 0.7152, 0.0722),
            ColorSpace::Bt2020 => (0.2627, 0.6780, 0.0593),
        };
        for y in 0..self.height {
            for x in 0..self.width {
                let y_idx = (y * frame.y_stride + x) as usize;
                let uv_x = x / 2;
                let uv_y = y / 2;
                let u_idx = (uv_y * frame.u_stride + uv_x) as usize;
                let v_idx = (uv_y * frame.v_stride + uv_x) as usize;
                if y_idx >= frame.y.len() || u_idx >= frame.u.len() || v_idx >= frame.v.len() {
                    continue;
                }
                let y_val = frame.y[y_idx] as f32;
                let u_val = frame.u[u_idx] as f32 - 128.0;
                let v_val = frame.v[v_idx] as f32 - 128.0;
                let r = y_val + 2.0 * (1.0 - kr_kg_kb.0) * v_val;
                let g = y_val - 2.0 * kr_kg_kb.0 / kr_kg_kb.1 * u_val - 2.0 * kr_kg_kb.2 / kr_kg_kb.1 * v_val;
                let b = y_val + 2.0 * (1.0 - kr_kg_kb.2) * u_val;
                let out_idx = ((y * self.width + x) * 3) as usize;
                if out_idx + 2 < output.len() {
                    output[out_idx] = r.clamp(0.0, 255.0) as u8;
                    output[out_idx + 1] = g.clamp(0.0, 255.0) as u8;
                    output[out_idx + 2] = b.clamp(0.0, 255.0) as u8;
                }
            }
        }
    }

    /// Convierte a RGBA (4 bytes por pixel)
    pub fn convert_to_rgba(&self, frame: &YuvFrame) -> Vec<u8> {
        let mut rgba = vec![0u8; (self.width * self.height * 4) as usize];
        let kr_kg_kb = match frame.color_space {
            ColorSpace::Bt601 => (0.299, 0.587, 0.114),
            ColorSpace::Bt709 => (0.2126, 0.7152, 0.0722),
            ColorSpace::Bt2020 => (0.2627, 0.6780, 0.0593),
        };
        for y in 0..self.height {
            for x in 0..self.width {
                let y_idx = (y * frame.y_stride + x) as usize;
                let uv_x = x / 2;
                let uv_y = y / 2;
                let u_idx = (uv_y * frame.u_stride + uv_x) as usize;
                let v_idx = (uv_y * frame.v_stride + uv_x) as usize;
                if y_idx >= frame.y.len() || u_idx >= frame.u.len() || v_idx >= frame.v.len() {
                    continue;
                }
                let y_val = frame.y[y_idx] as f32;
                let u_val = frame.u[u_idx] as f32 - 128.0;
                let v_val = frame.v[v_idx] as f32 - 128.0;
                let r = y_val + 2.0 * (1.0 - kr_kg_kb.0) * v_val;
                let g = y_val - 2.0 * kr_kg_kb.0 / kr_kg_kb.1 * u_val - 2.0 * kr_kg_kb.2 / kr_kg_kb.1 * v_val;
                let b = y_val + 2.0 * (1.0 - kr_kg_kb.2) * u_val;
                let out_idx = ((y * self.width + x) * 4) as usize;
                if out_idx + 3 < rgba.len() {
                    rgba[out_idx] = r.clamp(0.0, 255.0) as u8;
                    rgba[out_idx + 1] = g.clamp(0.0, 255.0) as u8;
                    rgba[out_idx + 2] = b.clamp(0.0, 255.0) as u8;
                    rgba[out_idx + 3] = 255;
                }
            }
        }
        rgba
    }
}

/// Frame decimator para downscale (low-power preview)
pub struct FrameDecimator {
    pub scale: u32,
}

impl FrameDecimator {
    pub fn new(scale: u32) -> Self {
        Self { scale: scale.max(1) }
    }

    pub fn decimate(&self, frame: &YuvFrame) -> YuvFrame {
        if self.scale == 1 {
            return frame.clone();
        }
        let new_w = frame.width / self.scale;
        let new_h = frame.height / self.scale;
        let mut y = vec![0u8; (new_w * new_h) as usize];
        let mut u = vec![0u8; ((new_w / 2) * (new_h / 2)) as usize];
        let mut v = vec![0u8; ((new_w / 2) * (new_h / 2)) as usize];
        for yi in 0..new_h {
            for xi in 0..new_w {
                let src_x = xi * self.scale;
                let src_y = yi * self.scale;
                let y_idx = (yi * new_w + xi) as usize;
                let src_idx = (src_y * frame.y_stride + src_x) as usize;
                y[y_idx] = frame.y[src_idx];
                if xi % 2 == 0 && yi % 2 == 0 {
                    let uv_x = xi / 2;
                    let uv_y = yi / 2;
                    let uv_idx = (uv_y * (new_w / 2) + uv_x) as usize;
                    let src_uv_x = src_x / 2;
                    let src_uv_y = src_y / 2;
                    let src_u_idx = (src_uv_y * frame.u_stride + src_uv_x) as usize;
                    let src_v_idx = (src_uv_y * frame.v_stride + src_uv_x) as usize;
                    u[uv_idx] = frame.u[src_u_idx];
                    v[uv_idx] = frame.v[src_v_idx];
                }
            }
        }
        YuvFrame {
            y, u, v,
            width: new_w,
            height: new_h,
            y_stride: new_w,
            u_stride: new_w / 2,
            v_stride: new_w / 2,
            color_space: frame.color_space,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_space_from_str() {
        assert_eq!(ColorSpace::from_str("bt709"), ColorSpace::Bt709);
        assert_eq!(ColorSpace::from_str("BT601"), ColorSpace::Bt601);
        assert_eq!(ColorSpace::from_str("bt2020"), ColorSpace::Bt2020);
    }

    #[test]
    fn test_yuv_frame_new() {
        let f = YuvFrame::new(320, 240);
        assert_eq!(f.width, 320);
        assert_eq!(f.height, 240);
        assert_eq!(f.y.len(), 320 * 240);
    }

    #[test]
    fn test_yuv_frame_with_cs() {
        let f = YuvFrame::new(16, 16).with_color_space(ColorSpace::Bt709);
        assert_eq!(f.color_space, ColorSpace::Bt709);
    }

    #[test]
    fn test_yuv_frame_fill_test_pattern() {
        let mut f = YuvFrame::new(16, 16);
        f.fill_test_pattern();
        assert!(f.y.iter().any(|&v| v != 0));
    }

    #[test]
    fn test_rgb_converter_new() {
        let c = RgbConverter::new(320, 240);
        assert_eq!(c.width, 320);
    }

    #[test]
    fn test_convert_dimensions() {
        let c = RgbConverter::new(16, 16);
        let f = YuvFrame::new(16, 16);
        let rgb = c.convert(&f);
        assert_eq!(rgb.len(), 16 * 16 * 3);
    }

    #[test]
    fn test_convert_white() {
        let c = RgbConverter::new(8, 8);
        let mut f = YuvFrame::new(8, 8);
        for v in f.y.iter_mut() { *v = 255; }
        for v in f.u.iter_mut() { *v = 128; }
        for v in f.v.iter_mut() { *v = 128; }
        let rgb = c.convert(&f);
        // Y=235+ da blanco (~240)
        assert!(rgb[0] > 200);
    }

    #[test]
    fn test_convert_black() {
        let c = RgbConverter::new(8, 8);
        let mut f = YuvFrame::new(8, 8);
        for v in f.y.iter_mut() { *v = 0; }
        for v in f.u.iter_mut() { *v = 128; }
        for v in f.v.iter_mut() { *v = 128; }
        let rgb = c.convert(&f);
        assert!(rgb[0] < 10);
    }

    #[test]
    fn test_convert_red() {
        let c = RgbConverter::new(4, 4);
        let mut f = YuvFrame::new(4, 4);
        for v in f.y.iter_mut() { *v = 82; } // Y para rojo
        for v in f.u.iter_mut() { *v = 90; } // U = 128-38
        for v in f.v.iter_mut() { *v = 240; } // V = 128+112
        let rgb = c.convert(&f);
        // R debe ser mayor que G y B
        assert!(rgb[0] > rgb[1]);
        assert!(rgb[0] > rgb[2]);
    }

    #[test]
    fn test_convert_to_rgba() {
        let c = RgbConverter::new(8, 8);
        let f = YuvFrame::new(8, 8);
        let rgba = c.convert_to_rgba(&f);
        assert_eq!(rgba.len(), 8 * 8 * 4);
        // Alpha siempre 255
        assert!(rgba.iter().skip(3).step_by(4).all(|&v| v == 255));
    }

    #[test]
    fn test_decimator_new() {
        let d = FrameDecimator::new(2);
        assert_eq!(d.scale, 2);
    }

    #[test]
    fn test_decimator_scale_one() {
        let d = FrameDecimator::new(1);
        let f = YuvFrame::new(16, 16);
        let out = d.decimate(&f);
        assert_eq!(out.width, 16);
    }

    #[test]
    fn test_decimator_half() {
        let d = FrameDecimator::new(2);
        let f = YuvFrame::new(16, 16);
        let out = d.decimate(&f);
        assert_eq!(out.width, 8);
        assert_eq!(out.height, 8);
    }

    #[test]
    fn test_decimator_quarter() {
        let d = FrameDecimator::new(4);
        let f = YuvFrame::new(16, 16);
        let out = d.decimate(&f);
        assert_eq!(out.width, 4);
        assert_eq!(out.height, 4);
    }

    #[test]
    fn test_decimator_min_scale() {
        let d = FrameDecimator::new(0);
        assert_eq!(d.scale, 1);
    }

    #[test]
    fn test_bt601_vs_bt709() {
        let c = RgbConverter::new(4, 4);
        let mut f601 = YuvFrame::new(4, 4).with_color_space(ColorSpace::Bt601);
        let mut f709 = YuvFrame::new(4, 4).with_color_space(ColorSpace::Bt709);
        for v in f601.y.iter_mut() { *v = 128; }
        for v in f601.u.iter_mut() { *v = 100; }
        for v in f601.v.iter_mut() { *v = 200; }
        f709.y.copy_from_slice(&f601.y);
        f709.u.copy_from_slice(&f601.u);
        f709.v.copy_from_slice(&f601.v);
        let r601 = c.convert(&f601);
        let r709 = c.convert(&f709);
        // BT.601 y BT.709 deben dar resultados diferentes
        assert!(r601[0] != r709[0] || r601[1] != r709[1] || r601[2] != r709[2]);
    }
}
