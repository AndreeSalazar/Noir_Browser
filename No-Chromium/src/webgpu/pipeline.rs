//! WebGPU Pipeline - Render pipeline configuration
use super::shaders::ShaderModule;
use super::buffer::VertexFormat;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PrimitiveTopology {
    PointList,
    LineList,
    LineStrip,
    TriangleList,
    TriangleStrip,
}

impl PrimitiveTopology {
    pub fn name(&self) -> &'static str {
        match self {
            PrimitiveTopology::PointList => "PointList",
            PrimitiveTopology::LineList => "LineList",
            PrimitiveTopology::LineStrip => "LineStrip",
            PrimitiveTopology::TriangleList => "TriangleList",
            PrimitiveTopology::TriangleStrip => "TriangleStrip",
        }
    }
}

#[derive(Debug, Clone)]
pub struct PipelineConfig {
    pub name: String,
    pub vertex_format: VertexFormat,
    pub topology: PrimitiveTopology,
    pub vertex_shader: ShaderModule,
    pub fragment_shader: Option<ShaderModule>,
    pub cull_mode: Option<CullMode>,
    pub depth_test: bool,
    pub depth_write: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CullMode {
    None,
    Front,
    Back,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            vertex_format: VertexFormat::basic_2d(),
            topology: PrimitiveTopology::TriangleList,
            vertex_shader: ShaderModule::new(
                "default_vertex",
                super::shaders::ShaderStage::Vertex,
                super::shaders::SHADER_SOLID_WGSL,
                "vs_main",
            ),
            fragment_shader: Some(ShaderModule::new(
                "default_fragment",
                super::shaders::ShaderStage::Fragment,
                super::shaders::SHADER_SOLID_WGSL,
                "fs_main",
            )),
            cull_mode: Some(CullMode::None),
            depth_test: false,
            depth_write: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Pipeline {
    pub id: u64,
    pub config: PipelineConfig,
    pub compiled: bool,
}

impl Pipeline {
    pub fn new(id: u64, config: PipelineConfig) -> Self {
        Self {
            id,
            config,
            compiled: false,
        }
    }

    pub fn compile(&mut self) {
        self.compiled = true;
    }
}
