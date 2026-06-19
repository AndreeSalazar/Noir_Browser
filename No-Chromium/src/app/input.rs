//! Input - Manejo de input (mouse, teclado)
//!
//! Funciones libres que operan sobre el contexto de la aplicación.

use winit::keyboard::{Key, NamedKey};

use super::context::AppContext;
use super::renderer;
use crate::parsers::layout::hit_test_link;

/// Maneja un click del mouse
pub fn handle_click(ctx: &mut AppContext) {
    let mx = ctx.mouse_x;
    let my = ctx.mouse_y;
    let w = ctx.width as f32;

    // Window controls (title bar)
    if my <= renderer::TITLE_BAR_HEIGHT as f32 {
        handle_title_bar_click(ctx, mx, w);
        return;
    }

    // Tab bar clicks
    if my >= renderer::TITLE_BAR_HEIGHT as f32
        && my <= (renderer::TITLE_BAR_HEIGHT + renderer::TAB_BAR_HEIGHT) as f32
    {
        handle_tab_bar_click(ctx, mx);
        return;
    }

    // Nav bar clicks
    let nav_y = (renderer::TITLE_BAR_HEIGHT + renderer::TAB_BAR_HEIGHT) as f32;
    let btn_h = 32.0;
    let btn_y = nav_y + (renderer::NAV_BAR_HEIGHT as f32 - btn_h) / 2.0;
    let btn_bottom = btn_y + btn_h;

    if my >= btn_y && my <= btn_bottom {
        handle_nav_bar_click(ctx, mx);
        return;
    }

    // Content area click
    let content_top = renderer::TOOLBAR_HEIGHT as f32;
    if my > content_top {
        handle_content_click(ctx, mx, my);
        return;
    }

    ctx.url_focused = false;
}

fn handle_title_bar_click(ctx: &mut AppContext, mx: f32, w: f32) {
    let ctrl_w = 46.0f32;

    let close_x = w - ctrl_w;
    if mx >= close_x {
        ctx.should_close = true;
        return;
    }

    let max_x = w - ctrl_w * 2.0;
    if mx >= max_x && mx < close_x {
        if let Some(window) = &ctx.window {
            ctx.is_maximized = !ctx.is_maximized;
            window.set_maximized(ctx.is_maximized);
        }
        return;
    }

    let min_x = w - ctrl_w * 3.0;
    if mx >= min_x && mx < max_x {
        if let Some(window) = &ctx.window {
            window.set_minimized(true);
        }
        return;
    }

    if mx < min_x {
        if let Some(window) = &ctx.window {
            let _ = window.drag_window();
        }
    }
}

fn handle_tab_bar_click(ctx: &mut AppContext, mx: f32) {
    let mut tx = 6.0f32;
    for i in 0..ctx.tabs.len() {
        let tab_w = renderer::TAB_WIDTH as f32;
        if mx >= tx && mx <= tx + tab_w {
            ctx.switch_tab(i);
            return;
        }
        tx += tab_w + renderer::TAB_SPACING as f32;
    }
    if mx >= tx && mx <= tx + 28.0 {
        ctx.new_tab();
    }
}

fn handle_nav_bar_click(ctx: &mut AppContext, mx: f32) {
    let mut bx = renderer::NAV_START_X as f32;

    if click_back_button(ctx, mx, bx) { return; }
    bx += renderer::NAV_BTN_SIZE as f32 + renderer::NAV_BTN_SPACING as f32;

    if click_forward_button(ctx, mx, bx) { return; }
    bx += renderer::NAV_BTN_SIZE as f32 + renderer::NAV_BTN_SPACING as f32;

    if click_reload_button(ctx, mx, bx) { return; }
    bx += renderer::NAV_BTN_SIZE as f32 + renderer::NAV_BTN_SPACING as f32;

    if click_home_button(ctx, mx, bx) { return; }
    bx += renderer::NAV_BTN_SIZE as f32 + 12.0;

    if mx >= bx && mx <= ctx.width as f32 {
        ctx.url_focused = true;
        ctx.url_cursor = ctx.url_bar.len();
    }
}

fn click_back_button(ctx: &mut AppContext, mx: f32, bx: f32) -> bool {
    if mx >= bx && mx <= bx + renderer::NAV_BTN_SIZE as f32 {
        if ctx.history_index > 0 {
            ctx.history_index -= 1;
            let url = ctx.history[ctx.history_index].clone();
            ctx.url_bar = url.clone();
            ctx.url_cursor = ctx.url_bar.len();
            ctx.navigate(url);
        }
        true
    } else {
        false
    }
}

fn click_forward_button(ctx: &mut AppContext, mx: f32, bx: f32) -> bool {
    if mx >= bx && mx <= bx + renderer::NAV_BTN_SIZE as f32 {
        if ctx.history_index < ctx.history.len().saturating_sub(1) {
            ctx.history_index += 1;
            let url = ctx.history[ctx.history_index].clone();
            ctx.url_bar = url.clone();
            ctx.url_cursor = ctx.url_bar.len();
            ctx.navigate(url);
        }
        true
    } else {
        false
    }
}

fn click_reload_button(ctx: &mut AppContext, mx: f32, bx: f32) -> bool {
    if mx >= bx && mx <= bx + renderer::NAV_BTN_SIZE as f32 {
        let url = ctx.tabs[ctx.active_tab].url.clone();
        if !url.is_empty() {
            ctx.navigate(url);
        }
        true
    } else {
        false
    }
}

fn click_home_button(ctx: &mut AppContext, mx: f32, bx: f32) -> bool {
    if mx >= bx && mx <= bx + renderer::NAV_BTN_SIZE as f32 {
        ctx.go_home();
        true
    } else {
        false
    }
}

fn handle_content_click(ctx: &mut AppContext, mx: f32, my: f32) {
    let layout_blocks = ctx.tabs[ctx.active_tab].layout_blocks.clone();
    let scroll_y = ctx.tabs[ctx.active_tab].scroll_y;
    if let Some(href) = hit_test_link(&layout_blocks, mx, my, scroll_y) {
        tracing::info!("Link clicked: {}", href);
        ctx.url_bar = href.clone();
        ctx.url_cursor = ctx.url_bar.len();
        ctx.navigate(href);
        if let Some(window) = &ctx.window {
            window.request_redraw();
        }
    }
}

/// Maneja una tecla presionada
pub fn handle_key(ctx: &mut AppContext, key: &Key, ctrl: bool) {
    if ctrl {
        handle_ctrl_key(ctx, key);
        return;
    }

    if !ctx.url_focused {
        handle_unfocused_key(ctx, key);
        return;
    }

    handle_url_input_key(ctx, key);
}

fn handle_ctrl_key(ctx: &mut AppContext, key: &Key) {
    if let Key::Character(c) = key {
        match c.as_str() {
            "t" | "T" => ctx.new_tab(),
            "w" | "W" => ctx.close_current_tab(),
            "l" | "L" => {
                ctx.url_focused = true;
                ctx.url_cursor = ctx.url_bar.len();
            }
            "r" | "R" => {
                let url = ctx.tabs[ctx.active_tab].url.clone();
                if !url.is_empty() {
                    ctx.navigate(url);
                }
            }
            "d" | "D" => ctx.new_tab(),
            "f" | "F" => {
                ctx.find_open = !ctx.find_open;
                if ctx.find_open {
                    ctx.find_query.clear();
                }
            }
            _ => {}
        }
    } else if let Key::Named(NamedKey::Tab) = key {
        if ctx.tabs.len() > 1 {
            let next = (ctx.active_tab + 1) % ctx.tabs.len();
            ctx.switch_tab(next);
        }
    } else if let Key::Named(NamedKey::F12) = key {
        ctx.console_open = !ctx.console_open;
    } else if let Key::Named(NamedKey::F1) = key {
        ctx.shortcuts_open = !ctx.shortcuts_open;
    }
}

fn handle_unfocused_key(ctx: &mut AppContext, key: &Key) {
    if let Key::Named(NamedKey::F5) = key {
        let url = ctx.tabs[ctx.active_tab].url.clone();
        if !url.is_empty() {
            ctx.navigate(url);
        }
    } else if let Key::Named(NamedKey::F11) = key {
        ctx.is_maximized = !ctx.is_maximized;
        if let Some(window) = &ctx.window {
            window.set_maximized(ctx.is_maximized);
        }
    } else if let Key::Named(NamedKey::F12) = key {
        ctx.console_open = !ctx.console_open;
    } else if let Key::Named(NamedKey::F1) = key {
        ctx.shortcuts_open = !ctx.shortcuts_open;
    } else if let Key::Named(NamedKey::Escape) = key {
        ctx.console_open = false;
        ctx.shortcuts_open = false;
        ctx.find_open = false;
    }
}

fn handle_url_input_key(ctx: &mut AppContext, key: &Key) {
    match key {
        Key::Named(NamedKey::Backspace) => {
            if ctx.url_cursor > 0 {
                ctx.url_cursor -= 1;
                ctx.url_bar.remove(ctx.url_cursor);
            }
        }
        Key::Named(NamedKey::Delete) => {
            if ctx.url_cursor < ctx.url_bar.len() {
                ctx.url_bar.remove(ctx.url_cursor);
            }
        }
        Key::Named(NamedKey::ArrowLeft) => {
            ctx.url_cursor = ctx.url_cursor.saturating_sub(1);
        }
        Key::Named(NamedKey::ArrowRight) => {
            if ctx.url_cursor < ctx.url_bar.len() {
                ctx.url_cursor += 1;
            }
        }
        Key::Named(NamedKey::Home) => ctx.url_cursor = 0,
        Key::Named(NamedKey::End) => ctx.url_cursor = ctx.url_bar.len(),
        Key::Named(NamedKey::Enter) => handle_url_submit(ctx),
        Key::Named(NamedKey::Escape) => {
            ctx.url_focused = false;
            ctx.find_open = false;
            ctx.console_open = false;
            ctx.shortcuts_open = false;
        }
        Key::Character(c) => {
            let ch = c.as_str();
            if !ch.is_empty() {
                for chr in ch.chars() {
                    ctx.url_bar.insert(ctx.url_cursor, chr);
                    ctx.url_cursor += 1;
                }
            }
        }
        _ => {}
    }
}

fn handle_url_submit(ctx: &mut AppContext) {
    if !ctx.url_bar.is_empty() {
        let url = ctx.resolve_url();
        ctx.navigate(url);
        ctx.url_focused = false;
    }
}
