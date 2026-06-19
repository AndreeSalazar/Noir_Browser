//! Event Loop - Bucle principal de eventos
//!
//! Usa winit 0.30 ApplicationHandler trait.

use std::num::NonZeroU32;
use std::rc::Rc;

use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{WindowAttributes, WindowId};

use super::config::AppConfig;
use super::context::AppContext;

/// Punto de entrada del event loop
pub fn run(context: AppContext) -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);

    let mut app = NoirApp::new(context);

    event_loop.run_app(&mut app)?;

    Ok(())
}

/// Wrapper que implementa ApplicationHandler
struct NoirApp {
    context: AppContext,
}

impl NoirApp {
    fn new(context: AppContext) -> Self {
        Self { context }
    }
}

impl ApplicationHandler for NoirApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.context.window.is_some() {
            return;
        }

        let attrs = WindowAttributes::default()
            .with_title("Noir Browser")
            .with_inner_size(LogicalSize::new(1280.0, 720.0))
            .with_min_inner_size(LogicalSize::new(800.0, 500.0))
            .with_decorations(false);

        let window = match event_loop.create_window(attrs) {
            Ok(w) => Rc::new(w),
            Err(e) => {
                tracing::error!("Failed to create window: {}", e);
                event_loop.exit();
                return;
            }
        };
        let size = window.inner_size();
        self.context.width = size.width;
        self.context.height = size.height;

        let context = match softbuffer::Context::new(Rc::clone(&window)) {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("Failed to create softbuffer context: {}", e);
                return;
            }
        };
        let surface = match softbuffer::Surface::new(&context, Rc::clone(&window)) {
            Ok(s) => s,
            Err(e) => {
                tracing::error!("Failed to create surface: {}", e);
                return;
            }
        };

        self.context.window = Some(window);
        self.context.surface = Some(surface);

        tracing::info!("Window created: {}x{}", self.context.width, self.context.height);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                tracing::info!("Close requested, shutting down...");
                event_loop.exit();
            }

            WindowEvent::RedrawRequested => {
                self.context.draw_frame();
            }

            WindowEvent::Resized(size) => {
                if size.width > 0 && size.height > 0 {
                    self.context.width = size.width;
                    self.context.height = size.height;
                    if let Some(surface) = &mut self.context.surface {
                        if let (Some(w), Some(h)) = (NonZeroU32::new(size.width), NonZeroU32::new(size.height)) {
                            if let Err(e) = surface.resize(w, h) {
                                tracing::error!("Failed to resize surface: {}", e);
                            }
                        }
                    }
                    self.context.recalculate_layout();
                    if let Some(window) = &self.context.window {
                        window.request_redraw();
                    }
                }
            }

            WindowEvent::CursorMoved { position, .. } => {
                self.context.mouse_x = position.x as f32;
                self.context.mouse_y = position.y as f32;
            }

            WindowEvent::MouseInput { state: ElementState::Pressed, .. } => {
                self.context.handle_click();
                if let Some(window) = &self.context.window {
                    window.request_redraw();
                }
            }

            WindowEvent::MouseWheel { delta, .. } => {
                let scroll_amount = match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => y * 60.0,
                    winit::event::MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
                };
                let active = self.context.active_tab;
                self.context.tabs[active].scroll_y -= scroll_amount;
                self.context.tabs[active].scroll_y = self.context.tabs[active].scroll_y.max(0.0);
                if let Some(window) = &self.context.window {
                    window.request_redraw();
                }
            }

            WindowEvent::KeyboardInput {
                event: KeyEvent { logical_key, state: ElementState::Pressed, .. },
                ..
            } => {
                let ctrl = self.context.modifiers.control_key();
                self.context.handle_key(&logical_key, ctrl);
                if let Some(window) = &self.context.window {
                    window.request_redraw();
                }
            }

            WindowEvent::ModifiersChanged(new_mods) => {
                self.context.modifiers = new_mods.state();
            }

            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self.context.should_close {
            event_loop.exit();
        }

        if self.context.fetching {
            self.context.process_fetch_result();
        }

        self.context.process_pending_timers();
        self.context.process_image_dirty();
    }
}
