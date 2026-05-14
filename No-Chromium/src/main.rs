mod generated_rust;
mod parsers;

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use generated_rust::vulkan_painter::VulkanPainter;
use generated_rust::html_elements::{HTMLElement, HtmlTag};
use generated_rust::css_engine::ComputedStyle;
use generated_rust::vulkan_hardware::HardwareLinker;
use generated_rust::bootstrapper::VulkanApp;
use parsers::html_lexer::HtmlLexer;

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("No-Chromium | Standard Web Engine")
        .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0))
        .build(&event_loop)
        .unwrap();

    // THE FUSION FINAL: Establishing REAL Hardware Bridge
    let _vulkan_app = VulkanApp::new(&window);

    println!("========================================");
    println!("     NO-CHROMIUM: STANDARDS ACTIVE      ");
    println!("========================================");

    // 0. Test the new Native Lexers
    println!("[Step 0] Initializing Sovereign Parsers...");
    let mut lexer = HtmlLexer::new("<video src='...'></video>");
    println!("[Parser] First Token Extracted: {:?}", lexer.consume_next());

    // 1. Simulate a YouTube Video Element from the Parser
    println!("[Step 1] Parsing YouTube Component...");
    let yt_video = HTMLElement {
        tag: HtmlTag::Video,
        attributes: std::collections::HashMap::new(), // Simplified for fix
        id: 5001,
    };
    println!("[+] HTML Element created: <video>");

    // 2. Apply CSS Styles from the Engine
    let mut style = ComputedStyle::default();
    style.display = Some("block".to_string());
    style.width = Some("100%".to_string());
    style.background_color = Some("#000000".to_string());
    
    println!("[Step 2] Applying CSS: display: {:?}, width: {:?}", 
        style.display, 
        style.width
    );

    let mut painter = VulkanPainter::new();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::RedrawRequested(_) => {
                // Now we pass BOTH the HTML element and its CSS style to Vulkan
                painter.add_element(&yt_video, &style);
                painter.flush();
            },
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            _ => (),
        }
    });
}
