use crate::vulkan_engine::core::device::VulkanDevice;
use crate::vulkan_engine::core::instance::VulkanInstance;
use crate::vulkan_engine::core::surface::VulkanSurface;
use crate::vulkan_engine::renderer::swapchain::SwapchainManager;

pub struct VulkanContext {
    pub swapchain_manager: SwapchainManager,
    pub device: VulkanDevice,
    pub surface: VulkanSurface,
    pub instance: VulkanInstance,
}

impl VulkanContext {
    pub fn new(window: &winit::window::Window) -> Self {
        let instance = VulkanInstance::new(window);
        let surface = VulkanSurface::new(&instance.entry, &instance.instance, window);
        let device = VulkanDevice::new(
            &instance.instance,
            surface.surface,
            &surface.surface_loader,
        );
        let swapchain_manager = SwapchainManager::new(
            &instance.instance,
            &device.device,
            device.physical_device,
            surface.surface,
            &surface.surface_loader,
        );

        Self {
            swapchain_manager,
            device,
            surface,
            instance,
        }
    }

    pub fn recreate_swapchain(&mut self, new_width: u32, new_height: u32) {
        self.swapchain_manager.recreate_swapchain(
            &self.device.device,
            self.device.physical_device,
            self.surface.surface,
            &self.surface.surface_loader,
            new_width,
            new_height,
        );
    }
}

impl Drop for VulkanContext {
    fn drop(&mut self) {
        println!("[*] Limpiando Vulkan Context (Modular)...");
        // We drop swapchain manually because it needs `device.device`.
        // The other fields (device, surface, instance) will be dropped in reverse declaration order, 
        // which means swapchain -> device -> surface -> instance.
        self.swapchain_manager.destroy(&self.device.device);
    }
}
