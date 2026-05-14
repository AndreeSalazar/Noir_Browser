// AUTO-GENERATED VULKAN CHROME UI BUILDER
pub struct UIVertex {
    pub x: f32, pub y: f32,
    pub r: f32, pub g: f32, pub b: f32, pub a: f32,
    pub u: f32, pub v: f32,
}

impl UIVertex {
    pub fn solid(x: f32, y: f32, r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { x, y, r, g, b, a, u: -1.0, v: -1.0 } // -1.0 indicates solid color to the shader
    }
    pub fn textured(x: f32, y: f32, r: f32, g: f32, b: f32, a: f32, u: f32, v: f32) -> Self {
        Self { x, y, r, g, b, a, u, v }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UIButton {
    Close,
    Minimize,
    Maximize,
}

#[derive(Debug, Clone, Copy)]
pub struct UIBoundingBox {
    pub button: UIButton,
    pub x_min: f32,
    pub x_max: f32,
    pub y_min: f32,
    pub y_max: f32,
}

pub fn get_ui_hitboxes(width: f32, _height: f32) -> Vec<UIBoundingBox> {
    vec![
        UIBoundingBox { button: UIButton::Close, x_min: width - 40.0, x_max: width, y_min: 0.0, y_max: 40.0 },
        UIBoundingBox { button: UIButton::Maximize, x_min: width - 80.0, x_max: width - 40.0, y_min: 0.0, y_max: 40.0 },
        UIBoundingBox { button: UIButton::Minimize, x_min: width - 120.0, x_max: width - 80.0, y_min: 0.0, y_max: 40.0 },
    ]
}

pub fn generate_chrome_vertices(width: f32, height: f32) -> Vec<f32> {
    let mut raw_data = Vec::new();
    
    // Normalizar a coordenadas de Vulkan (-1.0 a 1.0)
    // Top bar: 0 to 40px height
    let top_y = -1.0;
    let bottom_y = -1.0 + (40.0 / height) * 2.0;
    let left_x = -1.0;
    let right_x = 1.0;
    
    // 1. Background Bar (#1e1e2e)
    let bg_color = (0.117, 0.117, 0.180, 1.0);
    let bg_quad = [
        UIVertex::solid(left_x, top_y, bg_color.0, bg_color.1, bg_color.2, bg_color.3),
        UIVertex::solid(right_x, top_y, bg_color.0, bg_color.1, bg_color.2, bg_color.3),
        UIVertex::solid(left_x, bottom_y, bg_color.0, bg_color.1, bg_color.2, bg_color.3),
        UIVertex::solid(right_x, top_y, bg_color.0, bg_color.1, bg_color.2, bg_color.3),
        UIVertex::solid(right_x, bottom_y, bg_color.0, bg_color.1, bg_color.2, bg_color.3),
        UIVertex::solid(left_x, bottom_y, bg_color.0, bg_color.1, bg_color.2, bg_color.3),
    ];
    
    // 2. Close Button (#ff4d4d) right aligned (40x40)
    let close_left_x = 1.0 - (40.0 / width) * 2.0;
    let c_color = (1.0, 0.301, 0.301, 1.0);
    let close_quad = [
        UIVertex::solid(close_left_x, top_y, c_color.0, c_color.1, c_color.2, c_color.3),
        UIVertex::solid(right_x, top_y, c_color.0, c_color.1, c_color.2, c_color.3),
        UIVertex::solid(close_left_x, bottom_y, c_color.0, c_color.1, c_color.2, c_color.3),
        UIVertex::solid(right_x, top_y, c_color.0, c_color.1, c_color.2, c_color.3),
        UIVertex::solid(right_x, bottom_y, c_color.0, c_color.1, c_color.2, c_color.3),
        UIVertex::solid(close_left_x, bottom_y, c_color.0, c_color.1, c_color.2, c_color.3),
    ];

    // 3. Maximize Button (#4a4a6a) (40x40)
    let max_left_x = 1.0 - (80.0 / width) * 2.0;
    let m_color = (0.290, 0.290, 0.415, 1.0);
    let max_quad = [
        UIVertex::solid(max_left_x, top_y, m_color.0, m_color.1, m_color.2, m_color.3),
        UIVertex::solid(close_left_x, top_y, m_color.0, m_color.1, m_color.2, m_color.3),
        UIVertex::solid(max_left_x, bottom_y, m_color.0, m_color.1, m_color.2, m_color.3),
        UIVertex::solid(close_left_x, top_y, m_color.0, m_color.1, m_color.2, m_color.3),
        UIVertex::solid(close_left_x, bottom_y, m_color.0, m_color.1, m_color.2, m_color.3),
        UIVertex::solid(max_left_x, bottom_y, m_color.0, m_color.1, m_color.2, m_color.3),
    ];

    // 4. Minimize Button (#4a4a6a) (40x40)
    let min_left_x = 1.0 - (120.0 / width) * 2.0;
    let min_quad = [
        UIVertex::solid(min_left_x, top_y, m_color.0, m_color.1, m_color.2, m_color.3),
        UIVertex::solid(max_left_x, top_y, m_color.0, m_color.1, m_color.2, m_color.3),
        UIVertex::solid(min_left_x, bottom_y, m_color.0, m_color.1, m_color.2, m_color.3),
        UIVertex::solid(max_left_x, top_y, m_color.0, m_color.1, m_color.2, m_color.3),
        UIVertex::solid(max_left_x, bottom_y, m_color.0, m_color.1, m_color.2, m_color.3),
        UIVertex::solid(min_left_x, bottom_y, m_color.0, m_color.1, m_color.2, m_color.3),
    ];
    
    // 5. Separator (y=40 to y=41) - 10% white alpha
    let sep_top_y = bottom_y;
    let sep_bottom_y = -1.0 + (41.0 / height) * 2.0;
    let sep_color = (1.0, 1.0, 1.0, 0.1);
    let sep_quad = [
        UIVertex::solid(left_x, sep_top_y, sep_color.0, sep_color.1, sep_color.2, sep_color.3),
        UIVertex::solid(right_x, sep_top_y, sep_color.0, sep_color.1, sep_color.2, sep_color.3),
        UIVertex::solid(left_x, sep_bottom_y, sep_color.0, sep_color.1, sep_color.2, sep_color.3),
        UIVertex::solid(right_x, sep_top_y, sep_color.0, sep_color.1, sep_color.2, sep_color.3),
        UIVertex::solid(right_x, sep_bottom_y, sep_color.0, sep_color.1, sep_color.2, sep_color.3),
        UIVertex::solid(left_x, sep_bottom_y, sep_color.0, sep_color.1, sep_color.2, sep_color.3),
    ];

    // 6. URL Bar Background (#0d0d1a) (y=41 to y=71)
    let url_top_y = sep_bottom_y;
    let url_bottom_y = -1.0 + (71.0 / height) * 2.0;
    let url_color = (0.051, 0.051, 0.102, 1.0); // #0d0d1a approx
    let url_quad = [
        UIVertex::solid(left_x, url_top_y, url_color.0, url_color.1, url_color.2, url_color.3),
        UIVertex::solid(right_x, url_top_y, url_color.0, url_color.1, url_color.2, url_color.3),
        UIVertex::solid(left_x, url_bottom_y, url_color.0, url_color.1, url_color.2, url_color.3),
        UIVertex::solid(right_x, url_top_y, url_color.0, url_color.1, url_color.2, url_color.3),
        UIVertex::solid(right_x, url_bottom_y, url_color.0, url_color.1, url_color.2, url_color.3),
        UIVertex::solid(left_x, url_bottom_y, url_color.0, url_color.1, url_color.2, url_color.3),
    ];
    
    for v in bg_quad.iter().chain(close_quad.iter()).chain(max_quad.iter()).chain(min_quad.iter())
        .chain(sep_quad.iter()).chain(url_quad.iter()) {
        raw_data.push(v.x); raw_data.push(v.y);
        raw_data.push(v.r); raw_data.push(v.g); raw_data.push(v.b); raw_data.push(v.a);
        raw_data.push(v.u); raw_data.push(v.v);
    }
    raw_data
}