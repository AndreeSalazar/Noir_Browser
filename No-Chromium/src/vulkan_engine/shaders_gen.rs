// AUTO-GENERATED VULKAN SHADERS (GLSL -> SPIR-V)
use ash::vk;
use shaderc;

pub const VERTEX_SHADER_GLSL: &str = r#"
    #version 450
    layout(location = 0) in vec2 inPosition;
    layout(location = 1) in vec4 inColor;
    layout(location = 2) in vec2 inTexCoord;
    layout(location = 3) in vec4 inExtra; // box_w, box_h, radius, is_text
    
    layout(location = 0) out vec4 fragColor;
    layout(location = 1) out vec2 fragTexCoord;
    layout(location = 2) out vec4 fragExtra;
    
    void main() {
        if (inPosition.x < -1.0 || inPosition.x > 1.0 || inPosition.y < -1.0 || inPosition.y > 1.0) {
            gl_Position = vec4(-10.0, -10.0, -10.0, 1.0);
        } else {
            gl_Position = vec4(inPosition, 0.0, 1.0);
        }
        fragColor = inColor;
        fragTexCoord = inTexCoord;
        fragExtra = inExtra;
    }
"#;

pub const FRAGMENT_SHADER_GLSL: &str = r#"
    #version 450
    layout(location = 0) in vec4 fragColor;
    layout(location = 1) in vec2 fragTexCoord;
    layout(location = 2) in vec4 fragExtra;
    
    layout(binding = 0) uniform sampler2D texSampler;
    
    layout(location = 0) out vec4 outColor;
    
    void main() {
        float is_text = fragExtra.w;
        
        if (is_text > 0.5) {
            vec4 mask = texture(texSampler, fragTexCoord);
            float coverage = max(max(mask.r, mask.g), max(mask.b, mask.a));
            if (coverage <= 0.001) {
                discard;
            }
            vec3 rgb = fragColor.rgb * fragColor.a * mask.rgb;
            outColor = vec4(rgb, fragColor.a * coverage);
        } else {
            float box_w = fragExtra.x;
            float box_h = fragExtra.y;
            float radius = fragExtra.z;
            
            if (radius > 0.0 && fragTexCoord.x >= 0.0) {
                vec2 local_pos = fragTexCoord;
                vec2 center = vec2(box_w * 0.5, box_h * 0.5);
                vec2 d = abs(local_pos - center) - vec2(box_w * 0.5 - radius, box_h * 0.5 - radius);
                float dist = length(max(d, 0.0)) + min(max(d.x, d.y), 0.0) - radius;
                
                float alpha = 1.0 - smoothstep(-0.5, 0.5, dist);
                if (alpha <= 0.0) {
                    discard;
                }
                outColor = vec4(fragColor.rgb * fragColor.a * alpha, fragColor.a * alpha);
            } else {
                outColor = vec4(fragColor.rgb * fragColor.a, fragColor.a);
            }
        }
    }
"#;

pub struct ShaderModuleLoader;

impl ShaderModuleLoader {
    pub fn compile_glsl_to_spirv(source: &str, kind: shaderc::ShaderKind, name: &str) -> Vec<u32> {
        let compiler = shaderc::Compiler::new().expect("Failed to initialize shader compiler");
        let mut options = shaderc::CompileOptions::new().unwrap();
        options.set_optimization_level(shaderc::OptimizationLevel::Performance);

        let binary_result = compiler
            .compile_into_spirv(source, kind, name, "main", Some(&options))
            .expect("Failed to compile shader");

        binary_result.as_binary().to_vec()
    }

    pub fn create_shader_module(device: &ash::Device, code: &[u32]) -> vk::ShaderModule {
        let create_info = vk::ShaderModuleCreateInfo::builder().code(code);
        unsafe {
            device
                .create_shader_module(&create_info, None)
                .expect("Failed to create shader module")
        }
    }
}
