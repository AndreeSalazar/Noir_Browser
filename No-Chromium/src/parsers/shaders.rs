// AUTO-GENERATED SHADER CONSTANTS
// WebGPU replacement for Skia 2D rendering
pub const VERTEX_SHADER_GLSL: &str = r#"
    #version 450
    layout(location = 0) in vec2 inPosition;
    layout(location = 1) in vec3 inColor;
    layout(location = 0) out vec3 fragColor;
    void main() {
        gl_Position = vec4(inPosition, 0.0, 1.0);
        fragColor = inColor;
    }
"#;
pub const FRAGMENT_SHADER_GLSL: &str = r#"
    #version 450
    layout(location = 0) in vec3 fragColor;
    layout(location = 0) out vec4 outColor;
    void main() {
        outColor = vec4(fragColor, 1.0);
    }
"#;
