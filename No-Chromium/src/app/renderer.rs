use super::draw::{draw_rect, draw_text_noir, measure_text_width};
use super::state::NoirApp;
use super::theme::*;
use crate::parsers::layout::total_content_height;

impl NoirApp {
    pub fn draw_frame(&mut self) {
        let display_url = self.display_url();
        let url_color = self.url_text_color();
        let url_bar_empty = self.url_bar.is_empty();
        let url_focused = self.url_focused;
        let active_tab = self.active_tab;
        let tab_titles: Vec<String> = self.tabs.iter().map(|t| {
            if t.title.len() > 20 {
                format!("{}...", &t.title[..17])
            } else {
                t.title.clone()
            }
        }).collect();
        let active_url = self.tabs[active_tab].url.clone();
        let layout_blocks = self.tabs[active_tab].layout_blocks.clone();
        let scroll_y = self.tabs[active_tab].scroll_y;

        let (surface, window) = match (&mut self.surface, &self.window) {
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

        draw_title_bar(buf, stride, w);
        draw_tab_bar(buf, stride, w, &tab_titles, active_tab, tab_y_from_height());
        draw_nav_bar(buf, stride, w);
        draw_address_bar(buf, stride, w, &display_url, url_color, url_focused, url_bar_empty);
        draw_content_area(buf, stride, w, h, &active_url, &layout_blocks, scroll_y, self.fetching);
        draw_scroll_indicator(buf, stride, w, h, &layout_blocks, scroll_y);

        buffer.present().unwrap();
    }
}

fn tab_y_from_height() -> i32 {
    TITLE_BAR_HEIGHT as i32
}

fn draw_title_bar(buf: &mut [u32], stride: usize, w: i32) {
    draw_rect(buf, stride, 0, 0, w, TITLE_BAR_HEIGHT as i32, BG_TITLEBAR);
    draw_rect(buf, stride, 10, 10, 14, 14, ACCENT);
    draw_text_noir(buf, stride, w, 30, 11, "Noir Browser", TEXT_DIM, 1.0);

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

fn draw_tab_bar(buf: &mut [u32], stride: usize, w: i32, tab_titles: &[String], active_tab: usize, tab_y: i32) {
    draw_rect(buf, stride, 0, tab_y, w, TAB_BAR_HEIGHT as i32, BG_TAB_BAR);

    let mut tx = 4i32;
    for (i, title) in tab_titles.iter().enumerate() {
        let tab_w = TAB_WIDTH.min(w - tx - 100);
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

fn draw_nav_bar(buf: &mut [u32], stride: usize, w: i32) {
    let nav_y = (TITLE_BAR_HEIGHT + TAB_BAR_HEIGHT) as i32;
    draw_rect(buf, stride, 0, nav_y, w, NAV_BAR_HEIGHT as i32, BG_DARK);

    let btn_h = 34i32;
    let btn_y_pos = nav_y + (NAV_BAR_HEIGHT as i32 - btn_h) / 2;
    let mut bx = NAV_START_X;

    draw_rect(buf, stride, bx, btn_y_pos, NAV_BTN_SIZE, btn_h, BTN_BG);
    draw_text_noir(buf, stride, w, bx + 13, btn_y_pos + 9, "<", TEXT_WHITE, 1.2);
    bx += NAV_BTN_SIZE + NAV_BTN_SPACING;

    draw_rect(buf, stride, bx, btn_y_pos, NAV_BTN_SIZE, btn_h, BTN_BG);
    draw_text_noir(buf, stride, w, bx + 13, btn_y_pos + 9, ">", TEXT_WHITE, 1.2);
    bx += NAV_BTN_SIZE + NAV_BTN_SPACING;

    draw_rect(buf, stride, bx, btn_y_pos, NAV_BTN_SIZE, btn_h, BTN_BG);
    draw_text_noir(buf, stride, w, bx + 13, btn_y_pos + 9, "R", TEXT_WHITE, 1.2);
    bx += NAV_BTN_SIZE + NAV_BTN_SPACING;

    draw_rect(buf, stride, bx, btn_y_pos, NAV_BTN_SIZE, btn_h, BTN_BG);
    draw_text_noir(buf, stride, w, bx + 13, btn_y_pos + 9, "H", TEXT_WHITE, 1.2);
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
    bx += (NAV_BTN_SIZE + NAV_BTN_SPACING) * 3;
    bx += NAV_BTN_SIZE + 14;

    let ab_w = w - bx - 16;
    if ab_w <= 80 { return; }

    let ab_bg = if url_focused { BG_ADDRESS_BAR_FOCUS } else { BG_ADDRESS_BAR };
    draw_rect(buf, stride, bx, btn_y_pos, ab_w, btn_h, ab_bg);

    let text_x = bx + 14;
    let text_y = btn_y_pos + (btn_h / 2) - 5;

    if url_focused || !url_bar_empty {
        draw_text_noir(buf, stride, w, text_x, text_y, display_url, url_color, 1.0);

        if url_focused {
            let cursor_px = text_x + measure_text_width(display_url, 1.0) + 2;
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

fn draw_content_area(
    buf: &mut [u32],
    stride: usize,
    w: i32,
    h: i32,
    active_url: &str,
    layout_blocks: &[crate::parsers::layout::LayoutItem],
    scroll_y: f32,
    fetching: bool,
) {
    let content_y = TOOLBAR_HEIGHT as i32;
    let content_h = h - content_y;

    if active_url.is_empty() {
        draw_new_tab_page(buf, stride, w, content_y, content_h);
    } else if fetching {
        draw_text_noir(buf, stride, w, w / 2 - 50, content_y + 40, "Loading...", TEXT_DIM, 1.2);
        draw_text_noir(buf, stride, w, 30, content_y + 80, active_url, TEXT_PLACEHOLDER, 1.0);
    } else if !layout_blocks.is_empty() {
        render_layout_blocks(buf, stride, w, content_y, content_h, layout_blocks, scroll_y);
    } else {
        draw_text_noir(buf, stride, w, w / 2 - 50, content_y + 40, "Empty", TEXT_DIM, 1.2);
    }
}

fn draw_new_tab_page(buf: &mut [u32], stride: usize, w: i32, content_y: i32, content_h: i32) {
    let center_y = content_y + content_h / 2 - 80;

    draw_text_noir(buf, stride, w, w / 2 - 115, center_y, "NOIR", ACCENT, 3.5);
    draw_text_noir(buf, stride, w, w / 2 - 130, center_y + 55, "BROWSER", TEXT_DIM, 2.0);

    draw_text_noir(
        buf, stride, w, w / 2 - 170, center_y + 100,
        "Ultra-fast  |  Private  |  Vulkan-powered", TEXT_PLACEHOLDER, 1.0,
    );

    let link_y = center_y + 160;
    let links = [
        ("Google", LINK_GOOGLE),
        ("GitHub", LINK_GITHUB),
        ("YouTube", LINK_YOUTUBE),
        ("Rust", LINK_RUST),
    ];
    let total_w = links.len() as i32 * LINK_CARD_SIZE + (links.len() as i32 - 1) * LINK_CARD_SPACING;
    let start_x = w / 2 - total_w / 2;

    for (i, (name, color)) in links.iter().enumerate() {
        let lx = start_x + i as i32 * (LINK_CARD_SIZE + LINK_CARD_SPACING);

        draw_rect(buf, stride, lx, link_y, LINK_CARD_SIZE, LINK_CARD_SIZE, BG_LINK_CARD);
        draw_rect(buf, stride, lx, link_y, LINK_CARD_SIZE, 3, *color);

        let icon_size = 24;
        let icon_x = lx + (LINK_CARD_SIZE - icon_size) / 2;
        let icon_y = link_y + 24;
        draw_rect(buf, stride, icon_x, icon_y, icon_size, icon_size, *color);

        let label_w = name.len() as i32 * 7;
        let label_x = lx + (LINK_CARD_SIZE - label_w) / 2;
        draw_text_noir(buf, stride, w, label_x, link_y + LINK_CARD_SIZE + 14, name, TEXT_DIM, 1.0);
    }

    let shortcuts_y = link_y + LINK_CARD_SIZE + 40;
    let shortcuts = [
        ("yt / youtube", "YouTube Search"),
        ("gg / google", "Google Search"),
        ("gh / github", "GitHub Search"),
        ("ddg", "DuckDuckGo"),
        ("wiki", "Wikipedia"),
        ("crates", "Crates.io"),
    ];
    let sc_total_w = shortcuts.len() as i32 / 2 * 200 + (shortcuts.len() as i32 / 2 - 1) * 16;
    let sc_start_x = w / 2 - sc_total_w / 2;
    for (i, (cmd, desc)) in shortcuts.iter().enumerate() {
        let col = i % 2;
        let row = i / 2;
        let sx = sc_start_x + col as i32 * 216;
        let sy = shortcuts_y + row as i32 * 24;
        draw_text_noir(buf, stride, w, sx, sy, cmd, ACCENT, 1.0);
        draw_text_noir(buf, stride, w, sx + measure_text_width(cmd, 1.0) + 8, sy, desc, TEXT_PLACEHOLDER, 1.0);
    }
}

fn draw_scroll_indicator(buf: &mut [u32], stride: usize, w: i32, h: i32, layout_blocks: &[crate::parsers::layout::LayoutItem], scroll_y: f32) {
    if layout_blocks.is_empty() { return; }
    let content_y = TOOLBAR_HEIGHT as i32;
    let content_h = h - content_y;

    let total_h = total_content_height(layout_blocks);
    if total_h > content_h as f32 && content_h > 0 {
        let view_ratio = content_h as f32 / total_h;
        let scroll_ratio = scroll_y / (total_h - content_h as f32).max(1.0);
        let bar_h = (content_h as f32 * view_ratio).max(20.0);
        let bar_y = content_y as f32 + scroll_ratio * (content_h as f32 - bar_h);
        draw_rect(buf, stride, w - 6, bar_y as i32, 4, bar_h as i32, 0x40FFFFFF);
    }
}

fn render_layout_blocks(
    buf: &mut [u32],
    stride: usize,
    screen_w: i32,
    content_y: i32,
    content_h: i32,
    items: &[crate::parsers::layout::LayoutItem],
    scroll_y: f32,
) {
    use super::draw::{draw_rect, draw_text_noir};

    for item in items {
        match item {
            crate::parsers::layout::LayoutItem::Text(block) => {
                let screen_block_y = block.y - scroll_y + content_y as f32;

                if screen_block_y + block.h < content_y as f32 - 10.0 {
                    continue;
                }
                if screen_block_y > content_y as f32 + content_h as f32 + 10.0 {
                    continue;
                }

                if let Some(bg) = &block.bg_color {
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
            crate::parsers::layout::LayoutItem::Image(img) => {
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

                draw_rect(buf, stride, ix, iy, iw, ih, 0xFF1A1A1E);

                if let Some(cached) = crate::media::get_cached_image(&img.src) {
                    crate::media::draw_image_to_buffer(
                        buf, stride, &cached,
                        ix, iy, iw, ih,
                        screen_w, content_y + content_h,
                    );
                } else {
                    crate::media::draw_placeholder(
                        buf, stride,
                        ix, iy, iw, ih,
                        screen_w, content_y + content_h,
                        true,
                    );
                    let placeholder = if img.alt.is_empty() { "Loading..." } else { &img.alt };
                    draw_text_noir(buf, stride, screen_w, ix + 6, iy + ih / 2 - 4, placeholder, TEXT_DIM, 0.8);
                }
            }
        }
    }
}

pub fn rgba_to_u32(r: f32, g: f32, b: f32, a: f32) -> u32 {
    let ri = (r * 255.0) as u32;
    let gi = (g * 255.0) as u32;
    let bi = (b * 255.0) as u32;
    let ai = (a * 255.0) as u32;
    (ai << 24) | (ri << 16) | (gi << 8) | bi
}
