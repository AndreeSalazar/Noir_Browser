// AUTO-GENERATED VULKAN SHADERS (GLSL -> SPIR-V)
use ash::vk;
use shaderc;

pub const VERTEX_SHADER_GLSL: &str = r#"
    #version 450
    layout(location = 0) in vec2 inPosition;
    layout(location = 1) in vec2 inTexCoord;
    
    layout(location = 0) out vec2 fragTexCoord;
    
    void main() {
        gl_Position = vec4(inPosition, 0.0, 1.0);
        fragTexCoord = inTexCoord;
    }
"#;

pub const FRAGMENT_SHADER_GLSL: &str = r#"
    #version 450
    layout(location = 0) in vec2 fragTexCoord;
    
    layout(binding = 0) uniform sampler2D texSampler;
    
    layout(location = 0) out vec4 outColor;
    
    void main() {
        // Fontdue bitmap is often single channel (grayscale) or RGBA.
        // Assuming RGBA uploaded to texture.
        vec4 texColor = texture(texSampler, fragTexCoord);
        // Premium White text color applied over the bitmap alpha
        outColor = vec4(1.0, 1.0, 1.0, texColor.a);
    }
"#;

pub struct ShaderModuleLoader;

impl ShaderModuleLoader {
    pub fn compile_glsl_to_spirv(source: &str, kind: shaderc::ShaderKind, name: &str) -> Vec<u32> {
        let compiler = shaderc::Compiler::new().expect("Failed to initialize shader compiler");
        let mut options = shaderc::CompileOptions::new().unwrap();
        options.set_optimization_level(shaderc::OptimizationLevel::Performance);
        
        let binary_result = compiler.compile_into_spirv(
            source, kind, name, "main", Some(&options)
        ).expect("Failed to compile shader");
        
        binary_result.as_binary().to_vec()
    }

    pub fn create_shader_module(device: &ash::Device, code: &[u32]) -> vk::ShaderModule {
        let create_info = vk::ShaderModuleCreateInfo::builder().code(code);
        unsafe { device.create_shader_module(&create_info, None).expect("Failed to create shader module") }
    }
}