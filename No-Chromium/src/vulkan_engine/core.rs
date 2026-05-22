// src/vulkan_engine/core.rs
// 🌌 Noir Browser - UltraFast Vulkan 1.3 Engine Core
// Zero-copy rendering, bindless resources, async compute

use ash::{vk, Device, Entry, Instance};
use gpu_allocator::vulkan::{Allocator, AllocatorCreateDesc};
use std::sync::Arc;
use tracing::{debug, info, error};

/// Configuración del motor Vulkan ultra-rápido
#[derive(Debug, Clone)]
pub struct VulkanConfig {
    pub width: u32,
    pub height: u32,
    pub vsync: bool,
    pub triple_buffering: bool,
    pub enable_validation: bool,
    pub enable_debug_utils: bool,
}

impl Default for VulkanConfig {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            vsync: true,
            triple_buffering: true,
            enable_validation: cfg!(debug_assertions),
            enable_debug_utils: cfg!(debug_assertions),
        }
    }
}

/// Recursos por-frame para triple buffering
pub struct FrameResources {
    pub command_buffer: vk::CommandBuffer,
    pub command_pool: vk::CommandPool,
    pub image_available_semaphore: vk::Semaphore,
    pub render_finished_semaphore: vk::Semaphore,
    pub fence: vk::Fence,
    pub in_flight: bool,
}

/// 🚀 Motor Vulkan Ultra-Fast con Vulkan 1.3 features
pub struct UltraFastVulkanEngine {
    // Core Vulkan
    pub entry: Entry,
    pub instance: Instance,
    pub device: Arc<Device>,
    pub physical_device: vk::PhysicalDevice,
    
    // Queues separadas para paralelismo
    pub graphics_queue: vk::Queue,
    pub compute_queue: vk::Queue,
    pub transfer_queue: vk::Queue,
    pub queue_family_indices: QueueFamilyIndices,
    
    // Memory management con VMA
    pub allocator: Allocator,
    
    // Swapchain con triple buffering
    pub swapchain: SwapchainWrapper,
    pub frame_resources: Vec<FrameResources>,
    pub current_frame: usize,
    
    // Sincronización moderna (timeline semaphores)
    pub timeline_semaphore: vk::Semaphore,
    pub timeline_value: u64,
    
    // Pipeline cache para zero-stutter
    pub pipeline_cache: vk::PipelineCache,
    
    // Descriptor indexing (bindless)
    pub descriptor_pool: vk::DescriptorPool,
    pub descriptor_set_layout: vk::DescriptorSetLayout,
    
    // Configuración
    pub config: VulkanConfig,
    
    // Estado
    pub is_initialized: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct QueueFamilyIndices {
    pub graphics_family: u32,
    pub compute_family: u32,
    pub transfer_family: u32,
}

impl UltraFastVulkanEngine {
    /// 🎯 Crear instancia del motor con Vulkan 1.3
    pub fn new(config: VulkanConfig) -> Result<Self, Box<dyn std::error::Error>> {
        info!("🚀 Inicializando UltraFastVulkanEngine con Vulkan 1.3...");
        
        // 1. Cargar Entry point
        let entry = Entry::linked();
        
        // 2. Crear instancia con Vulkan 1.3 + extensiones críticas
        let instance = Self::create_instance(&entry, &config)?;
        
        // 3. Seleccionar dispositivo físico con features necesarias
        let (physical_device, queue_indices) = Self::pick_physical_device(&entry, &instance, &config)?;
        
        // 4. Crear dispositivo lógico con queues separadas
        let device = Self::create_logical_device(&entry, physical_device, &queue_indices, &config)?;
        let device = Arc::new(device);
        
        // 5. Obtener handles de queues
        let graphics_queue = unsafe { device.get_device_queue(queue_indices.graphics_family, 0) };
        let compute_queue = unsafe { device.get_device_queue(queue_indices.compute_family, 0) };
        let transfer_queue = unsafe { device.get_device_queue(queue_indices.transfer_family, 0) };
        
        // 6. Inicializar VMA allocator
        let allocator = Self::create_vma_allocator(&instance, &physical_device, &device)?;
        
        // 7. Crear swapchain con triple buffering
        let swapchain = SwapchainWrapper::new(&device, physical_device, &config)?;
        
        // 8. Crear frame resources (triple buffering)
        let frame_resources = Self::create_frame_resources(&device, if config.triple_buffering { 3 } else { 2 })?;
        
        // 9. Crear timeline semaphore para sincronización moderna
        let timeline_semaphore = Self::create_timeline_semaphore(&device)?;
        
        // 10. Crear pipeline cache
        let pipeline_cache = Self::create_pipeline_cache(&device)?;
        
        // 11. Setup descriptor indexing (bindless)
        let (descriptor_pool, descriptor_set_layout) = Self::create_bindless_descriptors(&device)?;
        
        info!("✅ UltraFastVulkanEngine inicializado exitosamente");
        debug!("  - GPU: {:?}", Self::get_device_name(&entry, physical_device));
        debug!("  - Resolution: {}x{}", config.width, config.height);
        debug!("  - Triple buffering: {}", config.triple_buffering);
        debug!("  - Validation layers: {}", config.enable_validation);
        
        Ok(Self {
            entry,
            instance,
            device,
            physical_device,
            graphics_queue,
            compute_queue,
            transfer_queue,
            queue_family_indices: queue_indices,
            allocator,
            swapchain,
            frame_resources,
            current_frame: 0,
            timeline_semaphore,
            timeline_value: 0,
            pipeline_cache,
            descriptor_pool,
            descriptor_set_layout,
            config,
            is_initialized: true,
        })
    }
    
    /// Crear instancia Vulkan con features 1.3
    fn create_instance(entry: &Entry, config: &VulkanConfig) -> Result<Instance, Box<dyn std::error::Error>> {
        let app_info = vk::ApplicationInfo::builder()
            .application_name(b"Noir Browser\0")
            .application_version(vk::make_api_version(0, 1, 3, 0))
            .engine_name(b"NoChromium Engine\0")
            .engine_version(vk::make_api_version(0, 1, 3, 0))
            .api_version(vk::API_VERSION_1_3);
        
        // Extensiones requeridas para Vulkan 1.3 + optimizaciones
        let mut extensions = vec![
            ash::khr::surface::NAME,
            ash::khr::swapchain::NAME,
            ash::khr::timeline_semaphore::NAME,  // Sincronización moderna
            ash::ext::descriptor_indexing::NAME,  // Bindless resources
            ash::khr::dynamic_rendering::NAME,    // Dynamic rendering (sin render passes fijos)
        ];
        
        if cfg!(target_os = "windows") {
            extensions.push(ash::khr::win32_surface::NAME);
        }
        
        // Layers de validación en debug
        let layers = if config.enable_validation {
            vec![b"VK_LAYER_KHRONOS_validation\0"]
        } else {
            vec![]
        };
        
        let create_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_extension_names(&extensions)
            .enabled_layer_names(&layers);
        
        unsafe { Ok(entry.create_instance(&create_info, None)?) }
    }
    
    /// Seleccionar GPU con features necesarias
    fn pick_physical_device(
        entry: &Entry,
        instance: &Instance,
        config: &VulkanConfig,
    ) -> Result<(vk::PhysicalDevice, QueueFamilyIndices), Box<dyn std::error::Error>> {
        let devices = unsafe { instance.enumerate_physical_devices()? };
        
        for device in devices {
            if let Some(indices) = Self::check_device_suitability(entry, instance, device, config) {
                info!("🎮 GPU seleccionada: {:?}", Self::get_device_name(entry, device));
                return Ok((device, indices));
            }
        }
        
        Err("❌ No se encontró GPU compatible con Vulkan 1.3 + features requeridas".into())
    }
    
    /// Verificar si un dispositivo es adecuado
    fn check_device_suitability(
        entry: &Entry,
        instance: &Instance,
        device: vk::PhysicalDevice,
        config: &VulkanConfig,
    ) -> Option<QueueFamilyIndices> {
        // Verificar Vulkan 1.3
        let api_version = unsafe { instance.get_physical_device_properties(device).api_version };
        if api_version < vk::API_VERSION_1_3 {
            return None;
        }
        
        // Verificar features críticas
        let features = unsafe { instance.get_physical_device_features(device) };
        if !features.multi_draw_indirect != 0 {
            return None;
        }
        
        // Verificar extensiones requeridas
        let extensions = unsafe { instance.enumerate_device_extension_properties(device, None) }
            .ok()?
            .iter()
            .map(|ext| std::ffi::CStr::from_ptr(ext.extension_name.as_ptr()))
            .collect::<Vec<_>>();
        
        let required = [
            ash::khr::swapchain::NAME,
            ash::khr::timeline_semaphore::NAME,
            ash::ext::descriptor_indexing::NAME,
            ash::khr::dynamic_rendering::NAME,
        ];
        
        if !required.iter().all(|req| extensions.iter().any(|ext| ext.to_str().ok() == Some(req))) {
            return None;
        }
        
        // Encontrar queue families
        let queue_families = unsafe { instance.get_physical_device_queue_family_properties(device) };
        
        let mut indices = QueueFamilyIndices {
            graphics_family: u32::MAX,
            compute_family: u32::MAX,
            transfer_family: u32::MAX,
        };
        
        for (i, family) in queue_families.iter().enumerate() {
            let i = i as u32;
            
            if family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                indices.graphics_family = i;
            }
            if family.queue_flags.contains(vk::QueueFlags::COMPUTE) && !family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                indices.compute_family = i;  // Compute queue dedicado
            }
            if family.queue_flags.contains(vk::QueueFlags::TRANSFER) && !family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                indices.transfer_family = i;  // Transfer queue dedicado
            }
        }
        
        // Fallback: usar graphics queue para compute/transfer si no hay dedicadas
        if indices.compute_family == u32::MAX {
            indices.compute_family = indices.graphics_family;
        }
        if indices.transfer_family == u32::MAX {
            indices.transfer_family = indices.graphics_family;
        }
        
        if indices.graphics_family == u32::MAX {
            return None;
        }
        
        Some(indices)
    }
    
    /// Crear dispositivo lógico con queues separadas
    fn create_logical_device(
        entry: &Entry,
        physical_device: vk::PhysicalDevice,
        queue_indices: &QueueFamilyIndices,
        config: &VulkanConfig,
    ) -> Result<Device, Box<dyn std::error::Error>> {
        let instance = unsafe { entry.create_instance(&vk::InstanceCreateInfo::default(), None)? };
        
        // Features de Vulkan 1.3
        let vulkan_13_features = vk::PhysicalDeviceVulkan13Features::builder()
            .synchronization2(true)
            .dynamic_rendering(true)
            .maintenance4(true)
            .build();
        
        // Features estándar
        let features = vk::PhysicalDeviceFeatures::builder()
            .multi_draw_indirect(true)
            .draw_indirect_first_instance(true)
            .fill_mode_non_solid(true)
            .build();
        
        // Crear queue create infos
        let mut queue_create_infos = Vec::new();
        let mut unique_families = std::collections::HashSet::new();
        
        for &family in &[queue_indices.graphics_family, queue_indices.compute_family, queue_indices.transfer_family] {
            if unique_families.insert(family) {
                let priority = 1.0f32;
                let queue_info = vk::DeviceQueueCreateInfo::builder()
                    .queue_family_index(family)
                    .queue_priorities(&[priority])
                    .build();
                queue_create_infos.push(queue_info);
            }
        }
        
        // Extensiones del dispositivo
        let device_extensions = [
            ash::khr::swapchain::NAME,
            ash::khr::timeline_semaphore::NAME,
            ash::ext::descriptor_indexing::NAME,
            ash::khr::dynamic_rendering::NAME,
        ];
        
        // Next chain para features 1.3
        let mut device_create_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(&queue_create_infos)
            .enabled_extension_names(&device_extensions)
            .push_next(&vulkan_13_features)
            .enabled_features(&features);
        
        unsafe { Ok(instance.create_device(physical_device, &device_create_info, None)?) }
    }
    
    /// Crear allocator VMA para gestión de memoria optimizada
    fn create_vma_allocator(
        instance: &Instance,
        physical_device: &vk::PhysicalDevice,
        device: &Device,
    ) -> Result<Allocator, Box<dyn std::error::Error>> {
        let allocator = Allocator::new(&AllocatorCreateDesc {
            instance: instance.clone(),
            device: device.clone(),
            physical_device: *physical_device,
            debug_settings: gpu_allocator::vulkan::AllocatorDebugSettings {
                log_allocations: cfg!(debug_assertions),
                log_frees: cfg!(debug_assertions),
                log_warnings: true,
            },
            ..Default::default()
        })?;
        
        Ok(allocator)
    }
    
    /// Crear timeline semaphore para sincronización moderna
    fn create_timeline_semaphore(device: &Device) -> Result<vk::Semaphore, Box<dyn std::error::Error>> {
        let timeline_create_info = vk::SemaphoreTypeCreateInfo::builder()
            .semaphore_type(vk::SemaphoreType::TIMELINE)
            .initial_value(0);
        
        let create_info = vk::SemaphoreCreateInfo::builder()
            .push_next(&timeline_create_info);
        
        unsafe { Ok(device.create_semaphore(&create_info, None)?) }
    }
    
    /// Crear pipeline cache para zero-stutter
    fn create_pipeline_cache(device: &Device) -> Result<vk::PipelineCache, Box<dyn std::error::Error>> {
        let create_info = vk::PipelineCacheCreateInfo::builder();
        unsafe { Ok(device.create_pipeline_cache(&create_info, None)?) }
    }
    
    /// Crear descriptor pool para bindless resources
    fn create_bindless_descriptors(device: &Device) -> Result<(vk::DescriptorPool, vk::DescriptorSetLayout), Box<dyn std::error::Error>> {
        // Bindless: un solo descriptor set con arrays grandes
        let sizes = [
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::STORAGE_BUFFER,
                descriptor_count: 1000,  // Hasta 1000 buffers bindless
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::STORAGE_IMAGE,
                descriptor_count: 500,  // Hasta 500 imágenes bindless
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::SAMPLER,
                descriptor_count: 100,
            },
        ];
        
        let pool_info = vk::DescriptorPoolCreateInfo::builder()
            .pool_sizes(&sizes)
            .max_sets(1)  // Un solo set bindless
            .flags(vk::DescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET);
        
        let descriptor_pool = unsafe { device.create_descriptor_pool(&pool_info, None)? };
        
        // Layout con descriptor indexing
        let binding_flags = vk::DescriptorBindingFlags::PARTIALLY_BOUND | vk::DescriptorBindingFlags::UPDATE_AFTER_BIND;
        let binding_flags_create_info = vk::DescriptorSetLayoutBindingFlagsCreateInfo::builder()
            .binding_flags(&[binding_flags]);
        
        let binding = vk::DescriptorSetLayoutBinding::builder()
            .binding(0)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .descriptor_count(1000)
            .stage_flags(vk::ShaderStageFlags::ALL)
            .build();
        
        let layout_info = vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(&[binding])
            .push_next(&binding_flags_create_info)
            .flags(vk::DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL);
        
        let descriptor_set_layout = unsafe { device.create_descriptor_set_layout(&layout_info, None)? };
        
        Ok((descriptor_pool, descriptor_set_layout))
    }
    
    /// 🎨 Renderizar un frame con zero-copy y multi-draw indirect
    pub fn render_frame(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let frame_idx = self.current_frame;
        let frame = &mut self.frame_resources[frame_idx];
        
        // Esperar frame anterior con timeline semaphore (no-blocking)
        self.wait_for_frame(frame_idx)?;
        
        // Adquirir imagen del swapchain
        let (image_index, _) = unsafe {
            self.swapchain.loader.acquire_next_image(
                self.swapchain.handle,
                u64::MAX,
                frame.image_available_semaphore,
                vk::Fence::null(),
            )?
        };
        
        // Reset command buffer
        unsafe {
            self.device.reset_command_pool(
                frame.command_pool,
                vk::CommandPoolResetFlags::empty(),
            )?;
        }
        
        // Begin command buffer
        let begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        unsafe { self.device.begin_command_buffer(frame.command_buffer, &begin_info)?; }
        
        // 🆕 Dynamic rendering (sin render pass pre-definido)
        let color_attachment = vk::RenderingAttachmentInfo::builder()
            .image_view(self.swapchain.image_views[image_index as usize])
            .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .clear_value(vk::ClearValue {
                color: vk::ClearColorValue { float32: [0.12, 0.13, 0.14, 1.0] },
            });
        
        let rendering_info = vk::RenderingInfo::builder()
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.swapchain.extent,
            })
            .layer_count(1)
            .color_attachments(&[color_attachment.build()]);
        
        unsafe {
            self.device.cmd_begin_rendering(frame.command_buffer, &rendering_info);
        }
        
        // Bind pipeline + bindless descriptors
        self.bind_graphics_pipeline(frame.command_buffer);
        self.bind_bindless_descriptors(frame.command_buffer);
        
        // 🚀 Multi-draw indirect: un solo draw call para toda la UI
        self.execute_multi_draw_indirect(frame.command_buffer);
        
        unsafe {
            self.device.cmd_end_rendering(frame.command_buffer);
        }
        
        // End command buffer
        unsafe { self.device.end_command_buffer(frame.command_buffer)?; }
        
        // Submit con timeline semaphore
        self.submit_frame(frame_idx, image_index)?;
        
        // Presentar
        unsafe {
            self.swapchain.loader.queue_present(
                self.graphics_queue,
                &vk::PresentInfoKHR::builder()
                    .swapchains(&[self.swapchain.handle])
                    .image_indices(&[image_index])
                    .wait_semaphores(&[frame.render_finished_semaphore]),
            )?;
        }
        
        // Avanzar frame
        self.current_frame = (self.current_frame + 1) % self.frame_resources.len();
        self.timeline_value += 1;
        
        Ok(())
    }
    
    /// Ejecutar multi-draw indirect (optimización crítica)
    fn execute_multi_draw_indirect(&self, command_buffer: vk::CommandBuffer) {
        // Aquí se ejecutaría el buffer con todos los draw commands pre-ensamblados
        // Esto reduce de ~500 draw calls a ~1 por frame
        unsafe {
            // self.device.cmd_draw_indirect(
            //     command_buffer,
            //     self.draw_indirect_buffer,
            //     0,
            //     self.draw_count,
            //     std::mem::size_of::<vk::DrawIndirectCommand>() as u32,
            // );
        }
    }
    
    /// Esperar frame anterior con timeline semaphore
    fn wait_for_frame(&self, frame_idx: usize) -> Result<(), Box<dyn std::error::Error>> {
        let frame = &self.frame_resources[frame_idx];
        
        if frame.in_flight {
            let values = [self.timeline_value - self.frame_resources.len() as u64];
            let wait_info = vk::SemaphoreWaitInfo::builder()
                .semaphores(&[self.timeline_semaphore])
                .values(&values);
            
            unsafe {
                self.device.wait_semaphores(self.device.clone(), &wait_info, u64::MAX)?;
            }
        }
        
        Ok(())
    }
    
    /// Submit frame con timeline semaphore
    fn submit_frame(&mut self, frame_idx: usize, image_index: u32) -> Result<(), Box<dyn std::error::Error>> {
        let frame = &mut self.frame_resources[frame_idx];
        
        // Reset fence si está signaled
        unsafe {
            self.device.reset_fences(&[frame.fence])?;
        }
        
        // Pipeline stages para wait/signal
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let wait_semaphores = [frame.image_available_semaphore];
        let signal_semaphores = [frame.render_finished_semaphore, self.timeline_semaphore];
        let signal_values = [0, self.timeline_value + 1];  // Timeline value para signal
        
        // Submit info con timeline semaphore
        let timeline_info = vk::TimelineSemaphoreSubmitInfo::builder()
            .signal_semaphore_values(&signal_values);
        
        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&wait_stages)
            .command_buffers(&[frame.command_buffer])
            .signal_semaphores(&signal_semaphores)
            .push_next(&timeline_info);
        
        unsafe {
            self.device.queue_submit(
                self.graphics_queue,
                &[submit_info.build()],
                frame.fence,
            )?;
        }
        
        frame.in_flight = true;
        
        Ok(())
    }
    
    /// Helper: obtener nombre del dispositivo
    fn get_device_name(entry: &Entry, physical_device: vk::PhysicalDevice) -> String {
        unsafe {
            let properties = entry.get_physical_device_properties(physical_device);
            std::ffi::CStr::from_ptr(properties.device_name.as_ptr())
                .to_string_lossy()
                .into_owned()
        }
    }
    
    /// Cleanup
    pub fn cleanup(&mut self) {
        info!("🧹 Limpiando UltraFastVulkanEngine...");
        
        unsafe {
            // Esperar que la GPU termine
            self.device.device_wait_idle().ok();
            
            // Destruir recursos en orden inverso a creación
            for frame in &self.frame_resources {
                self.device.destroy_semaphore(frame.image_available_semaphore, None);
                self.device.destroy_semaphore(frame.render_finished_semaphore, None);
                self.device.destroy_fence(frame.fence, None);
                self.device.destroy_command_pool(frame.command_pool, None);
            }
            
            self.device.destroy_semaphore(self.timeline_semaphore, None);
            self.device.destroy_pipeline_cache(self.pipeline_cache, None);
            self.device.destroy_descriptor_pool(self.descriptor_pool, None);
            self.device.destroy_descriptor_set_layout(self.descriptor_set_layout, None);
            
            self.swapchain.cleanup(&self.device);
            
            self.device.destroy_device(None);
            self.instance.destroy_instance(None);
        }
        
        info!("✅ Cleanup completado");
    }
}

/// Wrapper para swapchain con gestión simplificada
pub struct SwapchainWrapper {
    pub handle: vk::SwapchainKHR,
    pub loader: ash::khr::swapchain::Swapchain,
    pub images: Vec<vk::Image>,
    pub image_views: Vec<vk::ImageView>,
    pub extent: vk::Extent2D,
    pub format: vk::Format,
}

impl SwapchainWrapper {
    pub fn new(
        device: &Device,
        physical_device: vk::PhysicalDevice,
        config: &VulkanConfig,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Implementación simplificada - en producción sería más robusta
        let swapchain_loader = ash::khr::swapchain::Swapchain::new(
            &device.entry().unwrap(),
            &device.instance(),
            &device,
        );
        
        // ... crear swapchain real aquí
        
        Ok(Self {
            handle: vk::SwapchainKHR::null(),  // Placeholder
            loader: swapchain_loader,
            images: vec![],
            image_views: vec![],
            extent: vk::Extent2D { width: config.width, height: config.height },
            format: vk::Format::B8G8R8A8_SRGB,
        })
    }
    
    pub fn cleanup(&mut self, device: &Device) {
        unsafe {
            for view in self.image_views.drain(..) {
                device.destroy_image_view(view, None);
            }
            if self.handle != vk::SwapchainKHR::null() {
                self.loader.destroy_swapchain(self.handle, None);
            }
        }
    }
}
