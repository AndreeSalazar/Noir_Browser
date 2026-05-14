mod generated_rust;
mod vulkan_engine;
mod parsers;
mod layout;
mod ui;

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use generated_rust::vulkan_painter::VulkanPainter;
use parsers::css_engine::ComputedStyle;

// THE REAL VULKAN ENGINE
use vulkan_engine::setup::VulkanContext;
use vulkan_engine::real_renderer::RealRenderer;


fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("No-Chromium | Sovereign GPU Engine (Vulkan 1.3)")
        .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0))
        .with_decorations(false) // BORDERLESS NATIVO
        .build(&event_loop)
        .unwrap();

    println!("========================================");
    println!("     NO-CHROMIUM: AWAKENING THE GPU     ");
    println!("========================================");
fn extract_text_from_dom(nodes: &[crate::parsers::dom_tree::DomNode], out: &mut Vec<(String, f32)>, current_size: f32) {
    use crate::parsers::html_elements::HtmlTag;
    for node in nodes {
        if out.len() >= 4 { break; }
        match node {
            crate::parsers::dom_tree::DomNode::Element { tag, children, .. } => {
                if matches!(tag, HtmlTag::Script | HtmlTag::Noscript) {
                    continue;
                }
                let mut new_size = current_size;
                if let HtmlTag::Custom(name) = tag {
                    if name == "style" || name == "title" {
                        continue;
                    }
                } else {
                    match tag {
                        HtmlTag::H1 => new_size = 32.0,
                        HtmlTag::H2 => new_size = 24.0,
                        HtmlTag::H3 => new_size = 20.0,
                        HtmlTag::P => new_size = 16.0,
                        HtmlTag::A => new_size = 14.0,
                        _ => {}
                    }
                }
                extract_text_from_dom(children, out, new_size);
            }
            crate::parsers::dom_tree::DomNode::Text(t) => {
                let trimmed = t.trim();
                if trimmed.len() > 2 {
                    let limited: String = trimmed.chars().take(40).collect();
                    out.push((limited, current_size));
                }
            }
        }
    }
}

    // Phase 2: HTTP Client & html5ever DOM Tree
    let target_url = "https://example.com";
    let html = crate::parsers::http_client::fetch_html(target_url).unwrap_or_else(|_| "<h1>Network Error</h1>".to_string());
    let dom = crate::parsers::dom_tree::parse_html(&html);

    let mut extracted_texts = Vec::new();
    extract_text_from_dom(&dom, &mut extracted_texts, 24.0); // Default size 24.0
    
    use crate::layout::text_rasterizer::{RasterizedAtlas, TextRequest};
    let mut text_requests = Vec::new();
    
    // URL Bar Text (x=20, y=48 inside URL Bar)
    text_requests.push(TextRequest {
        text: target_url.to_string(),
        px_size: 16.0,
        pos_x: 20.0,
        pos_y: 48.0,
        color: [1.0, 1.0, 1.0, 1.0],
    });
    
    // Content Texts (starting y=80)
    let mut current_y = 80.0;
    for (text, size) in extracted_texts {
        text_requests.push(TextRequest {
            text,
            px_size: size,
            pos_x: 40.0,
            pos_y: current_y,
            color: [1.0, 1.0, 1.0, 1.0],
        });
        current_y += 30.0;
    }
    
    let mut extracted_style = ComputedStyle::default();
    extracted_style.background_color = Some("#1a1a1a".to_string());
    extracted_style.width = Some("100%".to_string());
    extracted_style.height = Some("100%".to_string());
    
    // MODULE 3: Rasterized Text (CPU Fontdue -> GPU Texture)
    let text_data = RasterizedAtlas::new(&text_requests);

    // Initialize Real Vulkan Hardware
    let mut vk_ctx = VulkanContext::new(&window);
    let mut renderer = Some(RealRenderer::new(&vk_ctx, text_data));
    
    let mut painter = VulkanPainter::new();

    let mut cursor_pos = winit::dpi::PhysicalPosition::new(0.0, 0.0);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                if let Some(mut r) = renderer.take() {
                    r.cleanup(&vk_ctx.device);
                }
                vk_ctx.cleanup();
                *control_flow = ControlFlow::Exit;
            },
            Event::WindowEvent {
                event: WindowEvent::CursorMoved { position, .. },
                ..
            } => {
                cursor_pos = position;
            },
            Event::WindowEvent {
                event: WindowEvent::Resized(new_size),
                ..
            } => {
                vk_ctx.recreate_swapchain(new_size.width, new_size.height);
                if let Some(r) = &mut renderer {
                    r.recreate_swapchain(&vk_ctx);
                }
            },
            Event::WindowEvent {
                event: WindowEvent::MouseInput { state: winit::event::ElementState::Pressed, button: winit::event::MouseButton::Left, .. },
                ..
            } => {
                let win_size = window.inner_size();
                let hitboxes = crate::ui::ui_gen::get_ui_hitboxes(win_size.width as f32, win_size.height as f32);
                let mut hit_button = false;
                
                for hb in hitboxes {
                    if cursor_pos.x >= hb.x_min as f64 && cursor_pos.x <= hb.x_max as f64 &&
                       cursor_pos.y >= hb.y_min as f64 && cursor_pos.y <= hb.y_max as f64 {
                           hit_button = true;
                           match hb.button {
                               crate::ui::ui_gen::UIButton::Close => {
                                   if let Some(mut r) = renderer.take() { r.cleanup(&vk_ctx.device); }
                                   vk_ctx.cleanup();
                                   *control_flow = ControlFlow::Exit;
                               },
                               crate::ui::ui_gen::UIButton::Minimize => {
                                   window.set_minimized(true);
                               },
                               crate::ui::ui_gen::UIButton::Maximize => {
                                   let is_max = window.is_maximized();
                                   window.set_maximized(!is_max);
                               }
                           }
                           break;
                       }
                }

                // Barra superior de 40px para arrastrar
                if !hit_button && cursor_pos.y < 40.0 {
                    let _ = window.drag_window();
                }
            },
            Event::RedrawRequested(_) => {
                if let Some(r) = &mut renderer {
                    r.draw_frame(&vk_ctx, &extracted_style);
                }
            },
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            _ => (),
        }
    });
}
