use super::glyphs::get_glyph_bitmap;

pub fn draw_rect(buf: &mut [u32], stride: usize, x: i32, y: i32, w: i32, h: i32, color: u32) {
    let sw = stride as i32;
    for row in y..y + h {
        if row < 0 || row * sw >= buf.len() as i32 {
            continue;
        }
        for col in x..x + w {
            if col >= 0 && col < sw {
                let idx = (row * sw + col) as usize;
                if idx < buf.len() {
                    buf[idx] = color;
                }
            }
        }
    }
}

pub fn draw_text_noir(
    buf: &mut [u32],
    stride: usize,
    screen_w: i32,
    x: i32,
    y: i32,
    text: &str,
    color: u32,
    scale: f32,
) {
    let sw = stride as i32;
    let char_w = (7.0 * scale) as i32;
    let spacing = (1.0 * scale) as i32;
    let r = ((color >> 16) & 0xFF) as u8;
    let g = ((color >> 8) & 0xFF) as u8;
    let b = (color & 0xFF) as u8;
    let pixel = 0xFF_000000 | ((r as u32) << 16) | ((g as u32) << 8) | b as u32;

    for (ci, ch) in text.chars().enumerate() {
        let cx = x + ci as i32 * (char_w + spacing);
        if cx + char_w > screen_w {
            break;
        }
        if ch == ' ' {
            continue;
        }
        let glyph = get_glyph_bitmap(ch);
        for gy in 0..glyph.len() {
            for gx in 0..glyph[0].len() {
                if glyph[gy][gx] {
                    for sy in 0..scale as i32 {
                        for sx in 0..scale as i32 {
                            let px = cx + gx as i32 + sx;
                            let py = y + gy as i32 + sy;
                            if px >= 0 && px < sw && py >= 0 {
                                let idx = (py * sw + px) as usize;
                                if idx < buf.len() {
                                    buf[idx] = pixel;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
