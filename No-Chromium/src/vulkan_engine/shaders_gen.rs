// AUTO-GENERATED VULKAN SHADERS (GLSL -> SPIR-V)
use ash::vk;
use shaderc;

pub const VERTEX_SHADER_GLSL: &str = r#"
    #version 450
    layout(location = 0) in vec2 inPosition;
    layout(location = 1) in vec4 inColor;
    layout(location = 2) in vec2 inTexCoord;
    
    layout(location = 0) out vec4 fragColor;
    layout(location = 1) out vec2 fragTexCoord;
    
    void main() {
        gl_Position = vec4(inPosition, 0.0, 1.0);
        fragColor = inColor;
        fragTexCoord = inTexCoord;
    }
"#;

pub const FRAGMENT_SHADER_GLSL: &str = r#"
    #version 450
    layout(location = 0) in vec4 fragColor;
    layout(location = 1) in vec2 fragTexCoord;
    
    layout(binding = 0) uniform sampler2D texSampler;
    
    layout(location = 0) out vec4 outColor;
    
    void main() {
        if (fragTexCoord.x < 0.0) {
            outColor = fragColor;
        } else {
            float dist = texture(texSampler, fragTexCoord).a;
            float smoothing = fwidth(dist);
            float alpha = smoothstep(0.5 - smoothing, 0.5 + smoothing, dist);
            outColor = vec4(fragColor.rgb, fragColor.a * alpha);
        }
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