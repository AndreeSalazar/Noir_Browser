use winit::{
    event::{Event, MouseScrollDelta, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy},
    window::{Window, WindowBuilder},
};

use crate::browser::{load_page_document, BrowserState, PageDocument};
use crate::render::quality::QualityProfile;
use crate::render::text::{RasterizedAtlas, TextRequest};
use crate::vulkan_engine::real_renderer::RealRenderer;
use crate::vulkan_engine::setup::VulkanContext;

const INITIAL_URL: &str = "https://example.com";

enum BrowserEvent {
    PageLoaded { url: String, document: PageDocument },
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
    let mut renderer: Option<RealRenderer> = None;
    let mut cursor_pos = winit::dpi::PhysicalPosition::new(0.0, 0.0);
    let event_proxy = event_loop.create_proxy();
    spawn_page_load(event_proxy.clone(), INITIAL_URL.to_string());
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
                    if let (Some(ctx), Some(r)) = (vk_ctx.as_ref(), renderer.as_mut()) {
                        r.update_text_atlas(ctx, new_atlas);
                    } else {
                        pending_atlas = Some(new_atlas);
                    }
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
                event: WindowEvent::Resized(new_size),
                ..
            } => {
                let Some(ctx) = vk_ctx.as_mut() else {
                    return;
                };

                if let Some(mut r) = renderer.take() {
                    r.cleanup(&ctx.device);
                }
                ctx.recreate_swapchain(new_size.width, new_size.height);
                let new_atlas = browser
                    .rerender_current_page(
                        quality.text_rasterization_options(),
                        new_size.width as f32,
                        new_size.height as f32,
                    )
                    .unwrap_or_else(|| {
                        loading_atlas(browser.current_url(), quality.text_rasterization_options())
                    });
                renderer = Some(RealRenderer::new(ctx, new_atlas, quality));
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
                    if let (Some(ctx), Some(r)) = (vk_ctx.as_ref(), renderer.as_mut()) {
                        r.update_text_atlas(ctx, new_atlas);
                    } else {
                        pending_atlas = Some(new_atlas);
                    }
                    window.request_redraw();
                }
            }
            Event::WindowEvent {
                event:
                    WindowEvent::MouseInput {
                        state: winit::event::ElementState::Pressed,
                        button: winit::event::MouseButton::Left,
                        ..
                    },
                ..
            } => {
                let win_size = window.inner_size();
                let hitboxes = crate::ui::ui_gen::get_ui_hitboxes(
                    win_size.width as f32,
                    win_size.height as f32,
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
                        }
                        break;
                    }
                }

                if !hit_button {
                    if let Some(url) = browser.link_at_y(cursor_pos.y as f32) {
                        println!("[Browser] Navigating to {}", url);
                        browser.set_pending_url(&url);
                        let loading = loading_atlas(
                            &url,
                            quality.text_rasterization_options(),
                        );

                        if let (Some(ctx), Some(r)) = (vk_ctx.as_ref(), renderer.as_mut()) {
                            r.update_text_atlas(ctx, loading);
                        } else {
                            pending_atlas = Some(loading);
                        }

                        spawn_page_load(event_proxy.clone(), url);
                        window.request_redraw();
                    } else if cursor_pos.y < 40.0 {
                        let _ = window.drag_window();
                    }
                }
            }
            Event::RedrawRequested(_) => {
                let win_size = window.inner_size();
                if let (Some(ctx), Some(r)) = (vk_ctx.as_ref(), renderer.as_mut()) {
                    r.draw_frame(
                        ctx,
                        browser.style(),
                        win_size.width as f32,
                        win_size.height as f32,
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
    renderer: &mut Option<RealRenderer>,
    pending_atlas: &mut Option<RasterizedAtlas>,
    quality: QualityProfile,
    current_url: &str,
) {
    if vk_ctx.is_some() {
        return;
    }

    let atlas = pending_atlas
        .take()
        .unwrap_or_else(|| loading_atlas(current_url, quality.text_rasterization_options()));
    let ctx = VulkanContext::new(window);
    let r = RealRenderer::new(&ctx, atlas, quality);
    *vk_ctx = Some(ctx);
    *renderer = Some(r);
    window.request_redraw();
}

fn spawn_page_load(proxy: EventLoopProxy<BrowserEvent>, url: String) {
    std::thread::spawn(move || {
        let document = load_page_document(&url);
        let _ = proxy.send_event(BrowserEvent::PageLoaded { url, document });
    });
}

fn loading_atlas(
    url: &str,
    text_options: crate::render::text::TextRasterizationOptions,
) -> RasterizedAtlas {
    RasterizedAtlas::with_options(
        &[
            TextRequest {
                text: url.to_string(),
                px_size: 16.0,
                is_bold: false,
                pos_x: 20.0,
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
        text_options,
    )
}

fn shutdown(
    renderer: &mut Option<RealRenderer>,
    vk_ctx: &mut Option<VulkanContext>,
    control_flow: &mut ControlFlow,
) {
    if let (Some(mut r), Some(ctx)) = (renderer.take(), vk_ctx.as_ref()) {
        r.cleanup(&ctx.device);
    }
    if let Some(mut ctx) = vk_ctx.take() {
        ctx.cleanup();
    }
    *control_flow = ControlFlow::Exit;
}
