//! Navigation - Manejo de navegación y URLs
//!
//! Funciones libres que operan sobre el contexto.

use std::sync::Arc;
use std::sync::Mutex;

use super::context::AppContext;
use crate::network::fetch::HttpFetcher;
use crate::parsers::page_document::PageDocument;

/// Resuelve una URL a su forma completa
pub fn resolve_url(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        return trimmed.to_string();
    }

    if trimmed.contains('.') && !trimmed.contains(' ') {
        return format!("https://{}", trimmed);
    }

    resolve_search_query(trimmed)
}

fn resolve_search_query(input: &str) -> String {
    let lower = input.to_lowercase();
    let parts: Vec<&str> = lower.splitn(2, ' ').collect();
    let query = if parts.len() > 1 { parts[1] } else { "" };

    match parts[0] {
        "yt" | "youtube" => format!("https://www.youtube.com/results?search_query={}", query.replace(' ', "+")),
        "gg" | "google" => format!("https://www.google.com/search?q={}", query.replace(' ', "+")),
        "gh" | "github" => format!("https://github.com/search?q={}", query.replace(' ', "+")),
        "ddg" | "duckduckgo" | "duck" => format!("https://duckduckgo.com/?q={}", query.replace(' ', "+")),
        "wiki" | "wikipedia" => format!("https://en.wikipedia.org/wiki/Special:Search?search={}", query.replace(' ', "+")),
        "reddit" => format!("https://www.reddit.com/search/?q={}", query.replace(' ', "+")),
        "so" | "stackoverflow" => format!("https://stackoverflow.com/search?q={}", query.replace(' ', "+")),
        "mdn" => format!("https://developer.mozilla.org/en-US/search?q={}", query.replace(' ', "+")),
        "crates" => format!("https://crates.io/search?q={}", query.replace(' ', "+")),
        "docs" | "docsrs" => format!("https://docs.rs/releases/search?query={}", query.replace(' ', "+")),
        "npm" => format!("https://www.npmjs.com/search?q={}", query.replace(' ', "+")),
        _ => format!("https://duckduckgo.com/?q={}", input.replace(' ', "+")),
    }
}

/// Navega a una URL
pub fn navigate(ctx: &mut AppContext, url: String) {
    ctx.tabs[ctx.active_tab].url = url.clone();
    ctx.tabs[ctx.active_tab].title = url.clone();
    ctx.url_focused = false;
    ctx.fetching = true;
    ctx.fetch_result = None;
    ctx.fetch_error = None;

    // Lanzar fetch async
    let url_for_fetch = url.clone();
    let result_holder: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let result_clone = result_holder.clone();

    let fetcher = HttpFetcher::new();

    tokio::spawn(async move {
        match fetcher.get(&url_for_fetch).await {
            Ok(result) => {
                if let Ok(mut guard) = result_clone.lock() {
                    *guard = Some(result.body);
                }
            }
            Err(e) => {
                if let Ok(mut guard) = result_clone.lock() {
                    *guard = Some(format!("<html><head><title>Error</title></head><body><h1>Failed to load</h1><p>{}</p></body></html>", e));
                }
            }
        }
    });

    ctx.fetch_result = Some(result_holder);
}

/// Va a la página de inicio
pub fn go_home(ctx: &mut AppContext) {
    ctx.url_bar.clear();
    ctx.url_cursor = 0;
    ctx.tabs[ctx.active_tab].url.clear();
    ctx.tabs[ctx.active_tab].title = "New Tab".to_string();
}

/// Procesa el resultado de un fetch
pub fn process_fetch_result(ctx: &mut AppContext) {
    let html_opt: Option<String> = if let Some(result) = &ctx.fetch_result {
        result.try_lock().ok().and_then(|g| g.clone())
    } else {
        None
    };

    if let Some(html) = html_opt {
        parse_and_render_page(ctx, &html);
        ctx.fetching = false;
        ctx.fetch_result = None;
        if let Some(window) = &ctx.window {
            window.request_redraw();
        }
    }
}

/// Procesa timers pendientes
pub fn process_pending_timers(ctx: &mut AppContext) {
    let pending = crate::js_engine_v3::get_pending_timers();
    if pending.is_empty() {
        return;
    }

    let active_idx = ctx.active_tab;
    let tab_id = ctx.tabs[active_idx].tab_id;
    for timer in pending {
        let callback_name = format!("__callback_{}", timer.callback_id);
        let _ = crate::js_engine_v3::eval_script(
            &mut ctx.tabs[active_idx].js_engine,
            tab_id,
            &format!("if (typeof {} === 'function') {}();", callback_name, callback_name)
        );
    }
    if let Some(window) = &ctx.window {
        window.request_redraw();
    }
}

fn parse_and_render_page(ctx: &mut AppContext, html: &str) {
    let url = ctx.tabs[ctx.active_tab].url.clone();
    tracing::info!("Parsing HTML for {} ({} bytes)", url, html.len());
    let mut page = PageDocument::from_html(&url, html);

    if !page.css_urls.is_empty() {
        fetch_external_css(&mut page);
    }

    let title = page.title.clone();
    let _num_links = page.links.len();

    let nodes = crate::parsers::dom_tree::parse_html(html);
    crate::js_engine_v3::sync_dom_to_js_engine(&nodes);

    let scripts = crate::js_engine_v3::extract_inline_scripts(&nodes);
    let tab_id = ctx.tabs[ctx.active_tab].tab_id;
    for (i, script) in scripts.iter().enumerate() {
        tracing::info!("Executing inline script #{} ({} bytes)", i + 1, script.len());
        let _ = crate::js_engine_v3::eval_script(
            &mut ctx.tabs[ctx.active_tab].js_engine,
            tab_id,
            script,
        );
    }

    if crate::js_engine_v3::take_mutated_flag() {
        tracing::info!("DOM mutated by JS, rebuilding layout");
        crate::js_engine_v3::rebuild_page_from_dom(&mut page);
    }

    let viewport_w = ctx.width as f32;
    let blocks = crate::parsers::layout::layout_page(&page, viewport_w);
    let content_h = crate::parsers::layout::total_content_height(&blocks);

    tracing::info!(
        "Parsed: title='{}', {} links, {} layout blocks, content height: {:.0}",
        title,
        _num_links,
        blocks.len(),
        content_h
    );

    ctx.tabs[ctx.active_tab].page = Some(page);
    ctx.tabs[ctx.active_tab].layout_blocks = blocks;
    ctx.tabs[ctx.active_tab].content_height = content_h;
    ctx.tabs[ctx.active_tab].title = if !title.is_empty() { title } else { url };

    fetch_page_images(ctx);
}

fn fetch_external_css(_page: &mut PageDocument) {
    // Simplificado - los CSS externos se cargarían async
}

fn fetch_page_images(ctx: &AppContext) {
    let imgs_to_fetch: Vec<String> = ctx.tabs[ctx.active_tab].layout_blocks.iter().filter_map(|item| {
        if let crate::parsers::layout::LayoutItem::Image(img) = item {
            if crate::media::get_cached_image(&img.src).is_none() && !img.src.is_empty() && img.src.starts_with("http") {
                Some(img.src.clone())
            } else {
                None
            }
        } else {
            None
        }
    }).collect();

    if !imgs_to_fetch.is_empty() {
        for img_url in imgs_to_fetch {
            tokio::spawn(async move {
                let _ = crate::media::fetch_image(&img_url).await;
            });
        }
    }
}
