#!/usr/bin/env bash
# scripts/setup_phase0.sh
# Setup inicial para Fase 0: Foundation (allocator + Vulkan optimizado)

set -e

echo "🚀 Configurando Noir Browser v2.0 - Fase 0: Foundation"

# 1. Actualizar Cargo.toml con perfil optimizado
echo "📦 Optimizando Cargo.toml para rendimiento máximo..."
cat >> Cargo.toml << 'EOF'

# === OPTIMIZACIONES PARA PRODUCCIÓN ===
[profile.release]
lto = "fat"              # Link-time optimization agresiva
codegen-units = 1        # Mejor optimización a cambio de build más lento
panic = "abort"          # Sin unwind tables, binario más pequeño
strip = true             # Eliminar símbolos de debug
opt-level = 3            # Máxima optimización

[profile.bench]
inherits = "release"
debug = true             # Mantener debug info para profiling con Tracy

# === DEPENDENCIAS OPTIMIZADAS ===
[dependencies]
# Vulkan core
ash = { version = "0.38", features = ["loaded", "debug"] }
gpu-alloc = "0.6"
gpu-alloc-ash = "0.7"

# Async runtime optimizado
tokio = { version = "1.35", features = ["full", "tracing"], default-features = false }

# Memory allocators
bumpalo = { version = "3.14", features = ["collections"] }
arena = "0.1"

# Networking HTTP/3
quinn = "0.10"
rustls = { version = "0.22", features = ["ring"], default-features = false }

# Profiling
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Search & caching
dashmap = "5.5"
lru = "0.12"

# Utilities
bytemuck = { version = "1.14", features = ["derive"] }
zerocopy = "0.7"

[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }
tracy-client = "0.16"

[[bench]]
name = "parsing_bench"
harness = false

[[bench]]
name = "layout_bench"
harness = false
EOF

# 2. Crear estructura de directorios optimizada
echo "📁 Creando estructura de directorios GPU-First..."
mkdir -p src/core/{gpu, memory, async}
mkdir -p src/parsing/{html, css, dom, js}
mkdir -p src/layout
mkdir -p src/search/{providers, cache}
mkdir -p src/network
mkdir -p shaders
mkdir -p tests/{perf,wpt,search}

# 3. Crear build.rs para compilación de shaders en compile-time
echo "⚙️  Generando build.rs para shader compilation..."
cat > build.rs << 'EOF'
use std::{env, fs, path::Path};

fn main() {
    println!("cargo:rerun-if-changed=shaders/");
    
    let out_dir = env::var("OUT_DIR").unwrap();
    let shader_dir = Path::new("shaders");
    
    // Compilar todos los .comp y .frag a SPIR-V
    for entry in fs::read_dir(shader_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        
        if path.extension().map_or(false, |ext| ext == "comp" || ext == "frag") {
            let shader_name = path.file_stem().unwrap().to_str().unwrap();
            let output_path = Path::new(&out_dir).join(format!("{}.spv", shader_name));
            
            // Usar glslc si está disponible, sino skip para desarrollo
            if let Ok(status) = std::process::Command::new("glslc")
                .arg(&path)
                .arg("-o")
                .arg(&output_path)
                .status()
            {
                if status.success() {
                    println!("✅ Compilado: {} → {:?}", path.display(), output_path);
                }
            } else {
                println!("⚠️  glslc no encontrado, usando shaders pre-compilados para dev");
            }
        }
    }
}
EOF

# 4. Crear archivo de configuración de Vulkan optimizado
echo "🎮 Generando vulkan_config.rs con optimizaciones..."
cat > src/core/gpu/vulkan_config.rs << 'EOF'
use ash::{vk, Device};

/// Configuración optimizada de Vulkan para Noir Browser v2.0
/// Principio: Zero-overhead, máximo paralelismo GPU
pub struct VulkanConfig {
    pub enable_validation: bool,
    pub enable_debug_utils: bool,
    pub async_compute_queue: bool,
    pub descriptor_pool_size: u32,
    pub max_frames_in_flight: u32,
}

impl Default for VulkanConfig {
    fn default() -> Self {
        Self {
            enable_validation: cfg!(debug_assertions),
            enable_debug_utils: cfg!(debug_assertions),
            async_compute_queue: true,  // Clave para parsing paralelo
            descriptor_pool_size: 1000,
            max_frames_in_flight: 3,    // Triple buffering optimizado
        }
    }
}

impl VulkanConfig {
    pub fn instance_extensions(&self) -> Vec<&'static str> {
        let mut extensions = vec![
            vk::EXT_DEBUG_UTILS_EXTENSION_NAME,
            vk::KHR_SURFACE_EXTENSION_NAME,
        ];
        
        // Platform-specific surface extensions
        #[cfg(target_os = "windows")]
        extensions.push(vk::KHR_WIN32_SURFACE_EXTENSION_NAME);
        #[cfg(target_os = "linux")]
        extensions.push(vk::KHR_XLIB_SURFACE_EXTENSION_NAME);
        #[cfg(target_os = "macos")]
        extensions.push(vk::EXT_METAL_SURFACE_EXTENSION_NAME);
        
        extensions
    }
    
    pub fn device_extensions(&self) -> Vec<&'static str> {
        vec![
            vk::KHR_SWAPCHAIN_EXTENSION_NAME,
            // Compute shaders para parsing/layout
            vk::KHR_SHADER_FLOAT_CONTROLS_EXTENSION_NAME,
        ]
    }
    
    pub fn queue_priorities(&self) -> Vec<f32> {
        // Prioridad alta para graphics, media para compute (parsing)
        vec![1.0, 0.7]
    }
}

/// Helper para crear pipelines optimizados
pub fn create_pipeline_cache(device: &Device) -> vk::PipelineCache {
    unsafe {
        device.create_pipeline_cache(
            &vk::PipelineCacheCreateInfo::builder()
                .initial_data(&[]),  // Cache vacío inicial, se llena en runtime
            None,
        ).expect("Failed to create pipeline cache")
    }
}
EOF

# 5. Crear allocator optimizado (piedra angular de zero-copy)
echo "🧠 Generando NoirAllocator (zero-copy arena)..."
cat > src/core/memory/allocator.rs << 'EOF'
use std::sync::atomic::{AtomicUsize, Ordering};
use ash::{vk, Device};

/// Arena allocator para recursos de frame único
/// Zero-copy: CPU escribe → GPU lee directamente desde host-visible memory
pub struct GpuArena {
    buffer: vk::Buffer,
    allocation: vk::DeviceMemory,
    offset: AtomicUsize,
    capacity: usize,
    device: Device,
}

impl GpuArena {
    pub fn new(device: Device, capacity: usize) -> Self {
        // Crear buffer host-visible para uploads rápidos
        let buffer_info = vk::BufferCreateInfo::builder()
            .size(capacity as u64)
            .usage(vk::BufferUsageFlags::TRANSFER_SRC | vk::BufferUsageFlags::TRANSFER_DST)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        
        let buffer = unsafe { device.create_buffer(&buffer_info, None) }.unwrap();
        
        // Allocar memoria host-visible + device-local (ideal para uploads)
        let mem_reqs = unsafe { device.get_buffer_memory_requirements(buffer) };
        let allocation = unsafe {
            device.allocate_memory(
                &vk::MemoryAllocateInfo::builder()
                    .allocation_size(mem_reqs.size)
                    .memory_type_index(0), // Host-visible type (simplificado)
                None,
            ).unwrap()
        };
        
        unsafe { device.bind_buffer_memory(buffer, allocation, 0).unwrap(); }
        
        Self {
            buffer,
            allocation,
            offset: AtomicUsize::new(0),
            capacity,
            device,
        }
    }
    
    /// Allocar espacio en la arena - O(1), thread-safe
    pub fn alloc<T: Copy>(&self, items: &[T]) -> Option<vk::DeviceSize> {
        let size = std::mem::size_of_val(items);
        let current = self.offset.fetch_add(size, Ordering::Relaxed);
        
        if current + size > self.capacity {
            self.offset.fetch_sub(size, Ordering::Relaxed); // Rollback
            return None;
        }
        
        // Copiar datos directamente a memory mapeada (zero-copy path)
        unsafe {
            let ptr = self.device.map_memory(
                self.allocation,
                current as u64,
                size as u64,
                vk::MemoryMapFlags::empty(),
            ).unwrap();
            
            std::ptr::copy_nonoverlapping(
                items.as_ptr() as *const u8,
                ptr as *mut u8,
                size,
            );
            
            self.device.unmap_memory(self.allocation);
        }
        
        Some(current as u64)
    }
    
    /// Resetear arena para reuso en siguiente frame - O(1)
    pub fn reset(&self) {
        self.offset.store(0, Ordering::Relaxed);
    }
    
    pub fn buffer(&self) -> vk::Buffer { self.buffer }
    pub fn allocation(&self) -> vk::DeviceMemory { self.allocation }
}

/// Ring buffer para uploads asíncronos (evita stalls de GPU)
pub struct UploadRing {
    buffers: Vec<GpuArena>,
    current: AtomicUsize,
    frames_in_flight: usize,
}

impl UploadRing {
    pub fn new(device: Device, size_per_frame: usize, frames: usize) -> Self {
        let buffers = (0..frames)
            .map(|_| GpuArena::new(device.clone(), size_per_frame))
            .collect();
        
        Self {
            buffers,
            current: AtomicUsize::new(0),
            frames_in_flight: frames,
        }
    }
    
    pub fn current_frame(&self) -> &GpuArena {
        let idx = self.current.load(Ordering::Relaxed) % self.frames_in_flight;
        &self.buffers[idx]
    }
    
    pub fn advance_frame(&self) {
        // Resetear buffer anterior antes de reusarlo
        let idx = self.current.load(Ordering::Relaxed) % self.frames_in_flight;
        self.buffers[idx].reset();
        
        self.current.fetch_add(1, Ordering::Relaxed);
    }
}
EOF

# 6. Crear .gitignore actualizado
echo "🔒 Actualizando .gitignore..."
cat > .gitignore << 'EOF'
# Build outputs
target/
*.spv
*.exe
*.pdb

# Profiling data
*.tracy
tracy_*.log

# IDE
.vscode/
.idea/
*.swp

# OS
.DS_Store
Thumbs.db

# Secrets
*.env
!*.env.example

# Shaders compiled (keep source only)
!shaders/*.comp
!shaders/*.frag
*.spv
EOF

echo "✅ Fase 0 setup completo!"
echo ""
echo "🎯 Próximos pasos:"
echo "  1. Ejecutar: cargo build --release"
echo "  2. Verificar: cargo bench (requiere glslc para shaders)"
echo "  3. Profiling: Ejecutar con TRACY=1 para ver bottlenecks"
echo ""
echo "📚 Documentación: Ver RECONSTRUCCION_v2.md para roadmap completo"
