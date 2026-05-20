use fontdue::{
    layout::{CoordinateSystem, Layout, LayoutSettings, TextStyle},
    Font, FontSettings,
};
use std::collections::HashMap;
use std::fs;
use std::sync::{Mutex, OnceLock};

const DEFAULT_OVERSAMPLE: f32 = 3.0;
const DEFAULT_ATLAS_PADDING: u32 = 2;

#[derive(Clone, Copy, Debug)]
pub struct TextRasterizationOptions {
    pub oversample: f32,
    pub atlas_padding: u32,
    pub gamma: f32,
    pub bitmap_mode: TextBitmapMode,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TextBitmapMode {
    AlphaMask,
    SubpixelMask,
}

impl TextRasterizationOptions {
    pub fn sharp_lcd() -> Self {
        Self {
            oversample: DEFAULT_OVERSAMPLE,
            atlas_padding: DEFAULT_ATLAS_PADDING,
            gamma: 1.15,
            bitmap_mode: TextBitmapMode::SubpixelMask,
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
pub struct AtlasImageRequest {
    pub rgba: std::sync::Arc<Vec<u8>>,
    pub width: u32,
    pub height: u32,
    pub pos_x: f32,
    pub pos_y: f32,
    pub dest_w: f32,
    pub dest_h: f32,
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

struct FontPair {
    regular: Font,
    bold: Font,
}

#[derive(Hash, PartialEq, Eq)]
struct GlyphCacheKey {
    font_slot: u8,
    glyph_index: u16,
    px_milli: u32,
    oversample: u32,
    gamma_milli: u32,
    bitmap_mode: TextBitmapMode,
}

#[derive(Clone)]
struct CachedGlyph {
    width: usize,
    height: usize,
    rgba: Vec<u8>,
}

static FONT_PAIR: OnceLock<FontPair> = OnceLock::new();
static GLYPH_CACHE: OnceLock<Mutex<HashMap<GlyphCacheKey, CachedGlyph>>> = OnceLock::new();

impl RasterizedAtlas {
    #[allow(dead_code)]
    pub fn new(requests: &[TextRequest]) -> Self {
        Self::with_options(requests, &[], TextRasterizationOptions::sharp_lcd())
    }

    pub fn with_options(
        requests: &[TextRequest],
        image_requests: &[AtlasImageRequest],
        options: TextRasterizationOptions,
    ) -> Self {
        println!(
            "[*] Rasterizando Atlas de Texto en CPU con {} peticiones y {} imágenes",
            requests.len(),
            image_requests.len()
        );
        let font_pair = load_font_pair();
        let font_reg = &font_pair.regular;
        let font_bold = &font_pair.bold;

        struct PreRaster {
            width: u32,
            height: u32,
            rgba: Vec<u8>,
            screen_x: f32,
            screen_y: f32,
            dest_w: f32,
            dest_h: f32,
            color: [f32; 4],
        }

        struct PositionedGlyph {
            x: f32,
            y: f32,
            width: usize,
            height: usize,
            rgba: Vec<u8>,
        }

        let mut pre_rasters = Vec::new();
        let mut max_atlas_w: u32 = 0;
        let mut total_atlas_h: u32 = 2; // 2px padding top

        for req in requests {
            let scale = options.oversample.max(1.0).round() as usize;
            let font_slot = if req.is_bold { 1 } else { 0 };
            let active_font = if req.is_bold { font_bold } else { font_reg };

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
                let cache_key = GlyphCacheKey {
                    font_slot,
                    glyph_index: glyph.key.glyph_index,
                    px_milli: (req.px_size * 1000.0).round() as u32,
                    oversample: scale as u32,
                    gamma_milli: (options.gamma * 1000.0).round() as u32,
                    bitmap_mode: options.bitmap_mode,
                };

                let cached = match cached_glyph(&cache_key) {
                    Some(cached) => cached,
                    None => {
                        let cached = rasterize_glyph(
                            active_font,
                            glyph.key.glyph_index,
                            req.px_size,
                            scale,
                            options,
                        );
                        glyph_cache()
                            .lock()
                            .unwrap()
                            .insert(cache_key, cached.clone());
                        cached
                    }
                };

                if cached.width == 0 || cached.height == 0 {
                    continue;
                }

                let x = glyph.x;
                let y = glyph.y;
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x + cached.width as f32);
                max_y = max_y.max(y + cached.height as f32);

                positioned_glyphs.push(PositionedGlyph {
                    x,
                    y,
                    width: cached.width,
                    height: cached.height,
                    rgba: cached.rgba,
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

            let mut line_rgba = vec![0u8; (padded_w * padded_h * 4) as usize];
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
                        let dst_idx = ((global_y * padded_w + global_x) as usize) * 4;
                        let src_idx = (y * glyph.width + x) * 4;
                        line_rgba[dst_idx] = line_rgba[dst_idx].max(glyph.rgba[src_idx]);
                        line_rgba[dst_idx + 1] =
                            line_rgba[dst_idx + 1].max(glyph.rgba[src_idx + 1]);
                        line_rgba[dst_idx + 2] =
                            line_rgba[dst_idx + 2].max(glyph.rgba[src_idx + 2]);
                        line_rgba[dst_idx + 3] =
                            line_rgba[dst_idx + 3].max(glyph.rgba[src_idx + 3]);
                    }
                }
            }

            if padded_w > max_atlas_w {
                max_atlas_w = padded_w;
            }

            pre_rasters.push(PreRaster {
                width: padded_w,
                height: padded_h,
                rgba: line_rgba,
                screen_x: req.pos_x - padding as f32,
                screen_y: req.pos_y - padding as f32,
                dest_w: padded_w as f32,
                dest_h: padded_h as f32,
                color: req.color,
            });

            total_atlas_h += padded_h + 2; // minimum 2px padding between lines
        }

        for img_req in image_requests {
            if img_req.width == 0 || img_req.height == 0 || img_req.rgba.len() != (img_req.width * img_req.height * 4) as usize {
                continue;
            }
            if img_req.width > max_atlas_w {
                max_atlas_w = img_req.width;
            }
            pre_rasters.push(PreRaster {
                width: img_req.width,
                height: img_req.height,
                rgba: (*img_req.rgba).clone(),
                screen_x: img_req.pos_x,
                screen_y: img_req.pos_y,
                dest_w: img_req.dest_w,
                dest_h: img_req.dest_h,
                color: [1.0, 1.0, 1.0, 1.0],
            });
            total_atlas_h += img_req.height + 2;
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
                    let src_idx = ((y * pr.width + x) as usize) * 4;
                    let a = pr.rgba[src_idx + 3];
                    if a > 0 {
                        let idx = ((py + y) * max_atlas_w + (px + x)) as usize * 4;
                        rgba_data[idx] = pr.rgba[src_idx];
                        rgba_data[idx + 1] = pr.rgba[src_idx + 1];
                        rgba_data[idx + 2] = pr.rgba[src_idx + 2];
                        rgba_data[idx + 3] = a;
                    }
                }
            }

            quads.push(TextQuad {
                x: pr.screen_x,
                y: pr.screen_y,
                w: pr.dest_w,
                h: pr.dest_h,
                u0: px as f32 / max_atlas_w as f32,
                v0: py as f32 / total_atlas_h as f32,
                u1: (px + pr.width) as f32 / max_atlas_w as f32,
                v1: (py + pr.height) as f32 / total_atlas_h as f32,
                color: pr.color,
            });

            current_y += pr.height + 2; // advance Y with padding
        }

        println!(
            "[+] Texture Atlas generado. Dimensión: {}x{} con {} textos.",
            max_atlas_w,
            total_atlas_h,
            quads.len()
        );

        Self {
            width: max_atlas_w,
            height: total_atlas_h,
            rgba_data,
            quads,
        }
    }
}

fn load_font_pair() -> &'static FontPair {
    FONT_PAIR.get_or_init(|| {
        let font_bytes_reg = read_font(&[
            "C:\\Windows\\Fonts\\segoeui.ttf",
            "C:\\Windows\\Fonts\\arial.ttf",
        ])
        .expect("Fallo al leer una fuente regular de Windows");
        let regular = Font::from_bytes(font_bytes_reg, FontSettings::default())
            .expect("Fallo al parsear la fuente regular");

        let font_bytes_bold = read_font(&[
            "C:\\Windows\\Fonts\\segoeuib.ttf",
            "C:\\Windows\\Fonts\\arialbd.ttf",
        ])
        .expect("Fallo al leer una fuente bold de Windows");
        let bold = Font::from_bytes(font_bytes_bold, FontSettings::default())
            .expect("Fallo al parsear la fuente bold");

        FontPair { regular, bold }
    })
}

fn glyph_cache() -> &'static Mutex<HashMap<GlyphCacheKey, CachedGlyph>> {
    GLYPH_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn cached_glyph(cache_key: &GlyphCacheKey) -> Option<CachedGlyph> {
    glyph_cache().lock().unwrap().get(cache_key).cloned()
}

fn rasterize_glyph(
    font: &Font,
    glyph_index: u16,
    px_size: f32,
    scale: usize,
    options: TextRasterizationOptions,
) -> CachedGlyph {
    let scale_f = scale as f32;
    match options.bitmap_mode {
        TextBitmapMode::AlphaMask => {
            let (metrics_hi, bitmap_hi) = font.rasterize_indexed(glyph_index, px_size * scale_f);
            let width = (metrics_hi.width as f32 / scale_f).ceil() as usize;
            let height = (metrics_hi.height as f32 / scale_f).ceil() as usize;
            let alpha = downsample_coverage(
                &bitmap_hi,
                metrics_hi.width,
                metrics_hi.height,
                width,
                height,
                scale,
                options.gamma,
            );
            CachedGlyph {
                width,
                height,
                rgba: alpha_to_rgba(&alpha),
            }
        }
        TextBitmapMode::SubpixelMask => {
            let (metrics_hi, bitmap_hi) =
                font.rasterize_indexed_subpixel(glyph_index, px_size * scale_f);
            let width = (metrics_hi.width as f32 / scale_f).ceil() as usize;
            let height = (metrics_hi.height as f32 / scale_f).ceil() as usize;
            CachedGlyph {
                width,
                height,
                rgba: downsample_lcd_coverage(
                    &bitmap_hi,
                    metrics_hi.width,
                    metrics_hi.height,
                    width,
                    height,
                    scale,
                    options.gamma,
                ),
            }
        }
    }
}

fn alpha_to_rgba(alpha: &[u8]) -> Vec<u8> {
    let mut rgba = vec![0u8; alpha.len() * 4];
    for (i, a) in alpha.iter().copied().enumerate() {
        let idx = i * 4;
        rgba[idx] = a;
        rgba[idx + 1] = a;
        rgba[idx + 2] = a;
        rgba[idx + 3] = a;
    }
    rgba
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

fn downsample_lcd_coverage(
    src: &[u8],
    src_w: usize,
    src_h: usize,
    dst_w: usize,
    dst_h: usize,
    scale: usize,
    gamma: f32,
) -> Vec<u8> {
    let mut dst = vec![0u8; dst_w * dst_h * 4];
    let src_stride = src_w * 3;

    for dy in 0..dst_h {
        for dx in 0..dst_w {
            let mut rgb = [0u32; 3];
            let mut count = 0u32;

            for sy in 0..scale {
                for sx in 0..scale {
                    let src_x = dx * scale + sx;
                    let src_y = dy * scale + sy;
                    if src_x < src_w && src_y < src_h {
                        let src_idx = src_y * src_stride + src_x * 3;
                        rgb[0] += src[src_idx] as u32;
                        rgb[1] += src[src_idx + 1] as u32;
                        rgb[2] += src[src_idx + 2] as u32;
                        count += 1;
                    }
                }
            }

            let idx = (dy * dst_w + dx) * 4;
            let mut max_channel = 0u8;
            for channel in 0..3 {
                let coverage = (rgb[channel] as f32 / count.max(1) as f32) / 255.0;
                let value = (coverage.powf(1.0 / gamma) * 255.0).round() as u8;
                dst[idx + channel] = value;
                max_channel = max_channel.max(value);
            }
            dst[idx + 3] = max_channel;
        }
    }

    dst
}

fn read_font(candidates: &[&str]) -> Option<Vec<u8>> {
    candidates.iter().find_map(|path| fs::read(path).ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_small_atlas_without_stalling() {
        let atlas = RasterizedAtlas::with_options(
            &[TextRequest {
                text: "Example Domain".to_string(),
                px_size: 24.0,
                is_bold: true,
                pos_x: 40.0,
                pos_y: 80.0,
                color: [1.0, 1.0, 1.0, 1.0],
            }],
            &[],
            TextRasterizationOptions::sharp_lcd(),
        );

        assert!(atlas.width > 0);
        assert!(atlas.height > 0);
        assert!(!atlas.rgba_data.is_empty());
        assert!(!atlas.quads.is_empty());
    }
}
