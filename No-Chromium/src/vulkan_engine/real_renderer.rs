use crate::render::quality::{PixelSnap, QualityProfile, TextFiltering};
use crate::render::text::{RasterizedAtlas, TextQuad};
use crate::ui::ui_gen::UIVertex;
use crate::vulkan_engine::memory_gen::MemoryManager;
use crate::vulkan_engine::pipeline_gen::PipelineManager;
use crate::vulkan_engine::setup::VulkanContext;
use crate::vulkan_engine::shaders_gen::{
    ShaderModuleLoader, FRAGMENT_SHADER_GLSL, VERTEX_SHADER_GLSL,
};
use ash::vk;
use vk_mem::Alloc;

pub struct RealRenderer {
    pub memory_manager: MemoryManager,
    pub pipeline_manager: PipelineManager,

    // Command & Sync
    pub command_pool: vk::CommandPool,
    pub command_buffer: vk::CommandBuffer,
    pub image_available_semaphore: vk::Semaphore,
    pub render_finished_semaphore: vk::Semaphore,
    pub in_flight_fence: vk::Fence,

    // Framebuffers
    pub framebuffers: Vec<vk::Framebuffer>,

    // Texture
    pub texture_image: vk::Image,
    pub texture_allocation: vk_mem::Allocation,
    pub texture_view: vk::ImageView,
    pub texture_sampler: vk::Sampler,

    // Descriptors
    pub descriptor_pool: vk::DescriptorPool,
    pub descriptor_set: vk::DescriptorSet,

    // Vertex Buffer
    pub vertex_buffer: vk::Buffer,
    pub vertex_allocation: vk_mem::Allocation,
    pub vertex_count: u32,

    // Layout State
    pub text_quads: Vec<TextQuad>,
    pub quality: QualityProfile,
}

impl RealRenderer {
    pub fn new(ctx: &VulkanContext, text_data: RasterizedAtlas, quality: QualityProfile) -> Self {
        unsafe {
            println!("[*] Inicializando Renderizador Real de Texturas...");

            // 1. Shaders
            let vert_spv = ShaderModuleLoader::compile_glsl_to_spirv(
                VERTEX_SHADER_GLSL,
                shaderc::ShaderKind::Vertex,
                "vertex.glsl",
            );
            let frag_spv = ShaderModuleLoader::compile_glsl_to_spirv(
                FRAGMENT_SHADER_GLSL,
                shaderc::ShaderKind::Fragment,
                "fragment.glsl",
            );

            let vert_module = ShaderModuleLoader::create_shader_module(&ctx.device, &vert_spv);
            let frag_module = ShaderModuleLoader::create_shader_module(&ctx.device, &frag_spv);

            // 2. Memory & Pipeline
            let memory_manager =
                MemoryManager::new(&ctx.instance, ctx.physical_device, &ctx.device);
            let pipeline_manager = PipelineManager::new(
                &ctx.device,
                ctx.surface_format,
                ctx.extent,
                vert_module,
                frag_module,
            );

            // 3. Command Pool & Buffers
            let pool_info = vk::CommandPoolCreateInfo::builder()
                .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
                .queue_family_index(ctx.queue_family_index);
            let command_pool = ctx.device.create_command_pool(&pool_info, None).unwrap();

            let alloc_info = vk::CommandBufferAllocateInfo::builder()
                .command_pool(command_pool)
                .level(vk::CommandBufferLevel::PRIMARY)
                .command_buffer_count(1);
            let command_buffer = ctx.device.allocate_command_buffers(&alloc_info).unwrap()[0];

            // 4. Staging Buffer for Texture
            let image_size = (text_data.width * text_data.height * 4) as u64;

            // 5. Staging Buffer for Texture
            let (staging_buffer, mut staging_alloc) =
                memory_manager.create_staging_buffer(image_size);
            let mapped_ptr = memory_manager
                .allocator
                .lock()
                .unwrap()
                .map_memory(&mut staging_alloc)
                .unwrap();
            std::ptr::copy_nonoverlapping(
                text_data.rgba_data.as_ptr(),
                mapped_ptr,
                image_size as usize,
            );
            memory_manager
                .allocator
                .lock()
                .unwrap()
                .unmap_memory(&mut staging_alloc);

            // 6. Texture Image
            let (texture_image, texture_allocation) =
                memory_manager.create_texture_image(text_data.width, text_data.height);

            // Record upload commands
            let begin_info = vk::CommandBufferBeginInfo::builder()
                .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
            ctx.device
                .begin_command_buffer(command_buffer, &begin_info)
                .unwrap();

            let barrier_to_dst = vk::ImageMemoryBarrier::builder()
                .old_layout(vk::ImageLayout::UNDEFINED)
                .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                .src_access_mask(vk::AccessFlags::empty())
                .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                .image(texture_image)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                });

            ctx.device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::TRANSFER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                std::slice::from_ref(&barrier_to_dst),
            );

            let copy_region = vk::BufferImageCopy::builder()
                .image_subresource(vk::ImageSubresourceLayers {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    mip_level: 0,
                    base_array_layer: 0,
                    layer_count: 1,
                })
                .image_extent(vk::Extent3D {
                    width: text_data.width,
                    height: text_data.height,
                    depth: 1,
                });

            ctx.device.cmd_copy_buffer_to_image(
                command_buffer,
                staging_buffer,
                texture_image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                std::slice::from_ref(&copy_region),
            );

            let barrier_to_shader = vk::ImageMemoryBarrier::builder()
                .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                .dst_access_mask(vk::AccessFlags::SHADER_READ)
                .image(texture_image)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                });

            ctx.device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                std::slice::from_ref(&barrier_to_shader),
            );

            // Execute the texture upload
            ctx.device.end_command_buffer(command_buffer).unwrap();
            let submit_info =
                vk::SubmitInfo::builder().command_buffers(std::slice::from_ref(&command_buffer));
            ctx.device
                .queue_submit(
                    ctx.present_queue,
                    std::slice::from_ref(&submit_info),
                    vk::Fence::null(),
                )
                .unwrap();
            ctx.device.queue_wait_idle(ctx.present_queue).unwrap();

            let text_quads = text_data.quads;

            // Cleanup staging
            memory_manager
                .allocator
                .lock()
                .unwrap()
                .destroy_buffer(staging_buffer, &mut staging_alloc);

            // 7. Texture View & Sampler
            let view_info = vk::ImageViewCreateInfo::builder()
                .image(texture_image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(vk::Format::R8G8B8A8_UNORM)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                });
            let texture_view = ctx.device.create_image_view(&view_info, None).unwrap();

            let sampler_info = vk::SamplerCreateInfo::builder()
                .mag_filter(vk::Filter::LINEAR)
                .min_filter(vk::Filter::LINEAR)
                .mipmap_mode(vk::SamplerMipmapMode::NEAREST)
                .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                .min_lod(0.0)
                .max_lod(0.0);
            let texture_sampler = ctx.device.create_sampler(&sampler_info, None).unwrap();

            // 8. Descriptor Set
            let pool_sizes = [vk::DescriptorPoolSize::builder()
                .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .build()];
            let desc_pool_info = vk::DescriptorPoolCreateInfo::builder()
                .pool_sizes(&pool_sizes)
                .max_sets(1);
            let descriptor_pool = ctx
                .device
                .create_descriptor_pool(&desc_pool_info, None)
                .unwrap();

            let alloc_info = vk::DescriptorSetAllocateInfo::builder()
                .descriptor_pool(descriptor_pool)
                .set_layouts(std::slice::from_ref(
                    &pipeline_manager.descriptor_set_layout,
                ));
            let descriptor_set = ctx.device.allocate_descriptor_sets(&alloc_info).unwrap()[0];

            let image_info = vk::DescriptorImageInfo::builder()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(texture_view)
                .sampler(texture_sampler);
            let write_desc = vk::WriteDescriptorSet::builder()
                .dst_set(descriptor_set)
                .dst_binding(0)
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(std::slice::from_ref(&image_info));
            ctx.device
                .update_descriptor_sets(std::slice::from_ref(&write_desc), &[]);

            // 9. Framebuffers
            let framebuffers: Vec<vk::Framebuffer> = ctx
                .present_image_views
                .iter()
                .map(|&view| {
                    let fb_info = vk::FramebufferCreateInfo::builder()
                        .render_pass(pipeline_manager.render_pass)
                        .attachments(std::slice::from_ref(&view))
                        .width(ctx.extent.width)
                        .height(ctx.extent.height)
                        .layers(1);
                    ctx.device.create_framebuffer(&fb_info, None).unwrap()
                })
                .collect();

            // Sync
            let semaphore_info = vk::SemaphoreCreateInfo::builder();
            let image_available_semaphore =
                ctx.device.create_semaphore(&semaphore_info, None).unwrap();
            let render_finished_semaphore =
                ctx.device.create_semaphore(&semaphore_info, None).unwrap();
            let fence_info = vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);
            let in_flight_fence = ctx.device.create_fence(&fence_info, None).unwrap();

            Self {
                memory_manager,
                pipeline_manager,
                command_pool,
                command_buffer,
                image_available_semaphore,
                render_finished_semaphore,
                in_flight_fence,
                framebuffers,
                texture_image,
                texture_allocation,
                texture_view,
                texture_sampler,
                descriptor_pool,
                descriptor_set,
                vertex_buffer: vk::Buffer::null(),
                vertex_allocation: std::mem::zeroed(),
                vertex_count: 0,
                text_quads,
                quality,
            }
        }
    }

    pub fn update_text_atlas(&mut self, ctx: &VulkanContext, text_data: RasterizedAtlas) {
        unsafe {
            ctx.device.device_wait_idle().unwrap();

            let (texture_image, texture_allocation, texture_view, texture_sampler) =
                self.upload_text_texture(ctx, &text_data);

            let image_info = vk::DescriptorImageInfo::builder()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(texture_view)
                .sampler(texture_sampler);
            let write_desc = vk::WriteDescriptorSet::builder()
                .dst_set(self.descriptor_set)
                .dst_binding(0)
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(std::slice::from_ref(&image_info));
            ctx.device
                .update_descriptor_sets(std::slice::from_ref(&write_desc), &[]);

            ctx.device.destroy_sampler(self.texture_sampler, None);
            ctx.device.destroy_image_view(self.texture_view, None);
            self.memory_manager
                .allocator
                .lock()
                .unwrap()
                .destroy_image(self.texture_image, &mut self.texture_allocation);

            self.texture_image = texture_image;
            self.texture_allocation = texture_allocation;
            self.texture_view = texture_view;
            self.texture_sampler = texture_sampler;
            self.text_quads = text_data.quads;
        }
    }

    fn upload_text_texture(
        &mut self,
        ctx: &VulkanContext,
        text_data: &RasterizedAtlas,
    ) -> (vk::Image, vk_mem::Allocation, vk::ImageView, vk::Sampler) {
        unsafe {
            let image_size = (text_data.width * text_data.height * 4) as u64;
            let (staging_buffer, mut staging_alloc) =
                self.memory_manager.create_staging_buffer(image_size);
            let mapped_ptr = self
                .memory_manager
                .allocator
                .lock()
                .unwrap()
                .map_memory(&mut staging_alloc)
                .unwrap();
            std::ptr::copy_nonoverlapping(
                text_data.rgba_data.as_ptr(),
                mapped_ptr,
                image_size as usize,
            );
            self.memory_manager
                .allocator
                .lock()
                .unwrap()
                .unmap_memory(&mut staging_alloc);

            let (texture_image, texture_allocation) = self
                .memory_manager
                .create_texture_image(text_data.width, text_data.height);

            ctx.device
                .reset_command_buffer(self.command_buffer, vk::CommandBufferResetFlags::empty())
                .unwrap();
            let begin_info = vk::CommandBufferBeginInfo::builder()
                .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
            ctx.device
                .begin_command_buffer(self.command_buffer, &begin_info)
                .unwrap();

            let barrier_to_dst = vk::ImageMemoryBarrier::builder()
                .old_layout(vk::ImageLayout::UNDEFINED)
                .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                .src_access_mask(vk::AccessFlags::empty())
                .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                .image(texture_image)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                });
            ctx.device.cmd_pipeline_barrier(
                self.command_buffer,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::TRANSFER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                std::slice::from_ref(&barrier_to_dst),
            );

            let copy_region = vk::BufferImageCopy::builder()
                .image_subresource(vk::ImageSubresourceLayers {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    mip_level: 0,
                    base_array_layer: 0,
                    layer_count: 1,
                })
                .image_extent(vk::Extent3D {
                    width: text_data.width,
                    height: text_data.height,
                    depth: 1,
                });
            ctx.device.cmd_copy_buffer_to_image(
                self.command_buffer,
                staging_buffer,
                texture_image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                std::slice::from_ref(&copy_region),
            );

            let barrier_to_shader = vk::ImageMemoryBarrier::builder()
                .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                .dst_access_mask(vk::AccessFlags::SHADER_READ)
                .image(texture_image)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                });
            ctx.device.cmd_pipeline_barrier(
                self.command_buffer,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                std::slice::from_ref(&barrier_to_shader),
            );

            ctx.device.end_command_buffer(self.command_buffer).unwrap();
            let submit_info = vk::SubmitInfo::builder()
                .command_buffers(std::slice::from_ref(&self.command_buffer));
            ctx.device
                .queue_submit(
                    ctx.present_queue,
                    std::slice::from_ref(&submit_info),
                    vk::Fence::null(),
                )
                .unwrap();
            ctx.device.queue_wait_idle(ctx.present_queue).unwrap();
            self.memory_manager
                .allocator
                .lock()
                .unwrap()
                .destroy_buffer(staging_buffer, &mut staging_alloc);

            let view_info = vk::ImageViewCreateInfo::builder()
                .image(texture_image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(vk::Format::R8G8B8A8_UNORM)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                });
            let texture_view = ctx.device.create_image_view(&view_info, None).unwrap();

            let sampler_info = vk::SamplerCreateInfo::builder()
                .mag_filter(vk::Filter::LINEAR)
                .min_filter(vk::Filter::LINEAR)
                .mipmap_mode(vk::SamplerMipmapMode::NEAREST)
                .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                .min_lod(0.0)
                .max_lod(0.0);
            let texture_sampler = ctx.device.create_sampler(&sampler_info, None).unwrap();

            (
                texture_image,
                texture_allocation,
                texture_view,
                texture_sampler,
            )
        }
    }

    pub fn draw_frame(
        &mut self,
        ctx: &VulkanContext,
        style: &crate::parsers::css_engine::ComputedStyle,
        win_width: f32,
        win_height: f32,
    ) {
        unsafe {
            ctx.device
                .wait_for_fences(&[self.in_flight_fence], true, std::u64::MAX)
                .unwrap();

            let (image_index, _) = match ctx.swapchain_loader.acquire_next_image(
                ctx.swapchain,
                std::u64::MAX,
                self.image_available_semaphore,
                vk::Fence::null(),
            ) {
                Ok(idx) => idx,
                Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                    println!("[Vulkan] Skip frame (Resize)");
                    return;
                }
                Err(e) => panic!("Failed to acquire image: {:?}", e),
            };

            ctx.device.reset_fences(&[self.in_flight_fence]).unwrap();

            // ==========================================
            // DYNAMIC LAYOUT ENGINE (DOM -> GPU VERTICES)
            // ==========================================
            let mut all_vertices =
                crate::ui::ui_gen::generate_chrome_vertices(win_width, win_height);
            let dom_vertices = crate::layout::layout_gen::LayoutEngine::build_dom_vertices(
                style, win_width, win_height,
            );
            for v in &dom_vertices {
                all_vertices.push(v.x);
                all_vertices.push(v.y);
                all_vertices.push(v.r);
                all_vertices.push(v.g);
                all_vertices.push(v.b);
                all_vertices.push(v.a);
                all_vertices.push(v.u);
                all_vertices.push(v.v);
            }

            // Text Quads (placed from Atlas)
            for tq in &self.text_quads {
                let scale = self.quality.device_pixel_ratio.max(1.0);
                let snap = |value: f32| match self.quality.pixel_snap {
                    PixelSnap::None => value,
                    PixelSnap::LogicalPixels => value.round(),
                    PixelSnap::PhysicalPixels => (value * scale).round() / scale,
                };
                let filter_bias = match self.quality.text_filtering {
                    TextFiltering::Linear => 0.0,
                    TextFiltering::SubpixelLinear => 0.0,
                };
                let _msaa_samples = self.quality.msaa_samples;

                let text_w = (tq.w / win_width) * 2.0;
                let text_h = (tq.h / win_height) * 2.0;
                let text_x = -1.0 + ((snap(tq.x) + filter_bias) / win_width) * 2.0;
                let text_y = -1.0 + ((snap(tq.y) + filter_bias) / win_height) * 2.0;

                let c = tq.color;
                let quad = [
                    UIVertex::textured(text_x, text_y, c[0], c[1], c[2], c[3], tq.u0, tq.v0),
                    UIVertex::textured(
                        text_x + text_w,
                        text_y,
                        c[0],
                        c[1],
                        c[2],
                        c[3],
                        tq.u1,
                        tq.v0,
                    ),
                    UIVertex::textured(
                        text_x,
                        text_y + text_h,
                        c[0],
                        c[1],
                        c[2],
                        c[3],
                        tq.u0,
                        tq.v1,
                    ),
                    UIVertex::textured(
                        text_x + text_w,
                        text_y,
                        c[0],
                        c[1],
                        c[2],
                        c[3],
                        tq.u1,
                        tq.v0,
                    ),
                    UIVertex::textured(
                        text_x + text_w,
                        text_y + text_h,
                        c[0],
                        c[1],
                        c[2],
                        c[3],
                        tq.u1,
                        tq.v1,
                    ),
                    UIVertex::textured(
                        text_x,
                        text_y + text_h,
                        c[0],
                        c[1],
                        c[2],
                        c[3],
                        tq.u0,
                        tq.v1,
                    ),
                ];

                for v in &quad {
                    all_vertices.push(v.x);
                    all_vertices.push(v.y);
                    all_vertices.push(v.r);
                    all_vertices.push(v.g);
                    all_vertices.push(v.b);
                    all_vertices.push(v.a);
                    all_vertices.push(v.u);
                    all_vertices.push(v.v);
                }
            }

            self.vertex_count = (all_vertices.len() / 8) as u32;

            // Destroy previous vertex buffer if it exists
            if self.vertex_buffer != vk::Buffer::null() {
                self.memory_manager
                    .allocator
                    .lock()
                    .unwrap()
                    .destroy_buffer(self.vertex_buffer, &mut self.vertex_allocation);
            }

            // Create new vertex buffer dynamically
            let vertex_size = (all_vertices.len() * std::mem::size_of::<f32>()) as u64;
            let (v_staging_buffer, mut v_staging_alloc) =
                self.memory_manager.create_staging_buffer(vertex_size);
            let v_mapped_ptr = self
                .memory_manager
                .allocator
                .lock()
                .unwrap()
                .map_memory(&mut v_staging_alloc)
                .unwrap();
            std::ptr::copy_nonoverlapping(
                all_vertices.as_ptr() as *const u8,
                v_mapped_ptr,
                vertex_size as usize,
            );
            self.memory_manager
                .allocator
                .lock()
                .unwrap()
                .unmap_memory(&mut v_staging_alloc);

            let v_buffer_info = vk::BufferCreateInfo::builder()
                .size(vertex_size)
                .usage(vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER)
                .sharing_mode(vk::SharingMode::EXCLUSIVE);
            let v_alloc_info = vk_mem::AllocationCreateInfo {
                usage: vk_mem::MemoryUsage::Auto,
                flags: vk_mem::AllocationCreateFlags::DEDICATED_MEMORY,
                ..Default::default()
            };
            let (vertex_buffer, vertex_allocation) = self
                .memory_manager
                .allocator
                .lock()
                .unwrap()
                .create_buffer(&v_buffer_info, &v_alloc_info)
                .unwrap();

            self.vertex_buffer = vertex_buffer;
            self.vertex_allocation = vertex_allocation;

            ctx.device
                .reset_command_buffer(self.command_buffer, vk::CommandBufferResetFlags::empty())
                .unwrap();

            let begin_info = vk::CommandBufferBeginInfo::builder()
                .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
            ctx.device
                .begin_command_buffer(self.command_buffer, &begin_info)
                .unwrap();

            // Record copy command for dynamic vertices
            let v_copy_region = vk::BufferCopy::builder().size(vertex_size);
            ctx.device.cmd_copy_buffer(
                self.command_buffer,
                v_staging_buffer,
                self.vertex_buffer,
                std::slice::from_ref(&v_copy_region),
            );

            // Setup memory barrier to ensure copy completes before vertex shader reads it
            let buffer_barrier = vk::BufferMemoryBarrier::builder()
                .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                .dst_access_mask(vk::AccessFlags::VERTEX_ATTRIBUTE_READ)
                .buffer(self.vertex_buffer)
                .size(vk::WHOLE_SIZE);
            ctx.device.cmd_pipeline_barrier(
                self.command_buffer,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::VERTEX_INPUT,
                vk::DependencyFlags::empty(),
                &[],
                std::slice::from_ref(&buffer_barrier),
                &[],
            );

            let clear_values = [vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.102, 0.102, 0.180, 1.0],
                },
            }];
            let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
                .render_pass(self.pipeline_manager.render_pass)
                .framebuffer(self.framebuffers[image_index as usize])
                .render_area(vk::Rect2D {
                    offset: vk::Offset2D { x: 0, y: 0 },
                    extent: ctx.extent,
                })
                .clear_values(&clear_values);

            ctx.device.cmd_begin_render_pass(
                self.command_buffer,
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            );
            ctx.device.cmd_bind_pipeline(
                self.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline_manager.graphics_pipeline,
            );

            let viewport = vk::Viewport::builder()
                .x(0.0)
                .y(0.0)
                .width(ctx.extent.width as f32)
                .height(ctx.extent.height as f32)
                .min_depth(0.0)
                .max_depth(1.0);
            ctx.device
                .cmd_set_viewport(self.command_buffer, 0, std::slice::from_ref(&viewport));

            let scissor = vk::Rect2D::builder()
                .offset(vk::Offset2D { x: 0, y: 0 })
                .extent(ctx.extent);
            ctx.device
                .cmd_set_scissor(self.command_buffer, 0, std::slice::from_ref(&scissor));

            ctx.device.cmd_bind_descriptor_sets(
                self.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline_manager.pipeline_layout,
                0,
                &[self.descriptor_set],
                &[],
            );
            ctx.device
                .cmd_bind_vertex_buffers(self.command_buffer, 0, &[self.vertex_buffer], &[0]);

            ctx.device
                .cmd_draw(self.command_buffer, self.vertex_count, 1, 0, 0);

            ctx.device.cmd_end_render_pass(self.command_buffer);
            ctx.device.end_command_buffer(self.command_buffer).unwrap();

            let wait_semaphores = [self.image_available_semaphore];
            let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
            let command_buffers = [self.command_buffer];
            let signal_semaphores = [self.render_finished_semaphore];

            let submit_info = vk::SubmitInfo::builder()
                .wait_semaphores(&wait_semaphores)
                .wait_dst_stage_mask(&wait_stages)
                .command_buffers(&command_buffers)
                .signal_semaphores(&signal_semaphores);

            ctx.device
                .queue_submit(
                    ctx.present_queue,
                    std::slice::from_ref(&submit_info),
                    self.in_flight_fence,
                )
                .unwrap();
            ctx.device.queue_wait_idle(ctx.present_queue).unwrap(); // Wait to safely destroy staging buffer

            self.memory_manager
                .allocator
                .lock()
                .unwrap()
                .destroy_buffer(v_staging_buffer, &mut v_staging_alloc);

            let swapchains = [ctx.swapchain];
            let image_indices = [image_index];
            let present_info = vk::PresentInfoKHR::builder()
                .wait_semaphores(&signal_semaphores)
                .swapchains(&swapchains)
                .image_indices(&image_indices);

            match ctx
                .swapchain_loader
                .queue_present(ctx.present_queue, &present_info)
            {
                Ok(_) => {}
                Err(vk::Result::ERROR_OUT_OF_DATE_KHR) | Err(vk::Result::SUBOPTIMAL_KHR) => {
                    println!("[Vulkan] Skip frame (Resize)");
                }
                Err(e) => panic!("Failed to present queue: {:?}", e),
            }
        }
    }

    pub fn cleanup(&mut self, device: &ash::Device) {
        unsafe {
            device.device_wait_idle().unwrap();

            // VMA Cleanup (crucial to prevent Assertion crashes)
            let alloc = self.memory_manager.allocator.lock().unwrap();

            // Note: v_staging_alloc was already destroyed in new(). We only need to destroy persistent ones.
            // But texture_allocation and vertex_allocation are not mutable here, we must use std::mem::replace or similar?
            // Actually, destroy_image takes &mut Allocation.
            alloc.destroy_buffer(self.vertex_buffer, &mut self.vertex_allocation);
            alloc.destroy_image(self.texture_image, &mut self.texture_allocation);

            device.destroy_sampler(self.texture_sampler, None);
            device.destroy_image_view(self.texture_view, None);

            for fb in &self.framebuffers {
                device.destroy_framebuffer(*fb, None);
            }

            device.destroy_descriptor_pool(self.descriptor_pool, None);

            // Destroy Pipeline Manager resources
            device.destroy_pipeline(self.pipeline_manager.graphics_pipeline, None);
            device.destroy_pipeline_layout(self.pipeline_manager.pipeline_layout, None);
            device.destroy_descriptor_set_layout(self.pipeline_manager.descriptor_set_layout, None);
            device.destroy_render_pass(self.pipeline_manager.render_pass, None);

            device.destroy_semaphore(self.image_available_semaphore, None);
            device.destroy_semaphore(self.render_finished_semaphore, None);
            device.destroy_fence(self.in_flight_fence, None);

            device.destroy_command_pool(self.command_pool, None);
        }
    }
}
