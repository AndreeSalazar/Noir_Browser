# ✅ Fase 0: Vulkan Ultra-Fast Base - Checklist de Implementación

> **Objetivo**: Motor Vulkan 1.3 funcional con <8ms frame time, triple buffering, zero-copy pipeline  
> **Duración estimada**: 2 semanas  
> **Estado actual**: 🟡 Estructura creada, métodos stub implementados

---

## 🎯 Criterios de Éxito (Definition of Done)

- [ ] `cargo build --features debug_vulkan` compila sin errores
- [ ] `cargo run --features debug_vulkan` abre ventana y muestra "Noir Browser"
- [ ] Frame time medido: <8ms promedio (125+ FPS)
- [ ] Triple buffering estable sin tearing
- [ ] Validation layers de Vulkan reportan 0 errores críticos
- [ ] RAM del proceso: <100MB en idle

---

## 🔧 Métodos Stub que Necesitan Implementación

Estos métodos en `src/vulkan_engine/core.rs` tienen `unimplemented!()` y deben completarse:

### 1. `create_instance()` - Línea ~200
```rust
// Requerimientos:
// - Habilitar VK_KHR_surface + VK_KHR_win32_surface (Windows)
// - Validation layers si feature debug_vulkan está activo
// - VK_EXT_debug_utils para logging

// Pasos:
// 1. Obtener layers disponibles con entry.enumerate_instance_layer_properties()
// 2. Filtrar "VK_LAYER_KHRONOS_validation" si enable_validation
// 3. Crear InstanceCreateInfo con extensions necesarias
// 4. entry.create_instance()
```

### 2. `create_surface()` - Línea ~210
```rust
// Windows-specific con winit:
use raw_window_handle::HasRawWindowHandle;
use ash::vk;

let raw_window_handle = window.raw_window_handle();
// Usar ash_window::create_surface() helper o implementar manualmente
// con vk::Win32SurfaceCreateInfoKHR
```

### 3. `pick_physical_device()` - Línea ~220
```rust
// Criterios de selección (en orden de prioridad):
// 1. Soporte para Vulkan 1.3+ (vk::API_VERSION_1_3)
// 2. Queue families: graphics + present + (opcional) compute separado
// 3. Extensions: VK_KHR_swapchain, VK_KHR_timeline_semaphore
// 4. Features: shaderInt64, descriptorIndexing (para bindless)
// 5. Preferir GPU dedicada sobre integrada

// Retornar QueueFamilyIndices con gráficos, present y compute
```

### 4. `create_logical_device()` - Línea ~235
```rust
// Configurar:
// - Queue create infos para graphics/present/compute
// - Device extensions: VK_KHR_swapchain, VK_KHR_timeline_semaphore
// - Device features: enable descriptorIndexing si enable_bindless
// - pNext chain con VkPhysicalDeviceVulkan13Features si Vulkan 1.3
```

### 5. `create_swapchain()` - Línea ~250
```rust
// Pasos clave:
// 1. Obtener surface capabilities con vkGetPhysicalDeviceSurfaceCapabilitiesKHR
// 2. Seleccionar formato: preferir B8G8R8A8_SRGB o R8G8B8A8_SRGB
// 3. Seleccionar present mode: MAILBOX_KHR (si disponible) o FIFO_KHR
// 4. Extent: usar currentExtent si no es u32::MAX, sino clamping de ventana
// 5. Crear swapchain con VK_SHARING_MODE_EXCLUSIVE
// 6. Obtener imágenes con vkGetSwapchainImagesKHR
```

### 6. `create_sync_objects()` - Línea ~270
```rust
// Crear por cada frame in flight:
// - vk::Semaphore para image_available (binary)
// - vk::Semaphore para render_finished (binary)  
// - vk::Semaphore para timeline (con VK_SEMAPHORE_TYPE_TIMELINE)
// - vk::Fence para CPU-GPU sync

// Para timeline semaphore:
let timeline_create_info = vk::SemaphoreTypeCreateInfo::default()
    .semaphore_type(vk::SemaphoreType::TIMELINE)
    .initial_value(0);
```

### 7. `create_command_buffers()` - Línea ~290
```rust
// Para cada frame:
// 1. vk::CommandPoolCreateInfo con queue_family_index
// 2. device.create_command_pool()
// 3. vk::CommandBufferAllocateInfo con level = PRIMARY
// 4. device.allocate_command_buffers()
```

### 8. `create_descriptors()` - Línea ~305
```rust
// Si enable_bindless:
// - Descriptor set layout con VK_DESCRIPTOR_BINDING_PARTIALLY_BOUND
// - Descriptor pool con tamaño grande para array de texturas
// - Usar descriptor indexing para acceso bindless

// Si no bindless: layout tradicional por material/texture
```

### 9. `recreate_swapchain()` - Línea ~320
```rust
// Llamar cuando:
// - Window resized
// - vkAcquireNextImage retorna SUBOPTIMAL_KHR o ERROR_OUT_OF_DATE_KHR

// Pasos:
// 1. device.device_wait_idle()
// 2. Liberar recursos antiguos del swapchain
// 3. Recrear swapchain con nuevo extent
// 4. Recrear framebuffers/image views si es necesario
```

---

## 🧪 Tests Mínimos para Fase 0

Crear en `tests/vulkan_init.rs`:

```rust
#[test]
#[ignore] // Requiere GPU con Vulkan 1.3
fn test_vulkan_init() {
    use winit::event_loop::EventLoop;
    use no_chromium::vulkan_engine::UltraFastVulkanEngine;
    
    let event_loop = EventLoop::new().unwrap();
    // Crear ventana headless o mock para testing
    // ...
}

#[test]
fn test_config_defaults() {
    use no_chromium::vulkan_engine::core::VulkanConfig;
    let config = VulkanConfig::default();
    assert_eq!(config.max_frames_in_flight, 3);
    assert_eq!(config.enable_bindless, true);
}
```

---

## 📊 Métricas a Medir

Usar `tracing` + `criterion` para benchmarks:

```rust
// En render_frame(), medir:
let frame_start = std::time::Instant::now();
// ... render logic ...
let frame_time = frame_start.elapsed();
tracing::debug!("[perf] Frame time: {:?}", frame_time);

// Benchmark con criterion:
#[bench]
fn bench_frame_render(b: &mut Bencher) {
    // Setup engine mock
    b.iter(|| engine.render_frame());
}
```

**Objetivos**:
- Frame time p50: <5ms
- Frame time p99: <8ms  
- CPU usage idle: <0.5%
- GPU memory: <200MB total

---

## 🚀 Comandos Útiles para Desarrollo

```powershell
# Build con validation layers (debug)
cargo build --features debug_vulkan

# Run con logging detallado
RUST_LOG=debug cargo run --features debug_vulkan

# Verificar Vulkan instalado
vulkaninfo --summary

# Benchmark de frame time
cargo bench --bench frame_time --features ultrafast

# Profiling con tracy (opcional)
cargo run --features tracy --profile release
```

---

## ⚠️ Errores Comunes y Soluciones

| Error | Causa Probable | Solución |
|-------|---------------|----------|
| `VK_ERROR_INITIALIZATION_FAILED` | Driver Vulkan desactualizado | Actualizar GPU drivers + Vulkan SDK 1.3+ |
| `VK_ERROR_EXTENSION_NOT_PRESENT` | Falta extensión en create_instance | Verificar `enumerate_instance_extension_properties()` |
| Validation layer no encontrada | Vulkan SDK sin validation layers | Reinstalar SDK con componente "Validation Layers" |
| Swapchain creation fails | Surface capabilities no compatibles | Verificar minImageCount, supported formats |
| Timeline semaphore error | Vulkan 1.2+ requerido | Check `vk::API_VERSION_1_2` en physical device |

---

## 📁 Archivos Clave para Fase 0

```
No-Chromium/
├── Cargo.toml              # ✅ Actualizado con workspace deps
├── src/
│   ├── main.rs            # ✅ Entry point minimal
│   ├── app.rs             # ✅ Winit event loop + Vulkan init
│   └── vulkan_engine/
│       ├── mod.rs         # ✅ Module declaration
│       └── core.rs        # 🟡 UltraFastVulkanEngine (stubs)
└── tests/
    └── vulkan_init.rs     # 🔲 Tests (pendiente)
```

---

## ✅ Próximo Paso Inmediato

**Implementar `create_instance()` en `src/vulkan_engine/core.rs`**:

```rust
fn create_instance(entry: &Entry, window: &Window, config: &VulkanConfig) -> Result<Instance> {
    use ash::vk;
    
    // 1. Obtener layers disponibles
    let available_layers = unsafe { entry.enumerate_instance_layer_properties()? };
    
    // 2. Filtrar validation layer si está habilitada
    let layers: Vec<&str> = if config.enable_validation {
        let validation_layer = b"VK_LAYER_KHRONOS_validation\0";
        if available_layers.iter().any(|l| {
            unsafe { std::ffi::CStr::from_ptr(l.layer_name.as_ptr()) } == validation_layer
        }) {
            vec![std::ffi::CStr::from_bytes_with_nul(validation_layer).unwrap()]
        } else {
            tracing::warn!("Validation layer requested but not available");
            vec![]
        }
    } else {
        vec![]
    };
    
    // 3. Extensions requeridas para ventana + debug
    let mut extensions = ash_window::enumerate_required_extensions(
        window.raw_display_handle()
    )?.iter().map(|e| e.as_ptr()).collect::<Vec<_>>();
    
    if config.enable_debug_utils {
        extensions.push(vk::EXT_DEBUG_UTILS_EXTENSION_NAME.as_ptr());
    }
    
    // 4. Crear instancia
    let app_info = vk::ApplicationInfo::default()
        .application_name(b"Noir Browser\0")
        .application_version(vk::make_api_version(0, 0, 1, 0))
        .engine_name(b"Noir Vulkan Engine\0")
        .engine_version(vk::make_api_version(0, 1, 0, 0))
        .api_version(vk::API_VERSION_1_3);
    
    let create_info = vk::InstanceCreateInfo::default()
        .application_info(&app_info)
        .enabled_layer_names(&layers)
        .enabled_extension_names(&extensions);
    
    let instance = unsafe { entry.create_instance(&create_info, None)? };
    
    tracing::info!("[vulkan] Instance created: Noir Browser");
    Ok(instance)
}
```

**Nota**: Necesitarás agregar `ash-window = "0.9"` a `Cargo.toml` como dependencia para `ash_window::enumerate_required_extensions`.

---

> 💡 **Consejo**: Implementa un método a la vez, compila y prueba después de cada cambio. Usa `#[cfg(feature = "debug_vulkan")]` para código de debug que no debe estar en release.

**¿Listo para implementar `create_instance()`?** 🚀
