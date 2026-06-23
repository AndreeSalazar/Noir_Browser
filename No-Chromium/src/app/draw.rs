//! Draw - Primitivas de dibujo básicas
//!
//! Funciones de bajo nivel para dibujar rectángulos y texto.

use super::glyphs::draw_glyph;

/// Dibuja un rectángulo sólido
pub fn draw_rect(buf: &mut [u32], stride: usize, x: i32, y: i32, w: i32, h: i32, color: u32) {
    for dy in 0..h {
        let py = y + dy;
        if py < 0 { continue; }
        let row_start = (py as usize) * stride;
        for dx in 0..w {
            let px = x + dx;
            if px < 0 { continue; }
            let idx = row_start + px as usize;
            if idx < buf.len() {
                buf[idx] = color;
            }
        }
    }
}

/// Caracteres soportados por el bitmap font
fn is_supported(ch: char) -> bool {
    ch.is_ascii_alphanumeric()
        || matches!(ch,
            ' ' | '.' | ',' | ':' | '!' | '?' | '/' | '-' | '_' | '+' | '='
            | '(' | ')' | '[' | ']' | '{' | '}' | '<' | '>' | '|' | '*' | '#' | '@'
            | '\'' | '"' | '~' | '^' | '`' | '%' | '&' | '$' | '\u{00A0}'
        )
}

/// Reemplaza caracteres no soportados con '?'
fn sanitize(text: &str) -> String {
    text.chars()
        .map(|c| if is_supported(c) || c == '\u{00A0}' { c } else { '?' })
        .collect()
}

/// Dibuja texto usando el bitmap font
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
    let char_w = (7.0 * scale) as i32;
    let char_h = (12.0 * scale) as i32;
    let mut cx = x;
    let clean = sanitize(text);
    for ch in clean.chars() {
        if cx >= screen_w {
            break;
        }
        draw_glyph(buf, stride, cx, y, ch, color, scale, char_w, char_h);
        cx += char_w + 1;
    }
}

/// Mide el ancho aproximado de un texto
pub fn measure_text_width(text: &str, scale: f32) -> f32 {
    let clean = sanitize(text);
    clean.chars().count() as f32 * (7.0 * scale + 1.0)
}

