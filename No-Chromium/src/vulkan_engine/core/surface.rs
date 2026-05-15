use ash::{vk, Instance, Entry};
use ash::extensions::khr::Surface;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};

pub struct VulkanSurface {
    pub surface_loader: Surface,
    pub surface: vk::SurfaceKHR,
}

impl VulkanSurface {
    pub fn new(entry: &Entry, instance: &Instance, window: &winit::window::Window) -> Self {
        unsafe {
            println!("[*] Vinculando Surface con Windows (winit)...");
            let surface = ash_window::create_surface(entry, instance, window.raw_display_handle(), window.raw_window_handle(), None).unwrap();
            let surface_loader = Surface::new(entry, instance);

            Self {
                surface_loader,
                surface,
            }
        }
    }
}
