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
    TabSelect(usize),
    TabClose(usize),
    NewTab,
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
    pub tabs: Vec<UIBoundingBox>,
    pub new_tab_btn: Option<UIBoundingBox>,
}

impl UILayout {
    pub fn new(width: f32, _height: f32, tabs_count: usize, scale_factor: f32) -> Self {
        let url_left = 168.0 * scale_factor;
        let url_right = (width - 150.0 * scale_factor).max(url_left + 120.0 * scale_factor);

        let mut tabs = Vec::new();
        let mut new_tab_btn = None;

        if tabs_count > 0 {
            let tab_w = ((width - 290.0 * scale_factor) / tabs_count as f32)
                .min(160.0 * scale_factor)
                .max(40.0 * scale_factor);
            for i in 0..tabs_count {
                let x_min = 12.0 * scale_factor + i as f32 * tab_w;
                let x_max = x_min + tab_w;
                
                // Add tab select hitbox
                tabs.push(UIBoundingBox {
                    button: UIButton::TabSelect(i),
                    x_min,
                    x_max: if tab_w > 60.0 * scale_factor { x_max - 24.0 * scale_factor } else { x_max },
                    y_min: 0.0,
                    y_max: 36.0 * scale_factor,
                });

                // Add close button hitbox if the tab is wide enough
                if tab_w > 60.0 * scale_factor {
                    tabs.push(UIBoundingBox {
                        button: UIButton::TabClose(i),
                        x_min: x_max - 24.0 * scale_factor,
                        x_max: x_max - 8.0 * scale_factor,
                        y_min: 10.0 * scale_factor,
                        y_max: 26.0 * scale_factor,
                    });
                }
            }

            let new_tab_x = 12.0 * scale_factor + tabs_count as f32 * tab_w + 6.0 * scale_factor;
            if new_tab_x + 28.0 * scale_factor < width - 150.0 * scale_factor {
                new_tab_btn = Some(UIBoundingBox {
                    button: UIButton::NewTab,
                    x_min: new_tab_x,
                    x_max: new_tab_x + 28.0 * scale_factor,
                    y_min: 6.0 * scale_factor,
                    y_max: 30.0 * scale_factor,
                });
            }
        }

        Self {
            back_btn: UIBoundingBox {
                button: UIButton::Back,
                x_min: 12.0 * scale_factor,
                x_max: 42.0 * scale_factor,
                y_min: 43.0 * scale_factor,
                y_max: 65.0 * scale_factor,
            },
            forward_btn: UIBoundingBox {
                button: UIButton::Forward,
                x_min: 48.0 * scale_factor,
                x_max: 78.0 * scale_factor,
                y_min: 43.0 * scale_factor,
                y_max: 65.0 * scale_factor,
            },
            reload_btn: UIBoundingBox {
                button: UIButton::Reload,
                x_min: 84.0 * scale_factor,
                x_max: 114.0 * scale_factor,
                y_min: 43.0 * scale_factor,
                y_max: 65.0 * scale_factor,
            },
            home_btn: UIBoundingBox {
                button: UIButton::Home,
                x_min: 120.0 * scale_factor,
                x_max: 150.0 * scale_factor,
                y_min: 43.0 * scale_factor,
                y_max: 65.0 * scale_factor,
            },
            address_bar: UIBoundingBox {
                button: UIButton::AddressBar,
                x_min: url_left,
                x_max: url_right,
                y_min: 43.0 * scale_factor,
                y_max: 65.0 * scale_factor,
            },
            minimize_btn: UIBoundingBox {
                button: UIButton::Minimize,
                x_min: width - 138.0 * scale_factor,
                x_max: width - 92.0 * scale_factor,
                y_min: 0.0,
                y_max: 36.0 * scale_factor,
            },
            maximize_btn: UIBoundingBox {
                button: UIButton::Maximize,
                x_min: width - 92.0 * scale_factor,
                x_max: width - 46.0 * scale_factor,
                y_min: 0.0,
                y_max: 36.0 * scale_factor,
            },
            close_btn: UIBoundingBox {
                button: UIButton::Close,
                x_min: width - 46.0 * scale_factor,
                x_max: width,
                y_min: 0.0,
                y_max: 36.0 * scale_factor,
            },
            tabs,
            new_tab_btn,
        }
    }

    pub fn get_hitboxes(&self) -> Vec<UIBoundingBox> {
        let mut hitboxes = vec![
            self.back_btn,
            self.forward_btn,
            self.reload_btn,
            self.home_btn,
            self.address_bar,
            self.minimize_btn,
            self.maximize_btn,
            self.close_btn,
        ];
        hitboxes.extend(self.tabs.iter().copied());
        if let Some(nt) = self.new_tab_btn {
            hitboxes.push(nt);
        }
        hitboxes
    }
}

pub fn get_ui_hitboxes(width: f32, height: f32, tabs_count: usize, scale_factor: f32) -> Vec<UIBoundingBox> {
    UILayout::new(width, height, tabs_count, scale_factor).get_hitboxes()
}

pub fn generate_chrome_vertices(
    width: f32,
    height: f32,
    tabs_count: usize,
    active_tab_index: usize,
    scale_factor: f32,
) -> Vec<f32> {
    let mut raw_data = Vec::new();
    let layout = UILayout::new(width, height, tabs_count, scale_factor);

    let chrome = (0.070, 0.074, 0.105, 1.0);
    let toolbar = (0.095, 0.098, 0.142, 1.0);
    let pill = (0.035, 0.038, 0.058, 1.0);
    let pill_edge = (0.180, 0.190, 0.260, 1.0);
    let subtle = (0.565, 0.600, 0.710, 1.0);
    let bright = (0.900, 0.925, 1.000, 1.0);
    let accent = (0.260, 0.520, 0.980, 1.0);

    push_quad_px(&mut raw_data, width, height, 0.0, 0.0, width, 36.0 * scale_factor, chrome);

    // Render Tabs
    if tabs_count > 0 {
        let tab_w = ((width - 290.0 * scale_factor) / tabs_count as f32)
            .min(160.0 * scale_factor)
            .max(40.0 * scale_factor);
        for i in 0..tabs_count {
            let x_min = 12.0 * scale_factor + i as f32 * tab_w;
            let is_active = i == active_tab_index;

            let tab_color = if is_active {
                toolbar
            } else {
                (0.082, 0.086, 0.122, 0.60)
            };

            push_pill_px(
                &mut raw_data,
                width,
                height,
                x_min,
                2.0 * scale_factor,
                tab_w - 4.0 * scale_factor,
                34.0 * scale_factor,
                tab_color,
                6.0 * scale_factor,
            );

            if is_active {
                // Neon glowing accent bar under active tab
                push_quad_px(
                    &mut raw_data,
                    width,
                    height,
                    x_min + 6.0 * scale_factor,
                    34.0 * scale_factor,
                    tab_w - 16.0 * scale_factor,
                    2.0 * scale_factor,
                    accent,
                );
                // Top highlight line
                push_quad_px(
                    &mut raw_data,
                    width,
                    height,
                    x_min + 6.0 * scale_factor,
                    2.0 * scale_factor,
                    tab_w - 16.0 * scale_factor,
                    1.0 * scale_factor,
                    (1.0, 1.0, 1.0, 0.12),
                );
            }

            // Draw "x" close button
            if tab_w > 60.0 * scale_factor {
                let close_x = x_min + tab_w - 20.0 * scale_factor;
                let close_y = 18.0 * scale_factor;
                let cross_color = if is_active { (0.75, 0.78, 0.88, 0.8) } else { (0.45, 0.48, 0.58, 0.6) };
                push_line_px(
                    &mut raw_data,
                    width,
                    height,
                    close_x - 4.0 * scale_factor,
                    close_y - 4.0 * scale_factor,
                    close_x + 4.0 * scale_factor,
                    close_y + 4.0 * scale_factor,
                    1.4 * scale_factor,
                    cross_color,
                );
                push_line_px(
                    &mut raw_data,
                    width,
                    height,
                    close_x + 4.0 * scale_factor,
                    close_y - 4.0 * scale_factor,
                    close_x - 4.0 * scale_factor,
                    close_y + 4.0 * scale_factor,
                    1.4 * scale_factor,
                    cross_color,
                );
            }
        }

        // Draw "+" (NewTab) button
        if let Some(nt) = layout.new_tab_btn {
            let btn_color = (0.125, 0.132, 0.180, 0.82);
            push_circle_fill(
                &mut raw_data,
                width,
                height,
                nt.x_min + 14.0 * scale_factor,
                nt.y_min + 12.0 * scale_factor,
                11.0 * scale_factor,
                btn_color,
            );
            push_circle_outline(
                &mut raw_data,
                width,
                height,
                nt.x_min + 14.0 * scale_factor,
                nt.y_min + 12.0 * scale_factor,
                11.0 * scale_factor,
                1.0 * scale_factor,
                (0.20, 0.22, 0.30, 0.50),
            );
            let plus_x = nt.x_min + 14.0 * scale_factor;
            let plus_y = nt.y_min + 12.0 * scale_factor;
            push_line_px(&mut raw_data, width, height, plus_x - 4.0 * scale_factor, plus_y, plus_x + 4.0 * scale_factor, plus_y, 1.6 * scale_factor, subtle);
            push_line_px(&mut raw_data, width, height, plus_x, plus_y - 4.0 * scale_factor, plus_x, plus_y + 4.0 * scale_factor, 1.6 * scale_factor, subtle);
        }
    }

    push_quad_px(
        &mut raw_data,
        width,
        height,
        0.0,
        36.0 * scale_factor,
        width,
        36.0 * scale_factor,
        toolbar,
    );
    push_quad_px(
        &mut raw_data,
        width,
        height,
        0.0,
        71.0 * scale_factor,
        width,
        1.0 * scale_factor,
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
        8.0 * scale_factor,
    );
    push_pill_outline(
        &mut raw_data,
        width,
        height,
        layout.address_bar.x_min,
        layout.address_bar.y_min,
        url_w,
        url_h,
        1.5 * scale_factor,
        (0.120, 0.650, 0.820, 0.85),
        8.0 * scale_factor,
    );
    push_icon_star(
        &mut raw_data,
        width,
        height,
        layout.address_bar.x_max - 20.0 * scale_factor,
        layout.address_bar.y_min + 11.0 * scale_factor,
        (0.950, 0.750, 0.200, 1.0),
        scale_factor,
    );

    push_nav_button(&mut raw_data, width, height, layout.back_btn.x_min, layout.back_btn.y_min, subtle, scale_factor);
    push_icon_back(&mut raw_data, width, height, layout.back_btn.x_min + 15.0 * scale_factor, layout.back_btn.y_min + 11.0 * scale_factor, bright, scale_factor);

    push_nav_button(
        &mut raw_data,
        width,
        height,
        layout.forward_btn.x_min,
        layout.forward_btn.y_min,
        (0.420, 0.450, 0.540, 1.0),
        scale_factor,
    );
    push_icon_forward(
        &mut raw_data,
        width,
        height,
        layout.forward_btn.x_min + 15.0 * scale_factor,
        layout.forward_btn.y_min + 11.0 * scale_factor,
        (0.420, 0.450, 0.540, 1.0),
        scale_factor,
    );

    push_nav_button(&mut raw_data, width, height, layout.reload_btn.x_min, layout.reload_btn.y_min, subtle, scale_factor);
    push_icon_reload(&mut raw_data, width, height, layout.reload_btn.x_min + 15.0 * scale_factor, layout.reload_btn.y_min + 11.0 * scale_factor, bright, scale_factor);

    push_nav_button(&mut raw_data, width, height, layout.home_btn.x_min, layout.home_btn.y_min, subtle, scale_factor);
    push_icon_home(&mut raw_data, width, height, layout.home_btn.x_min + 15.0 * scale_factor, layout.home_btn.y_min + 11.0 * scale_factor, bright, scale_factor);

    push_icon_lock(&mut raw_data, width, height, layout.address_bar.x_min + 16.0 * scale_factor, layout.address_bar.y_min + 11.0 * scale_factor, accent, scale_factor);
    
    push_icon_globe(&mut raw_data, width, height, width - 120.0 * scale_factor, 54.0 * scale_factor, subtle, scale_factor);
    push_icon_shield(
        &mut raw_data,
        width,
        height,
        width - 80.0 * scale_factor,
        54.0 * scale_factor,
        (0.330, 0.690, 0.520, 1.0),
        scale_factor,
    );

    push_window_button(
        &mut raw_data,
        width,
        height,
        layout.minimize_btn.x_min,
        UIButton::Minimize,
        scale_factor,
    );
    push_window_button(
        &mut raw_data,
        width,
        height,
        layout.maximize_btn.x_min,
        UIButton::Maximize,
        scale_factor,
    );
    push_window_button(&mut raw_data, width, height, layout.close_btn.x_min, UIButton::Close, scale_factor);
    raw_data
}

fn push_nav_button(
    raw: &mut Vec<f32>,
    width: f32,
    height: f32,
    x: f32,
    y: f32,
    color: (f32, f32, f32, f32),
    scale_factor: f32,
) {
    push_pill_px(
        raw,
        width,
        height,
        x,
        y,
        30.0 * scale_factor,
        22.0 * scale_factor,
        (0.125, 0.132, 0.180, 0.82),
        7.0 * scale_factor,
    );
    push_quad_px(raw, width, height, x, y, 30.0 * scale_factor, 1.0 * scale_factor, (1.0, 1.0, 1.0, 0.08));
    push_quad_px(
        raw,
        width,
        height,
        x + 5.0 * scale_factor,
        y + 20.0 * scale_factor,
        20.0 * scale_factor,
        1.0 * scale_factor,
        (0.0, 0.0, 0.0, 0.18),
    );
    let _ = color;
}

fn push_window_button(raw: &mut Vec<f32>, width: f32, height: f32, x: f32, button: UIButton, scale_factor: f32) {
    let (bg, fg) = match button {
        UIButton::Close => ((0.930, 0.220, 0.250, 1.0), (1.0, 1.0, 1.0, 1.0)),
        UIButton::Maximize => ((0.155, 0.164, 0.225, 1.0), (0.760, 0.800, 0.900, 1.0)),
        UIButton::Minimize => ((0.130, 0.138, 0.190, 1.0), (0.760, 0.800, 0.900, 1.0)),
        _ => ((0.130, 0.138, 0.190, 1.0), (0.760, 0.800, 0.900, 1.0)),
    };
    push_quad_px(raw, width, height, x, 0.0, 46.0 * scale_factor, 36.0 * scale_factor, bg);
    match button {
        UIButton::Close => {
            push_line_px(raw, width, height, x + 17.0 * scale_factor, 12.0 * scale_factor, x + 29.0 * scale_factor, 24.0 * scale_factor, 2.0 * scale_factor, fg);
            push_line_px(raw, width, height, x + 29.0 * scale_factor, 12.0 * scale_factor, x + 17.0 * scale_factor, 24.0 * scale_factor, 2.0 * scale_factor, fg);
        }
        UIButton::Maximize => {
            push_line_px(raw, width, height, x + 17.0 * scale_factor, 12.0 * scale_factor, x + 29.0 * scale_factor, 12.0 * scale_factor, 1.6 * scale_factor, fg);
            push_line_px(raw, width, height, x + 29.0 * scale_factor, 12.0 * scale_factor, x + 29.0 * scale_factor, 24.0 * scale_factor, 1.6 * scale_factor, fg);
            push_line_px(raw, width, height, x + 29.0 * scale_factor, 24.0 * scale_factor, x + 17.0 * scale_factor, 24.0 * scale_factor, 1.6 * scale_factor, fg);
            push_line_px(raw, width, height, x + 17.0 * scale_factor, 24.0 * scale_factor, x + 17.0 * scale_factor, 12.0 * scale_factor, 1.6 * scale_factor, fg);
        }
        UIButton::Minimize => {
            push_line_px(raw, width, height, x + 16.0 * scale_factor, 21.0 * scale_factor, x + 30.0 * scale_factor, 21.0 * scale_factor, 2.0 * scale_factor, fg);
        }
        _ => {}
    }
}

fn push_pill_outline(
    raw: &mut Vec<f32>,
    width: f32,
    height: f32,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    thickness: f32,
    color: (f32, f32, f32, f32),
    radius: f32,
) {
    push_line_px(raw, width, height, x + radius, y, x + w - radius, y, thickness, color);
    push_line_px(raw, width, height, x + radius, y + h, x + w - radius, y + h, thickness, color);
    let segments = 12;
    let r = radius;
    let left_cx = x + radius;
    let left_cy = y + h * 0.5;
    let mut last = (left_cx, left_cy + r);
    for i in 1..=segments {
        let t = std::f32::consts::FRAC_PI_2 + (i as f32 / segments as f32) * std::f32::consts::PI;
        let next = (left_cx + t.cos() * r, left_cy + t.sin() * r);
        push_line_px(raw, width, height, last.0, last.1, next.0, next.1, thickness, color);
        last = next;
    }
    let right_cx = x + w - radius;
    let right_cy = y + h * 0.5;
    let mut last = (right_cx, right_cy - r);
    for i in 1..=segments {
        let t = -std::f32::consts::FRAC_PI_2 + (i as f32 / segments as f32) * std::f32::consts::PI;
        let next = (right_cx + t.cos() * r, right_cy + t.sin() * r);
        push_line_px(raw, width, height, last.0, last.1, next.0, next.1, thickness, color);
        last = next;
    }
}

fn push_icon_star(
    raw: &mut Vec<f32>,
    width: f32,
    height: f32,
    cx: f32,
    cy: f32,
    color: (f32, f32, f32, f32),
    scale_factor: f32,
) {
    let s = 0.70 * scale_factor;
    let map_x = |x_svg: f32| cx + (x_svg - 12.0) * s;
    let map_y = |y_svg: f32| cy + (y_svg - 12.0) * s;
    let thickness = 1.6 * scale_factor;
    let points = [
        (12.0, 7.5),
        (13.1, 9.7),
        (15.5, 10.05),
        (13.75, 11.75),
        (14.17, 14.15),
        (12.0, 13.0),
        (9.83, 14.15),
        (10.25, 11.75),
        (8.5, 10.05),
        (10.9, 9.7),
        (12.0, 7.5),
    ];
    for pair in points.windows(2) {
        push_line_px(raw, width, height, map_x(pair[0].0), map_y(pair[0].1), map_x(pair[1].0), map_y(pair[1].1), thickness, color);
    }
}

fn push_icon_back(
    raw: &mut Vec<f32>,
    width: f32,
    height: f32,
    cx: f32,
    cy: f32,
    color: (f32, f32, f32, f32),
    scale_factor: f32,
) {
    let s = 0.75 * scale_factor;
    let map_x = |x_svg: f32| cx + (x_svg - 12.0) * s;
    let map_y = |y_svg: f32| cy + (y_svg - 12.0) * s;
    let thickness = 2.2 * scale_factor;
    push_line_px(raw, width, height, map_x(15.0), map_y(6.0), map_x(9.0), map_y(12.0), thickness, color);
    push_line_px(raw, width, height, map_x(9.0), map_y(12.0), map_x(15.0), map_y(18.0), thickness, color);
    push_line_px(raw, width, height, map_x(9.0), map_y(12.0), map_x(20.0), map_y(12.0), thickness, color);
}

fn push_icon_forward(
    raw: &mut Vec<f32>,
    width: f32,
    height: f32,
    cx: f32,
    cy: f32,
    color: (f32, f32, f32, f32),
    scale_factor: f32,
) {
    let s = 0.75 * scale_factor;
    let map_x = |x_svg: f32| cx + (x_svg - 12.0) * s;
    let map_y = |y_svg: f32| cy + (y_svg - 12.0) * s;
    let thickness = 2.2 * scale_factor;
    push_line_px(raw, width, height, map_x(9.0), map_y(6.0), map_x(15.0), map_y(12.0), thickness, color);
    push_line_px(raw, width, height, map_x(15.0), map_y(12.0), map_x(9.0), map_y(18.0), thickness, color);
    push_line_px(raw, width, height, map_x(4.0), map_y(12.0), map_x(15.0), map_y(12.0), thickness, color);
}

fn push_icon_reload(
    raw: &mut Vec<f32>,
    width: f32,
    height: f32,
    cx: f32,
    cy: f32,
    color: (f32, f32, f32, f32),
    scale_factor: f32,
) {
    let s = 0.75 * scale_factor;
    let map_x = |x_svg: f32| cx + (x_svg - 12.0) * s;
    let map_y = |y_svg: f32| cy + (y_svg - 12.0) * s;
    let thickness = 2.0 * scale_factor;
    let segments = 24;
    let center_x = map_x(12.0);
    let center_y = map_y(12.0);
    let r = 8.0 * s;
    let max_angle = 1.75 * std::f32::consts::TAU;
    let mut last = (center_x + r, center_y);
    for i in 1..=segments {
        let t = (i as f32 / segments as f32) * max_angle;
        let next = (center_x + t.cos() * r, center_y + t.sin() * r);
        push_line_px(raw, width, height, last.0, last.1, next.0, next.1, thickness, color);
        last = next;
    }
    push_line_px(raw, width, height, map_x(20.0), map_y(4.0), map_x(20.0), map_y(10.0), thickness, color);
    push_line_px(raw, width, height, map_x(20.0), map_y(10.0), map_x(14.0), map_y(10.0), thickness, color);
}

fn push_icon_home(
    raw: &mut Vec<f32>,
    width: f32,
    height: f32,
    cx: f32,
    cy: f32,
    color: (f32, f32, f32, f32),
    scale_factor: f32,
) {
    let s = 0.75 * scale_factor;
    let map_x = |x_svg: f32| cx + (x_svg - 12.0) * s;
    let map_y = |y_svg: f32| cy + (y_svg - 12.0) * s;
    let thickness = 2.0 * scale_factor;
    push_line_px(raw, width, height, map_x(4.0), map_y(11.0), map_x(12.0), map_y(4.0), thickness, color);
    push_line_px(raw, width, height, map_x(12.0), map_y(4.0), map_x(20.0), map_y(11.0), thickness, color);
    push_line_px(raw, width, height, map_x(6.5), map_y(10.5), map_x(6.5), map_y(20.0), thickness, color);
    push_line_px(raw, width, height, map_x(6.5), map_y(20.0), map_x(17.5), map_y(20.0), thickness, color);
    push_line_px(raw, width, height, map_x(17.5), map_y(20.0), map_x(17.5), map_y(10.5), thickness, color);
    push_line_px(raw, width, height, map_x(10.0), map_y(20.0), map_x(10.0), map_y(15.0), thickness, color);
    push_line_px(raw, width, height, map_x(10.0), map_y(15.0), map_x(14.0), map_y(15.0), thickness, color);
    push_line_px(raw, width, height, map_x(14.0), map_y(15.0), map_x(14.0), map_y(20.0), thickness, color);
}

fn push_icon_lock(
    raw: &mut Vec<f32>,
    width: f32,
    height: f32,
    cx: f32,
    cy: f32,
    color: (f32, f32, f32, f32),
    scale_factor: f32,
) {
    let s = 0.75 * scale_factor;
    let map_x = |x_svg: f32| cx + (x_svg - 12.0) * s;
    let map_y = |y_svg: f32| cy + (y_svg - 12.0) * s;
    let thickness = 1.8 * scale_factor;
    push_pill_outline(raw, width, height, map_x(5.0), map_y(10.0), 14.0 * s, 10.0 * s, thickness, color, 2.0 * s);
    push_line_px(raw, width, height, map_x(8.0), map_y(10.0), map_x(8.0), map_y(7.0), thickness, color);
    push_line_px(raw, width, height, map_x(16.0), map_y(7.0), map_x(16.0), map_y(10.0), thickness, color);
    let segments = 10;
    let arc_cx = map_x(12.0);
    let arc_cy = map_y(7.0);
    let r = 4.0 * s;
    let mut last = (arc_cx - r, arc_cy);
    for i in 1..=segments {
        let t = std::f32::consts::PI - (i as f32 / segments as f32) * std::f32::consts::PI;
        let next = (arc_cx + t.cos() * r, arc_cy + t.sin() * r);
        push_line_px(raw, width, height, last.0, last.1, next.0, next.1, thickness, color);
        last = next;
    }
    push_line_px(raw, width, height, map_x(12.0), map_y(14.0), map_x(12.0), map_y(16.0), thickness, color);
}

fn push_icon_globe(
    raw: &mut Vec<f32>,
    width: f32,
    height: f32,
    cx: f32,
    cy: f32,
    color: (f32, f32, f32, f32),
    scale_factor: f32,
) {
    push_circle_outline(raw, width, height, cx, cy, 8.0 * scale_factor, 2.5 * scale_factor, color);
    push_line_px(raw, width, height, cx - 7.0 * scale_factor, cy, cx + 7.0 * scale_factor, cy, 2.0 * scale_factor, color);
    push_line_px(raw, width, height, cx, cy - 8.0 * scale_factor, cx, cy + 8.0 * scale_factor, 2.0 * scale_factor, color);
    push_line_px(
        raw,
        width,
        height,
        cx - 4.5 * scale_factor,
        cy - 6.0 * scale_factor,
        cx + 4.5 * scale_factor,
        cy - 6.0 * scale_factor,
        1.8 * scale_factor,
        color,
    );
    push_line_px(
        raw,
        width,
        height,
        cx - 4.5 * scale_factor,
        cy + 6.0 * scale_factor,
        cx + 4.5 * scale_factor,
        cy + 6.0 * scale_factor,
        1.8 * scale_factor,
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
    scale_factor: f32,
) {
    let points = [
        (cx, cy - 9.0 * scale_factor),
        (cx + 7.0 * scale_factor, cy - 5.0 * scale_factor),
        (cx + 5.0 * scale_factor, cy + 5.0 * scale_factor),
        (cx, cy + 9.0 * scale_factor),
        (cx - 5.0 * scale_factor, cy + 5.0 * scale_factor),
        (cx - 7.0 * scale_factor, cy - 5.0 * scale_factor),
        (cx, cy - 9.0 * scale_factor),
    ];
    for pair in points.windows(2) {
        push_line_px(
            raw, width, height, pair[0].0, pair[0].1, pair[1].0, pair[1].1, 2.5 * scale_factor, color,
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
    _width: f32,
    _height: f32,
    p0: (f32, f32),
    p1: (f32, f32),
    p2: (f32, f32),
    color: (f32, f32, f32, f32),
) {
    for p in [p0, p1, p2] {
        let x = p.0;
        let y = p.1;
        let v = UIVertex::solid(x, y, color.0, color.1, color.2, color.3);
        raw.push(v.x);
        raw.push(v.y);
        raw.push(v.r);
        raw.push(v.g);
        raw.push(v.b);
        raw.push(v.a);
        raw.push(v.u);
        raw.push(v.v);
        raw.push(v.box_w);
        raw.push(v.box_h);
        raw.push(v.radius);
        raw.push(v.is_text);
    }
}
