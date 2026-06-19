#version 450
layout(location = 0) in vec4 fragColor;
layout(location = 1) in vec2 fragTexCoord;
layout(location = 2) in vec4 fragExtra;

layout(binding = 0) uniform sampler2D texSampler;

layout(location = 0) out vec4 outColor;

void main() {
    float is_text = fragExtra.w;
    
    if (is_text > 1.5) {
        // Actual Images (is_text == 2.0)
        vec4 texColor = texture(texSampler, fragTexCoord);
        if (texColor.a <= 0.001) {
            discard;
        }
        outColor = texColor * fragColor;
    } else if (is_text > 0.5) {
        // Text glyph (is_text == 1.0)
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
