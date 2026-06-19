pub mod config;
pub mod draw;
pub mod glyphs;
pub mod state;
pub mod theme;

mod input;
mod navigation;
mod renderer;

use anyhow::Result;
use std::num::NonZeroU32;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{WindowAttributes, WindowId};

pub use config::AppConfig;
use state::NoirApp;

use crate::network::fetch::HttpFetcher;
use crate::parsers::page_document::PageDocument;
use crate::parsers::layout::{layout_page, total_content_height};

impl ApplicationHandler for NoirApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let attrs = WindowAttributes::default()
            .with_title("Noir Browser")
            .with_inner_size(LogicalSize::new(1280.0, 720.0))
            .with_min_inner_size(LogicalSize::new(800.0, 500.0))
            .with_decorations(false);

        let window = Rc::new(event_loop.create_window(attrs).unwrap());
        let size = window.inner_size();
        self.width = size.width;
        self.height = size.height;

        let context = softbuffer::Context::new(Rc::clone(&window)).unwrap();
        let surface = softbuffer::Surface::new(&context, Rc::clone(&window)).unwrap();

        self.window = Some(window);
        self.surface = Some(surface);

        tracing::info!("Window created: {}x{}", self.width, self.height);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                tracing::info!("Close requested, shutting down...");
                event_loop.exit();
            }

            WindowEvent::RedrawRequested => {
                self.draw_frame();
            }

            WindowEvent::Resized(size) => {
                if size.width > 0 && size.height > 0 {
                    self.width = size.width;
                    self.height = size.height;
                    if let Some(surface) = &mut self.surface {
                        surface.resize(
                            NonZeroU32::new(size.width).unwrap(),
                            NonZeroU32::new(size.height).unwrap(),
                        )
                        .unwrap();
                    }
                    self.window.as_ref().unwrap().request_redraw();
                }
            }

            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_x = position.x as f32;
                self.mouse_y = position.y as f32;
            }

            WindowEvent::MouseInput { state: ElementState::Pressed, .. } => {
                self.handle_click();
                self.window.as_ref().unwrap().request_redraw();
            }

            WindowEvent::MouseWheel { delta, .. } => {
                let scroll_amount = match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => y * 60.0,
                    winit::event::MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
                };
                self.tabs[self.active_tab].scroll_y -= scroll_amount;
                self.tabs[self.active_tab].scroll_y = self.tabs[self.active_tab].scroll_y.max(0.0);
                self.window.as_ref().unwrap().request_redraw();
            }

            WindowEvent::KeyboardInput {
                event: KeyEvent { logical_key, state: ElementState::Pressed, .. },
                ..
            } => {
                let ctrl = self.modifiers.control_key();
                self.handle_key(&logical_key, ctrl);
                self.window.as_ref().unwrap().request_redraw();
            }

            WindowEvent::ModifiersChanged(new_mods) => {
                self.modifiers = new_mods.state();
            }

            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self.should_close {
            event_loop.exit();
        }

        if self.fetching {
            self.process_fetch_result();
        }

        self.process_pending_timers();
        self.process_image_dirty();
    }
}

impl NoirApp {
    fn process_fetch_result(&mut self) {
        let html_opt: Option<String> = if let Some(result) = &self.fetch_result {
            result.try_lock().ok().and_then(|g| g.clone())
        } else {
            None
        };

        if let Some(html) = html_opt {
            self.parse_and_render_page(&html);
            self.fetching = false;
            self.fetch_result = None;
            if let Some(window) = &self.window {
                window.request_redraw();
            }
        }
    }

    fn parse_and_render_page(&mut self, html: &str) {
        let url = self.tabs[self.active_tab].url.clone();
        tracing::info!("Parsing HTML for {} ({} bytes)", url, html.len());
        let mut page = PageDocument::from_html(&url, html);

        if !page.css_urls.is_empty() {
            self.fetch_external_css(&mut page);
        }

        let title = page.title.clone();
        let num_links = page.links.len();

        let nodes = crate::parsers::dom_tree::parse_html(html);
        crate::js_engine::dom_sync::sync_dom_to_js_engine(&nodes);

        let scripts = crate::js_engine::dom_sync::extract_inline_scripts(&nodes);
        let tab_id = self.tabs[self.active_tab].tab_id;
        for (i, script) in scripts.iter().enumerate() {
            tracing::info!("Executing inline script #{} ({} bytes)", i + 1, script.len());
            match self.tabs[self.active_tab].js_engine.eval_script(tab_id, script) {
                Ok(result) => {
                    if !result.is_empty() && result != "undefined" {
                        tracing::info!("Script result: {}", result);
                    }
                }
                Err(e) => {
                    tracing::warn!("Script error: {}", e);
                }
            }
        }

        if crate::js_engine::dom_bridge::take_mutated_flag() {
            tracing::info!("DOM mutated by JS, rebuilding layout");
            crate::js_engine::dom_sync::rebuild_page_from_dom(&mut page);
        }

        let viewport_w = self.width as f32;
        let blocks = layout_page(&page, viewport_w);
        let content_h = total_content_height(&blocks);

        tracing::info!(
            "Parsed: title='{}', {} links, {} layout blocks, content height: {:.0}",
            title, num_links, blocks.len(), content_h
        );

        self.tabs[self.active_tab].page = Some(page);
        self.tabs[self.active_tab].layout_blocks = blocks;
        self.tabs[self.active_tab].content_height = content_h;
        self.tabs[self.active_tab].title = if !title.is_empty() { title } else { url };

        self.fetch_page_images();
    }

    fn fetch_external_css(&self, page: &mut PageDocument) {
        tracing::info!("Fetching {} external CSS files", page.css_urls.len());
        for css_url in &page.css_urls {
            let css_url_clone = css_url.clone();
            let result_holder: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
            let result_clone = result_holder.clone();
            tokio::spawn(async move {
                let client = reqwest::Client::builder()
                    .timeout(std::time::Duration::from_secs(5))
                    .build()
                    .unwrap_or_default();
                let css = match client.get(&css_url_clone).send().await {
                    Ok(resp) => match resp.text().await {
                        Ok(c) => Some(c),
                        Err(_) => None,
                    },
                    Err(_) => None,
                };
                if let Ok(mut guard) = result_clone.lock() {
                    *guard = css;
                }
            });
            if let Ok(guard) = result_holder.lock() {
                if let Some(css) = guard.as_ref() {
                    tracing::info!("Loaded CSS from {} ({} bytes)", css_url, css.len());
                    page.style_blocks.push(css.clone());
                }
            };
        }
    }

    fn fetch_page_images(&self) {
        let imgs_to_fetch: Vec<String> = self.tabs[self.active_tab].layout_blocks.iter().filter_map(|item| {
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
            tracing::info!("Queueing {} images for async fetch", imgs_to_fetch.len());
            for img_url in imgs_to_fetch {
                tokio::spawn(async move {
                    crate::media::fetch_image(&img_url).await;
                });
            }
        }
    }

    fn process_pending_timers(&mut self) {
        let pending = crate::js_engine::bindings::get_pending_timers();
        if pending.is_empty() { return; }

        let tab_id = self.tabs[self.active_tab].tab_id;
        for timer in pending {
            let callback_name = format!("__callback_{}", timer.callback_id);
            let _ = self.tabs[self.active_tab].js_engine.eval_script(tab_id, &format!(
                "if (typeof {} === 'function') {}();", callback_name, callback_name
            ));
        }
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    fn process_image_dirty(&self) {
        if crate::media::take_image_dirty() {
            if let Some(window) = &self.window {
                window.request_redraw();
            }
        }
    }
}

pub async fn run(config: AppConfig) -> Result<()> {
    tracing::info!(
        "Starting Noir Browser window (model: {:?})",
        config.process_model
    );

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);

    let mut app = NoirApp::new();

    event_loop.run_app(&mut app)?;

    Ok(())
}
