use ash::{vk, Device, Instance};
use gpu_allocator::vulkan::{Allocator, AllocatorCreateDesc};
use gpu_allocator::AllocatorDebugSettings;
use std::sync::{Arc, Mutex};

pub struct MemoryAllocator {
    pub allocator: Arc<Mutex<Allocator>>,
}

impl MemoryAllocator {
    pub fn new(instance: &Instance, physical_device: vk::PhysicalDevice, device: &Device) -> Self {
        println!("[*] Inicializando gpu-allocator...");
        
        let allocator = Allocator::new(&AllocatorCreateDesc {
            instance: instance.clone(),
            device: device.clone(),
            physical_device,
            debug_settings: AllocatorDebugSettings {
                log_leaks_on_shutdown: true,
                ..Default::default()
            },
            buffer_device_address: false,
            allocation_sizes: Default::default(),
        }).expect("Failed to create gpu-allocator Allocator");

        Self {
            allocator: Arc::new(Mutex::new(allocator)),
        }
    }
}
