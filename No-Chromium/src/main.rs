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
use parsers::html_lexer_legacy::HtmlLexer;
use parsers::css_engine::ComputedStyle;

// THE REAL VULKAN ENGINE
use vulkan_engine::setup::VulkanContext;
use vulkan_engine::real_renderer::RealRenderer;
use parsers::html_elements::{HTMLElement, HtmlTag};
use layout::text_rasterizer::RasterizedText;

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
fn extract_text_from_dom(nodes: &[crate::parsers::dom_tree::DomNode]) -> String {
    use crate::parsers::html_elements::HtmlTag;
    for node in nodes {
        match node {
            crate::parsers::dom_tree::DomNode::Element { tag, children, .. } => {
                if matches!(tag, HtmlTag::Script | HtmlTag::Noscript | HtmlTag::Custom(_)) {
                    continue;
                }
                let child_text = extract_text_from_dom(children);
                if !child_text.is_empty() {
                    return child_text;
                }
            }
            crate::parsers::dom_tree::DomNode::Text(t) => {
                let trimmed = t.trim();
                if trimmed.len() > 2 {
                    // Limit length to avoid huge textures
                    let limited: String = trimmed.chars().take(40).collect();
                    return limited;
                }
            }
        }
    }
    String::new()
}

    // Phase 2: HTTP Client & html5ever DOM Tree
    let html = crate::parsers::http_client::fetch_html("https://example.com").unwrap_or_else(|_| "<h1>Network Error</h1>".to_string());
    let dom = crate::parsers::dom_tree::parse_html(&html);

    let extracted_text = extract_text_from_dom(&dom);
    let final_text = if extracted_text.is_empty() {
        "Noir Browser DOM".to_string()
    } else {
        extracted_text
    };

    let mut extracted_style = ComputedStyle::default();
    extracted_style.background_color = Some("#1a1a1a".to_string());
    extracted_style.width = Some("100%".to_string());
    extracted_style.height = Some("100%".to_string());
    
    // MODULE 3: Rasterized Text (CPU Fontdue -> GPU Texture)
    let text_data = crate::layout::text_rasterizer::RasterizedText::new(&final_text, 60.0);

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
