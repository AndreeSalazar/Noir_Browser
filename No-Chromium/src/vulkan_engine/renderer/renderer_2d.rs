use crate::render::quality::{PixelSnap, QualityProfile, TextFiltering};
use crate::render::text::{RasterizedAtlas, TextQuad};
use crate::ui::ui_gen::UIVertex;
use crate::vulkan_engine::context::VulkanContext;
use crate::vulkan_engine::memory::allocator::MemoryAllocator;
use crate::vulkan_engine::memory::buffer::{create_staging_buffer, create_buffer, destroy_buffer};
use crate::vulkan_engine::memory::image::{create_texture_image, destroy_image};
use ash::vk;
use std::ffi::CString;
use shaderc;
use gpu_allocator::MemoryLocation;

const VERTEX_SHADER_SOURCE: &str = include_str!("../shaders/shader_2d.vert");
const FRAGMENT_SHADER_SOURCE: &str = include_str!("../shaders/shader_2d.frag");

pub struct Renderer2D {
    pub memory_allocator: MemoryAllocator,
    pub render_pass: vk::RenderPass,
    pub descriptor_set_layout: vk::DescriptorSetLayout,
    pub pipeline_layout: vk::PipelineLayout,
    pub graphics_pipeline: vk::Pipeline,

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
    pub texture_allocation: gpu_allocator::vulkan::Allocation,
    pub texture_view: vk::ImageView,
    pub texture_sampler: vk::Sampler,

    // Descriptors
    pub descriptor_pool: vk::DescriptorPool,
    pub descriptor_set: vk::DescriptorSet,

    // Vertex Buffer
    pub vertex_buffer: vk::Buffer,
    pub vertex_allocation: gpu_allocator::vulkan::Allocation,
    pub vertex_count: u32,

    // Layout State
    pub text_quads: Vec<TextQuad>,
    pub quality: QualityProfile,
}

impl Renderer2D {
    pub fn new(ctx: &VulkanContext, text_data: RasterizedAtlas, quality: QualityProfile) -> Self {
        unsafe {
            println!("[*] Inicializando Renderizador 2D con Push Constants...");

            // 1. Shaders
            let compiler = shaderc::Compiler::new().expect("Failed to initialize shader compiler");
            let mut options = shaderc::CompileOptions::new().unwrap();
            options.set_optimization_level(shaderc::OptimizationLevel::Performance);

            let vert_spv = compiler
                .compile_into_spirv(VERTEX_SHADER_SOURCE, shaderc::ShaderKind::Vertex, "shader_2d.vert", "main", Some(&options))
                .expect("Failed to compile vertex shader");
            let frag_spv = compiler
                .compile_into_spirv(FRAGMENT_SHADER_SOURCE, shaderc::ShaderKind::Fragment, "shader_2d.frag", "main", Some(&options))
                .expect("Failed to compile fragment shader");

            let vert_create_info = vk::ShaderModuleCreateInfo::builder().code(vert_spv.as_binary());
            let frag_create_info = vk::ShaderModuleCreateInfo::builder().code(frag_spv.as_binary());

            let vert_module = ctx.device.device.create_shader_module(&vert_create_info, None).unwrap();
            let frag_module = ctx.device.device.create_shader_module(&frag_create_info, None).unwrap();

            // 2. Memory Allocator
            let memory_allocator = MemoryAllocator::new(
                &ctx.instance.instance,
                ctx.device.physical_device,
                &ctx.device.device,
            );

            // 3. Descriptor Set Layout
            let sampler_layout_binding = vk::DescriptorSetLayoutBinding::builder()
                .binding(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT)
                .build();

            let layout_info = vk::DescriptorSetLayoutCreateInfo::builder()
                .bindings(std::slice::from_ref(&sampler_layout_binding));

            let descriptor_set_layout = ctx.device.device
                .create_descriptor_set_layout(&layout_info, None)
                .unwrap();

            // 4. Render Pass
            let color_attachment = vk::AttachmentDescription::builder()
                .format(ctx.swapchain_manager.surface_format.format)
                .samples(vk::SampleCountFlags::TYPE_1)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::STORE)
                .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);

            let color_attachment_ref = vk::AttachmentReference::builder()
                .attachment(0)
                .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

            let subpass = vk::SubpassDescription::builder()
                .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
                .color_attachments(std::slice::from_ref(&color_attachment_ref));

            let dependency = vk::SubpassDependency::builder()
                .src_subpass(vk::SUBPASS_EXTERNAL)
                .dst_subpass(0)
                .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
                .src_access_mask(vk::AccessFlags::empty())
                .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
                .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE);

            let render_pass_info = vk::RenderPassCreateInfo::builder()
                .attachments(std::slice::from_ref(&color_attachment))
                .subpasses(std::slice::from_ref(&subpass))
                .dependencies(std::slice::from_ref(&dependency));

            let render_pass = ctx.device.device.create_render_pass(&render_pass_info, None).unwrap();

            // 5. Pipeline Layout & Push Constants
            let push_constant_range = vk::PushConstantRange::builder()
                .stage_flags(vk::ShaderStageFlags::VERTEX)
                .offset(0)
                .size(64) // mat4 (64 bytes)
                .build();

            let pipeline_layout_info = vk::PipelineLayoutCreateInfo::builder()
                .set_layouts(std::slice::from_ref(&descriptor_set_layout))
                .push_constant_ranges(std::slice::from_ref(&push_constant_range));

            let pipeline_layout = ctx.device.device
                .create_pipeline_layout(&pipeline_layout_info, None)
                .unwrap();

            // 6. Graphics Pipeline
            let main_function_name = CString::new("main").unwrap();

            let vert_shader_stage_info = vk::PipelineShaderStageCreateInfo::builder()
                .stage(vk::ShaderStageFlags::VERTEX)
                .module(vert_module)
                .name(&main_function_name);

            let frag_shader_stage_info = vk::PipelineShaderStageCreateInfo::builder()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(frag_module)
                .name(&main_function_name);

            let shader_stages = [
                vert_shader_stage_info.build(),
                frag_shader_stage_info.build(),
            ];

            let vertex_binding_description = vk::VertexInputBindingDescription::builder()
                .binding(0)
                .stride(48) // 12 floats * 4 bytes
                .input_rate(vk::VertexInputRate::VERTEX)
                .build();

            let vertex_attribute_descriptions = [
                vk::VertexInputAttributeDescription::builder()
                    .binding(0)
                    .location(0)
                    .format(vk::Format::R32G32_SFLOAT)
                    .offset(0)
                    .build(),
                vk::VertexInputAttributeDescription::builder()
                    .binding(0)
                    .location(1)
                    .format(vk::Format::R32G32B32A32_SFLOAT)
                    .offset(8)
                    .build(),
                vk::VertexInputAttributeDescription::builder()
                    .binding(0)
                    .location(2)
                    .format(vk::Format::R32G32_SFLOAT)
                    .offset(24)
                    .build(),
                vk::VertexInputAttributeDescription::builder()
                    .binding(0)
                    .location(3)
                    .format(vk::Format::R32G32B32A32_SFLOAT)
                    .offset(32)
                    .build(),
            ];

            let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::builder()
                .vertex_binding_descriptions(std::slice::from_ref(&vertex_binding_description))
                .vertex_attribute_descriptions(&vertex_attribute_descriptions);

            let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::builder()
                .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
                .primitive_restart_enable(false);

            let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
            let dynamic_state_info = vk::PipelineDynamicStateCreateInfo::builder()
                .dynamic_states(&dynamic_states);

            let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
                .viewport_count(1)
                .scissor_count(1);

            let rasterizer = vk::PipelineRasterizationStateCreateInfo::builder()
                .depth_clamp_enable(false)
                .rasterizer_discard_enable(false)
                .polygon_mode(vk::PolygonMode::FILL)
                .line_width(1.0)
                .cull_mode(vk::CullModeFlags::BACK)
                .front_face(vk::FrontFace::CLOCKWISE)
                .depth_bias_enable(false);

            let multisampling = vk::PipelineMultisampleStateCreateInfo::builder()
                .sample_shading_enable(false)
                .rasterization_samples(vk::SampleCountFlags::TYPE_1);

            let color_blend_attachment = vk::PipelineColorBlendAttachmentState::builder()
                .color_write_mask(
                    vk::ColorComponentFlags::R
                        | vk::ColorComponentFlags::G
                        | vk::ColorComponentFlags::B
                        | vk::ColorComponentFlags::A,
                )
                .blend_enable(true)
                .src_color_blend_factor(vk::BlendFactor::ONE)
                .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
                .color_blend_op(vk::BlendOp::ADD)
                .src_alpha_blend_factor(vk::BlendFactor::ONE)
                .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
                .alpha_blend_op(vk::BlendOp::ADD);

            let color_blending = vk::PipelineColorBlendStateCreateInfo::builder()
                .logic_op_enable(false)
                .attachments(std::slice::from_ref(&color_blend_attachment));

            let pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
                .stages(&shader_stages)
                .vertex_input_state(&vertex_input_info)
                .input_assembly_state(&input_assembly)
                .viewport_state(&viewport_state)
                .rasterization_state(&rasterizer)
                .multisample_state(&multisampling)
                .color_blend_state(&color_blending)
                .dynamic_state(&dynamic_state_info)
                .layout(pipeline_layout)
                .render_pass(render_pass)
                .subpass(0);

            let graphics_pipeline = ctx.device.device
                .create_graphics_pipelines(
                    vk::PipelineCache::null(),
                    std::slice::from_ref(&pipeline_info),
                    None,
                )
                .expect("Failed to create Graphics Pipeline")[0];

            ctx.device.device.destroy_shader_module(vert_module, None);
            ctx.device.device.destroy_shader_module(frag_module, None);

            // 7. Command Pool & Sync
            let pool_info = vk::CommandPoolCreateInfo::builder()
                .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
                .queue_family_index(ctx.device.queue_family_index);
            let command_pool = ctx.device.device.create_command_pool(&pool_info, None).unwrap();

            let alloc_info = vk::CommandBufferAllocateInfo::builder()
                .command_pool(command_pool)
                .level(vk::CommandBufferLevel::PRIMARY)
                .command_buffer_count(1);
            let command_buffer = ctx.device.device.allocate_command_buffers(&alloc_info).unwrap()[0];

            // 8. Upload Texture
            let image_size = (text_data.width * text_data.height * 4) as u64;
            let (staging_buffer, staging_alloc) = create_staging_buffer(
                &ctx.device.device,
                &memory_allocator.allocator,
                image_size,
            );

            let mapped_ptr = staging_alloc.mapped_ptr().expect("Failed to map memory").as_ptr();
            std::ptr::copy_nonoverlapping(
                text_data.rgba_data.as_ptr(),
                mapped_ptr as *mut u8,
                image_size as usize,
            );

            let (texture_image, texture_allocation) = create_texture_image(
                &ctx.device.device,
                &memory_allocator.allocator,
                text_data.width,
                text_data.height,
            );

            let begin_info = vk::CommandBufferBeginInfo::builder()
                .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
            ctx.device.device.begin_command_buffer(command_buffer, &begin_info).unwrap();

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

            ctx.device.device.cmd_pipeline_barrier(
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

            ctx.device.device.cmd_copy_buffer_to_image(
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

            ctx.device.device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                std::slice::from_ref(&barrier_to_shader),
            );

            ctx.device.device.end_command_buffer(command_buffer).unwrap();
            let submit_info = vk::SubmitInfo::builder().command_buffers(std::slice::from_ref(&command_buffer));
            ctx.device.device.queue_submit(ctx.device.present_queue, std::slice::from_ref(&submit_info), vk::Fence::null()).unwrap();
            ctx.device.device.queue_wait_idle(ctx.device.present_queue).unwrap();

            destroy_buffer(&ctx.device.device, &memory_allocator.allocator, staging_buffer, staging_alloc);

            // 9. Texture View & Sampler
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
            let texture_view = ctx.device.device.create_image_view(&view_info, None).unwrap();

            let sampler_info = vk::SamplerCreateInfo::builder()
                .mag_filter(vk::Filter::LINEAR)
                .min_filter(vk::Filter::LINEAR)
                .mipmap_mode(vk::SamplerMipmapMode::NEAREST)
                .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                .min_lod(0.0)
                .max_lod(0.0);
            let texture_sampler = ctx.device.device.create_sampler(&sampler_info, None).unwrap();

            // 10. Descriptor Pool & Set
            let pool_sizes = [vk::DescriptorPoolSize::builder()
                .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .build()];
            let desc_pool_info = vk::DescriptorPoolCreateInfo::builder()
                .pool_sizes(&pool_sizes)
                .max_sets(1);
            let descriptor_pool = ctx.device.device.create_descriptor_pool(&desc_pool_info, None).unwrap();

            let alloc_info = vk::DescriptorSetAllocateInfo::builder()
                .descriptor_pool(descriptor_pool)
                .set_layouts(std::slice::from_ref(&descriptor_set_layout));
            let descriptor_set = ctx.device.device.allocate_descriptor_sets(&alloc_info).unwrap()[0];

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
            ctx.device.device.update_descriptor_sets(std::slice::from_ref(&write_desc), &[]);

            // 11. Framebuffers
            let framebuffers: Vec<vk::Framebuffer> = ctx.swapchain_manager.present_image_views
                .iter()
                .map(|&view| {
                    let fb_info = vk::FramebufferCreateInfo::builder()
                        .render_pass(render_pass)
                        .attachments(std::slice::from_ref(&view))
                        .width(ctx.swapchain_manager.extent.width)
                        .height(ctx.swapchain_manager.extent.height)
                        .layers(1);
                    ctx.device.device.create_framebuffer(&fb_info, None).unwrap()
                })
                .collect();

            // 12. Sync
            let semaphore_info = vk::SemaphoreCreateInfo::builder();
            let image_available_semaphore = ctx.device.device.create_semaphore(&semaphore_info, None).unwrap();
            let render_finished_semaphore = ctx.device.device.create_semaphore(&semaphore_info, None).unwrap();
            let fence_info = vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);
            let in_flight_fence = ctx.device.device.create_fence(&fence_info, None).unwrap();

            Self {
                memory_allocator,
                render_pass,
                descriptor_set_layout,
                pipeline_layout,
                graphics_pipeline,
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
                text_quads: text_data.quads,
                quality,
            }
        }
    }

    pub fn update_text_atlas(&mut self, ctx: &VulkanContext, text_data: RasterizedAtlas) {
        unsafe {
            ctx.device.device.device_wait_idle().unwrap();

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
            ctx.device.device.update_descriptor_sets(std::slice::from_ref(&write_desc), &[]);

            ctx.device.device.destroy_sampler(self.texture_sampler, None);
            ctx.device.device.destroy_image_view(self.texture_view, None);
            destroy_image(&ctx.device.device, &self.memory_allocator.allocator, self.texture_image, std::mem::replace(&mut self.texture_allocation, std::mem::zeroed()));

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
    ) -> (vk::Image, gpu_allocator::vulkan::Allocation, vk::ImageView, vk::Sampler) {
        unsafe {
            let image_size = (text_data.width * text_data.height * 4) as u64;
            let (staging_buffer, staging_alloc) = create_staging_buffer(
                &ctx.device.device,
                &self.memory_allocator.allocator,
                image_size,
            );

            let mapped_ptr = staging_alloc.mapped_ptr().expect("Failed to map memory").as_ptr();
            std::ptr::copy_nonoverlapping(
                text_data.rgba_data.as_ptr(),
                mapped_ptr as *mut u8,
                image_size as usize,
            );

            let (texture_image, texture_allocation) = create_texture_image(
                &ctx.device.device,
                &self.memory_allocator.allocator,
                text_data.width,
                text_data.height,
            );

            ctx.device.device
                .reset_command_buffer(self.command_buffer, vk::CommandBufferResetFlags::empty())
                .unwrap();
            let begin_info = vk::CommandBufferBeginInfo::builder()
                .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
            ctx.device.device.begin_command_buffer(self.command_buffer, &begin_info).unwrap();

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
            ctx.device.device.cmd_pipeline_barrier(
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
            ctx.device.device.cmd_copy_buffer_to_image(
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
            ctx.device.device.cmd_pipeline_barrier(
                self.command_buffer,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                std::slice::from_ref(&barrier_to_shader),
            );

            ctx.device.device.end_command_buffer(self.command_buffer).unwrap();
            let submit_info = vk::SubmitInfo::builder().command_buffers(std::slice::from_ref(&self.command_buffer));
            ctx.device.device.queue_submit(ctx.device.present_queue, std::slice::from_ref(&submit_info), vk::Fence::null()).unwrap();
            ctx.device.device.queue_wait_idle(ctx.device.present_queue).unwrap();
            destroy_buffer(&ctx.device.device, &self.memory_allocator.allocator, staging_buffer, staging_alloc);

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
            let texture_view = ctx.device.device.create_image_view(&view_info, None).unwrap();

            let sampler_info = vk::SamplerCreateInfo::builder()
                .mag_filter(vk::Filter::LINEAR)
                .min_filter(vk::Filter::LINEAR)
                .mipmap_mode(vk::SamplerMipmapMode::NEAREST)
                .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                .min_lod(0.0)
                .max_lod(0.0);
            let texture_sampler = ctx.device.device.create_sampler(&sampler_info, None).unwrap();

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
        boxes: &[crate::browser::RenderBox],
        win_width: f32,
        win_height: f32,
        tabs_count: usize,
        active_tab_index: usize,
        scale_factor: f32,
        hovered_button: Option<crate::ui::ui_gen::UIButton>,
    ) {
        unsafe {
            ctx.device.device
                .wait_for_fences(&[self.in_flight_fence], true, std::u64::MAX)
                .unwrap();

            let (image_index, _) = match ctx.swapchain_manager.swapchain_loader.acquire_next_image(
                ctx.swapchain_manager.swapchain,
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

            ctx.device.device.reset_fences(&[self.in_flight_fence]).unwrap();

            // ==========================================
            // DYNAMIC LAYOUT ENGINE (DOM -> GPU VERTICES)
            // ==========================================
            let dom_vertices = crate::layout::layout_gen::LayoutEngine::build_dom_vertices(
                boxes, win_width, win_height,
            );
            let mut all_vertices = Vec::with_capacity(dom_vertices.len() * 12 + 1024);
            for v in &dom_vertices {
                all_vertices.push(v.x);
                all_vertices.push(v.y);
                all_vertices.push(v.r);
                all_vertices.push(v.g);
                all_vertices.push(v.b);
                all_vertices.push(v.a);
                all_vertices.push(v.u);
                all_vertices.push(v.v);
                all_vertices.push(v.box_w);
                all_vertices.push(v.box_h);
                all_vertices.push(v.radius);
                all_vertices.push(v.is_text);
            }
            let chrome_vertices = crate::ui::ui_gen::generate_chrome_vertices(
                win_width,
                win_height,
                tabs_count,
                active_tab_index,
                scale_factor,
                hovered_button,
            );
            all_vertices.extend_from_slice(&chrome_vertices);

            // Text Quads (placed from Atlas using raw pixel coordinates)
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

                let text_x = snap(tq.x) + filter_bias;
                let text_y = snap(tq.y) + filter_bias;
                let text_w = tq.w;
                let text_h = tq.h;

                let c = tq.color;
                let quad = [
                    UIVertex::textured(text_x, text_y, c[0], c[1], c[2], c[3], tq.u0, tq.v0, tq.is_text),
                    UIVertex::textured(text_x + text_w, text_y, c[0], c[1], c[2], c[3], tq.u1, tq.v0, tq.is_text),
                    UIVertex::textured(text_x, text_y + text_h, c[0], c[1], c[2], c[3], tq.u0, tq.v1, tq.is_text),
                    UIVertex::textured(text_x + text_w, text_y, c[0], c[1], c[2], c[3], tq.u1, tq.v0, tq.is_text),
                    UIVertex::textured(text_x + text_w, text_y + text_h, c[0], c[1], c[2], c[3], tq.u1, tq.v1, tq.is_text),
                    UIVertex::textured(text_x, text_y + text_h, c[0], c[1], c[2], c[3], tq.u0, tq.v1, tq.is_text),
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
                    all_vertices.push(v.box_w);
                    all_vertices.push(v.box_h);
                    all_vertices.push(v.radius);
                    all_vertices.push(v.is_text);
                }
            }

            self.vertex_count = (all_vertices.len() / 12) as u32;

            // Destroy previous vertex buffer if it exists
            if self.vertex_buffer != vk::Buffer::null() {
                destroy_buffer(
                    &ctx.device.device,
                    &self.memory_allocator.allocator,
                    self.vertex_buffer,
                    std::mem::replace(&mut self.vertex_allocation, std::mem::zeroed()),
                );
            }

            // Create new vertex buffer dynamically
            let vertex_size = (all_vertices.len() * std::mem::size_of::<f32>()) as u64;
            let (v_staging_buffer, v_staging_alloc) = create_staging_buffer(
                &ctx.device.device,
                &self.memory_allocator.allocator,
                vertex_size,
            );
            let v_mapped_ptr = v_staging_alloc.mapped_ptr().expect("Failed to map memory").as_ptr();
            std::ptr::copy_nonoverlapping(
                all_vertices.as_ptr() as *const u8,
                v_mapped_ptr as *mut u8,
                vertex_size as usize,
            );

            let (vertex_buffer, vertex_allocation) = create_buffer(
                &ctx.device.device,
                &self.memory_allocator.allocator,
                vertex_size,
                vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
                MemoryLocation::GpuOnly,
                "Vertex Buffer",
            );

            self.vertex_buffer = vertex_buffer;
            self.vertex_allocation = vertex_allocation;

            ctx.device.device
                .reset_command_buffer(self.command_buffer, vk::CommandBufferResetFlags::empty())
                .unwrap();

            let begin_info = vk::CommandBufferBeginInfo::builder()
                .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
            ctx.device.device.begin_command_buffer(self.command_buffer, &begin_info).unwrap();

            // Record copy command for dynamic vertices
            let v_copy_region = vk::BufferCopy::builder().size(vertex_size);
            ctx.device.device.cmd_copy_buffer(
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
            ctx.device.device.cmd_pipeline_barrier(
                self.command_buffer,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::VERTEX_INPUT,
                vk::DependencyFlags::empty(),
                &[],
                std::slice::from_ref(&buffer_barrier),
                &[],
            );

            let clear_color = style
                .background_color
                .as_deref()
                .map(crate::layout::layout_gen::LayoutEngine::parse_color)
                .unwrap_or((0.102, 0.102, 0.180, 1.0));
            let clear_values = [vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [clear_color.0, clear_color.1, clear_color.2, clear_color.3],
                },
            }];
            let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
                .render_pass(self.render_pass)
                .framebuffer(self.framebuffers[image_index as usize])
                .render_area(vk::Rect2D {
                    offset: vk::Offset2D { x: 0, y: 0 },
                    extent: ctx.swapchain_manager.extent,
                })
                .clear_values(&clear_values);

            ctx.device.device.cmd_begin_render_pass(
                self.command_buffer,
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            );
            ctx.device.device.cmd_bind_pipeline(
                self.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.graphics_pipeline,
            );

            let viewport = vk::Viewport::builder()
                .x(0.0)
                .y(0.0)
                .width(ctx.swapchain_manager.extent.width as f32)
                .height(ctx.swapchain_manager.extent.height as f32)
                .min_depth(0.0)
                .max_depth(1.0);
            ctx.device.device
                .cmd_set_viewport(self.command_buffer, 0, std::slice::from_ref(&viewport));

            let scissor = vk::Rect2D::builder()
                .offset(vk::Offset2D { x: 0, y: 0 })
                .extent(ctx.swapchain_manager.extent);
            ctx.device.device
                .cmd_set_scissor(self.command_buffer, 0, std::slice::from_ref(&scissor));

            // Orthographic projection matrix push constant
            let w = win_width;
            let h = win_height;
            let ortho_matrix = [
                2.0 / w,  0.0,      0.0, 0.0,
                0.0,      2.0 / h,  0.0, 0.0,
                0.0,      0.0,      1.0, 0.0,
                -1.0,     -1.0,     0.0, 1.0,
            ];

            let ortho_matrix_bytes = std::slice::from_raw_parts(
                ortho_matrix.as_ptr() as *const u8,
                std::mem::size_of_val(&ortho_matrix),
            );

            ctx.device.device.cmd_push_constants(
                self.command_buffer,
                self.pipeline_layout,
                vk::ShaderStageFlags::VERTEX,
                0,
                ortho_matrix_bytes,
            );

            ctx.device.device.cmd_bind_descriptor_sets(
                self.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline_layout,
                0,
                &[self.descriptor_set],
                &[],
            );
            ctx.device.device
                .cmd_bind_vertex_buffers(self.command_buffer, 0, &[self.vertex_buffer], &[0]);

            ctx.device.device
                .cmd_draw(self.command_buffer, self.vertex_count, 1, 0, 0);

            ctx.device.device.cmd_end_render_pass(self.command_buffer);
            ctx.device.device.end_command_buffer(self.command_buffer).unwrap();

            let wait_semaphores = [self.image_available_semaphore];
            let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
            let command_buffers = [self.command_buffer];
            let signal_semaphores = [self.render_finished_semaphore];

            let submit_info = vk::SubmitInfo::builder()
                .wait_semaphores(&wait_semaphores)
                .wait_dst_stage_mask(&wait_stages)
                .command_buffers(&command_buffers)
                .signal_semaphores(&signal_semaphores);

            ctx.device.device
                .queue_submit(
                    ctx.device.present_queue,
                    std::slice::from_ref(&submit_info),
                    self.in_flight_fence,
                )
                .unwrap();
            ctx.device.device.queue_wait_idle(ctx.device.present_queue).unwrap();

            destroy_buffer(&ctx.device.device, &self.memory_allocator.allocator, v_staging_buffer, v_staging_alloc);

            let swapchains = [ctx.swapchain_manager.swapchain];
            let image_indices = [image_index];
            let present_info = vk::PresentInfoKHR::builder()
                .wait_semaphores(&signal_semaphores)
                .swapchains(&swapchains)
                .image_indices(&image_indices);

            match ctx.swapchain_manager.swapchain_loader
                .queue_present(ctx.device.present_queue, &present_info)
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

            if self.vertex_buffer != vk::Buffer::null() {
                destroy_buffer(device, &self.memory_allocator.allocator, self.vertex_buffer, std::mem::replace(&mut self.vertex_allocation, std::mem::zeroed()));
            }
            destroy_image(device, &self.memory_allocator.allocator, self.texture_image, std::mem::replace(&mut self.texture_allocation, std::mem::zeroed()));

            device.destroy_sampler(self.texture_sampler, None);
            device.destroy_image_view(self.texture_view, None);

            for fb in &self.framebuffers {
                device.destroy_framebuffer(*fb, None);
            }

            device.destroy_descriptor_pool(self.descriptor_pool, None);

            device.destroy_pipeline(self.graphics_pipeline, None);
            device.destroy_pipeline_layout(self.pipeline_layout, None);
            device.destroy_descriptor_set_layout(self.descriptor_set_layout, None);
            device.destroy_render_pass(self.render_pass, None);

            device.destroy_semaphore(self.image_available_semaphore, None);
            device.destroy_semaphore(self.render_finished_semaphore, None);
            device.destroy_fence(self.in_flight_fence, None);

            device.destroy_command_pool(self.command_pool, None);
        }
    }
}
