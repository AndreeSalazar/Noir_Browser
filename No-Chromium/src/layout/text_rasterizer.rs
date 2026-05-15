use fontdue::{
    layout::{CoordinateSystem, Layout, LayoutSettings, TextStyle},
    Font, FontSettings,
};
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

        struct PositionedGlyph {
            x: f32,
            y: f32,
            width: usize,
            height: usize,
            bitmap: Vec<u8>,
        }

        let mut pre_rasters = Vec::new();
        let mut max_atlas_w: u32 = 0;
        let mut total_atlas_h: u32 = 2; // 2px padding top

        for req in requests {
            let scale = options.oversample.max(1.0).round() as usize;
            let scale_f = scale as f32;
            let active_font = if req.is_bold { &font_bold } else { &font_reg };

            let fonts = [active_font];
            let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
            layout.reset(&LayoutSettings {
                x: 0.0,
                y: 0.0,
                ..LayoutSettings::default()
            });
            layout.append(&fonts, &TextStyle::new(&req.text, req.px_size, 0));

            let mut positioned_glyphs = Vec::new();
            let mut min_x = f32::MAX;
            let mut min_y = f32::MAX;
            let mut max_x = f32::MIN;
            let mut max_y = f32::MIN;

            for glyph in layout.glyphs() {
                let (metrics_hi, bitmap_hi) =
                    active_font.rasterize_indexed(glyph.key.glyph_index, req.px_size * scale_f);
                if metrics_hi.width == 0 || metrics_hi.height == 0 {
                    continue;
                }

                let width = (metrics_hi.width as f32 / scale_f).ceil() as usize;
                let height = (metrics_hi.height as f32 / scale_f).ceil() as usize;
                let bitmap = downsample_coverage(
                    &bitmap_hi,
                    metrics_hi.width,
                    metrics_hi.height,
                    width,
                    height,
                    scale,
                    options.gamma,
                );

                let x = glyph.x;
                let y = glyph.y;
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x + width as f32);
                max_y = max_y.max(y + height as f32);

                positioned_glyphs.push(PositionedGlyph {
                    x,
                    y,
                    width,
                    height,
                    bitmap,
                });
            }

            if positioned_glyphs.is_empty() {
                continue;
            }

            let padding = options.atlas_padding;
            let content_w = (max_x - min_x).ceil().max(1.0) as u32;
            let content_h = (max_y - min_y).ceil().max(1.0) as u32;
            let padded_w = content_w + padding * 2;
            let padded_h = content_h + padding * 2;

            let mut line_buffer = vec![0u8; (padded_w * padded_h) as usize];
            for glyph in positioned_glyphs {
                let glyph_x = (padding as f32 + glyph.x - min_x).round().max(0.0) as u32;
                let glyph_y = (padding as f32 + glyph.y - min_y).round().max(0.0) as u32;

                for y in 0..glyph.height {
                    for x in 0..glyph.width {
                        let global_x = glyph_x + x as u32;
                        let global_y = glyph_y + y as u32;
                        if global_x >= padded_w || global_y >= padded_h {
                            continue;
                        }
                        let idx = (global_y * padded_w + global_x) as usize;
                        let alpha = glyph.bitmap[y * glyph.width + x];
                        line_buffer[idx] = std::cmp::max(line_buffer[idx], alpha);
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

fn downsample_coverage(
    src: &[u8],
    src_w: usize,
    src_h: usize,
    dst_w: usize,
    dst_h: usize,
    scale: usize,
    gamma: f32,
) -> Vec<u8> {
    let mut dst = vec![0u8; dst_w * dst_h];
    for dy in 0..dst_h {
        for dx in 0..dst_w {
            let mut sum = 0u32;
            let mut count = 0u32;

            for sy in 0..scale {
                for sx in 0..scale {
                    let src_x = dx * scale + sx;
                    let src_y = dy * scale + sy;
                    if src_x < src_w && src_y < src_h {
                        sum += src[src_y * src_w + src_x] as u32;
                        count += 1;
                    }
                }
            }

            let coverage = (sum as f32 / count.max(1) as f32) / 255.0;
            dst[dy * dst_w + dx] = (coverage.powf(1.0 / gamma) * 255.0).round() as u8;
        }
    }
    dst
}

fn read_font(candidates: &[&str]) -> Option<Vec<u8>> {
    candidates.iter().find_map(|path| fs::read(path).ok())
}
