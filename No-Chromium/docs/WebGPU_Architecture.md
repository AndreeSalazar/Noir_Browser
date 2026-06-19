# Noir Browser - Arquitectura GPU

## Migración de Vulkan a WebGPU

Noir Browser ha migrado completamente de **Vulkan directo** a **WebGPU** como su API de GPU.

### Por qué WebGPU en lugar de Vulkan directo

| Característica | WebGPU | Vulkan directo |
|---|---|---|
| Multi-backend automático | ✅ Sí (Vulkan/Metal/DX12) | ❌ No |
| Estándar web (W3C) | ✅ Sí | ❌ No |
| Promise-based (compatible con JS) | ✅ Sí | ❌ No |
| Compute shaders | ✅ Sí | ✅ Sí |
| Líneas de código (renderer básico) | ~500 | ~3000+ |
| Multiplataforma automática | ✅ Sí | ❌ Manual |
| Sandbox seguro | ✅ Sí | ❌ Driver-level |
| Integración con JS engine | ✅ Nativa | ❌ Manual |

### Arquitectura actual

```
┌─────────────────────────────────────────┐
│         Noir Browser                    │
├─────────────────────────────────────────┤
│  ┌──────────┐  ┌──────────┐            │
│  │ JS v3    │  │ WASM v2  │            │
│  └──────────┘  └──────────┘            │
│         │                │                │
│         └────────┬───────┘                │
│                  │                         │
│         ┌────────▼────────┐               │
│         │  WebGPU Bridge  │               │
│         └────────┬────────┘               │
│                  │                         │
│         ┌────────▼────────┐               │
│         │  WebGPU Module  │               │
│         │  (multi-backend)│               │
│         └────────┬────────┘               │
│                  │                         │
│    ┌─────────┬───┴────┬─────────┐         │
│    ▼         ▼        ▼         ▼         │
│  Windows   Linux    macOS    Android       │
│   DX12    Vulkan    Metal    Vulkan        │
└─────────────────────────────────────────┘
```

### Módulos GPU

- `src/webgpu/` - WebGPU module (11 archivos)
  - `device.rs` - GPU adapter/device abstraction
  - `shaders.rs` - WGSL shader modules
  - `buffer.rs` - GPU memory management
  - `texture.rs` - GPU textures
  - `pipeline.rs` - Render pipelines
  - `renderer.rs` - 2D renderer
  - `compute.rs` - Compute shaders
  - `bridge.rs` - JS <-> WebGPU bridge
  - `pwa.rs` - PWA support
  - `integration.rs` - Integrated renderer

### Archivos archive/ (código experimental)

El directorio `src/archive/` contiene código experimental de fases anteriores,
incluyendo un bootstrapper de Vulkan que ya no se usa. Este código se mantiene
solo como referencia histórica.

## Configuración

En `src/app/config.rs`:
- `debug_webgpu: bool` - Habilita logging detallado de WebGPU
- El flag anterior `debug_vulkan` ha sido renombrado a `debug_webgpu`
