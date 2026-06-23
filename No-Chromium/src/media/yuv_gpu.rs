//! YUV to RGB GPU Shader - Compute shader para WebGPU
//!
//! Convierte YUV a RGB usando GPU compute shader (más rápido que CPU).

pub const YUV_TO_RGB_SHADER: &str = r#"
struct YuvBuffer {
    width: u32,
    height: u32,
    y_stride: u32,
    u_stride: u32,
    v_stride: u32,
    color_space: u32, // 0=BT.601, 1=BT.709, 2=BT.2020
    y_offset: u32,
    u_offset: u32,
    v_offset: u32,
}

@group(0) @binding(0) var yuv_data: texture_2d<f32>;
@group(0) @binding(1) var output_data: texture_storage_2d<rgba8unorm, write>;

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let coords = vec2<u32>(gid.x, gid.y);
    let dims = textureDimensions(output_data);
    if (coords.x >= dims.x || coords.y >= dims.y) {
        return;
    }

    let uv_x = coords.x / 2u;
    let uv_y = coords.y / 2u;

    // Load Y, U, V samples
    let y = textureLoad(yuv_data, vec2<u32>(coords.x, coords.y), 0).r;
    let u = textureLoad(yuv_data, vec2<u32>(uv_x, uv_y), 1).r - 0.5;
    let v = textureLoad(yuv_data, vec2<u32>(uv_x, uv_y), 2).r - 0.5;

    // BT.601 coefficients
    let r = y + 1.402 * v;
    let g = y - 0.344136 * u - 0.714136 * v;
    let b = y + 1.772 * u;

    let rgb = vec4<f32>(
        clamp(r, 0.0, 1.0),
        clamp(g, 0.0, 1.0),
        clamp(b, 0.0, 1.0),
        1.0
    );

    textureStore(output_data, coords, rgb);
}
"#;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShaderColorSpace {
    Bt601,
    Bt709,
    Bt2020,
}

impl ShaderColorSpace {
    pub fn to_u32(&self) -> u32 {
        match self {
            Self::Bt601 => 0,
            Self::Bt709 => 1,
            Self::Bt2020 => 2,
        }
    }

    pub fn from_u32(v: u32) -> Self {
        match v {
            1 => Self::Bt709,
            2 => Self::Bt2020,
            _ => Self::Bt601,
        }
    }
}

#[derive(Debug, Clone)]
pub struct YuvGpuConfig {
    pub width: u32,
    pub height: u32,
    pub color_space: ShaderColorSpace,
    pub workgroup_size_x: u32,
    pub workgroup_size_y: u32,
}

impl YuvGpuConfig {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            color_space: ShaderColorSpace::Bt601,
            workgroup_size_x: 8,
            workgroup_size_y: 8,
        }
    }

    pub fn with_color_space(mut self, cs: ShaderColorSpace) -> Self {
        self.color_space = cs;
        self
    }

    pub fn workgroup_count(&self) -> (u32, u32) {
        let x = (self.width + self.workgroup_size_x - 1) / self.workgroup_size_x;
        let y = (self.height + self.workgroup_size_y - 1) / self.workgroup_size_y;
        (x, y)
    }

    pub fn workgroup_count_total(&self) -> u32 {
        let (x, y) = self.workgroup_count();
        x * y
    }
}

pub struct YuvGpuConverter {
    pub config: YuvGpuConfig,
    pub shader_source: String,
    pub total_runs: u64,
    pub total_pixels: u64,
    pub failures: u32,
}

impl YuvGpuConverter {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            config: YuvGpuConfig::new(width, height),
            shader_source: YUV_TO_RGB_SHADER.to_string(),
            total_runs: 0,
            total_pixels: 0,
            failures: 0,
        }
    }

    pub fn with_color_space(mut self, cs: ShaderColorSpace) -> Self {
        self.config = self.config.with_color_space(cs);
        self
    }

    pub fn run(&mut self, _yuv: &[u8]) -> Result<Vec<u8>, String> {
        self.total_runs += 1;
        let pixel_count = self.config.width as u64 * self.config.height as u64;
        self.total_pixels += pixel_count;
        // CPU fallback (la GPU se integrará en integración con wgpu)
        let mut rgb = vec![0u8; (pixel_count * 4) as usize];
        // BT.601 conversion
        for y in 0..self.config.height {
            for x in 0..self.config.width {
                let y_idx = (y * self.config.width + x) as usize;
                if y_idx * 2 + 1 >= _yuv.len() { break; }
                let y_val = _yuv[y_idx] as f32;
                let uv_x = (x / 2) as usize;
                let uv_y = (y / 2) as usize;
                let uv_idx = uv_y * (self.config.width as usize / 2) + uv_x;
                let u_val = if uv_idx + (_yuv.len() / 2) < _yuv.len() {
                    _yuv[uv_idx + (_yuv.len() / 2)] as f32 - 128.0
                } else { 0.0 };
                let v_val = if uv_idx + (_yuv.len() * 3 / 4) < _yuv.len() {
                    _yuv[uv_idx + (_yuv.len() * 3 / 4)] as f32 - 128.0
                } else { 0.0 };
                let r = y_val + 1.402 * v_val;
                let g = y_val - 0.344136 * u_val - 0.714136 * v_val;
                let b = y_val + 1.772 * u_val;
                let out_idx = (y * self.config.width + x) as usize * 4;
                if out_idx + 3 < rgb.len() {
                    rgb[out_idx] = r.clamp(0.0, 255.0) as u8;
                    rgb[out_idx + 1] = g.clamp(0.0, 255.0) as u8;
                    rgb[out_idx + 2] = b.clamp(0.0, 255.0) as u8;
                    rgb[out_idx + 3] = 255;
                }
            }
        }
        Ok(rgb)
    }

    pub fn fps(&self) -> f32 {
        if self.total_runs == 0 { return 0.0; }
        self.total_pixels as f32 / self.total_runs as f32
    }

    pub fn shader_lines(&self) -> usize {
        self.shader_source.lines().count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shader_source_not_empty() {
        assert!(!YUV_TO_RGB_SHADER.is_empty());
    }

    #[test]
    fn test_shader_has_compute() {
        assert!(YUV_TO_RGB_SHADER.contains("@compute"));
        assert!(YUV_TO_RGB_SHADER.contains("@workgroup_size"));
    }

    #[test]
    fn test_color_space_to_u32() {
        assert_eq!(ShaderColorSpace::Bt601.to_u32(), 0);
        assert_eq!(ShaderColorSpace::Bt709.to_u32(), 1);
        assert_eq!(ShaderColorSpace::Bt2020.to_u32(), 2);
    }

    #[test]
    fn test_color_space_from_u32() {
        assert_eq!(ShaderColorSpace::from_u32(0), ShaderColorSpace::Bt601);
        assert_eq!(ShaderColorSpace::from_u32(1), ShaderColorSpace::Bt709);
        assert_eq!(ShaderColorSpace::from_u32(99), ShaderColorSpace::Bt601);
    }

    #[test]
    fn test_config_new() {
        let c = YuvGpuConfig::new(1920, 1080);
        assert_eq!(c.width, 1920);
        assert_eq!(c.workgroup_size_x, 8);
    }

    #[test]
    fn test_config_workgroup_count() {
        let c = YuvGpuConfig::new(1920, 1080);
        let (x, y) = c.workgroup_count();
        assert_eq!(x, 240); // 1920/8
        assert_eq!(y, 135); // 1080/8
    }

    #[test]
    fn test_config_workgroup_count_partial() {
        let c = YuvGpuConfig::new(100, 100);
        let (x, y) = c.workgroup_count();
        assert_eq!(x, 13); // ceil(100/8)
        assert_eq!(y, 13);
    }

    #[test]
    fn test_config_total() {
        let c = YuvGpuConfig::new(100, 100);
        assert_eq!(c.workgroup_count_total(), 13 * 13);
    }

    #[test]
    fn test_converter_new() {
        let c = YuvGpuConverter::new(640, 480);
        assert_eq!(c.config.width, 640);
        assert_eq!(c.total_runs, 0);
    }

    #[test]
    fn test_converter_with_color_space() {
        let c = YuvGpuConverter::new(640, 480).with_color_space(ShaderColorSpace::Bt709);
        assert_eq!(c.config.color_space, ShaderColorSpace::Bt709);
    }

    #[test]
    fn test_converter_run() {
        let mut c = YuvGpuConverter::new(8, 8);
        // 8x8 Y plane + 4x4 U + 4x4 V = 64 + 16 + 16 = 96
        let yuv = vec![128u8; 96];
        let rgb = c.run(&yuv).unwrap();
        assert_eq!(rgb.len(), 8 * 8 * 4);
        assert_eq!(c.total_runs, 1);
    }

    #[test]
    fn test_converter_white() {
        let mut c = YuvGpuConverter::new(4, 4);
        let mut yuv = vec![128u8; 4 * 4 + 2 * 2 + 2 * 2];
        // Y=255 para blanco
        for i in 0..(4 * 4) {
            yuv[i] = 255;
        }
        // U/V = 128 (neutral)
        for i in 0..(2 * 2) {
            yuv[4*4 + i] = 128;
            yuv[4*4 + 2*2 + i] = 128;
        }
        let rgb = c.run(&yuv).unwrap();
        // White should be high values
        assert!(rgb[0] > 200);
    }

    #[test]
    fn test_converter_fps() {
        let mut c = YuvGpuConverter::new(100, 100);
        let yuv = vec![0u8; 15000];
        let _ = c.run(&yuv);
        assert!(c.fps() > 0.0);
    }

    #[test]
    fn test_converter_shader_lines() {
        let c = YuvGpuConverter::new(8, 8);
        assert!(c.shader_lines() > 20);
    }

    #[test]
    fn test_shader_has_color_clamp() {
        assert!(YUV_TO_RGB_SHADER.contains("clamp"));
    }

    #[test]
    fn test_shader_yuv_formula() {
        assert!(YUV_TO_RGB_SHADER.contains("1.402"));
        assert!(YUV_TO_RGB_SHADER.contains("1.772"));
    }
}
