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

        // Get the primary monitor size to limit the window
        let monitor_size = event_loop
            .primary_monitor()
            .map(|m| {
                let sf = m.scale_factor();
                let logical = m.size();
                LogicalSize::new(
                    (logical.width as f64 / sf) as f64,
                    (logical.height as f64 / sf) as f64,
                )
            })
            .unwrap_or(LogicalSize::new(1920.0, 1080.0));

        // Limit window to 90% of monitor size
        let target_w = (monitor_size.width * 0.9).min(1920.0).max(800.0);
        let target_h = (monitor_size.height * 0.9).min(1080.0).max(500.0);

        let attrs = WindowAttributes::default()
            .with_title("Noir Browser")
            .with_inner_size(LogicalSize::new(target_w, target_h))
            .with_min_inner_size(LogicalSize::new(800.0, 500.0))
            .with_max_inner_size(LogicalSize::new(
                (monitor_size.width * 0.95).min(2560.0),
                (monitor_size.height * 0.95).min(1440.0),
            ))
            .with_decorations(false);

        let window = match event_loop.create_window(attrs) {
            Ok(w) => Rc::new(w),
            Err(e) => {
                tracing::error!("Failed to create window: {}", e);
                event_loop.exit();
                return;
            }
        };

        // Center window on screen
        if let Some(monitor) = event_loop.primary_monitor() {
            let scale = monitor.scale_factor();
            let mon_size = monitor.size();
            let win_size = window.outer_size();
            let mon_logical_w = (mon_size.width as f64 / scale) as i32;
            let mon_logical_h = (mon_size.height as f64 / scale) as i32;
            let win_logical_w = (win_size.width as f64 / scale) as i32;
            let win_logical_h = (win_size.height as f64 / scale) as i32;
            let x = (mon_logical_w - win_logical_w) / 2;
            let y = (mon_logical_h - win_logical_h) / 2;
            let _ = window.set_outer_position(winit::dpi::LogicalPosition::new(x.max(0), y.max(0)));
        }

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
                    // Clamp size to prevent oversized windows
                    let max_w = 5120;  // 5K width max
                    let max_h = 2880;  // 5K height max
                    let clamped_w = size.width.min(max_w);
                    let clamped_h = size.height.min(max_h);

                    self.context.width = clamped_w;
                    self.context.height = clamped_h;
                    if let Some(surface) = &mut self.context.surface {
                        if let (Some(w), Some(h)) = (NonZeroU32::new(clamped_w), NonZeroU32::new(clamped_h)) {
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
                self.context.update_hover();
                if let Some(window) = &self.context.window {
                    if self.context.is_hovering_link {
                        window.set_cursor_icon(winit::window::CursorIcon::Pointer);
                    } else {
                        window.set_cursor_icon(winit::window::CursorIcon::Default);
                    }
                }
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
                // FASE A4: usar ScrollState con inercia
                self.context.tabs[active].scroll.scroll_by(scroll_amount);
                self.context.tabs[active].scroll_y = self.context.tabs[active].scroll.offset_y;
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

        // FASE A3: record frame metrics
        self.context.metrics.record_frame();

        if self.context.fetching {
            self.context.process_fetch_result();
            self.context.tick_animation();
            if let Some(window) = &self.context.window {
                window.request_redraw();
            }
        }

        self.context.process_pending_timers();
        self.context.process_image_dirty();
    }
}
