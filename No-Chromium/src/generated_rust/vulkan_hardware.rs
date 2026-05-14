// AUTO-GENERATED VULKAN HARDWARE LINKER
use ash::{vk, Device, Instance};
use ash::extensions::khr::Swapchain;

pub struct HardwareLinker {
    pub swapchain_loader: Swapchain,
    pub swapchain: vk::SwapchainKHR,
    pub images: Vec<vk::Image>,
}

impl HardwareLinker {
    pub fn clear_screen(device: &Device, cmd: vk::CommandBuffer, image: vk::Image) {
        let clear_color = vk::ClearColorValue {
            float32: [0.05, 0.05, 0.15, 1.0], // The No-Chromium Premium Dark Blue
        };
        let range = vk::ImageSubresourceRange::builder()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .level_count(1)
            .layer_count(1);

        unsafe {
            device.cmd_clear_color_image(cmd, image, vk::ImageLayout::GENERAL, &clear_color, &[range.build()]);
        }
    }
}