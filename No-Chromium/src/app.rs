use winit::{
    event::{
        ElementState, Event, MouseButton, MouseScrollDelta, StartCause, VirtualKeyCode, WindowEvent,
    },
    event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy},
    window::{Window, WindowBuilder},
};

use crate::browser::{load_page_document, BrowserState, PageDocument, PageClickResult};
use crate::render::quality::QualityProfile;
use crate::render::text::{RasterizedAtlas, TextRequest};
use crate::vulkan_engine::renderer::renderer_2d::Renderer2D;
use crate::vulkan_engine::context::VulkanContext;

const INITIAL_URL: &str = "https://www.google.com";

#[derive(Clone, Debug)]
pub enum BrowserEvent {
    PageLoaded { url: String, document: PageDocument },
    ImageLoaded { url: String },
}

static EVENT_PROXY: std::sync::OnceLock<winit::event_loop::EventLoopProxy<BrowserEvent>> = std::sync::OnceLock::new();

pub fn get_event_proxy() -> Option<winit::event_loop::EventLoopProxy<BrowserEvent>> {
    EVENT_PROXY.get().cloned()
}

pub fn set_event_proxy(proxy: winit::event_loop::EventLoopProxy<BrowserEvent>) {
    let _ = EVENT_PROXY.set(proxy);
}

pub fn run() {
    let event_loop = EventLoopBuilder::<BrowserEvent>::with_user_event().build();
    let window = WindowBuilder::new()
        .with_title("No-Chromium | Sovereign GPU Engine (Vulkan 1.3)")
        .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0))
        .with_decorations(false)
        .build(&event_loop)
        .unwrap();

    println!("========================================");
    println!("     NO-CHROMIUM: AWAKENING THE GPU     ");
    println!("========================================");

    let quality = QualityProfile::ultra_native(window.scale_factor() as f32);
    let mut browser = BrowserState::new(INITIAL_URL);
    let mut pending_atlas: Option<RasterizedAtlas> = None;
    let mut vk_ctx: Option<VulkanContext> = None;
    let mut renderer: Option<Renderer2D> = None;
    let mut cursor_pos = winit::dpi::PhysicalPosition::new(0.0, 0.0);
    let mut address_focused = false;
    let mut address_input = INITIAL_URL.to_string();
    let event_proxy = event_loop.create_proxy();
    set_event_proxy(event_proxy.clone());
    let rt = tokio::runtime::Runtime::new().unwrap();
    let _enter_guard = rt.enter();

    // Trigger startup pre-caching of images
    crate::media::image_manager::pre_cache_resources(event_proxy.clone());

    spawn_page_load(rt.handle().clone(), event_proxy.clone(), INITIAL_URL.to_string());
    window.request_redraw();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::NewEvents(StartCause::Init) => {
                ensure_gpu_ready(
                    &window,
                    &mut vk_ctx,
                    &mut renderer,
                    &mut pending_atlas,
                    quality,
                    browser.current_url(),
                );
            }
            Event::UserEvent(BrowserEvent::PageLoaded { url, document }) => {
                let win_size = window.inner_size();
                if let Some(new_atlas) = browser.accept_loaded_document(
                    url,
                    document,
                    quality.text_rasterization_options(),
                    win_size.width as f32,
                    win_size.height as f32,
                ) {
                    if !address_focused {
                        address_input = browser.current_url().to_string();
                    }
                    apply_atlas(
                        &mut renderer,
                        &mut pending_atlas,
                        vk_ctx.as_ref(),
                        new_atlas,
                    );
                    window.request_redraw();
                }
            }
            Event::UserEvent(BrowserEvent::ImageLoaded { url }) => {
                println!("[App Loop] Image loaded and decoded: {}", url);
                let win_size = window.inner_size();
                if let Some(new_atlas) = browser.rerender_current_page(
                    quality.text_rasterization_options(),
                    win_size.width as f32,
                    win_size.height as f32,
                ) {
                    apply_atlas(
                        &mut renderer,
                        &mut pending_atlas,
                        vk_ctx.as_ref(),
                        new_atlas,
                    );
                    window.request_redraw();
                }
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                shutdown(&mut renderer, &mut vk_ctx, control_flow);
            }
            Event::WindowEvent {
                event: WindowEvent::CursorMoved { position, .. },
                ..
            } => {
                cursor_pos = position;
            }
            Event::WindowEvent {
                event: WindowEvent::ReceivedCharacter(ch),
                ..
            } => {
                if address_focused && !ch.is_control() {
                    address_input.push(ch);
                    update_address_preview(
                        &mut browser,
                        &mut renderer,
                        &mut pending_atlas,
                        vk_ctx.as_ref(),
                        &window,
                        quality,
                        &address_input,
                    );
                } else if !ch.is_control() {
                    if browser.handle_page_char(ch) {
                        let current_url = browser.current_url().to_string();
                        let atlas = browser.rerender_current_page(
                            quality.text_rasterization_options(),
                            window.inner_size().width as f32,
                            window.inner_size().height as f32,
                        ).unwrap_or_else(|| {
                            loading_atlas(&current_url, quality.text_rasterization_options(), window.inner_size().width as f32)
                        });
                        apply_atlas(&mut renderer, &mut pending_atlas, vk_ctx.as_ref(), atlas);
                        window.request_redraw();
                    }
                }
            }
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput { input, .. },
                ..
            } => {
                if input.state == ElementState::Pressed {
                    let ctrl = input.modifiers.ctrl();
                    match input.virtual_keycode {
                        Some(VirtualKeyCode::T) if ctrl => {
                            address_focused = false;
                            browser.open_tab(INITIAL_URL);
                            address_input = INITIAL_URL.to_string();
                            begin_navigation(
                                &mut browser,
                                rt.handle().clone(),
                                &event_proxy,
                                &mut renderer,
                                &mut pending_atlas,
                                vk_ctx.as_ref(),
                                &window,
                                quality,
                                INITIAL_URL,
                            );
                            return;
                        }
                        Some(VirtualKeyCode::W) if ctrl => {
                            address_focused = false;
                            let i = browser.active_tab_index;
                            browser.close_tab(i);
                            address_input = browser.current_url().to_string();
                            let current_url = browser.current_url().to_string();
                            let atlas = if let Some(new_atlas) = browser.rerender_current_page(
                                quality.text_rasterization_options(),
                                window.inner_size().width as f32,
                                window.inner_size().height as f32,
                            ) {
                                new_atlas
                            } else {
                                if browser.current_tab().document.is_none() {
                                    spawn_page_load(rt.handle().clone(), event_proxy.clone(), current_url.clone());
                                }
                                loading_atlas(
                                    &current_url,
                                    quality.text_rasterization_options(),
                                    window.inner_size().width as f32,
                                )
                            };
                            apply_atlas(&mut renderer, &mut pending_atlas, vk_ctx.as_ref(), atlas);
                            window.request_redraw();
                            return;
                        }
                        Some(VirtualKeyCode::Tab) if ctrl => {
                            address_focused = false;
                            let next_idx = (browser.active_tab_index + 1) % browser.tabs.len();
                            browser.switch_tab(next_idx);
                            address_input = browser.current_url().to_string();
                            let current_url = browser.current_url().to_string();
                            let atlas = if let Some(new_atlas) = browser.rerender_current_page(
                                quality.text_rasterization_options(),
                                window.inner_size().width as f32,
                                window.inner_size().height as f32,
                            ) {
                                new_atlas
                            } else {
                                loading_atlas(
                                    &current_url,
                                    quality.text_rasterization_options(),
                                    window.inner_size().width as f32,
                                )
                            };
                            apply_atlas(&mut renderer, &mut pending_atlas, vk_ctx.as_ref(), atlas);
                            window.request_redraw();
                            return;
                        }
                        _ => {}
                    }
                }

                if address_focused && input.state == ElementState::Pressed {
                    match input.virtual_keycode {
                        Some(VirtualKeyCode::Back) => {
                            address_input.pop();
                            update_address_preview(
                                &mut browser,
                                &mut renderer,
                                &mut pending_atlas,
                                vk_ctx.as_ref(),
                                &window,
                                quality,
                                &address_input,
                            );
                        }
                        Some(VirtualKeyCode::Return) => {
                            if let Some(url) = normalize_address_input(&address_input) {
                                address_focused = false;
                                address_input = url.clone();
                                begin_navigation(
                                    &mut browser,
                                    rt.handle().clone(),
                                    &event_proxy,
                                    &mut renderer,
                                    &mut pending_atlas,
                                    vk_ctx.as_ref(),
                                    &window,
                                    quality,
                                    &address_input,
                                );
                            }
                        }
                        Some(VirtualKeyCode::Escape) => {
                            address_focused = false;
                            address_input = browser.current_url().to_string();
                            update_address_preview(
                                &mut browser,
                                &mut renderer,
                                &mut pending_atlas,
                                vk_ctx.as_ref(),
                                &window,
                                quality,
                                &address_input,
                            );
                        }
                        _ => {}
                    }
                } else if input.state == ElementState::Pressed {
                    match input.virtual_keycode {
                        Some(VirtualKeyCode::Back) => {
                            if browser.handle_page_backspace() {
                                let current_url = browser.current_url().to_string();
                                let atlas = browser.rerender_current_page(
                                    quality.text_rasterization_options(),
                                    window.inner_size().width as f32,
                                    window.inner_size().height as f32,
                                ).unwrap_or_else(|| {
                                    loading_atlas(&current_url, quality.text_rasterization_options(), window.inner_size().width as f32)
                                });
                                apply_atlas(&mut renderer, &mut pending_atlas, vk_ctx.as_ref(), atlas);
                                window.request_redraw();
                            }
                        }
                        Some(VirtualKeyCode::Return) => {
                            if let Some(submit_url) = browser.handle_page_return() {
                                println!("[Browser] Form submitted via Enter to {}", submit_url);
                                address_input = submit_url.clone();
                                begin_navigation(
                                    &mut browser,
                                    rt.handle().clone(),
                                    &event_proxy,
                                    &mut renderer,
                                    &mut pending_atlas,
                                    vk_ctx.as_ref(),
                                    &window,
                                    quality,
                                    &submit_url,
                                );
                            }
                        }
                        Some(VirtualKeyCode::Escape) => {
                            // Blur the focused input
                            browser.current_tab_mut().focused_input_idx = None;
                            let current_url = browser.current_url().to_string();
                            let atlas = browser.rerender_current_page(
                                quality.text_rasterization_options(),
                                window.inner_size().width as f32,
                                window.inner_size().height as f32,
                                ).unwrap_or_else(|| {
                                    loading_atlas(&current_url, quality.text_rasterization_options(), window.inner_size().width as f32)
                                });
                            apply_atlas(&mut renderer, &mut pending_atlas, vk_ctx.as_ref(), atlas);
                            window.request_redraw();
                        }
                        _ => {}
                    }
                }
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(new_size),
                ..
            } => {
                let Some(ctx) = vk_ctx.as_mut() else {
                    return;
                };

                if let Some(mut r) = renderer.take() {
                    r.cleanup(&ctx.device.device);
                }
                ctx.recreate_swapchain(new_size.width, new_size.height);
                let new_atlas = browser
                    .rerender_current_page(
                        quality.text_rasterization_options(),
                        new_size.width as f32,
                        new_size.height as f32,
                    )
                    .unwrap_or_else(|| {
                        loading_atlas(
                            browser.current_url(),
                            quality.text_rasterization_options(),
                            new_size.width as f32,
                        )
                    });
                renderer = Some(Renderer2D::new(ctx, new_atlas, quality));
            }
            Event::WindowEvent {
                event: WindowEvent::MouseWheel { delta, .. },
                ..
            } => {
                let scroll_delta = match delta {
                    MouseScrollDelta::LineDelta(_, y) => -y * 72.0,
                    MouseScrollDelta::PixelDelta(pos) => -pos.y as f32,
                };

                let win_size = window.inner_size();
                if let Some(new_atlas) = browser.scroll_by(
                    scroll_delta,
                    quality.text_rasterization_options(),
                    win_size.width as f32,
                    win_size.height as f32,
                ) {
                    apply_atlas(
                        &mut renderer,
                        &mut pending_atlas,
                        vk_ctx.as_ref(),
                        new_atlas,
                    );
                    window.request_redraw();
                }
            }
            Event::WindowEvent {
                event:
                    WindowEvent::MouseInput {
                        state: ElementState::Pressed,
                        button: MouseButton::Left,
                        ..
                    },
                ..
            } => {
                let win_size = window.inner_size();
                let scale_factor = window.scale_factor() as f32;
                let hitboxes = crate::ui::ui_gen::get_ui_hitboxes(
                    win_size.width as f32,
                    win_size.height as f32,
                    browser.tabs.len(),
                    scale_factor,
                );
                let mut hit_button = false;

                for hb in hitboxes {
                    if cursor_pos.x >= hb.x_min as f64
                        && cursor_pos.x <= hb.x_max as f64
                        && cursor_pos.y >= hb.y_min as f64
                        && cursor_pos.y <= hb.y_max as f64
                    {
                        hit_button = true;
                        match hb.button {
                            crate::ui::ui_gen::UIButton::Back => {
                                address_focused = false;
                                if let Some(url) = browser.go_back() {
                                    address_input = url.clone();
                                    begin_pending_load(
                                        rt.handle().clone(),
                                        &event_proxy,
                                        &mut renderer,
                                        &mut pending_atlas,
                                        vk_ctx.as_ref(),
                                        &window,
                                        quality,
                                        &url,
                                    );
                                }
                            }
                            crate::ui::ui_gen::UIButton::Forward => {
                                address_focused = false;
                                if let Some(url) = browser.go_forward() {
                                    address_input = url.clone();
                                    begin_pending_load(
                                        rt.handle().clone(),
                                        &event_proxy,
                                        &mut renderer,
                                        &mut pending_atlas,
                                        vk_ctx.as_ref(),
                                        &window,
                                        quality,
                                        &url,
                                    );
                                }
                            }
                            crate::ui::ui_gen::UIButton::Reload => {
                                address_focused = false;
                                let url = browser.reload();
                                address_input = url.clone();
                                begin_pending_load(
                                    rt.handle().clone(),
                                    &event_proxy,
                                    &mut renderer,
                                    &mut pending_atlas,
                                    vk_ctx.as_ref(),
                                    &window,
                                    quality,
                                    &url,
                                );
                            }
                            crate::ui::ui_gen::UIButton::Home => {
                                address_focused = false;
                                address_input = INITIAL_URL.to_string();
                                begin_navigation(
                                    &mut browser,
                                    rt.handle().clone(),
                                    &event_proxy,
                                    &mut renderer,
                                    &mut pending_atlas,
                                    vk_ctx.as_ref(),
                                    &window,
                                    quality,
                                    INITIAL_URL,
                                );
                            }
                            crate::ui::ui_gen::UIButton::AddressBar => {
                                address_focused = true;
                                address_input = browser.current_url().to_string();
                                update_address_preview(
                                    &mut browser,
                                    &mut renderer,
                                    &mut pending_atlas,
                                    vk_ctx.as_ref(),
                                    &window,
                                    quality,
                                    &address_input,
                                );
                            }
                            crate::ui::ui_gen::UIButton::Close => {
                                shutdown(&mut renderer, &mut vk_ctx, control_flow);
                            }
                            crate::ui::ui_gen::UIButton::Minimize => {
                                window.set_minimized(true);
                            }
                            crate::ui::ui_gen::UIButton::Maximize => {
                                let is_max = window.is_maximized();
                                window.set_maximized(!is_max);
                            }
                            crate::ui::ui_gen::UIButton::TabSelect(i) => {
                                address_focused = false;
                                browser.switch_tab(i);
                                address_input = browser.current_url().to_string();
                                let current_url = browser.current_url().to_string();
                                let atlas = if let Some(new_atlas) = browser.rerender_current_page(
                                    quality.text_rasterization_options(),
                                    win_size.width as f32,
                                    win_size.height as f32,
                                ) {
                                    new_atlas
                                } else {
                                    loading_atlas(
                                        &current_url,
                                        quality.text_rasterization_options(),
                                        win_size.width as f32,
                                    )
                                };
                                apply_atlas(&mut renderer, &mut pending_atlas, vk_ctx.as_ref(), atlas);
                                window.request_redraw();
                            }
                            crate::ui::ui_gen::UIButton::TabClose(i) => {
                                address_focused = false;
                                browser.close_tab(i);
                                address_input = browser.current_url().to_string();
                                let current_url = browser.current_url().to_string();
                                let atlas = if let Some(new_atlas) = browser.rerender_current_page(
                                    quality.text_rasterization_options(),
                                    win_size.width as f32,
                                    win_size.height as f32,
                                ) {
                                    new_atlas
                                } else {
                                    if browser.current_tab().document.is_none() {
                                        spawn_page_load(rt.handle().clone(), event_proxy.clone(), current_url.clone());
                                    }
                                    loading_atlas(
                                        &current_url,
                                        quality.text_rasterization_options(),
                                        win_size.width as f32,
                                    )
                                };
                                apply_atlas(&mut renderer, &mut pending_atlas, vk_ctx.as_ref(), atlas);
                                window.request_redraw();
                            }
                            crate::ui::ui_gen::UIButton::NewTab => {
                                address_focused = false;
                                browser.open_tab(INITIAL_URL);
                                address_input = INITIAL_URL.to_string();
                                begin_navigation(
                                    &mut browser,
                                    rt.handle().clone(),
                                    &event_proxy,
                                    &mut renderer,
                                    &mut pending_atlas,
                                    vk_ctx.as_ref(),
                                    &window,
                                    quality,
                                    INITIAL_URL,
                                );
                            }
                        }
                        break;
                    }
                }

                if !hit_button {
                    address_focused = false;
                    let click_res = browser.handle_page_click(cursor_pos.x as f32, cursor_pos.y as f32);
                    match click_res {
                        PageClickResult::Navigate(url) => {
                            println!("[Browser] Navigating to {}", url);
                            address_input = url.clone();
                            begin_navigation(
                                &mut browser,
                                rt.handle().clone(),
                                &event_proxy,
                                &mut renderer,
                                &mut pending_atlas,
                                vk_ctx.as_ref(),
                                &window,
                                quality,
                                &url,
                            );
                        }
                        PageClickResult::Submit(submit_url) => {
                            println!("[Browser] Submitting form to {}", submit_url);
                            address_input = submit_url.clone();
                            begin_navigation(
                                &mut browser,
                                rt.handle().clone(),
                                &event_proxy,
                                &mut renderer,
                                &mut pending_atlas,
                                vk_ctx.as_ref(),
                                &window,
                                quality,
                                &submit_url,
                            );
                        }
                        PageClickResult::InputFocused | PageClickResult::None => {
                            let current_url = browser.current_url().to_string();
                            let atlas = browser.rerender_current_page(
                                quality.text_rasterization_options(),
                                win_size.width as f32,
                                win_size.height as f32,
                            ).unwrap_or_else(|| {
                                loading_atlas(&current_url, quality.text_rasterization_options(), win_size.width as f32)
                            });
                            apply_atlas(&mut renderer, &mut pending_atlas, vk_ctx.as_ref(), atlas);
                            window.request_redraw();

                            if matches!(click_res, PageClickResult::None) && cursor_pos.y < (36.0 * window.scale_factor()) {
                                let _ = window.drag_window();
                            }
                        }
                    }
                }
            }
            Event::RedrawRequested(_) => {
                let win_size = window.inner_size();
                if let (Some(ctx), Some(r)) = (vk_ctx.as_ref(), renderer.as_mut()) {
                    r.draw_frame(
                        ctx,
                        browser.style(),
                        browser.layout_boxes(),
                        win_size.width as f32,
                        win_size.height as f32,
                        browser.tabs.len(),
                        browser.active_tab_index,
                        window.scale_factor() as f32,
                    );
                }
            }
            _ => (),
        }
    });
}

fn ensure_gpu_ready(
    window: &Window,
    vk_ctx: &mut Option<VulkanContext>,
    renderer: &mut Option<Renderer2D>,
    pending_atlas: &mut Option<RasterizedAtlas>,
    quality: QualityProfile,
    current_url: &str,
) {
    if vk_ctx.is_some() {
        return;
    }

    let atlas = pending_atlas.take().unwrap_or_else(|| {
        loading_atlas(
            current_url,
            quality.text_rasterization_options(),
            window.inner_size().width as f32,
        )
    });
    let ctx = VulkanContext::new(window);
    let r = Renderer2D::new(&ctx, atlas, quality);
    *vk_ctx = Some(ctx);
    *renderer = Some(r);
    window.request_redraw();
}

fn spawn_page_load(rt: tokio::runtime::Handle, proxy: EventLoopProxy<BrowserEvent>, url: String) {
    rt.spawn(async move {
        let document = load_page_document(&url).await;
        let _ = proxy.send_event(BrowserEvent::PageLoaded { url, document });
    });
}

fn begin_navigation(
    browser: &mut BrowserState,
    rt: tokio::runtime::Handle,
    proxy: &EventLoopProxy<BrowserEvent>,
    renderer: &mut Option<Renderer2D>,
    pending_atlas: &mut Option<RasterizedAtlas>,
    vk_ctx: Option<&VulkanContext>,
    window: &Window,
    quality: QualityProfile,
    url: &str,
) {
    browser.navigate_new(url);
    begin_pending_load(rt, proxy, renderer, pending_atlas, vk_ctx, window, quality, url);
}

fn begin_pending_load(
    rt: tokio::runtime::Handle,
    proxy: &EventLoopProxy<BrowserEvent>,
    renderer: &mut Option<Renderer2D>,
    pending_atlas: &mut Option<RasterizedAtlas>,
    vk_ctx: Option<&VulkanContext>,
    window: &Window,
    quality: QualityProfile,
    url: &str,
) {
    let win_size = window.inner_size();
    let loading = loading_atlas(
        url,
        quality.text_rasterization_options(),
        win_size.width as f32,
    );
    apply_atlas(renderer, pending_atlas, vk_ctx, loading);
    spawn_page_load(rt, proxy.clone(), url.to_string());
    window.request_redraw();
}

fn update_address_preview(
    browser: &mut BrowserState,
    renderer: &mut Option<Renderer2D>,
    pending_atlas: &mut Option<RasterizedAtlas>,
    vk_ctx: Option<&VulkanContext>,
    window: &Window,
    quality: QualityProfile,
    address_text: &str,
) {
    let win_size = window.inner_size();
    let atlas = browser
        .rerender_with_address(
            address_text,
            quality.text_rasterization_options(),
            win_size.width as f32,
            win_size.height as f32,
        )
        .unwrap_or_else(|| {
            loading_atlas(
                address_text,
                quality.text_rasterization_options(),
                win_size.width as f32,
            )
        });
    apply_atlas(renderer, pending_atlas, vk_ctx, atlas);
    window.request_redraw();
}

fn apply_atlas(
    renderer: &mut Option<Renderer2D>,
    pending_atlas: &mut Option<RasterizedAtlas>,
    vk_ctx: Option<&VulkanContext>,
    atlas: RasterizedAtlas,
) {
    if let (Some(ctx), Some(r)) = (vk_ctx, renderer.as_mut()) {
        r.update_text_atlas(ctx, atlas);
    } else {
        *pending_atlas = Some(atlas);
    }
}

fn loading_atlas(
    url: &str,
    text_options: crate::render::text::TextRasterizationOptions,
    viewport_width: f32,
) -> RasterizedAtlas {
    RasterizedAtlas::with_options(
        &[
            TextRequest {
                text: compact_url_text(url, viewport_width),
                px_size: 16.0,
                is_bold: false,
                pos_x: 202.0,
                pos_y: 48.0,
                color: [1.0, 1.0, 1.0, 1.0],
            },
            TextRequest {
                text: "Cargando...".to_string(),
                px_size: 20.0,
                is_bold: true,
                pos_x: 40.0,
                pos_y: 92.0,
                color: [1.0, 1.0, 1.0, 1.0],
            },
        ],
        &[],
        text_options,
    )
}

fn normalize_address_input(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        return Some(trimmed.to_string());
    }

    if trimmed.contains('.') && !trimmed.contains(' ') {
        return Some(format!("https://{trimmed}"));
    }

    Some(format!(
        "https://duckduckgo.com/?q={}",
        encode_search_query(trimmed)
    ))
}

fn encode_search_query(input: &str) -> String {
    input
        .bytes()
        .flat_map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                vec![byte as char]
            }
            b' ' => vec!['+'],
            _ => {
                let hex = b"0123456789ABCDEF";
                vec![
                    '%',
                    hex[(byte >> 4) as usize] as char,
                    hex[(byte & 0x0F) as usize] as char,
                ]
            }
        })
        .collect()
}

fn compact_url_text(url: &str, viewport_width: f32) -> String {
    let max_chars = ((viewport_width - 250.0).max(160.0) / 8.5) as usize;
    if url.chars().count() <= max_chars {
        return url.to_string();
    }

    let keep_start = (max_chars / 2).saturating_sub(2).max(12);
    let keep_end = max_chars.saturating_sub(keep_start + 3).max(8);
    let start: String = url.chars().take(keep_start).collect();
    let end: String = url
        .chars()
        .rev()
        .take(keep_end)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    format!("{start}...{end}")
}

fn shutdown(
    renderer: &mut Option<Renderer2D>,
    vk_ctx: &mut Option<VulkanContext>,
    control_flow: &mut ControlFlow,
) {
    if let (Some(mut r), Some(ctx)) = (renderer.take(), vk_ctx.as_ref()) {
        r.cleanup(&ctx.device.device);
    }
    if let Some(_ctx) = vk_ctx.take() {
        // Drop automatically cleans up contexts
    }
    *control_flow = ControlFlow::Exit;
}
