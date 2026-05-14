import os

class VulkanLayoutGen:
    def __init__(self, output_dir="No-Chromium/src/vulkan_engine"):
        self.output_dir = output_dir

    def run(self):
        print("[*] Generando módulo layout_gen.rs para CSS -> GPU Vertices...")
        code = [
            "// AUTO-GENERATED VULKAN LAYOUT ENGINE",
            "use crate::vulkan_engine::ui_gen::UIVertex;",
            "use crate::generated_rust::css_engine::ComputedStyle;",
            "",
            "pub struct LayoutEngine;",
            "",
            "impl LayoutEngine {",
            "    pub fn parse_color(hex: &str) -> (f32, f32, f32, f32) {",
            "        let hex = hex.trim_start_matches('#');",
            "        if hex.len() == 6 {",
            "            let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255) as f32 / 255.0;",
            "            let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(255) as f32 / 255.0;",
            "            let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255) as f32 / 255.0;",
            "            (r, g, b, 1.0)",
            "        } else {",
            "            (1.0, 1.0, 1.0, 1.0) // Default to white",
            "        }",
            "    }",
            "",
            "    pub fn parse_px(val: &str) -> f32 {",
            "        let clean = val.replace(\"px\", \"\").trim().to_string();",
            "        clean.parse::<f32>().unwrap_or(0.0)",
            "    }",
            "",
            "    pub fn build_dom_vertices(style: &ComputedStyle, window_width: f32, window_height: f32) -> Vec<UIVertex> {",
            "        let mut vertices = Vec::new();",
            "        ",
            "        // 1. Extraer propiedades CSS",
            "        let width = if let Some(w) = &style.width { Self::parse_px(w) } else { 100.0 };",
            "        let height = if let Some(h) = &style.height { Self::parse_px(h) } else { 100.0 };",
            "        let color_hex = if let Some(c) = &style.background_color { c.clone() } else { \"#ffffff\".to_string() };",
            "        let (r, g, b, a) = Self::parse_color(&color_hex);",
            "",
            "        // 2. Calcular coordenadas (Top-Left 0,0 default offset por ahora debajo del Top Bar 40px)",
            "        let px_x = 0.0;",
            "        let px_y = 40.0; // Debajo del Custom Chrome",
            "",
            "        // 3. Convertir a NDC (-1.0 a 1.0)",
            "        let ndc_x = -1.0 + (px_x / window_width) * 2.0;",
            "        let ndc_y = -1.0 + (px_y / window_height) * 2.0;",
            "        let ndc_w = (width / window_width) * 2.0;",
            "        let ndc_h = (height / window_height) * 2.0;",
            "",
            "        // 4. Generar el Quad (Solid Color)",
            "        vertices.push(UIVertex::solid(ndc_x, ndc_y, r, g, b, a));",
            "        vertices.push(UIVertex::solid(ndc_x + ndc_w, ndc_y, r, g, b, a));",
            "        vertices.push(UIVertex::solid(ndc_x, ndc_y + ndc_h, r, g, b, a));",
            "        ",
            "        vertices.push(UIVertex::solid(ndc_x + ndc_w, ndc_y, r, g, b, a));",
            "        vertices.push(UIVertex::solid(ndc_x + ndc_w, ndc_y + ndc_h, r, g, b, a));",
            "        vertices.push(UIVertex::solid(ndc_x, ndc_y + ndc_h, r, g, b, a));",
            "",
            "        vertices",
            "    }",
            "}",
        ]
        with open(os.path.join(self.output_dir, "layout_gen.rs"), "w", encoding='utf-8') as f:
            f.write("\n".join(code))
        
        # update mod.rs
        mod_file = os.path.join(self.output_dir, "mod.rs")
        if os.path.exists(mod_file):
            with open(mod_file, "r") as f:
                content = f.read()
            if "pub mod layout_gen;" not in content:
                with open(mod_file, "a") as f:
                    f.write("\npub mod layout_gen;\n")

if __name__ == "__main__":
    VulkanLayoutGen().run()
