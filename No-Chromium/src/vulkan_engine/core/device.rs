use ash::{vk, Instance, Device};
use ash::extensions::khr::Swapchain;
use std::ops::Deref;

pub struct VulkanDevice {
    pub physical_device: vk::PhysicalDevice,
    pub device: Device,
    pub queue_family_index: u32,
    pub present_queue: vk::Queue,
}

impl VulkanDevice {
    pub fn new(instance: &Instance, surface: vk::SurfaceKHR, surface_loader: &ash::extensions::khr::Surface) -> Self {
        unsafe {
            println!("[*] Seleccionando Hardware...");
            let physical_devices = instance.enumerate_physical_devices().unwrap();
            let (physical_device, queue_family_index) = physical_devices.into_iter().find_map(|pdevice| {
                let props = instance.get_physical_device_queue_family_properties(pdevice);
                for (index, family) in props.iter().enumerate() {
                    let supports_graphic_and_surface = family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                        && surface_loader.get_physical_device_surface_support(pdevice, index as u32, surface).unwrap();
                    if supports_graphic_and_surface {
                        return Some((pdevice, index as u32));
                    }
                }
                None
            }).expect("No suitable hardware found");

            println!("[*] Creando Device Lógico y Cola...");
            let priorities = [1.0];
            let queue_info = vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(queue_family_index)
                .queue_priorities(&priorities);
            
            let device_extension_names_raw = [Swapchain::name().as_ptr()];
            let features = vk::PhysicalDeviceFeatures::builder();
            
            let device_create_info = vk::DeviceCreateInfo::builder()
                .queue_create_infos(std::slice::from_ref(&queue_info))
                .enabled_extension_names(&device_extension_names_raw)
                .enabled_features(&features);

            let device = instance.create_device(physical_device, &device_create_info, None).unwrap();
            let present_queue = device.get_device_queue(queue_family_index, 0);

            Self {
                physical_device,
                device,
                queue_family_index,
                present_queue,
            }
        }
    }
}

impl Drop for VulkanDevice {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_device(None);
        }
    }
}

impl Deref for VulkanDevice {
    type Target = ash::Device;

    fn deref(&self) -> &Self::Target {
        &self.device
    }
}
