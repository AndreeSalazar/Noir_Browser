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
        use std::sync::OnceLock;
        static START_TIME: OnceLock<std::time::Instant> = OnceLock::new();
        let start = START_TIME.get_or_init(std::time::Instant::now);
        let elapsed = start.elapsed().as_secs_f32();
        let pulse = (elapsed * 3.5).sin() * 0.5 + 0.5; // range 0.0 to 1.0

        let mut vertices = Vec::new();

        for b in boxes {
            let x = b.x;
            let y = b.y;
            let w = b.w;
            let h = b.h;

            let mut r = b.color[0];
            let mut g = b.color[1];
            let mut b_col = b.color[2];
            let mut a = b.color[3];
            let radius = b.radius;

            // Detect Google Search Input Border
            let is_search_border = (radius - 22.0).abs() < 0.1 && (b.color[0] - 0.35).abs() < 0.1;
            let is_search_border_focused = (radius - 22.0).abs() < 0.1 && (b.color[0] - 0.26).abs() < 0.1;
            // Detect Google Buttons Border
            let is_button_border = (radius - 18.0).abs() < 0.1 && (b.color[0] - 0.12).abs() < 0.1;

            if is_search_border || is_search_border_focused {
                // Neon glowing cyan border
                r = 0.120;
                g = 0.650 + pulse * 0.15;
                b_col = 0.820 + pulse * 0.18;
                a = 0.7 + pulse * 0.3;

                // Push an outer neon glow box behind it!
                let glow_offset = 2.0 + pulse * 3.0;
                let gx = x - glow_offset;
                let gy = y - glow_offset;
                let gw = w + glow_offset * 2.0;
                let gh = h + glow_offset * 2.0;
                let gr = radius + glow_offset;
                let ga = 0.15 + pulse * 0.15; // soft glow alpha

                vertices.push(UIVertex::solid_box(gx, gy, r, g, b_col, ga, 0.0, 0.0, gw, gh, gr));
                vertices.push(UIVertex::solid_box(gx + gw, gy, r, g, b_col, ga, gw, 0.0, gw, gh, gr));
                vertices.push(UIVertex::solid_box(gx, gy + gh, r, g, b_col, ga, 0.0, gh, gw, gh, gr));

                vertices.push(UIVertex::solid_box(gx + gw, gy, r, g, b_col, ga, gw, 0.0, gw, gh, gr));
                vertices.push(UIVertex::solid_box(gx + gw, gy + gh, r, g, b_col, ga, gw, gh, gw, gh, gr));
                vertices.push(UIVertex::solid_box(gx, gy + gh, r, g, b_col, ga, 0.0, gh, gw, gh, gr));
            } else if is_button_border {
                // Neon cyan border for button
                r = 0.120;
                g = 0.650 + pulse * 0.10;
                b_col = 0.820 + pulse * 0.12;
                a = 0.4 + pulse * 0.3;

                // Push a soft outer glow box behind it
                let glow_offset = 1.0 + pulse * 2.0;
                let gx = x - glow_offset;
                let gy = y - glow_offset;
                let gw = w + glow_offset * 2.0;
                let gh = h + glow_offset * 2.0;
                let gr = radius + glow_offset;
                let ga = 0.10 + pulse * 0.10;

                vertices.push(UIVertex::solid_box(gx, gy, r, g, b_col, ga, 0.0, 0.0, gw, gh, gr));
                vertices.push(UIVertex::solid_box(gx + gw, gy, r, g, b_col, ga, gw, 0.0, gw, gh, gr));
                vertices.push(UIVertex::solid_box(gx, gy + gh, r, g, b_col, ga, 0.0, gh, gw, gh, gr));

                vertices.push(UIVertex::solid_box(gx + gw, gy, r, g, b_col, ga, gw, 0.0, gw, gh, gr));
                vertices.push(UIVertex::solid_box(gx + gw, gy + gh, r, g, b_col, ga, gw, gh, gw, gh, gr));
                vertices.push(UIVertex::solid_box(gx, gy + gh, r, g, b_col, ga, 0.0, gh, gw, gh, gr));
            }

            vertices.push(UIVertex::solid_box(x, y, r, g, b_col, a, 0.0, 0.0, w, h, radius));
            vertices.push(UIVertex::solid_box(x + w, y, r, g, b_col, a, w, 0.0, w, h, radius));
            vertices.push(UIVertex::solid_box(x, y + h, r, g, b_col, a, 0.0, h, w, h, radius));

            vertices.push(UIVertex::solid_box(x + w, y, r, g, b_col, a, w, 0.0, w, h, radius));
            vertices.push(UIVertex::solid_box(x + w, y + h, r, g, b_col, a, w, h, w, h, radius));
            vertices.push(UIVertex::solid_box(x, y + h, r, g, b_col, a, 0.0, h, w, h, radius));
        }

        vertices
    }
}
