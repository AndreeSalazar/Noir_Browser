use ash::{vk, Device};
use gpu_allocator::vulkan::{Allocator, Allocation, AllocationCreateDesc, AllocationScheme};
use gpu_allocator::MemoryLocation;
use std::sync::{Arc, Mutex};

pub fn create_texture_image(
    device: &Device,
    allocator: &Arc<Mutex<Allocator>>,
    width: u32,
    height: u32,
) -> (vk::Image, Allocation) {
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

    let image = unsafe { device.create_image(&image_info, None).expect("Failed to create image") };
    let requirements = unsafe { device.get_image_memory_requirements(image) };

    let allocation = allocator
        .lock()
        .unwrap()
        .allocate(&AllocationCreateDesc {
            name: "Texture Image",
            requirements,
            location: MemoryLocation::GpuOnly,
            linear: false,
            allocation_scheme: AllocationScheme::GpuAllocatorManaged,
        })
        .expect("Failed to allocate image memory");

    unsafe {
        device
            .bind_image_memory(image, allocation.memory(), allocation.offset())
            .expect("Failed to bind image memory");
    }

    (image, allocation)
}

pub fn destroy_image(
    device: &Device,
    allocator: &Arc<Mutex<Allocator>>,
    image: vk::Image,
    allocation: Allocation,
) {
    unsafe {
        device.destroy_image(image, None);
    }
    allocator
        .lock()
        .unwrap()
        .free(allocation)
        .expect("Failed to free image memory");
}
