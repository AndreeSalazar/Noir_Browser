# 🚀 Noir Browser - Guía de Inicio Rápido (Reconstrucción Ultra-Fast)

> **Versión:** 0.2.0-ultrafast  
> **Objetivo:** Reconstruir Noir Browser para máximo rendimiento con Vulkan 1.3 + búsqueda web nativa

---

## 📋 Prerrequisitos

```bash
# Rust toolchain (1.75+)
rustup update stable
rustup default stable

# Vulkan SDK 1.3+ (https://vulkan.lunarg.com/)
# Windows: Instalar desde LunarG SDK
# Linux: sudo apt install libvulkan-dev vulkan-tools
# macOS: MoltenVK via Homebrew

# Dependencias de sistema (Windows)
# - Visual Studio Build Tools 2022 con "Desktop development with C++"
# - Windows SDK 10+

# Verificar instalación
cargo --version          # >= 1.75
vulkaninfo --version     # >= 1.3
```

---

## 🎯 Primeros Pasos (Día 1)

### 1. Clonar y preparar estructura

```bash
cd C:\Users\andre\OneDrive\Desktop\Noir_Browser\No-Chromium

# Verificar estructura nueva
tree /F /A  # Windows
# o
find . -type f -name "*.rs" | head -20  # Linux/Mac

# Limpiar builds anteriores
cargo clean
```

### 2. Compilar con features ultrafast

```bash
# Build de desarrollo con logging
cargo build --features debug_vulkan

# Build de release optimizado (para benchmarks)
cargo build --release --features ultrafast

# Ejecutar con tracing
RUST_LOG=no_chromium=debug cargo run --features debug_vulkan
```

### 3. Verificar Vulkan 1.3

```bash
# Ejecutar test de inicialización
cargo test vulkan_init --features debug_vulkan -- --nocapture

# Deberías ver:
# ✅ UltraFastVulkanEngine inicializado exitosamente
# 🎮 GPU seleccionada: "NVIDIA GeForce RTX XXX"
# - Resolution: 1920x1080
# - Triple buffering: true
```

---

## 🔧 Configuración Recomendada

### `Cargo.toml` - Features activas

```toml
[features]
default = ["ultrafast"]
ultrafast = []  # Vulkan 1.3 + bindless + async compute
debug_vulkan = ["ash/debug"]  # Validation layers (solo dev)
```

### Variables de entorno para debugging

```bash
# Habilitar validation layers de Vulkan
export VK_LAYER_PATH="C:\VulkanSDK\1.3.xx.x\Bin"  # Windows
export VK_INSTANCE_LAYERS=VK_LAYER_KHRONOS_validation

# Logging detallado
export RUST_LOG=no_chromium=trace,ash=debug

# Profiling con Tracy (opcional)
export TRACY_ENABLE=1
```

---

## 🧪 Tests Esenciales

```bash
# Test de inicialización Vulkan
cargo test test_vulkan_init --features debug_vulkan

# Test de memoria VMA (sin leaks)
cargo test test_vma_allocator --release

# Benchmark de frame time
cargo bench --bench frame_time --features ultrafast

# Test de búsqueda web (mock)
cargo test web_search_mock
```

### Ejemplo: Test de inicialización

```rust
// tests/vulkan_init.rs
#[test]
fn test_ultrafast_vulkan_init() {
    let config = VulkanConfig {
        width: 1280,
        height: 720,
        enable_validation: true,
        ..Default::default()
    };
    
    let engine = UltraFastVulkanEngine::new(config).unwrap();
    
    assert!(engine.is_initialized);
    assert!(engine.timeline_semaphore != vk::Semaphore::null());
    assert_eq!(engine.frame_resources.len(), 3);  // Triple buffering
}
```

---

## 🐛 Debugging Común

### ❌ "No GPU compatible found"

```bash
# Verificar que Vulkan 1.3 está instalado
vulkaninfo | grep "apiVersion"

# Si usas laptop con GPU integrada + dedicada:
# - Forzar uso de GPU dedicada en configuración de Windows/NVIDIA
# - O usar variable de entorno:
set VK_ICD_FILENAMES=C:\VulkanSDK\1.3.xx.x\Bin\nv-vk64.json  # NVIDIA
```

### ❌ "Validation layer errors"

```bash
# En debug, los validation layers son normales al inicio
# Si persisten en release, desactivar:
cargo build --release  # Sin features debug_vulkan

# Para ver detalles:
export VK_DBG_LAYER_ACTION_LOG_MSGS=1
```

### ❌ "Memory leak detected by VMA"

```bash
# Habilitar logging de VMA en Cargo.toml:
[dependencies]
gpu-allocator = { version = "0.25", features = ["vulkan", "debug"] }

# Revisar que todos los recursos se destruyen en cleanup()
# Usar RenderDoc para visualizar recursos GPU: https://renderdoc.org/
```

---

## 📊 Métricas de Rendimiento (Objetivos)

| Métrica | Comando | Objetivo |
|---------|---------|----------|
| **Frame time** | `cargo bench frame_time` | < 5ms (200fps) |
| **RAM por tab** | `cargo run --profile=release` + Task Manager | < 50MB |
| **CPU idle** | `cargo run` + Process Explorer | < 0.5% |
| **Cold start** | `time cargo run --release` | < 1s |
| **Draw calls/frame** | RenderDoc capture | < 100 (bindless) |

### Script de benchmark rápido

```bash
#!/bin/bash
# scripts/benchmark.sh

echo "🔥 Benchmark Noir Browser Ultra-Fast"
echo "====================================="

echo -e "\n📦 Build release..."
cargo build --release --features ultrafast

echo -e "\n⏱️  Cold start time..."
time target/release/no-chromium --headless --test-page example.com

echo -e "\n🎮 Frame time benchmark (100 frames)..."
cargo bench --bench frame_time --features ultrafast

echo -e "\n💾 Memory usage (simulated)..."
cargo test test_memory_usage --release -- --nocapture

echo -e "\n✅ Benchmark completado"
```

---

## 🔄 Workflow de Desarrollo

### Día típico de desarrollo

```bash
# 1. Pull latest + limpiar
git pull
cargo clean -p no-chromium

# 2. Build con hot-reload de shaders (usar cargo-watch)
cargo install cargo-watch
cargo watch -x "build --features debug_vulkan" -w src/ -w shaders/

# 3. Ejecutar con logging
RUST_LOG=debug cargo run --features debug_vulkan

# 4. Testear cambios específicos
cargo test module_name --features debug_vulkan

# 5. Benchmark antes de commit
cargo bench --bench critical_path
```

### Estructura de commits recomendada

```
feat(vulkan): add bindless descriptor indexing
- Implement VK_EXT_descriptor_indexing
- Reduce draw calls by 70% via multi-draw indirect

fix(memory): prevent VMA leak in swapchain recreation
- Add explicit cleanup for image views
- Add test case for leak detection

perf(layout): move box resolution to compute shader
- Zero-copy DOM → GPU buffer
- Frame time: 12ms → 4ms on 1080p
```

---

## 🆘 Soporte y Recursos

### Documentación oficial
- [Vulkan 1.3 Spec](https://registry.khronos.org/vulkan/specs/1.3/html/)
- [Ash Rust Bindings](https://docs.rs/ash/latest/ash/)
- [VMA Allocator](https://gpuopen-librariesandsdks.github.io/VulkanMemoryAllocator/html/)
- [Boa JS Engine](https://boa-dev.github.io/boa/)

### Herramientas recomendadas
- [RenderDoc](https://renderdoc.org/) - Debugging gráfico Vulkan
- [NVidia Nsight](https://developer.nvidia.com/nsight-graphics) - Profiling GPU
- [Tracy](https://github.com/wolfpld/tracy) - Profiling en tiempo real
- [Web Platform Tests](https://github.com/web-platform-tests/wpt) - Compatibilidad web

### Comunidad
- Discord: `#noir-browser-dev` (invitación en README principal)
- GitHub Issues: Usar etiquetas `[Fase-0]`, `[vulkan]`, `[bug]`, `[enhancement]`
- RFCs: Crear issue con etiqueta `[RFC]` para cambios de arquitectura

---

## 🎯 Checklist: ¿Listo para la Fase 0?

- [ ] Vulkan SDK 1.3+ instalado y `vulkaninfo` funciona
- [ ] `cargo build --features debug_vulkan` compila sin errores
- [ ] Test `test_vulkan_init` pasa con tu GPU
- [ ] Frame time inicial medido (baseline)
- [ ] RenderDoc puede capturar un frame de prueba
- [ ] Entendiste la arquitectura `UltraFastVulkanEngine`

✅ Si todo está en verde: **¡Comienza la Fase 0!** 🚀

---

> 💡 **Pro Tip:** Usa `cargo expand` para ver el código macro-expanded y debuggear problemas complejos de Ash/Vulkan.
> 
> ```bash
> cargo install cargo-expand
> cargo expand vulkan_engine::core::UltraFastVulkanEngine::new
> ```

**¿Preguntas?** Abre un issue con la etiqueta `[help]` o únete al Discord para soporte en tiempo real.

¡A codificar! 🦀⚡
