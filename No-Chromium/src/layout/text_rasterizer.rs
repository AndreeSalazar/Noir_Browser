use fontdue::{Font, FontSettings};
use std::fs;

const DEFAULT_OVERSAMPLE: f32 = 3.0;
const DEFAULT_ATLAS_PADDING: u32 = 2;

#[derive(Clone, Copy, Debug)]
pub struct TextRasterizationOptions {
    pub oversample: f32,
    pub atlas_padding: u32,
    pub gamma: f32,
}

impl TextRasterizationOptions {
    pub fn sharp_lcd() -> Self {
        Self {
            oversample: DEFAULT_OVERSAMPLE,
            atlas_padding: DEFAULT_ATLAS_PADDING,
            gamma: 1.15,
        }
    }
}

#[derive(Clone, Debug)]
pub struct TextRequest {
    pub text: String,
    pub px_size: f32,
    pub is_bold: bool,
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
    #[allow(dead_code)]
    pub fn new(requests: &[TextRequest]) -> Self {
        Self::with_options(requests, TextRasterizationOptions::sharp_lcd())
    }

    pub fn with_options(requests: &[TextRequest], options: TextRasterizationOptions) -> Self {
        println!("[*] Rasterizando Atlas de Texto en CPU con {} peticiones", requests.len());
        
        let font_bytes_reg = read_font(&[
            "C:\\Windows\\Fonts\\segoeui.ttf",
            "C:\\Windows\\Fonts\\arial.ttf",
        ])
        .expect("Fallo al leer una fuente regular de Windows");
        let font_reg = Font::from_bytes(font_bytes_reg, FontSettings::default())
            .expect("Fallo al parsear la fuente regular");
            
        let font_bytes_bold = read_font(&[
            "C:\\Windows\\Fonts\\segoeuib.ttf",
            "C:\\Windows\\Fonts\\arialbd.ttf",
        ])
        .expect("Fallo al leer una fuente bold de Windows");
        let font_bold = Font::from_bytes(font_bytes_bold, FontSettings::default())
            .expect("Fallo al parsear la fuente bold");

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
            let mut pen_x = 0.0_f32;
            let mut min_y = 0_i32;
            let mut max_y = 0_i32;
            let scale = options.oversample.max(1.0).round() as usize;
            let scale_f = scale as f32;
            let active_font = if req.is_bold { &font_bold } else { &font_reg };
            
            for c in req.text.chars() {
                if c == ' ' {
                    pen_x += req.px_size * 0.28;
                    line_glyphs.push(None);
                    continue;
                }
                let (metrics_3x, bitmap_3x) = active_font.rasterize(c, req.px_size * scale_f);
                
                let dw = (metrics_3x.width as f32 / scale_f).ceil() as usize;
                let dh = (metrics_3x.height as f32 / scale_f).ceil() as usize;
                let mut bitmap_1x = vec![0u8; dw * dh];
                
                for dy in 0..dh {
                    for dx in 0..dw {
                        let mut sum = 0u32;
                        let mut count = 0u32;
                        for sy in 0..scale {
                            for sx in 0..scale {
                                let src_x = dx * scale + sx;
                                let src_y = dy * scale + sy;
                                if src_x < metrics_3x.width && src_y < metrics_3x.height {
                                    sum += bitmap_3x[src_y * metrics_3x.width + src_x] as u32;
                                    count += 1;
                                }
                            }
                        }
                        let coverage = (sum as f32 / count.max(1) as f32) / 255.0;
                        bitmap_1x[dy * dw + dx] = (coverage.powf(1.0 / options.gamma) * 255.0).round() as u8;
                    }
                }
                
                let mut metrics_1x = metrics_3x;
                metrics_1x.width = dw;
                metrics_1x.height = dh;
                metrics_1x.xmin = (metrics_3x.xmin as f32 / scale_f).round() as i32;
                metrics_1x.ymin = (metrics_3x.ymin as f32 / scale_f).round() as i32;
                metrics_1x.advance_width = metrics_3x.advance_width / scale_f;
                metrics_1x.advance_height = metrics_3x.advance_height / scale_f;
                
                min_y = min_y.min(metrics_1x.ymin);
                max_y = max_y.max(metrics_1x.ymin + metrics_1x.height as i32);
                pen_x += metrics_1x.advance_width;
                line_glyphs.push(Some((metrics_1x, bitmap_1x, pen_x)));
            }

            let w = pen_x.ceil().max(1.0) as u32;
            let h = (max_y - min_y).max(1) as u32;
            if w == 0 || h == 0 {
                continue;
            }

            let padding = options.atlas_padding;
            let padded_w = w + padding * 2;
            let padded_h = h + padding * 2;

            let mut line_buffer = vec![0u8; (padded_w * padded_h) as usize];
            let mut cursor_x = padding as f32;
            for glyph_opt in line_glyphs {
                match glyph_opt {
                    Some((metrics, bitmap, next_pen_x)) => {
                        let glyph_x = (cursor_x + metrics.xmin as f32).round().max(0.0) as u32;
                        let glyph_y = (padding as i32 + metrics.ymin - min_y).max(0) as u32;
                        for y in 0..metrics.height {
                            for x in 0..metrics.width {
                                let global_x = glyph_x + x as u32;
                                let global_y = glyph_y + y as u32;
                                if global_x >= padded_w || global_y >= padded_h {
                                    continue;
                                }
                                let idx = (global_y * padded_w + global_x) as usize;
                                let alpha = bitmap[y * metrics.width + x];
                                line_buffer[idx] = std::cmp::max(line_buffer[idx], alpha);
                            }
                        }
                        cursor_x = padding as f32 + next_pen_x;
                    }
                    None => {
                        cursor_x += req.px_size * 0.28;
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
                screen_x: req.pos_x - padding as f32,
                screen_y: req.pos_y - padding as f32,
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
        for pr in pre_rasters {
            let px: u32 = 2; // 2px left padding
            let py: u32 = current_y;

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

fn read_font(candidates: &[&str]) -> Option<Vec<u8>> {
    candidates.iter().find_map(|path| fs::read(path).ok())
}
