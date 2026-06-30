//! Navigation - Manejo de navegación y URLs
//!
//! Funciones libres que operan sobre el contexto.

use std::sync::Arc;
use std::sync::Mutex;

use super::context::AppContext;
use crate::network::fetch::HttpFetcher;
use crate::network::url_resolver::url_encode;
use crate::parsers::page_document::PageDocument;

/// Resuelve una URL a su forma completa
/// Ahora usa el URL Resolver RFC 3986 para casos complejos
pub fn resolve_url(input: &str) -> String {
    let trimmed = input.trim();

    // Empty string -> default search
    if trimmed.is_empty() {
        return resolve_search_query("");
    }

    // Si ya es URL absoluta, retornar
    if trimmed.starts_with("http://") || trimmed.starts_with("https://")
        || trimmed.starts_with("file://") || trimmed.starts_with("data:")
        || trimmed.starts_with("about:") {
        return trimmed.to_string();
    }

    // Si parece un dominio (contiene punto, sin espacios), agregar https://
    if trimmed.contains('.') && !trimmed.contains(' ') {
        return format!("https://{}", trimmed);
    }

    // Si tiene path relativo (empieza con /), no podemos resolver sin base URL
    if trimmed.starts_with('/') {
        return format!("https://{}", trimmed.trim_start_matches('/'));
    }

    resolve_search_query(trimmed)
}

fn resolve_search_query(input: &str) -> String {
    let lower = input.to_lowercase();
    let parts: Vec<&str> = lower.splitn(2, ' ').collect();
    let query = if parts.len() > 1 { parts[1] } else { "" };
    let encoded_query = url_encode(query);
    let encoded_input = url_encode(input);

    match parts[0] {
        "yt" | "youtube" if query.trim().is_empty() => "https://www.youtube.com/".to_string(),
        "yt" | "youtube" => format!("https://www.youtube.com/results?search_query={}", encoded_query),
        "gg" | "google" => format!("https://www.google.com/search?q={}", encoded_query),
        "gh" | "github" => format!("https://github.com/search?q={}", encoded_query),
        "ddg" | "duckduckgo" | "duck" if query.trim().is_empty() => "https://html.duckduckgo.com/html/".to_string(),
        "ddg" | "duckduckgo" | "duck" => format!("https://html.duckduckgo.com/html/?q={}", encoded_query),
        "wiki" | "wikipedia" => format!("https://en.wikipedia.org/wiki/Special:Search?search={}", encoded_query),
        "reddit" => format!("https://www.reddit.com/search/?q={}", encoded_query),
        "so" | "stackoverflow" => format!("https://stackoverflow.com/search?q={}", encoded_query),
        "mdn" => format!("https://developer.mozilla.org/en-US/search?q={}", encoded_query),
        "crates" => format!("https://crates.io/search?q={}", encoded_query),
        "docs" | "docsrs" => format!("https://docs.rs/releases/search?query={}", encoded_query),
        "npm" => format!("https://www.npmjs.com/search?q={}", encoded_query),
        _ => format!("https://html.duckduckgo.com/html/?q={}", encoded_input),
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
    let error_holder: Arc<Mutex<Option<crate::app::error_pages::ErrorPage>>> = Arc::new(Mutex::new(None));
    let error_clone = error_holder.clone();

    let fetcher = HttpFetcher::new();

    tokio::spawn(async move {
        match fetcher.get(&url_for_fetch).await {
            Ok(result) => {
                if let Ok(mut guard) = result_clone.lock() {
                    *guard = Some(result.body);
                }
            }
            Err(e) => {
                // FASE A4: Error pages bonitas
                let err_str = format!("{}", e);
                let kind = if err_str.contains("dns") || err_str.contains("resolve") {
                    crate::app::error_pages::ErrorKind::DnsFailure
                } else if err_str.contains("timeout") {
                    crate::app::error_pages::ErrorKind::Timeout
                } else if err_str.contains("refused") {
                    crate::app::error_pages::ErrorKind::ConnectionRefused
                } else if err_str.contains("tls") || err_str.contains("ssl") {
                    crate::app::error_pages::ErrorKind::TlsError
                } else {
                    crate::app::error_pages::ErrorKind::Unknown
                };
                let error_page = crate::app::error_pages::ErrorPage::new(kind, &url_for_fetch)
                    .with_detail(&err_str);
                // Generar HTML de error
                let html = error_page_to_html(&error_page);
                if let Ok(mut guard) = result_clone.lock() {
                    *guard = Some(html);
                }
                if let Ok(mut guard) = error_clone.lock() {
                    *guard = Some(error_page);
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

fn error_page_to_html(error: &crate::app::error_pages::ErrorPage) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head><title>{}</title>
<style>
body {{ font-family: -apple-system, sans-serif; background: #12121a; color: #e0e0e8; padding: 80px; text-align: center; }}
h1 {{ font-size: 24px; margin-bottom: 16px; color: #ff6b6b; }}
.icon {{ font-size: 64px; margin-bottom: 24px; color: #ff6b6b; }}
p {{ font-size: 14px; color: #a0a0a8; margin: 8px 0; }}
.url {{ font-size: 12px; color: #666; font-family: monospace; margin: 16px 0; padding: 8px; background: #1a1a22; border-radius: 4px; }}
button {{ background: #2a2a35; color: #fff; border: 1px solid #3a3a45; padding: 8px 16px; border-radius: 4px; cursor: pointer; margin-top: 16px; }}
</style>
</head>
<body>
<div class="icon">{}</div>
<h1>{}</h1>
<p>{}</p>
<div class="url">{}</div>
<button onclick="location.reload()">Reload</button>
</body>
</html>"#,
        error.kind.title(),
        error.kind.icon(),
        error.kind.title(),
        error.kind.suggestion(),
        error.url,
    )
}

fn parse_and_render_page(ctx: &mut AppContext, html: &str) {
    let url = ctx.tabs[ctx.active_tab].url.clone();
    tracing::info!("Parsing HTML for {} ({} bytes)", url, html.len());
    let mut page = PageDocument::from_html(&url, html);

    // PASO 2: Detectar y cargar CSS externos (FASE A1)
    let mut css_loader = crate::parsers::css_loader::CssLoader::new();
    css_loader.extract_css_links_from_html(&url, html);
    if !css_loader.links.is_empty() {
        tracing::info!("Found {} external CSS files", css_loader.links.len());
        // En la realidad, fetcheamos async. Por ahora solo loggeamos.
        fetch_external_css(&mut css_loader, ctx);
    }
    page.css_loader = Some(css_loader);

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

fn fetch_external_css(loader: &mut crate::parsers::css_loader::CssLoader, _ctx: &AppContext) {
    // PASO 2: Fetch async de CSS externos
    for link in loader.links.clone() {
        if link.loaded { continue; }
        let url = link.resolved_url.clone();
        tokio::spawn(async move {
            // Use HttpFetcher to download CSS
            let fetcher = HttpFetcher::new();
            match fetcher.get(&url).await {
                Ok(result) => {
                    tracing::info!("Fetched CSS: {} ({} bytes)", url, result.body.len());
                    // En la realidad, esto se aplicaria via una cola thread-safe
                    // Por ahora solo loggeamos
                }
                Err(e) => {
                    tracing::warn!("Failed to fetch CSS {}: {}", url, e);
                }
            }
        });
    }
}

fn fetch_page_images(ctx: &AppContext) {
    // PASO 3: Usar Image Loader (FASE A2) para tracking de imagenes
    let mut image_loader = crate::media::image_loader::ImageLoader::new();

    let page_url = ctx.tabs[ctx.active_tab].url.clone();
    let html = String::new();  // En la realidad, hariamos esto cuando parseamos
    image_loader.extract_from_html(&page_url, &html);

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
