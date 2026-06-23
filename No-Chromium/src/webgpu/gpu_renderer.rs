//! WebGPU Renderer - GPU rendering via wgpu
//!
//! Renderiza el navegador usando la GPU a través del estándar WebGPU.
//! Funciona en Vulkan (Linux/Android), Metal (macOS/iOS) y DirectX 12 (Windows).

#![allow(dead_code)]

use std::sync::Arc;
use std::sync::Mutex;
use wgpu::util::DeviceExt;

/// Estado de un rectángulo pendiente de dibujar
#[derive(Debug, Clone, Copy)]
pub struct DrawRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub color: [f32; 4],
}

/// Uniform buffer compartido con el shader
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RectUniform {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub color: [f32; 4],
}

/// WGSL shader para renderizar rectángulos
pub const RECT_SHADER: &str = r#"
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

struct Uniforms {
    rect: vec4<f32>,
    color: vec4<f32>,
};
@group(0) @binding(0) var<uniform> u: Uniforms;

@vertex
fn vs_main(@builtin(vertex_index) idx: u32) -> VertexOutput {
    var positions = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(0.0, 1.0),
    );

    let p = positions[idx];
    let rect = u.rect;

    var out: VertexOutput;
    out.position = vec4<f32>(rect.x + p.x * rect.z, rect.y + p.y * rect.w, 0.0, 1.0);
    out.color = u.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
"#;

/// Información del GPU adapter
#[derive(Debug, Clone)]
pub struct AdapterInfo {
    pub name: String,
    pub vendor: String,
    pub backend: String,
}

/// Estado del renderer WebGPU
pub struct GpuState {
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
    pub pipeline: Arc<wgpu::RenderPipeline>,
    pub bind_group: Arc<wgpu::BindGroup>,
    pub uniform_buffer: Arc<wgpu::Buffer>,
    pub adapter_info: AdapterInfo,
}

/// Crea el estado WebGPU (device, queue, pipeline)
pub async fn create_gpu_state(adapter: &wgpu::Adapter) -> Result<GpuState, String> {
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: Some("noir-gpu-device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::downlevel_defaults(),
            memory_hints: wgpu::MemoryHints::Performance,
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
            trace: wgpu::Trace::Off,
        })
        .await
        .map_err(|e| format!("Failed to create device: {}", e))?;

    let device = Arc::new(device);
    let queue = Arc::new(queue);

    // Create shader
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("rect-shader"),
        source: wgpu::ShaderSource::Wgsl(RECT_SHADER.into()),
    });

    // Create uniform buffer
    let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("rect-uniform"),
        size: std::mem::size_of::<RectUniform>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    // Create bind group layout
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("rect-bind-layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("rect-bind-group"),
        layout: &bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform_buffer.as_entire_binding(),
        }],
    });

    // Create pipeline
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("rect-pipeline-layout"),
        bind_group_layouts: &[Some(&bind_group_layout)],
        immediate_size: 0,
    });

    // Use a basic format for the pipeline
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("rect-pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: Some(std::num::NonZero::new(1).unwrap()),
        cache: None,
    });

    let info = adapter.get_info();
    let adapter_info = AdapterInfo {
        name: info.name.clone(),
        vendor: format!("{:?}", info.vendor),
        backend: format!("{:?}", info.backend),
    };

    Ok(GpuState {
        device,
        queue,
        pipeline: Arc::new(pipeline),
        bind_group: Arc::new(bind_group),
        uniform_buffer: Arc::new(uniform_buffer),
        adapter_info,
    })
}

/// Crea una instancia wgpu
pub fn create_instance() -> wgpu::Instance {
    wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        flags: wgpu::InstanceFlags::default(),
        memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
        backend_options: wgpu::BackendOptions::default(),
        display: None,
    })
}

/// Solicita un adaptador
pub async fn request_adapter(
    instance: &wgpu::Instance,
) -> Result<wgpu::Adapter, wgpu::RequestAdapterError> {
    instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            compatible_surface: None,
        })
        .await
}

/// Convierte coordenadas de pixel a NDC (Normalized Device Coordinates)
pub fn pixel_to_ndc(x: f32, y: f32, w: f32, h: f32, screen_w: f32, screen_h: f32) -> RectUniform {
    let ndc_x = (x / screen_w) * 2.0 - 1.0;
    let ndc_y = 1.0 - (y / screen_h) * 2.0;
    let ndc_w = (w / screen_w) * 2.0;
    let ndc_h = (h / screen_h) * 2.0;

    RectUniform {
        x: ndc_x,
        y: ndc_y,
        w: ndc_w,
        h: -ndc_h, // Invert Y
        color: [1.0, 1.0, 1.0, 1.0],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_draw_rect_struct() {
        let rect = DrawRect {
            x: 10.0,
            y: 20.0,
            w: 100.0,
            h: 50.0,
            color: [1.0, 0.0, 0.0, 1.0],
        };
        assert_eq!(rect.x, 10.0);
        assert_eq!(rect.color[0], 1.0);
    }

    #[test]
    fn test_rect_uniform_size() {
        let size = std::mem::size_of::<RectUniform>();
        assert_eq!(size, 32);
    }

    #[test]
    fn test_rect_uniform_alignment() {
        let uniform = RectUniform {
            x: 0.0, y: 0.0, w: 1.0, h: 1.0,
            color: [0.0; 4],
        };
        let bytes = bytemuck::bytes_of(&uniform);
        assert_eq!(bytes.len(), 32);
    }

    #[test]
    fn test_wgsl_shader_not_empty() {
        assert!(!RECT_SHADER.is_empty());
        assert!(RECT_SHADER.contains("@vertex"));
        assert!(RECT_SHADER.contains("@fragment"));
    }

    #[test]
    fn test_draw_rect_default() {
        let rect = DrawRect {
            x: 0.0,
            y: 0.0,
            w: 0.0,
            h: 0.0,
            color: [0.0; 4],
        };
        assert_eq!(rect.x, 0.0);
    }

    #[test]
    fn test_wgsl_has_uniform_struct() {
        assert!(RECT_SHADER.contains("struct Uniforms"));
    }

    #[test]
    fn test_wgsl_has_vertex_output() {
        assert!(RECT_SHADER.contains("VertexOutput"));
    }

    #[test]
    fn test_pixel_to_ndc_origin() {
        let u = pixel_to_ndc(0.0, 0.0, 100.0, 100.0, 800.0, 600.0);
        assert!((u.x - (-1.0)).abs() < 0.001);
    }

    #[test]
    fn test_pixel_to_ndc_center() {
        let u = pixel_to_ndc(400.0, 300.0, 100.0, 100.0, 800.0, 600.0);
        assert!(u.x.abs() < 0.001);
    }

    #[test]
    fn test_pixel_to_ndc_size() {
        let u = pixel_to_ndc(0.0, 0.0, 800.0, 600.0, 800.0, 600.0);
        assert!((u.w - 2.0).abs() < 0.001);
    }

    #[test]
    fn test_create_instance() {
        let _instance = create_instance();
    }

    #[test]
    fn test_adapter_info_struct() {
        let info = AdapterInfo {
            name: "Test".to_string(),
            vendor: "TestVendor".to_string(),
            backend: "Vulkan".to_string(),
        };
        assert_eq!(info.name, "Test");
    }
}
