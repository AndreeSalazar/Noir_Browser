#version 450
layout(location = 0) in vec2 inPosition;
layout(location = 1) in vec4 inColor;
layout(location = 2) in vec2 inTexCoord;
layout(location = 3) in vec4 inExtra; // box_w, box_h, radius, is_text

layout(location = 0) out vec4 fragColor;
layout(location = 1) out vec2 fragTexCoord;
layout(location = 2) out vec4 fragExtra;

layout(push_constant) uniform PushConstants {
    mat4 proj;
} pcs;

void main() {
    vec4 projected = pcs.proj * vec4(inPosition, 0.0, 1.0);
    if (projected.x < -1.0 || projected.x > 1.0 || projected.y < -1.0 || projected.y > 1.0) {
        gl_Position = vec4(-10.0, -10.0, -10.0, 1.0);
    } else {
        gl_Position = projected;
    }
    fragColor = inColor;
    fragTexCoord = inTexCoord;
    fragExtra = inExtra;
}
