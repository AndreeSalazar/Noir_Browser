// AUTO-GENERATED VULKAN CHROME UI BUILDER
pub struct UIVertex {
    pub x: f32,
    pub y: f32,
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
    pub u: f32,
    pub v: f32,
    pub box_w: f32,
    pub box_h: f32,
    pub radius: f32,
    pub is_text: f32,
}

impl UIVertex {
    pub fn solid(x: f32, y: f32, r: f32, g: f32, b: f32, a: f32) -> Self {
        Self {
            x,
            y,
            r,
            g,
            b,
            a,
            u: -1.0,
            v: -1.0,
            box_w: 0.0,
            box_h: 0.0,
            radius: 0.0,
            is_text: 0.0,
        }
    }
    
    pub fn solid_box(x: f32, y: f32, r: f32, g: f32, b: f32, a: f32, u: f32, v: f32, box_w: f32, box_h: f32, radius: f32) -> Self {
        Self {
            x, y, r, g, b, a, u, v, box_w, box_h, radius, is_text: 0.0,
        }
    }
    pub fn textured(x: f32, y: f32, r: f32, g: f32, b: f32, a: f32, u: f32, v: f32) -> Self {
        Self {
            x,
            y,
            r,
            g,
            b,
            a,
            u,
            v,
            box_w: 0.0,
            box_h: 0.0,
            radius: 0.0,
            is_text: 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UIButton {
    Back,
    Forward,
    Reload,
    Home,
    AddressBar,
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

pub struct UILayout {
    pub back_btn: UIBoundingBox,
    pub forward_btn: UIBoundingBox,
    pub reload_btn: UIBoundingBox,
    pub home_btn: UIBoundingBox,
    pub address_bar: UIBoundingBox,
    pub minimize_btn: UIBoundingBox,
    pub maximize_btn: UIBoundingBox,
    pub close_btn: UIBoundingBox,
}

impl UILayout {
    pub fn new(width: f32, _height: f32) -> Self {
        let url_left = 168.0;
        let url_right = (width - 150.0).max(url_left + 120.0);

        Self {
            back_btn: UIBoundingBox {
                button: UIButton::Back,
                x_min: 12.0,
                x_max: 42.0,
                y_min: 43.0,
                y_max: 65.0,
            },
            forward_btn: UIBoundingBox {
                button: UIButton::Forward,
                x_min: 48.0,
                x_max: 78.0,
                y_min: 43.0,
                y_max: 65.0,
            },
            reload_btn: UIBoundingBox {
                button: UIButton::Reload,
                x_min: 84.0,
                x_max: 114.0,
                y_min: 43.0,
                y_max: 65.0,
            },
            home_btn: UIBoundingBox {
                button: UIButton::Home,
                x_min: 120.0,
                x_max: 150.0,
                y_min: 43.0,
                y_max: 65.0,
            },
            address_bar: UIBoundingBox {
                button: UIButton::AddressBar,
                x_min: url_left,
                x_max: url_right,
                y_min: 43.0,
                y_max: 65.0,
            },
            minimize_btn: UIBoundingBox {
                button: UIButton::Minimize,
                x_min: width - 138.0,
                x_max: width - 92.0,
                y_min: 0.0,
                y_max: 36.0,
            },
            maximize_btn: UIBoundingBox {
                button: UIButton::Maximize,
                x_min: width - 92.0,
                x_max: width - 46.0,
                y_min: 0.0,
                y_max: 36.0,
            },
            close_btn: UIBoundingBox {
                button: UIButton::Close,
                x_min: width - 46.0,
                x_max: width,
                y_min: 0.0,
                y_max: 36.0,
            },
        }
    }

    pub fn get_hitboxes(&self) -> Vec<UIBoundingBox> {
        vec![
            self.back_btn,
            self.forward_btn,
            self.reload_btn,
            self.home_btn,
            self.address_bar,
            self.minimize_btn,
            self.maximize_btn,
            self.close_btn,
        ]
    }
}

pub fn get_ui_hitboxes(width: f32, height: f32) -> Vec<UIBoundingBox> {
    UILayout::new(width, height).get_hitboxes()
}

pub fn generate_chrome_vertices(width: f32, height: f32) -> Vec<f32> {
    let mut raw_data = Vec::new();
    let layout = UILayout::new(width, height);

    let chrome = (0.070, 0.074, 0.105, 1.0);
    let toolbar = (0.095, 0.098, 0.142, 1.0);
    let pill = (0.035, 0.038, 0.058, 1.0);
    let pill_edge = (0.180, 0.190, 0.260, 1.0);
    let subtle = (0.565, 0.600, 0.710, 1.0);
    let bright = (0.900, 0.925, 1.000, 1.0);
    let accent = (0.260, 0.520, 0.980, 1.0);

    push_quad_px(&mut raw_data, width, height, 0.0, 0.0, width, 36.0, chrome);
    push_quad_px(
        &mut raw_data,
        width,
        height,
        0.0,
        36.0,
        width,
        36.0,
        toolbar,
    );
    push_quad_px(
        &mut raw_data,
        width,
        height,
        0.0,
        71.0,
        width,
        1.0,
        (1.0, 1.0, 1.0, 0.09),
    );

    let url_w = layout.address_bar.x_max - layout.address_bar.x_min;
    let url_h = layout.address_bar.y_max - layout.address_bar.y_min;

    push_pill_px(
        &mut raw_data,
        width,
        height,
        layout.address_bar.x_min,
        layout.address_bar.y_min,
        url_w,
        url_h,
        pill,
        8.0,
    );
    push_quad_px(
        &mut raw_data,
        width,
        height,
        layout.address_bar.x_min,
        layout.address_bar.y_min,
        url_w,
        1.0,
        pill_edge,
    );
    push_quad_px(
        &mut raw_data,
        width,
        height,
        layout.address_bar.x_min,
        layout.address_bar.y_max - 1.0,
        url_w,
        1.0,
        (0.0, 0.0, 0.0, 0.26),
    );

    push_nav_button(&mut raw_data, width, height, layout.back_btn.x_min, layout.back_btn.y_min, subtle);
    push_icon_back(&mut raw_data, width, height, layout.back_btn.x_min + 15.0, layout.back_btn.y_min + 11.0, bright);

    push_nav_button(
        &mut raw_data,
        width,
        height,
        layout.forward_btn.x_min,
        layout.forward_btn.y_min,
        (0.420, 0.450, 0.540, 1.0),
    );
    push_icon_forward(
        &mut raw_data,
        width,
        height,
        layout.forward_btn.x_min + 15.0,
        layout.forward_btn.y_min + 11.0,
        (0.420, 0.450, 0.540, 1.0),
    );

    push_nav_button(&mut raw_data, width, height, layout.reload_btn.x_min, layout.reload_btn.y_min, subtle);
    push_icon_reload(&mut raw_data, width, height, layout.reload_btn.x_min + 15.0, layout.reload_btn.y_min + 11.0, bright);

    push_nav_button(&mut raw_data, width, height, layout.home_btn.x_min, layout.home_btn.y_min, subtle);
    push_icon_home(&mut raw_data, width, height, layout.home_btn.x_min + 15.0, layout.home_btn.y_min + 11.0, bright);

    push_icon_lock(&mut raw_data, width, height, layout.address_bar.x_min + 16.0, layout.address_bar.y_min + 11.0, accent);
    
    // Some icons are relative to the right edge of the window
    push_icon_globe(&mut raw_data, width, height, width - 130.0, 18.0, subtle);
    push_icon_shield(
        &mut raw_data,
        width,
        height,
        width - 102.0,
        18.0,
        (0.330, 0.690, 0.520, 1.0),
    );

    push_window_button(
        &mut raw_data,
        width,
        height,
        layout.minimize_btn.x_min,
        UIButton::Minimize,
    );
    push_window_button(
        &mut raw_data,
        width,
        height,
        layout.maximize_btn.x_min,
        UIButton::Maximize,
    );
    push_window_button(&mut raw_data, width, height, layout.close_btn.x_min, UIButton::Close);
    raw_data
}

fn push_nav_button(
    raw: &mut Vec<f32>,
    width: f32,
    height: f32,
    x: f32,
    y: f32,
    color: (f32, f32, f32, f32),
) {
    push_pill_px(
        raw,
        width,
        height,
        x,
        y,
        30.0,
        22.0,
        (0.125, 0.132, 0.180, 0.82),
        7.0,
    );
    push_quad_px(raw, width, height, x, y, 30.0, 1.0, (1.0, 1.0, 1.0, 0.08));
    push_quad_px(
        raw,
        width,
        height,
        x + 5.0,
        y + 20.0,
        20.0,
        1.0,
        (0.0, 0.0, 0.0, 0.18),
    );
    let _ = color;
}

fn push_window_button(raw: &mut Vec<f32>, width: f32, height: f32, x: f32, button: UIButton) {
    let (bg, fg) = match button {
        UIButton::Close => ((0.930, 0.220, 0.250, 1.0), (1.0, 1.0, 1.0, 1.0)),
        UIButton::Maximize => ((0.155, 0.164, 0.225, 1.0), (0.760, 0.800, 0.900, 1.0)),
        UIButton::Minimize => ((0.130, 0.138, 0.190, 1.0), (0.760, 0.800, 0.900, 1.0)),
        _ => ((0.130, 0.138, 0.190, 1.0), (0.760, 0.800, 0.900, 1.0)),
    };
    push_quad_px(raw, width, height, x, 0.0, 46.0, 36.0, bg);
    match button {
        UIButton::Close => {
            push_line_px(raw, width, height, x + 17.0, 12.0, x + 29.0, 24.0, 2.0, fg);
            push_line_px(raw, width, height, x + 29.0, 12.0, x + 17.0, 24.0, 2.0, fg);
        }
        UIButton::Maximize => {
            push_line_px(raw, width, height, x + 17.0, 12.0, x + 29.0, 12.0, 1.6, fg);
            push_line_px(raw, width, height, x + 29.0, 12.0, x + 29.0, 24.0, 1.6, fg);
            push_line_px(raw, width, height, x + 29.0, 24.0, x + 17.0, 24.0, 1.6, fg);
            push_line_px(raw, width, height, x + 17.0, 24.0, x + 17.0, 12.0, 1.6, fg);
        }
        UIButton::Minimize => {
            push_line_px(raw, width, height, x + 16.0, 21.0, x + 30.0, 21.0, 2.0, fg);
        }
        _ => {}
    }
}

fn push_icon_back(
    raw: &mut Vec<f32>,
    width: f32,
    height: f32,
    cx: f32,
    cy: f32,
    color: (f32, f32, f32, f32),
) {
    push_line_px(raw, width, height, cx + 5.0, cy, cx - 5.0, cy, 2.0, color);
    push_line_px(raw, width, height, cx - 5.0, cy, cx, cy - 5.0, 2.0, color);
    push_line_px(raw, width, height, cx - 5.0, cy, cx, cy + 5.0, 2.0, color);
}

fn push_icon_forward(
    raw: &mut Vec<f32>,
    width: f32,
    height: f32,
    cx: f32,
    cy: f32,
    color: (f32, f32, f32, f32),
) {
    push_line_px(raw, width, height, cx - 5.0, cy, cx + 5.0, cy, 2.0, color);
    push_line_px(raw, width, height, cx + 5.0, cy, cx, cy - 5.0, 2.0, color);
    push_line_px(raw, width, height, cx + 5.0, cy, cx, cy + 5.0, 2.0, color);
}

fn push_icon_reload(
    raw: &mut Vec<f32>,
    width: f32,
    height: f32,
    cx: f32,
    cy: f32,
    color: (f32, f32, f32, f32),
) {
    let mut last: Option<(f32, f32)> = None;
    for i in 0..18 {
        let t = 0.45 + i as f32 * 0.255;
        let p = (cx + t.cos() * 6.0, cy + t.sin() * 6.0);
        if let Some(prev) = last {
            push_line_px(raw, width, height, prev.0, prev.1, p.0, p.1, 1.6, color);
        }
        last = Some(p);
    }
    push_line_px(
        raw,
        width,
        height,
        cx + 5.6,
        cy - 4.0,
        cx + 8.0,
        cy - 8.0,
        1.6,
        color,
    );
    push_line_px(
        raw,
        width,
        height,
        cx + 5.6,
        cy - 4.0,
        cx + 2.0,
        cy - 5.8,
        1.6,
        color,
    );
}

fn push_icon_home(
    raw: &mut Vec<f32>,
    width: f32,
    height: f32,
    cx: f32,
    cy: f32,
    color: (f32, f32, f32, f32),
) {
    push_line_px(raw, width, height, cx - 7.0, cy, cx, cy - 7.0, 1.8, color);
    push_line_px(raw, width, height, cx, cy - 7.0, cx + 7.0, cy, 1.8, color);
    push_line_px(
        raw,
        width,
        height,
        cx - 5.0,
        cy,
        cx - 5.0,
        cy + 7.0,
        1.8,
        color,
    );
    push_line_px(
        raw,
        width,
        height,
        cx + 5.0,
        cy,
        cx + 5.0,
        cy + 7.0,
        1.8,
        color,
    );
    push_line_px(
        raw,
        width,
        height,
        cx - 5.0,
        cy + 7.0,
        cx + 5.0,
        cy + 7.0,
        1.8,
        color,
    );
}

fn push_icon_lock(
    raw: &mut Vec<f32>,
    width: f32,
    height: f32,
    cx: f32,
    cy: f32,
    color: (f32, f32, f32, f32),
) {
    push_quad_px(raw, width, height, cx - 4.5, cy - 1.0, 9.0, 7.0, color);
    push_line_px(
        raw,
        width,
        height,
        cx - 3.2,
        cy - 1.0,
        cx - 3.2,
        cy - 5.0,
        1.6,
        color,
    );
    push_line_px(
        raw,
        width,
        height,
        cx + 3.2,
        cy - 1.0,
        cx + 3.2,
        cy - 5.0,
        1.6,
        color,
    );
    push_line_px(
        raw,
        width,
        height,
        cx - 3.2,
        cy - 5.0,
        cx + 3.2,
        cy - 5.0,
        1.6,
        color,
    );
}

fn push_icon_globe(
    raw: &mut Vec<f32>,
    width: f32,
    height: f32,
    cx: f32,
    cy: f32,
    color: (f32, f32, f32, f32),
) {
    push_circle_outline(raw, width, height, cx, cy, 8.0, 1.4, color);
    push_line_px(raw, width, height, cx - 7.0, cy, cx + 7.0, cy, 1.2, color);
    push_line_px(raw, width, height, cx, cy - 8.0, cx, cy + 8.0, 1.2, color);
    push_line_px(
        raw,
        width,
        height,
        cx - 4.5,
        cy - 6.0,
        cx + 4.5,
        cy - 6.0,
        1.1,
        color,
    );
    push_line_px(
        raw,
        width,
        height,
        cx - 4.5,
        cy + 6.0,
        cx + 4.5,
        cy + 6.0,
        1.1,
        color,
    );
}

fn push_icon_shield(
    raw: &mut Vec<f32>,
    width: f32,
    height: f32,
    cx: f32,
    cy: f32,
    color: (f32, f32, f32, f32),
) {
    let points = [
        (cx, cy - 9.0),
        (cx + 7.0, cy - 5.0),
        (cx + 5.0, cy + 5.0),
        (cx, cy + 9.0),
        (cx - 5.0, cy + 5.0),
        (cx - 7.0, cy - 5.0),
        (cx, cy - 9.0),
    ];
    for pair in points.windows(2) {
        push_line_px(
            raw, width, height, pair[0].0, pair[0].1, pair[1].0, pair[1].1, 1.5, color,
        );
    }
}

fn push_circle_outline(
    raw: &mut Vec<f32>,
    width: f32,
    height: f32,
    cx: f32,
    cy: f32,
    radius: f32,
    thickness: f32,
    color: (f32, f32, f32, f32),
) {
    let segments = 22;
    let mut last = (cx + radius, cy);
    for i in 1..=segments {
        let t = i as f32 / segments as f32 * std::f32::consts::TAU;
        let next = (cx + t.cos() * radius, cy + t.sin() * radius);
        push_line_px(
            raw, width, height, last.0, last.1, next.0, next.1, thickness, color,
        );
        last = next;
    }
}

fn push_pill_px(
    raw: &mut Vec<f32>,
    width: f32,
    height: f32,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    color: (f32, f32, f32, f32),
    radius: f32,
) {
    push_quad_px(
        raw,
        width,
        height,
        x + radius,
        y,
        w - radius * 2.0,
        h,
        color,
    );
    push_quad_px(
        raw,
        width,
        height,
        x,
        y + radius,
        w,
        h - radius * 2.0,
        color,
    );
    push_circle_fill(raw, width, height, x + radius, y + radius, radius, color);
    push_circle_fill(
        raw,
        width,
        height,
        x + w - radius,
        y + radius,
        radius,
        color,
    );
    push_circle_fill(
        raw,
        width,
        height,
        x + radius,
        y + h - radius,
        radius,
        color,
    );
    push_circle_fill(
        raw,
        width,
        height,
        x + w - radius,
        y + h - radius,
        radius,
        color,
    );
}

fn push_circle_fill(
    raw: &mut Vec<f32>,
    width: f32,
    height: f32,
    cx: f32,
    cy: f32,
    radius: f32,
    color: (f32, f32, f32, f32),
) {
    let segments = 14;
    for i in 0..segments {
        let a = i as f32 / segments as f32 * std::f32::consts::TAU;
        let b = (i + 1) as f32 / segments as f32 * std::f32::consts::TAU;
        push_triangle_px(
            raw,
            width,
            height,
            (cx, cy),
            (cx + a.cos() * radius, cy + a.sin() * radius),
            (cx + b.cos() * radius, cy + b.sin() * radius),
            color,
        );
    }
}

fn push_quad_px(
    raw: &mut Vec<f32>,
    width: f32,
    height: f32,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    color: (f32, f32, f32, f32),
) {
    let p0 = (x, y);
    let p1 = (x + w, y);
    let p2 = (x, y + h);
    let p3 = (x + w, y + h);
    push_triangle_px(raw, width, height, p0, p1, p2, color);
    push_triangle_px(raw, width, height, p1, p3, p2, color);
}

fn push_line_px(
    raw: &mut Vec<f32>,
    width: f32,
    height: f32,
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    thickness: f32,
    color: (f32, f32, f32, f32),
) {
    let dx = x1 - x0;
    let dy = y1 - y0;
    let len = (dx * dx + dy * dy).sqrt().max(0.001);
    let nx = -dy / len * thickness * 0.5;
    let ny = dx / len * thickness * 0.5;
    let p0 = (x0 + nx, y0 + ny);
    let p1 = (x1 + nx, y1 + ny);
    let p2 = (x0 - nx, y0 - ny);
    let p3 = (x1 - nx, y1 - ny);
    push_triangle_px(raw, width, height, p0, p1, p2, color);
    push_triangle_px(raw, width, height, p1, p3, p2, color);
}

fn push_triangle_px(
    raw: &mut Vec<f32>,
    width: f32,
    height: f32,
    p0: (f32, f32),
    p1: (f32, f32),
    p2: (f32, f32),
    color: (f32, f32, f32, f32),
) {
    for p in [p0, p1, p2] {
        let x = -1.0 + (p.0 / width) * 2.0;
        let y = -1.0 + (p.1 / height) * 2.0;
        let v = UIVertex::solid(x, y, color.0, color.1, color.2, color.3);
        raw.push(v.x);
        raw.push(v.y);
        raw.push(v.r);
        raw.push(v.g);
        raw.push(v.b);
        raw.push(v.a);
        raw.push(v.u);
        raw.push(v.v);
    }
}
