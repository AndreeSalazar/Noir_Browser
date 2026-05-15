use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use crate::browser::BrowserState;
use crate::render::quality::QualityProfile;
use crate::vulkan_engine::real_renderer::RealRenderer;
use crate::vulkan_engine::setup::VulkanContext;

const INITIAL_URL: &str = "https://example.com";

pub fn run() {
    let event_loop = EventLoop::new();
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
    let initial_width = window.inner_size().width as f32;
    let text_data = browser.load_current_page(quality.text_rasterization_options(), initial_width);

    let mut vk_ctx = VulkanContext::new(&window);
    let mut renderer = Some(RealRenderer::new(&vk_ctx, text_data, quality));
    let mut cursor_pos = winit::dpi::PhysicalPosition::new(0.0, 0.0);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
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
                vk_ctx.recreate_swapchain(new_size.width, new_size.height);
                if let Some(r) = &mut renderer {
                    r.recreate_swapchain(&vk_ctx);
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
                        let win_size = window.inner_size();
                        let new_atlas = browser.navigate_to(
                            &url,
                            quality.text_rasterization_options(),
                            win_size.width as f32,
                        );

                        if let Some(mut r) = renderer.take() {
                            r.cleanup(&vk_ctx.device);
                        }

                        renderer = Some(RealRenderer::new(&vk_ctx, new_atlas, quality));
                        window.request_redraw();
                    } else if cursor_pos.y < 40.0 {
                        let _ = window.drag_window();
                    }
                }
            }
            Event::RedrawRequested(_) => {
                let win_size = window.inner_size();
                if let Some(r) = &mut renderer {
                    r.draw_frame(
                        &vk_ctx,
                        browser.style(),
                        win_size.width as f32,
                        win_size.height as f32,
                    );
                }
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            _ => (),
        }
    });
}

fn shutdown(
    renderer: &mut Option<RealRenderer>,
    vk_ctx: &mut VulkanContext,
    control_flow: &mut ControlFlow,
) {
    if let Some(mut r) = renderer.take() {
        r.cleanup(&vk_ctx.device);
    }
    vk_ctx.cleanup();
    *control_flow = ControlFlow::Exit;
}
