// AUTO-GENERATED VULKAN LAYOUT ENGINE
use crate::ui::ui_gen::UIVertex;

pub struct LayoutEngine;

impl LayoutEngine {
    pub fn parse_color(hex: &str) -> (f32, f32, f32, f32) {
        let hex = hex.trim().trim_start_matches('#');
        if hex.len() == 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255) as f32 / 255.0;
            let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(255) as f32 / 255.0;
            let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255) as f32 / 255.0;
            (r, g, b, 1.0)
        } else {
            (1.0, 1.0, 1.0, 1.0) // Default to white
        }
    }

    pub fn parse_px_or_percent(val: &str, base: f32) -> f32 {
        let clean = val.trim();
        if let Some(percent) = clean.strip_suffix('%') {
            return percent.trim().parse::<f32>().unwrap_or(100.0) * base / 100.0;
        }

        clean
            .replace("px", "")
            .trim()
            .parse::<f32>()
            .unwrap_or(base)
    }

    pub fn build_dom_vertices(
        boxes: &[crate::browser::RenderBox],
        _window_width: f32,
        _window_height: f32,
    ) -> Vec<UIVertex> {
        let mut vertices = Vec::new();

        for b in boxes {
            let x = b.x;
            let y = b.y;
            let w = b.w;
            let h = b.h;

            let (r, g, b_col, a) = (b.color[0], b.color[1], b.color[2], b.color[3]);

            vertices.push(UIVertex::solid_box(x, y, r, g, b_col, a, 0.0, 0.0, b.w, b.h, b.radius));
            vertices.push(UIVertex::solid_box(x + w, y, r, g, b_col, a, b.w, 0.0, b.w, b.h, b.radius));
            vertices.push(UIVertex::solid_box(x, y + h, r, g, b_col, a, 0.0, b.h, b.w, b.h, b.radius));

            vertices.push(UIVertex::solid_box(x + w, y, r, g, b_col, a, b.w, 0.0, b.w, b.h, b.radius));
            vertices.push(UIVertex::solid_box(x + w, y + h, r, g, b_col, a, b.w, b.h, b.w, b.h, b.radius));
            vertices.push(UIVertex::solid_box(x, y + h, r, g, b_col, a, 0.0, b.h, b.w, b.h, b.radius));
        }

        vertices
    }
}
