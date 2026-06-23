//! Chrome UI Renderer - Dibuja la interfaz Chrome-like
//!
//! Constantes de layout y funciones de dibujo para la UI.

use super::context::AppContext;
use super::draw::{draw_rect, draw_text_noir, measure_text_width};
use crate::parsers::layout::LayoutItem;
use crate::parsers::layout::total_content_height;

// === LAYOUT CONSTANTS ===
pub const TITLE_BAR_HEIGHT: u32 = 32;
pub const TAB_BAR_HEIGHT: u32 = 36;
pub const NAV_BAR_HEIGHT: u32 = 44;
pub const TOOLBAR_HEIGHT: u32 = TITLE_BAR_HEIGHT + TAB_BAR_HEIGHT + NAV_BAR_HEIGHT;
pub const TAB_WIDTH: u32 = 200;
pub const TAB_SPACING: i32 = 4;
pub const NAV_BTN_SIZE: u32 = 34;
pub const NAV_BTN_SPACING: i32 = 6;
pub const NAV_START_X: i32 = 8;

// === COLORS ===
const BG_CONTENT: u32 = 0xFF12121A;
const BG_TITLEBAR: u32 = 0xFF1A1A22;
const BG_TAB_BAR: u32 = 0xFF15151E;
const BG_ADDRESS_BAR: u32 = 0xFF1E1E26;
const BG_ADDRESS_BAR_FOCUS: u32 = 0xFF2A2A35;
const BG_DARK: u32 = 0xFF0E0E14;
const BG_LINK_CARD: u32 = 0xFF1F1F28;
const ACCENT: u32 = 0xFFFF3344;
const TEXT_WHITE: u32 = 0xFFFFFFFF;
const TEXT_DIM: u32 = 0xFFB0B0B8;
const TEXT_PLACEHOLDER: u32 = 0xFF707078;
const BTN_BG: u32 = 0xFF2A2A35;
const CLOSE_RED: u32 = 0xFFE53935;
const GREEN: u32 = 0xFF4CAF50;
const LINK_GOOGLE: u32 = 0xFF4285F4;
const LINK_GITHUB: u32 = 0xFF24292E;
const LINK_YOUTUBE: u32 = 0xFFFF0000;
const LINK_RUST: u32 = 0xFFCE422B;

/// Dibuja un frame completo
pub fn draw(ctx: &mut AppContext) {
    let display_url = ctx.url_bar.clone();
    let url_color = 0xFFE0E0E8;
    let url_bar_empty = ctx.url_bar.is_empty();
    let url_focused = ctx.url_focused;
    let active_tab = ctx.active_tab;
    let active_url = ctx.tabs[active_tab].url.clone();
    let active_title = ctx.tabs[active_tab].title.clone();
    let layout_blocks = ctx.tabs[active_tab].layout_blocks.clone();
    let scroll_y = ctx.tabs[active_tab].scroll_y;
    let fetching = ctx.fetching;
    let anim_frame = ctx.loading_anim_frame;
    let console_open = ctx.console_open;
    let console_messages = ctx.console_messages.clone();
    let find_open = ctx.find_open;
    let find_query = ctx.find_query.clone();
    let shortcuts_open = ctx.shortcuts_open;
    let is_https = active_url.starts_with("https://");

    let (surface, window) = match (&mut ctx.surface, &ctx.window) {
        (Some(s), Some(w)) => (s, w),
        _ => return,
    };

    let size = window.inner_size();
    let width = size.width.max(1);
    let height = size.height.max(1);

    let mut buffer = surface.buffer_mut().unwrap();
    let buf = buffer.as_mut();
    let stride = width as usize;

    for pixel in buf.iter_mut() {
        *pixel = BG_CONTENT;
    }

    let w = width as i32;
    let h = height as i32;

    let tab_titles: Vec<String> = ctx.tabs.iter().map(|t| {
        if t.title.len() > 20 {
            format!("{}...", &t.title[..17])
        } else {
            t.title.clone()
        }
    }).collect();

    // Title bar (with dynamic page title)
    draw_title_bar(buf, stride, w, &tab_titles, active_tab, &active_title, is_https);

    // Tab bar
    let tab_y = TITLE_BAR_HEIGHT as i32;
    draw_tab_bar(buf, stride, w, tab_y, &tab_titles, active_tab);

    // Nav bar
    let nav_y = (TITLE_BAR_HEIGHT + TAB_BAR_HEIGHT) as i32;
    draw_nav_bar(buf, stride, w, nav_y, is_https, fetching, anim_frame);

    // Address bar
    draw_address_bar(buf, stride, w, &display_url, url_color, url_focused, url_bar_empty);

    // Progress bar
    if fetching {
        let progress = (anim_frame % 100) as f32 / 100.0;
        let bar_w = (w as f32 * progress) as i32;
        draw_rect(buf, stride, 0, TOOLBAR_HEIGHT as i32, bar_w, 2, ACCENT);
    }

    // Content
    let content_y = TOOLBAR_HEIGHT as i32;
    let content_h = h - content_y;

    if active_url.is_empty() {
        draw_new_tab_page(buf, stride, w, content_y, content_h);
    } else if fetching {
        draw_loading_animation(buf, stride, w, content_y, content_h, anim_frame);
        draw_text_noir(buf, stride, w, 30, content_y + content_h - 30, &active_url, TEXT_PLACEHOLDER, 1.0);
    } else if !layout_blocks.is_empty() {
        render_layout_blocks(buf, stride, w, content_y, content_h, &layout_blocks, scroll_y);
    } else if let Some(err) = &ctx.fetch_error {
        draw_error_page(buf, stride, w, content_y, content_h, err, &active_url);
    } else {
        draw_text_noir(buf, stride, w, w / 2 - 50, content_y + 40, "Empty", TEXT_DIM, 1.2);
    }

    // Scroll indicator
    if !layout_blocks.is_empty() {
        let total_h = total_content_height(&layout_blocks);
        if total_h > content_h as f32 && content_h > 0 {
            let view_ratio = content_h as f32 / total_h;
            let scroll_ratio = scroll_y / (total_h - content_h as f32).max(1.0);
            let bar_h = (content_h as f32 * view_ratio).max(20.0);
            let bar_y = content_y as f32 + scroll_ratio * (content_h as f32 - bar_h);
            draw_rect(buf, stride, w - 6, bar_y as i32, 4, bar_h as i32, 0x40FFFFFF);
        }
    }

    // Find in page
    if find_open {
        draw_find_bar(buf, stride, w, h, &find_query);
    }

    // Console panel
    if console_open {
        draw_console_panel(buf, stride, w, h, &console_messages);
    }

    // Keyboard shortcuts
    if shortcuts_open {
        draw_shortcuts_panel(buf, stride, w, h);
    }

    buffer.present().unwrap();
}

fn draw_title_bar(buf: &mut [u32], stride: usize, w: i32, _tab_titles: &[String], _active_tab: usize, page_title: &str, is_https: bool) {
    draw_rect(buf, stride, 0, 0, w, TITLE_BAR_HEIGHT as i32, BG_TITLEBAR);
    draw_rect(buf, stride, 10, 10, 14, 14, ACCENT);
    draw_text_noir(buf, stride, w, 30, 11, "Noir Browser", TEXT_DIM, 1.0);

    // Page title in the middle (or HTTPS indicator)
    let display_title = if page_title.is_empty() || page_title == "New Tab" {
        String::from("Noir Browser")
    } else {
        page_title.to_string()
    };
    let title_w = measure_text_width(&display_title, 1.0) as i32;
    let title_x = (w - title_w) / 2;
    draw_text_noir(buf, stride, w, title_x, 11, &display_title, TEXT_WHITE, 1.0);

    // HTTPS lock icon next to title
    if is_https {
        let lock_x = title_x - 18;
        draw_rect(buf, stride, lock_x, 11, 4, 5, GREEN);
        draw_rect(buf, stride, lock_x - 1, 8, 6, 4, GREEN);
        draw_text_noir(buf, stride, w, lock_x, 16, "v", GREEN, 0.6);
    }

    let ctrl_w = 46;
    let ctrl_h = TITLE_BAR_HEIGHT as i32;

    let min_x = w - ctrl_w * 3;
    draw_rect(buf, stride, min_x, 0, ctrl_w, ctrl_h, BTN_BG);
    draw_text_noir(buf, stride, w, min_x + 18, 11, "-", TEXT_DIM, 1.2);

    let max_x = w - ctrl_w * 2;
    draw_rect(buf, stride, max_x, 0, ctrl_w, ctrl_h, BTN_BG);
    draw_rect(buf, stride, max_x + 17, 11, 10, 10, TEXT_DIM);
    draw_rect(buf, stride, max_x + 18, 12, 8, 8, BG_TITLEBAR);

    let close_x = w - ctrl_w;
    draw_rect(buf, stride, close_x, 0, ctrl_w, ctrl_h, CLOSE_RED);
    draw_text_noir(buf, stride, w, close_x + 17, 11, "X", TEXT_WHITE, 1.0);
}

fn draw_tab_bar(buf: &mut [u32], stride: usize, w: i32, tab_y: i32, tab_titles: &[String], active_tab: usize) {
    draw_rect(buf, stride, 0, tab_y, w, TAB_BAR_HEIGHT as i32, BG_TAB_BAR);

    let mut tx = 4i32;
    for (i, title) in tab_titles.iter().enumerate() {
        let tab_w = TAB_WIDTH.min(w as u32 - tx as u32 - 100) as i32;
        if tx + tab_w > w - 100 { break; }

        let ty = tab_y + 4;
        let th = TAB_BAR_HEIGHT as i32 - 8;

        if i == active_tab {
            draw_rect(buf, stride, tx, ty, tab_w, th, BG_ADDRESS_BAR);
            draw_rect(buf, stride, tx, ty, tab_w, 2, ACCENT);
        } else {
            draw_rect(buf, stride, tx, ty, tab_w, th, BG_TAB_BAR);
        }

        draw_rect(buf, stride, tx + 8, ty + (th / 2) - 3, 6, 6, ACCENT);

        let text_color = if i == active_tab { TEXT_WHITE } else { TEXT_DIM };
        draw_text_noir(buf, stride, w, tx + 20, ty + (th / 2) - 4, title, text_color, 0.9);
        draw_text_noir(buf, stride, w, tx + tab_w - 18, ty + (th / 2) - 4, "x", TEXT_DIM, 0.8);

        tx += tab_w + TAB_SPACING;
    }

    if tx + 34 < w {
        draw_rect(buf, stride, tx + 2, tab_y + 7, 28, TAB_BAR_HEIGHT as i32 - 14, BTN_BG);
        draw_text_noir(buf, stride, w, tx + 10, tab_y + 11, "+", TEXT_DIM, 1.2);
    }
}

fn draw_nav_bar(buf: &mut [u32], stride: usize, w: i32, nav_y: i32, is_https: bool, fetching: bool, anim_frame: u32) {
    draw_rect(buf, stride, 0, nav_y, w, NAV_BAR_HEIGHT as i32, BG_DARK);

    let btn_h = 34i32;
    let btn_y_pos = nav_y + (NAV_BAR_HEIGHT as i32 - btn_h) / 2;
    let mut bx = NAV_START_X;

    // Back button
    draw_rect(buf, stride, bx, btn_y_pos, NAV_BTN_SIZE as i32, btn_h, BTN_BG);
    draw_text_noir(buf, stride, w, bx + 13, btn_y_pos + 9, "<", TEXT_WHITE, 1.2);
    bx += NAV_BTN_SIZE as i32 + NAV_BTN_SPACING;

    // Forward button
    draw_rect(buf, stride, bx, btn_y_pos, NAV_BTN_SIZE as i32, btn_h, BTN_BG);
    draw_text_noir(buf, stride, w, bx + 13, btn_y_pos + 9, ">", TEXT_WHITE, 1.2);
    bx += NAV_BTN_SIZE as i32 + NAV_BTN_SPACING;

    // Reload button (animated when fetching)
    draw_rect(buf, stride, bx, btn_y_pos, NAV_BTN_SIZE as i32, btn_h, BTN_BG);
    if fetching {
        draw_loading_spinner(buf, stride, bx + NAV_BTN_SIZE as i32 / 2 - 4, btn_y_pos + btn_h / 2 - 4, 8, anim_frame);
    } else {
        draw_text_noir(buf, stride, w, bx + 13, btn_y_pos + 9, "R", TEXT_WHITE, 1.2);
    }
    bx += NAV_BTN_SIZE as i32 + NAV_BTN_SPACING;

    // Home button
    draw_rect(buf, stride, bx, btn_y_pos, NAV_BTN_SIZE as i32, btn_h, BTN_BG);
    draw_text_noir(buf, stride, w, bx + 13, btn_y_pos + 9, "H", TEXT_WHITE, 1.2);
    bx += NAV_BTN_SIZE as i32 + 14;

    // HTTPS indicator in address bar
    if is_https {
        let lock_x = bx + 2;
        let lock_y = btn_y_pos + 12;
        draw_rect(buf, stride, lock_x, lock_y + 3, 6, 5, GREEN);
        draw_rect(buf, stride, lock_x + 1, lock_y, 4, 4, GREEN);
    }
}

fn draw_address_bar(
    buf: &mut [u32],
    stride: usize,
    w: i32,
    display_url: &str,
    url_color: u32,
    url_focused: bool,
    url_bar_empty: bool,
) {
    let nav_y = (TITLE_BAR_HEIGHT + TAB_BAR_HEIGHT) as i32;
    let btn_h = 34i32;
    let btn_y_pos = nav_y + (NAV_BAR_HEIGHT as i32 - btn_h) / 2;
    let mut bx = NAV_START_X;
    bx += (NAV_BTN_SIZE as i32 + NAV_BTN_SPACING) * 3;
    bx += NAV_BTN_SIZE as i32 + 14;

    let ab_w = w - bx - 16;
    if ab_w <= 80 { return; }

    let ab_bg = if url_focused { BG_ADDRESS_BAR_FOCUS } else { BG_ADDRESS_BAR };
    draw_rect(buf, stride, bx, btn_y_pos, ab_w, btn_h, ab_bg);

    let text_x = bx + 14;
    let text_y = btn_y_pos + (btn_h / 2) - 5;

    if url_focused || !url_bar_empty {
        draw_text_noir(buf, stride, w, text_x, text_y, display_url, url_color, 1.0);

        if url_focused {
            let cursor_px = text_x + measure_text_width(display_url, 1.0) as i32 + 2;
            draw_rect(buf, stride, cursor_px, text_y, 2, 10, TEXT_WHITE);
        }
    } else {
        draw_text_noir(buf, stride, w, text_x, text_y, "Search or enter URL...", TEXT_PLACEHOLDER, 1.0);
    }

    if !url_bar_empty {
        let lock_x = bx + ab_w - 30;
        let lock_y = btn_y_pos + (btn_h / 2) - 5;
        draw_rect(buf, stride, lock_x, lock_y + 3, 8, 7, GREEN);
        draw_rect(buf, stride, lock_x + 1, lock_y, 6, 5, GREEN);
    }
}

fn draw_new_tab_page(buf: &mut [u32], stride: usize, w: i32, content_y: i32, content_h: i32) {
    let center_y = content_y + content_h / 2 - 80;

    draw_text_noir(buf, stride, w, w / 2 - 115, center_y, "NOIR", ACCENT, 3.5);
    draw_text_noir(buf, stride, w, w / 2 - 130, center_y + 55, "BROWSER", TEXT_DIM, 2.0);

    draw_text_noir(
        buf, stride, w, w / 2 - 170, center_y + 100,
        "Ultra-fast  |  Private  |  WebGPU-powered", TEXT_PLACEHOLDER, 1.0,
    );

    let link_y = center_y + 160;
    let links = [
        ("Google", LINK_GOOGLE),
        ("GitHub", LINK_GITHUB),
        ("YouTube", LINK_YOUTUBE),
        ("Rust", LINK_RUST),
    ];
    let total_w = links.len() as i32 * 120 + (links.len() as i32 - 1) * 16;
    let start_x = w / 2 - total_w / 2;

    for (i, (name, color)) in links.iter().enumerate() {
        let lx = start_x + i as i32 * (120 + 16);

        draw_rect(buf, stride, lx, link_y, 120, 120, BG_LINK_CARD);
        draw_rect(buf, stride, lx, link_y, 120, 3, *color);

        let icon_size = 24;
        let icon_x = lx + (120 - icon_size) / 2;
        let icon_y = link_y + 24;
        draw_rect(buf, stride, icon_x, icon_y, icon_size, icon_size, *color);

        let label_w = name.len() as i32 * 7;
        let label_x = lx + (120 - label_w) / 2;
        draw_text_noir(buf, stride, w, label_x, link_y + 120 + 14, name, TEXT_DIM, 1.0);
    }
}

fn render_layout_blocks(
    buf: &mut [u32],
    stride: usize,
    screen_w: i32,
    content_y: i32,
    content_h: i32,
    items: &[LayoutItem],
    scroll_y: f32,
) {
    for item in items {
        match item {
            LayoutItem::Text(block) => {
                // CSS display: none - skip element
                if block.display == "none" { continue; }
                // CSS visibility: hidden - skip but keep space
                if !block.visible { continue; }

                let screen_block_y = block.y - scroll_y + content_y as f32;

                if screen_block_y + block.h < content_y as f32 - 10.0 {
                    continue;
                }
                if screen_block_y > content_y as f32 + content_h as f32 + 10.0 {
                    continue;
                }

                if let Some(bg) = &block.bg_color {
                    // Solo pintar bg si NO es el default (negro/transparente)
                    // y NO es blanco puro (eso causa cuadrados blancos feos)
                    let is_default = bg[0] < 0.05 && bg[1] < 0.05 && bg[2] < 0.05;
                    let is_white = bg[0] > 0.95 && bg[1] > 0.95 && bg[2] > 0.95;
                    if !is_default && !is_white {
                        let bg_u32 = rgba_to_u32(bg[0], bg[1], bg[2], bg[3]);
                        draw_rect(
                            buf,
                            stride,
                            block.x as i32 - 4,
                            screen_block_y as i32 - 2,
                            (block.w + block.padding_left + 8.0) as i32,
                            (block.h + block.padding_top + 4.0) as i32,
                            bg_u32,
                        );
                    } else if is_white {
                        // Para bg blanco, dibujar un border sutil
                        let x = block.x as i32 - 4;
                        let y = screen_block_y as i32 - 2;
                        let w = (block.w + block.padding_left + 8.0) as i32;
                        let h = (block.h + block.padding_top + 4.0) as i32;
                        draw_rect(buf, stride, x, y, w, 1, 0xFF5599FF);
                        draw_rect(buf, stride, x, y + h - 1, w, 1, 0xFF5599FF);
                        draw_rect(buf, stride, x, y, 1, h, 0xFF5599FF);
                        draw_rect(buf, stride, x + w - 1, y, 1, h, 0xFF5599FF);
                    }
                }

                if block.is_link {
                    let underline_y = screen_block_y as i32 + block.h as i32 - 1;
                    draw_rect(
                        buf,
                        stride,
                        block.x as i32,
                        underline_y,
                        block.w as i32,
                        1,
                        0xFF6699FF,
                    );
                }

                let color_u32 = rgba_to_u32(block.color[0], block.color[1], block.color[2], block.color[3]);
                let font_scale = block.font_size / 14.0;
                draw_text_noir(
                    buf,
                    stride,
                    screen_w,
                    block.x as i32,
                    screen_block_y as i32,
                    &block.text,
                    color_u32,
                    font_scale,
                );
            }
            LayoutItem::Image(img) => {
                let screen_img_y = img.y - scroll_y + content_y as f32;

                if screen_img_y + img.h < content_y as f32 - 10.0 {
                    continue;
                }
                if screen_img_y > content_y as f32 + content_h as f32 + 10.0 {
                    continue;
                }

                let ix = img.x as i32;
                let iy = screen_img_y as i32;
                let iw = img.w as i32;
                let ih = img.h as i32;

                // Image background with rounded border feel
                draw_rect(buf, stride, ix, iy, iw, ih, 0xFF1A1A22);
                // Top/bottom border for separation
                draw_rect(buf, stride, ix, iy, iw, 1, 0xFF2A2A35);
                draw_rect(buf, stride, ix, iy + ih - 1, iw, 1, 0xFF2A2A35);

                if let Some(cached) = crate::media::get_cached_image(&img.src) {
                    crate::media::draw_image_to_buffer(
                        buf, stride, &cached,
                        ix, iy, iw, ih,
                        screen_w, content_y + content_h,
                    );
                } else {
                    // Loading placeholder with shimmer effect
                    let bg_color = if img.lazy { 0xFF18181F } else { 0xFF202028 };
                    draw_rect(buf, stride, ix, iy, iw, ih, bg_color);
                    // Center icon (image frame)
                    let icon_w = 40;
                    let icon_h = 30;
                    let icon_x = ix + (iw - icon_w) / 2;
                    let icon_y = iy + (ih - icon_h) / 2;
                    draw_rect(buf, stride, icon_x, icon_y, icon_w, icon_h, 0xFF3A3A45);
                    // Mountain icon
                    let m_x = icon_x + 4;
                    let m_y = icon_y + icon_h - 6;
                    draw_rect(buf, stride, m_x, m_y, 6, 4, 0xFF505060);
                    draw_rect(buf, stride, m_x + 4, m_y - 4, 6, 8, 0xFF505060);
                    draw_rect(buf, stride, m_x + 12, m_y - 2, 6, 6, 0xFF505060);
                    draw_rect(buf, stride, m_x + 18, m_y - 6, 8, 10, 0xFF505060);
                    draw_rect(buf, stride, m_x + 24, m_y - 1, 8, 5, 0xFF505060);
                    // Sun
                    draw_rect(buf, stride, icon_x + icon_w - 10, icon_y + 4, 4, 4, 0xFFFFCC55);
                    // Alt text or "Loading..."
                    let placeholder = if img.alt.is_empty() {
                        if img.lazy { "..." } else { "Loading..." }
                    } else {
                        &img.alt
                    };
                    let txt_w = measure_text_width(placeholder, 0.8) as i32;
                    draw_text_noir(buf, stride, screen_w, ix + (iw - txt_w) / 2, iy + ih / 2 + icon_h / 2 + 4, placeholder, TEXT_DIM, 0.8);
                }
            }
            LayoutItem::Video(vid) => {
                let screen_vid_y = vid.y - scroll_y + content_y as f32;

                if screen_vid_y + vid.h < content_y as f32 - 10.0 {
                    continue;
                }
                if screen_vid_y > content_y as f32 + content_h as f32 + 10.0 {
                    continue;
                }

                let vx = vid.x as i32;
                let vy = screen_vid_y as i32;
                let vw = vid.w as i32;
                let vh = vid.h as i32;

                // Video background (dark gradient: solid for now)
                draw_rect(buf, stride, vx, vy, vw, vh, 0xFF0A0A12);
                // Subtle border
                draw_rect(buf, stride, vx, vy, vw, 1, 0xFF2A2A35);
                draw_rect(buf, stride, vx, vy + vh - 1, vw, 1, 0xFF2A2A35);
                draw_rect(buf, stride, vx, vy, 1, vh, 0xFF2A2A35);
                draw_rect(buf, stride, vx + vw - 1, vy, 1, vh, 0xFF2A2A35);

                // Centered play button (circular, red like YouTube)
                let btn_size = 70;
                let btn_x = vx + (vw - btn_size) / 2;
                let btn_y = vy + (vh - btn_size) / 2;
                // Red filled circle (approximation as square with rounded edges)
                draw_rect(buf, stride, btn_x + 8, btn_y, btn_size - 16, btn_size, 0xCCFF0000);
                draw_rect(buf, stride, btn_x + 4, btn_y + 4, btn_size - 8, btn_size - 8, 0xCCFF0000);
                draw_rect(buf, stride, btn_x, btn_y + 8, btn_size, btn_size - 16, 0xCCFF0000);
                draw_rect(buf, stride, btn_x + 2, btn_y + 4, 4, btn_size - 8, 0xCCFF0000);
                draw_rect(buf, stride, btn_x + btn_size - 6, btn_y + 4, 4, btn_size - 8, 0xCCFF0000);
                draw_rect(buf, stride, btn_x + 4, btn_y + 2, 4, btn_size - 4, 0xCCFF0000);
                draw_rect(buf, stride, btn_x + btn_size - 8, btn_y + 2, 4, btn_size - 4, 0xCCFF0000);
                // White triangle play icon
                let tri_w = 22;
                let tri_h = 26;
                let tri_x = btn_x + btn_size / 2 + 2;
                let tri_y = btn_y + (btn_size - tri_h) / 2;
                for row in 0..tri_h {
                    let progress = row as f32 / tri_h as f32;
                    let width_at_row = (tri_w as f32 * (1.0 - progress * 0.3)).max(4.0) as i32;
                    let x_start = tri_x - ((tri_w - width_at_row) / 2) - 4;
                    draw_rect(buf, stride, x_start, tri_y + row, width_at_row, 1, 0xFFFFFFFF);
                }

                // Video label (top-left)
                let label = if vid.src.contains("youtube") {
                    "YouTube Video"
                } else if vid.src.contains("vimeo") {
                    "Vimeo Video"
                } else if vid.src.contains(".mp4") {
                    "MP4 Video"
                } else {
                    "Video"
                };
                draw_text_noir(buf, stride, screen_w, vx + 10, vy + 10, label, 0xFFCCCCCC, 0.9);

                // Bottom controls bar (semi-transparent)
                if vid.controls {
                    let ctrl_h = 32;
                    let ctrl_y = vy + vh - ctrl_h;
                    draw_rect(buf, stride, vx, ctrl_y, vw, ctrl_h, 0xDD000000);
                    // Play/pause button
                    draw_rect(buf, stride, vx + 8, ctrl_y + 8, 14, 16, 0xFFFFFFFF);
                    // Time display
                    draw_text_noir(buf, stride, screen_w, vx + 30, ctrl_y + 12, "0:00 / 0:00", 0xFFEEEEEE, 0.7);
                    // Progress bar
                    let prog_x = vx + 100;
                    let prog_w = vw - 200;
                    let prog_y = ctrl_y + 17;
                    draw_rect(buf, stride, prog_x, prog_y, prog_w, 4, 0xFF444444);
                    draw_rect(buf, stride, prog_x, prog_y, 20, 4, 0xFFFF0000);
                    // Volume icon
                    draw_rect(buf, stride, vx + vw - 80, ctrl_y + 8, 16, 14, 0xFFAAAAAA);
                    // Fullscreen icon
                    draw_rect(buf, stride, vx + vw - 40, ctrl_y + 8, 16, 14, 0xFFAAAAAA);
                }
            }
        }
    }
}

fn rgba_to_u32(r: f32, g: f32, b: f32, a: f32) -> u32 {
    let ri = (r * 255.0) as u32;
    let gi = (g * 255.0) as u32;
    let bi = (b * 255.0) as u32;
    let ai = (a * 255.0) as u32;
    (ai << 24) | (ri << 16) | (gi << 8) | bi
}

// === NEW UI FUNCTIONS ===

fn draw_loading_spinner(buf: &mut [u32], stride: usize, cx: i32, cy: i32, size: i32, frame: u32) {
    let segments = 8;
    let angle_offset = (frame % 60) as f32 * 0.1;
    for i in 0..segments {
        let angle = angle_offset + (i as f32) * std::f32::consts::TAU / segments as f32;
        let alpha = 1.0 - (i as f32 / segments as f32);
        let color = if alpha > 0.5 { TEXT_WHITE } else { 0x80FFFFFF };
        let r = (size as f32) * 0.7;
        let x = cx + (angle.cos() * r) as i32;
        let y = cy + (angle.sin() * r) as i32;
        draw_rect(buf, stride, x - 1, y - 1, 3, 3, color);
    }
}

fn draw_loading_animation(buf: &mut [u32], stride: usize, w: i32, content_y: i32, content_h: i32, frame: u32) {
    let cx = w / 2;
    let cy = content_y + content_h / 2;
    draw_loading_spinner(buf, stride, cx - 12, cy - 12, 24, frame);
    draw_text_noir(buf, stride, w, cx - 50, cy + 30, "Loading...", TEXT_WHITE, 1.4);
}

fn draw_error_page(buf: &mut [u32], stride: usize, w: i32, content_y: i32, content_h: i32, error: &str, url: &str) {
    draw_rect(buf, stride, 0, content_y, w, content_h, 0xFF1A0E0E);
    let cx = w / 2;
    let cy = content_y + content_h / 2;

    // Error icon (large X)
    draw_rect(buf, stride, cx - 30, cy - 60, 12, 60, 0xFFE53935);
    draw_rect(buf, stride, cx + 18, cy - 60, 12, 60, 0xFFE53935);

    draw_text_noir(buf, stride, w, cx - 60, cy + 20, "Failed to load page", TEXT_WHITE, 1.8);
    draw_text_noir(buf, stride, w, cx - 80, cy + 70, url, TEXT_DIM, 1.0);

    let truncated_err: String = if error.len() > 80 {
        format!("{}...", &error[..77])
    } else {
        error.to_string()
    };
    draw_text_noir(buf, stride, w, cx - 100, cy + 110, &truncated_err, 0xFFE57373, 0.9);

    draw_text_noir(buf, stride, w, cx - 60, cy + 160, "Press F5 to retry", TEXT_DIM, 1.0);
}

fn draw_console_panel(buf: &mut [u32], stride: usize, w: i32, h: i32, messages: &[super::context::ConsoleMessage]) {
    let panel_h = h / 3;
    let panel_y = h - panel_h;

    // Background
    draw_rect(buf, stride, 0, panel_y, w, panel_h, 0xEE0E0E14);
    // Top border
    draw_rect(buf, stride, 0, panel_y, w, 1, ACCENT);
    // Title
    draw_text_noir(buf, stride, w, 10, panel_y + 8, "Console (F12 to close)", TEXT_WHITE, 1.0);

    // Messages
    let max_lines = ((panel_h - 30) / 14) as usize;
    let start = if messages.len() > max_lines {
        messages.len() - max_lines
    } else {
        0
    };

    for (i, msg) in messages[start..].iter().enumerate() {
        let y = panel_y + 30 + (i as i32) * 14;
        let color = match msg.level {
            super::context::ConsoleLevel::Error => 0xFFFF6B6B,
            super::context::ConsoleLevel::Warn => 0xFFFFD93D,
            super::context::ConsoleLevel::Info => 0xFF6BCFFF,
            _ => TEXT_DIM,
        };
        let prefix = match msg.level {
            super::context::ConsoleLevel::Error => "[ERROR]",
            super::context::ConsoleLevel::Warn => "[WARN]",
            super::context::ConsoleLevel::Info => "[INFO]",
            super::context::ConsoleLevel::Log => "[LOG]",
        };
        let truncated: String = if msg.text.len() > 200 {
            format!("{}...", &msg.text[..197])
        } else {
            msg.text.clone()
        };
        draw_text_noir(buf, stride, w, 10, y, prefix, color, 0.8);
        draw_text_noir(buf, stride, w, 70, y, &truncated, TEXT_WHITE, 0.8);
    }
}

fn draw_find_bar(buf: &mut [u32], stride: usize, w: i32, h: i32, query: &str) {
    let bar_w = 400;
    let bar_h = 36;
    let bar_x = (w - bar_w) / 2;
    let bar_y = 80;

    draw_rect(buf, stride, bar_x, bar_y, bar_w, bar_h, BG_ADDRESS_BAR);
    draw_rect(buf, stride, bar_x, bar_y, bar_w, 1, ACCENT);
    draw_text_noir(buf, stride, w, bar_x + 12, bar_y + 12, "Find:", TEXT_DIM, 1.0);
    draw_text_noir(buf, stride, w, bar_x + 60, bar_y + 12, query, TEXT_WHITE, 1.0);
    let cursor_x = bar_x + 60 + measure_text_width(query, 1.0) as i32 + 2;
    draw_rect(buf, stride, cursor_x, bar_y + 10, 2, 16, TEXT_WHITE);
    draw_text_noir(buf, stride, w, bar_x + bar_w - 60, bar_y + 12, "Esc", TEXT_DIM, 0.8);
}

fn draw_shortcuts_panel(buf: &mut [u32], stride: usize, w: i32, h: i32) {
    let panel_w = 500;
    let panel_h = 380;
    let panel_x = (w - panel_w) / 2;
    let panel_y = (h - panel_h) / 2;

    draw_rect(buf, stride, panel_x, panel_y, panel_w, panel_h, 0xEE1A1A22);
    draw_rect(buf, stride, panel_x, panel_y, panel_w, 1, ACCENT);

    draw_text_noir(buf, stride, w, panel_x + 20, panel_y + 12, "Keyboard Shortcuts", TEXT_WHITE, 1.4);
    draw_text_noir(buf, stride, w, panel_x + 20, panel_y + 14, "_____________", TEXT_DIM, 1.0);

    let shortcuts = [
        ("Ctrl+T", "New tab"),
        ("Ctrl+W", "Close tab"),
        ("Ctrl+L", "Focus address bar"),
        ("Ctrl+R / F5", "Reload page"),
        ("Ctrl+Tab", "Next tab"),
        ("Ctrl+D", "New tab"),
        ("F1", "Show shortcuts"),
        ("F11", "Toggle fullscreen"),
        ("F12", "Toggle console"),
        ("Ctrl+F", "Find in page"),
        ("Esc", "Close dialogs"),
    ];

    for (i, (key, desc)) in shortcuts.iter().enumerate() {
        let y = panel_y + 50 + (i as i32) * 26;
        draw_text_noir(buf, stride, w, panel_x + 30, y, key, ACCENT, 1.0);
        draw_text_noir(buf, stride, w, panel_x + 200, y, desc, TEXT_WHITE, 1.0);
    }

    draw_text_noir(buf, stride, w, panel_x + 20, panel_y + panel_h - 25, "Press F1 to close", TEXT_DIM, 0.9);
}
