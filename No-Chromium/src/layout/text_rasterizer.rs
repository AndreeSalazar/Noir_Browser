use fontdue::{Font, FontSettings};
use std::fs;

#[derive(Clone, Debug)]
pub struct TextRequest {
    pub text: String,
    pub px_size: f32,
    pub pos_x: f32,
    pub pos_y: f32,
    pub color: [f32; 4], // r, g, b, a
}

#[derive(Clone, Debug)]
pub struct TextQuad {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub u0: f32,
    pub v0: f32,
    pub u1: f32,
    pub v1: f32,
    pub color: [f32; 4],
}

pub struct RasterizedAtlas {
    pub width: u32,
    pub height: u32,
    pub rgba_data: Vec<u8>,
    pub quads: Vec<TextQuad>,
}

impl RasterizedAtlas {
    pub fn new(requests: &[TextRequest]) -> Self {
        println!("[*] Rasterizando Atlas de Texto en CPU con {} peticiones", requests.len());
        
        let font_bytes = fs::read("C:\\Windows\\Fonts\\arial.ttf")
            .expect("Fallo al leer la fuente Arial de Windows");
            
        let font = Font::from_bytes(font_bytes, FontSettings::default())
            .expect("Fallo al parsear la fuente Arial");

        struct PreRaster {
            width: u32,
            height: u32,
            buffer: Vec<u8>, // alpha
            screen_x: f32,
            screen_y: f32,
            color: [f32; 4],
        }

        let mut pre_rasters = Vec::new();
        let mut max_atlas_w: u32 = 0;
        let mut total_atlas_h: u32 = 2; // 2px padding top

        for req in requests {
            let mut line_glyphs = Vec::new();
            let mut w: u32 = 0;
            let mut h: u32 = 0;
            for c in req.text.chars() {
                if c == ' ' {
                    w += (req.px_size * 0.25) as u32;
                    line_glyphs.push(None);
                    continue;
                }
                let (metrics, bitmap) = font.rasterize(c, req.px_size);
                line_glyphs.push(Some((metrics, bitmap)));
                w += metrics.width as u32 + 1; // 1px intra-letter padding
                if metrics.height as u32 > h {
                    h = metrics.height as u32;
                }
            }

            if w == 0 || h == 0 {
                continue;
            }

            let radius = 4;
            let padded_w = w + radius * 2;
            let padded_h = h + radius * 2;

            let mut line_buffer = vec![0u8; (padded_w * padded_h) as usize];
            let mut cursor_x = radius;
            for glyph_opt in line_glyphs {
                match glyph_opt {
                    Some((metrics, bitmap)) => {
                        for y in 0..metrics.height {
                            for x in 0..metrics.width {
                                let global_x = cursor_x + x as u32;
                                let global_y = radius + y as u32;
                                let idx = (global_y * padded_w + global_x) as usize;
                                let alpha = bitmap[y * metrics.width + x];
                                line_buffer[idx] = std::cmp::max(line_buffer[idx], alpha);
                            }
                        }
                        cursor_x += metrics.width as u32 + 1;
                    }
                    None => {
                        cursor_x += (req.px_size * 0.25) as u32;
                    }
                }
            }

            if padded_w > max_atlas_w {
                max_atlas_w = padded_w;
            }

            pre_rasters.push(PreRaster {
                width: padded_w,
                height: padded_h,
                buffer: line_buffer,
                screen_x: req.pos_x - radius as f32,
                screen_y: req.pos_y - radius as f32,
                color: req.color,
            });

            total_atlas_h += padded_h + 2; // minimum 2px padding between lines
        }

        if max_atlas_w == 0 {
            max_atlas_w = 4;
            total_atlas_h = 4;
        }

        // Add 2px right padding to width
        max_atlas_w += 2;

        let mut rgba_data = vec![0u8; (max_atlas_w * total_atlas_h * 4) as usize];
        let mut quads = Vec::new();

        let mut current_y: u32 = 2; // initial top padding
        for mut pr in pre_rasters {
            let px: u32 = 2; // 2px left padding
            let py: u32 = current_y;

            let radius = 4;
            let mut sdf_buffer = vec![0u8; (pr.width * pr.height) as usize];
            for y in 0..pr.height {
                for x in 0..pr.width {
                    let idx = (y * pr.width + x) as usize;
                    let val = pr.buffer[idx];
                    let is_inside = val > 127;
                    
                    let mut min_dist = (radius * radius) as f32;
                    let y_min = y.saturating_sub(radius as u32);
                    let y_max = (y + radius as u32).min(pr.height - 1);
                    let x_min = x.saturating_sub(radius as u32);
                    let x_max = (x + radius as u32).min(pr.width - 1);
                    
                    for sy in y_min..=y_max {
                        for sx in x_min..=x_max {
                            let s_idx = (sy * pr.width + sx) as usize;
                            let s_val = pr.buffer[s_idx];
                            let s_inside = s_val > 127;
                            if is_inside != s_inside {
                                let dx = sx as f32 - x as f32;
                                let dy = sy as f32 - y as f32;
                                let dist_sq = dx*dx + dy*dy;
                                if dist_sq < min_dist {
                                    min_dist = dist_sq;
                                }
                            }
                        }
                    }
                    
                    let dist = min_dist.sqrt();
                    let mut normalized = if is_inside {
                        0.5 + (dist / (radius as f32)) * 0.5
                    } else {
                        0.5 - (dist / (radius as f32)) * 0.5
                    };
                    normalized = normalized.clamp(0.0, 1.0);
                    sdf_buffer[idx] = (normalized * 255.0) as u8;
                }
            }
            pr.buffer = sdf_buffer;

            for y in 0..pr.height {
                for x in 0..pr.width {
                    let a = pr.buffer[(y * pr.width + x) as usize];
                    if a > 0 {
                        let idx = ((py + y) * max_atlas_w + (px + x)) as usize * 4;
                        rgba_data[idx] = 255;
                        rgba_data[idx + 1] = 255;
                        rgba_data[idx + 2] = 255;
                        rgba_data[idx + 3] = a;
                    }
                }
            }

            quads.push(TextQuad {
                x: pr.screen_x,
                y: pr.screen_y,
                w: pr.width as f32,
                h: pr.height as f32,
                u0: px as f32 / max_atlas_w as f32,
                v0: py as f32 / total_atlas_h as f32,
                u1: (px + pr.width) as f32 / max_atlas_w as f32,
                v1: (py + pr.height) as f32 / total_atlas_h as f32,
                color: pr.color,
            });

            current_y += pr.height + 2; // advance Y with padding
        }

        println!("[+] Texture Atlas generado. Dimensión: {}x{} con {} textos.", max_atlas_w, total_atlas_h, quads.len());

        Self {
            width: max_atlas_w,
            height: total_atlas_h,
            rgba_data,
            quads,
        }
    }
}
