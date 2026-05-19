use ash::{vk, Device};
use gpu_allocator::vulkan::{Allocator, Allocation, AllocationCreateDesc, AllocationScheme};
use gpu_allocator::MemoryLocation;
use std::sync::{Arc, Mutex};

pub fn create_buffer(
    device: &Device,
    allocator: &Arc<Mutex<Allocator>>,
    size: u64,
    usage: vk::BufferUsageFlags,
    location: MemoryLocation,
    name: &str,
) -> (vk::Buffer, Allocation) {
    let buffer_info = vk::BufferCreateInfo::builder()
        .size(size)
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);

    let buffer = unsafe { device.create_buffer(&buffer_info, None).expect("Failed to create buffer") };
    let requirements = unsafe { device.get_buffer_memory_requirements(buffer) };

    let allocation = allocator
        .lock()
        .unwrap()
        .allocate(&AllocationCreateDesc {
            name,
            requirements,
            location,
            linear: true,
            allocation_scheme: AllocationScheme::GpuAllocatorManaged,
        })
        .expect("Failed to allocate buffer memory");

    unsafe {
        device
            .bind_buffer_memory(buffer, allocation.memory(), allocation.offset())
            .expect("Failed to bind buffer memory");
    }

    (buffer, allocation)
}

pub fn create_staging_buffer(
    device: &Device,
    allocator: &Arc<Mutex<Allocator>>,
    size: u64,
) -> (vk::Buffer, Allocation) {
    create_buffer(
        device,
        allocator,
        size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        MemoryLocation::CpuToGpu,
        "Staging Buffer",
    )
}

pub fn destroy_buffer(
    device: &Device,
    allocator: &Arc<Mutex<Allocator>>,
    buffer: vk::Buffer,
    allocation: Allocation,
) {
    unsafe {
        device.destroy_buffer(buffer, None);
    }
    allocator
        .lock()
        .unwrap()
        .free(allocation)
        .expect("Failed to free buffer memory");
}
