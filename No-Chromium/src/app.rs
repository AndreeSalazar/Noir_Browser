//! App Loop - Fase 0: Vulkan Ultra-Fast Base
//! 
//! Inicializa winit (ventana) + Vulkan engine + event loop básico

use anyhow::Result;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes, WindowId},
};

use crate::vulkan_engine::UltraFastVulkanEngine;

/// Estado principal de la aplicación
pub struct App {
    window: Option<Window>,
    vulkan_engine: Option<UltraFastVulkanEngine>,
    #[allow(dead_code)]
    window_id: Option<WindowId>,
}

impl App {
    pub fn new() -> Self {
        Self {
            window: None,
            vulkan_engine: None,
            window_id: None,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Crear ventana
        let attributes = WindowAttributes::default()
            .with_title("Noir Browser - Vulkan Ultra-Fast")
            .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0));
        
        let window = event_loop.create_window(attributes).expect("Failed to create window");
        self.window_id = Some(window.id());
        
        // Inicializar Vulkan engine (stub for Phase 0)
        #[cfg(not(feature = "debug_vulkan"))]
        let enable_validation = false;
        #[cfg(feature = "debug_vulkan")]
        let enable_validation = true;
        
        match UltraFastVulkanEngine::new() {
            Ok(engine) => {
                tracing::info!("[vulkan] Engine initialized successfully (stub)");
                self.vulkan_engine = Some(engine);
            }
            Err(e) => {
                tracing::error!("[vulkan] Failed to initialize: {}", e);
                // En debug, continuar sin Vulkan para testing
                #[cfg(feature = "debug_vulkan")]
                {
                    tracing::warn!("[vulkan] Running in fallback mode");
                }
            }
        }
        
        self.window = Some(window);
    }

    fn window_event(&mut self, _event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                tracing::info!("[app] Close requested, shutting down...");
                // Cleanup Vulkan resources
                if let Some(mut engine) = self.vulkan_engine.take() {
                    let _ = engine.cleanup();
                }
                // _event_loop.exit(); // winit 0.29 no tiene exit() en ActiveEventLoop
            }
            WindowEvent::Resized(size) => {
                if let Some(ref mut engine) = self.vulkan_engine {
                    engine.on_resize(size.width, size.height);
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(ref mut engine) = self.vulkan_engine {
                    if let Err(e) = engine.render_frame() {
                        tracing::error!("[render] Frame error: {}", e);
                    }
                }
            }
            _ => {}
        }
    }
}

/// Punto de entrada principal del loop de aplicación
pub fn run() -> Result<()> {
    tracing::info!("[app] Starting Noir Browser (Fase 0: Vulkan Ultra-Fast)");
    
    let event_loop = EventLoop::new()?;
    let mut app = App::new();
    
    event_loop.run_app(&mut app)?;
    
    tracing::info!("[app] Shutdown complete");
    Ok(())
}
