// AUTO-GENERATED VULKAN MEMORY MANAGER (vk-mem VMA)
use ash::{vk, Device, Instance};
use std::sync::{Arc, Mutex};
use vk_mem::{Alloc, Allocator, AllocatorCreateInfo};

pub struct MemoryManager {
    pub allocator: Arc<Mutex<Allocator>>,
}

impl MemoryManager {
    pub fn new(instance: &Instance, physical_device: vk::PhysicalDevice, device: &Device) -> Self {
        println!("[*] Inicializando Vulkan Memory Allocator (vk-mem)...");
        let allocator_info = AllocatorCreateInfo::new(instance, device, physical_device);
        let allocator = Allocator::new(allocator_info).expect("Failed to create VMA Allocator");

        Self {
            allocator: Arc::new(Mutex::new(allocator)),
        }
    }

    pub fn create_staging_buffer(&self, size: u64) -> (vk::Buffer, vk_mem::Allocation) {
        let buffer_info = vk::BufferCreateInfo::builder()
            .size(size)
            .usage(vk::BufferUsageFlags::TRANSFER_SRC)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let alloc_info = vk_mem::AllocationCreateInfo {
            usage: vk_mem::MemoryUsage::Auto,
            flags: vk_mem::AllocationCreateFlags::HOST_ACCESS_SEQUENTIAL_WRITE,
            ..Default::default()
        };

        unsafe {
            self.allocator
                .lock()
                .unwrap()
                .create_buffer(&buffer_info, &alloc_info)
                .expect("Failed to create staging buffer")
        }
    }

    pub fn create_texture_image(&self, width: u32, height: u32) -> (vk::Image, vk_mem::Allocation) {
        let image_info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .extent(vk::Extent3D {
                width,
                height,
                depth: 1,
            })
            .mip_levels(1)
            .array_layers(1)
            .format(vk::Format::R8G8B8A8_UNORM)
            .tiling(vk::ImageTiling::OPTIMAL)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .usage(vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .samples(vk::SampleCountFlags::TYPE_1);

        let alloc_info = vk_mem::AllocationCreateInfo {
            usage: vk_mem::MemoryUsage::Auto,
            flags: vk_mem::AllocationCreateFlags::DEDICATED_MEMORY,
            ..Default::default()
        };

        unsafe {
            self.allocator
                .lock()
                .unwrap()
                .create_image(&image_info, &alloc_info)
                .expect("Failed to create texture image")
        }
    }
}
