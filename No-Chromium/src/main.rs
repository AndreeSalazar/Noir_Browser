mod generated_rust;
mod parsers;
mod vulkan_engine;
mod text_rasterizer;

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use generated_rust::vulkan_painter::VulkanPainter;
use generated_rust::html_elements::{HTMLElement, HtmlTag};
use generated_rust::css_engine::ComputedStyle;
use parsers::html_lexer::HtmlLexer;

// THE REAL VULKAN ENGINE
use vulkan_engine::setup::VulkanContext;
use vulkan_engine::real_renderer::RealRenderer;

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("No-Chromium | Sovereign GPU Engine (Vulkan 1.3)")
        .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0))
        .build(&event_loop)
        .unwrap();

    println!("========================================");
    println!("     NO-CHROMIUM: AWAKENING THE GPU     ");
    println!("========================================");

    // Initialize Real Vulkan Hardware
    let vk_ctx = VulkanContext::new(&window);
    let renderer = RealRenderer::new(&vk_ctx);

    // 0. Test the new Native Lexers
    let mut lexer = HtmlLexer::new("<video src='...'></video>");
    println!("[Parser] First Token Extracted: {:?}", lexer.consume_next());

    // 1. Simulate a YouTube Video Element from the Parser
    let yt_video = HTMLElement {
        tag: HtmlTag::Video,
        attributes: std::collections::HashMap::new(),
        id: 5001,
    };

    // 2. Apply CSS Styles from the Engine
    let mut style = ComputedStyle::default();
    style.display = Some("block".to_string());
    style.width = Some("100%".to_string());
    style.background_color = Some("#1a1a2e".to_string());
    
    let mut painter = VulkanPainter::new();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::RedrawRequested(_) => {
                painter.add_element(&yt_video, &style);
                painter.flush(); 
                
                // VULKAN: Execute the REAL hardware drawing command
                renderer.draw_frame(&vk_ctx);
            },
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            _ => (),
        }
    });
}
