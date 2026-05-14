use ash::vk;
use crate::vulkan_engine::setup::VulkanContext;

pub struct RealRenderer {
    pub command_pool: vk::CommandPool,
    pub command_buffer: vk::CommandBuffer,
    pub image_available_semaphore: vk::Semaphore,
    pub render_finished_semaphore: vk::Semaphore,
    pub in_flight_fence: vk::Fence,
}

impl RealRenderer {
    pub fn new(ctx: &VulkanContext) -> Self {
        unsafe {
            println!("[*] Creando Command Pool y Primitivas de Sincronización...");
            let pool_info = vk::CommandPoolCreateInfo::builder()
                .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
                .queue_family_index(ctx.queue_family_index);
            
            let command_pool = ctx.device.create_command_pool(&pool_info, None).unwrap();

            let alloc_info = vk::CommandBufferAllocateInfo::builder()
                .command_pool(command_pool)
                .level(vk::CommandBufferLevel::PRIMARY)
                .command_buffer_count(1);
            
            let command_buffer = ctx.device.allocate_command_buffers(&alloc_info).unwrap()[0];

            let semaphore_info = vk::SemaphoreCreateInfo::builder();
            let image_available_semaphore = ctx.device.create_semaphore(&semaphore_info, None).unwrap();
            let render_finished_semaphore = ctx.device.create_semaphore(&semaphore_info, None).unwrap();
            
            let fence_info = vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);
            let in_flight_fence = ctx.device.create_fence(&fence_info, None).unwrap();

            Self {
                command_pool,
                command_buffer,
                image_available_semaphore,
                render_finished_semaphore,
                in_flight_fence,
            }
        }
    }

    pub fn draw_frame(&self, ctx: &VulkanContext) {
        unsafe {
            // 1. Wait for previous frame
            ctx.device.wait_for_fences(&[self.in_flight_fence], true, std::u64::MAX).unwrap();

            // 2. Acquire next image
            let (image_index, _is_suboptimal) = match ctx.swapchain_loader.acquire_next_image(
                ctx.swapchain,
                std::u64::MAX,
                self.image_available_semaphore,
                vk::Fence::null()
            ) {
                Ok((idx, suboptimal)) => (idx, suboptimal),
                Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                    println!("[Vulkan] Swapchain Out of Date (Resized/Maximized). Skipping frame...");
                    return;
                }
                Err(e) => panic!("Failed to acquire next image: {:?}", e),
            };

            // 3. Reset fence
            ctx.device.reset_fences(&[self.in_flight_fence]).unwrap();

            // 4. Record command buffer
            ctx.device.reset_command_buffer(self.command_buffer, vk::CommandBufferResetFlags::empty()).unwrap();
            
            let begin_info = vk::CommandBufferBeginInfo::builder()
                .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
            
            ctx.device.begin_command_buffer(self.command_buffer, &begin_info).unwrap();

            let image = ctx.present_images[image_index as usize];

            // Barrier 1: Undefined -> Transfer Dst
            let barrier_to_clear = vk::ImageMemoryBarrier::builder()
                .old_layout(vk::ImageLayout::UNDEFINED)
                .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .image(image)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                })
                .src_access_mask(vk::AccessFlags::empty())
                .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE);

            ctx.device.cmd_pipeline_barrier(
                self.command_buffer,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::TRANSFER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                std::slice::from_ref(&barrier_to_clear),
            );

            // Clear Color: #1a1a2e (r: 0.102, g: 0.102, b: 0.180, a: 1.0)
            let clear_color = vk::ClearColorValue { float32: [0.102, 0.102, 0.180, 1.0] };
            let clear_range = vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            };

            ctx.device.cmd_clear_color_image(
                self.command_buffer,
                image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &clear_color,
                std::slice::from_ref(&clear_range),
            );

            // Barrier 2: Transfer Dst -> Present Src
            let barrier_to_present = vk::ImageMemoryBarrier::builder()
                .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                .new_layout(vk::ImageLayout::PRESENT_SRC_KHR)
                .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .image(image)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                })
                .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                .dst_access_mask(vk::AccessFlags::MEMORY_READ);

            ctx.device.cmd_pipeline_barrier(
                self.command_buffer,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                std::slice::from_ref(&barrier_to_present),
            );

            ctx.device.end_command_buffer(self.command_buffer).unwrap();

            // 5. Submit
            let wait_semaphores = [self.image_available_semaphore];
            let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
            let command_buffers = [self.command_buffer];
            let signal_semaphores = [self.render_finished_semaphore];

            let submit_info = vk::SubmitInfo::builder()
                .wait_semaphores(&wait_semaphores)
                .wait_dst_stage_mask(&wait_stages)
                .command_buffers(&command_buffers)
                .signal_semaphores(&signal_semaphores);

            ctx.device.queue_submit(ctx.present_queue, std::slice::from_ref(&submit_info), self.in_flight_fence).unwrap();

            // 6. Present
            let swapchains = [ctx.swapchain];
            let image_indices = [image_index];

            let present_info = vk::PresentInfoKHR::builder()
                .wait_semaphores(&signal_semaphores)
                .swapchains(&swapchains)
                .image_indices(&image_indices);

            match ctx.swapchain_loader.queue_present(ctx.present_queue, &present_info) {
                Ok(_) => {}
                Err(vk::Result::ERROR_OUT_OF_DATE_KHR) | Err(vk::Result::SUBOPTIMAL_KHR) => {
                    println!("[Vulkan] Swapchain Out of Date during present (Resized/Maximized). Skipping frame...");
                }
                Err(e) => panic!("Failed to present queue: {:?}", e),
            }
        }
    }
}
