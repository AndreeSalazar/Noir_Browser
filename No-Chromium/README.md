# 🌌 Noir Browser - No-Chromium Core

> **Arquitectura Fusionada**: Chrome × Tor × Vulkan  
> **Lema**: Velocidad de Chrome + Privacidad de Tor + Vulkan Ultra-Fast

## 📋 Tabla de Contenidos

- [🧬 Filosofía de Diseño](#-filosofía-de-diseño)
- [🏗️ Arquitectura Multi-Proceso](#-arquitectura-multi-proceso)
- [📁 Estructura del Proyecto](#-estructura-del-proyecto)
- [⚡ Características Principales](#-características-principales)
- [🔧 Configuración y Build](#-configuración-y-build)
- [🗺️ Roadmap de Desarrollo](#-roadmap-de-desarrollo)
- [🚀 Primeros Pasos](#-primeros-pasos)

---

## 🧬 Filosofía de Diseño

| Principio | Fuente | Implementación en Noir |
|-----------|--------|----------------------|
| **Aislamiento por proceso** | Chrome Site Isolation | `tokio::task` con memoria separada por dominio |
| **Privacidad por defecto** | Tor Browser | Sin telemetría, cookies partitioned, fingerprint jitter |
| **Renderizado GPU puro** | Innovación Noir | Vulkan 1.3, zero-copy, bindless, triple buffering |
| **Servicios adaptativos** | Chrome Servicification | Auto-escala: 1 proceso en 4GB RAM, procesos separados en 16GB |
| **Anonimato de red** | Tor Onion Routing | SOCKS5 chain opcional, circuit rotation, DNS over HTTPS |
| **Evitar escritura a disco** | Tor Disk Avoidance | Cache efímera en `mmap` anónimo, `zeroize` al cerrar |

---

## 🏗️ Arquitectura Multi-Proceso (Rust Native)

```
┌─────────────────────────────────────────────────────────────────┐
│                        NOIR BROWSER                             │
│                                                                 │
│  ┌─────────────────────┐    ┌───────────────────────────────┐   │
│  │   BROWSER PROCESS   │◄──►│       GPU PROCESS             │   │
│  │  (UI + Navigation)  │ IPC│  (Vulkan Engine - ash)        │   │
│  │                     │    │  - Frame composer             │   │
│  │ - Address bar       │    │  - Shader pipelines           │   │
│  │ - Tab management    │    │  - Bindless descriptors       │   │
│  │ - Cookie jar (FPI)  │    │  - Timeline semaphores        │   │
│  │ - History (RAM)     │    │  - MSDF text rasterizer       │   │
│  └─────────┬───────────┘    └───────────────────────────────┘   │
│            │                                                    │
│            ▼                                                    │
│  ┌─────────────────────┐    ┌───────────────────────────────┐   │
│  │  RENDERER PROCESS 1 │    │  NETWORK PROCESS              │   │
│  │  (tab: example.com) │    │                               │   │
│  │                     │    │  - HTTP/HTTPS fetch           │   │
│  │ - HTML Parser       │    │  - DNS-over-HTTPS resolver    │   │
│  │ - CSS Cascade       │    │  - SOCKS5 proxy (Tor mode)    │   │
│  │ - Layout Engine     │    │  - Pre-cache async            │   │
│  │ - JS Engine (Boa)   │    │  - Certificate pinning        │   │
│  └─────────┬───────────┘    └───────────────────────────────┘   │
│            │                                                    │
│            ▼                                                    │
│  ┌─────────────────────┐    ┌───────────────────────────────┐   │
│  │  RENDERER PROCESS 2 │    │  UTILITY PROCESSES            │   │
│  │  (tab: news.ycombin)|◄──►│                               │   │
│  │                     │    │  - Image decoder (async)      │   │
│  │ - DOM Isolated      │    │  - Font loader (MSDF)         │   │
│  │ - Cookie Partition  │    │  - Video decoder (NVDEC)      │   │
│  │ - Script Sandbox    │    │  - Search indexer (SQLite)    │   │
│  └─────────────────────┘    └───────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

### Comunicación IPC (Canales MPSC Tokio)

Los mensajes entre procesos usan canales tipo-safe definidos en `src/utils/ipc.rs`:

```rust
// Browser ↔ Renderer
pub enum BrowserMessage {
    Navigate { url: String, tab_id: TabId },
    StopLoading { tab_id: TabId },
    GetTitle { tab_id: TabId, reply: oneshot::Sender<String> },
    CloseTab { tab_id: TabId },
}

// Renderer ↔ GPU
pub enum RenderMessage {
    SubmitFrame { commands: Vec<CommandBuffer>, sem: Semaphore },
    SwapChainInvalid,
    Resize { width: u32, height: u32 },
}

// Renderer ↔ Network
pub enum NetworkMessage {
    FetchUrl { url: Url, headers: Headers, reply: oneshot::Sender<Response> },
    WebSocketConnect { url: Url, reply: oneshot::Sender<WsStream> },
    DnsResolve { hostname: String, reply: oneshot::Sender<IpAddr> },
}
```

---

## 📁 Estructura del Proyecto

```
No-Chromium/
├── Cargo.toml                      # Configuración con features
├── README.md                       # Este documento
├── src/
│   ├── main.rs                     # Entry point + auto-scaling
│   ├── lib.rs                      # API pública exportada
│   ├── app.rs                      # UI loop (winit)
│   ├── browser/                    # Proceso Browser
│   │   ├── mod.rs                  # Coordinador + tipos base
│   │   ├── coordinator.rs          # Lógica de coordinación
│   │   ├── tab_manager.rs          # Gestión de tabs + IPC
│   │   ├── navigation.rs           # Navigation flow (Chrome-style)
│   │   └── privacy/                # Módulo de privacidad
│   │       ├── mod.rs              # First-party isolation
│   │       ├── fpi.rs              # Cookie/storage partitioning
│   │       └── fingerprint.rs      # Canvas/WebGL jitter
│   ├── renderer/                   # Proceso Renderer
│   │   ├── mod.rs                  # Entry point renderer
│   │   ├── html_parser.rs          # Zero-copy HTML parser
│   │   ├── css_cascade.rs          # CSS specificity + inheritance
│   │   ├── layout_engine.rs        # Block/inline → Flexbox/Grid
│   │   └── js_engine/              # Motor JavaScript
│   │       ├── mod.rs              # Boa integration
│   │       └── boa_bridge.rs       # document/window bindings
│   ├── vulkan_engine/              # Proceso GPU
│   │   ├── mod.rs                  # Exportaciones públicas
│   │   ├── core.rs                 # UltraFastVulkanEngine
│   │   ├── shaders/                # Shaders GLSL
│   │   │   ├── ui.comp             # UI compositing
│   │   │   ├── text_msdf.frag      # MSDF text rendering
│   │   │   └── image.frag          # Image sampling
│   │   └── bindless.rs             # Descriptor indexing
│   ├── network/                    # Proceso Network
│   │   ├── mod.rs                  # Coordinador de red
│   │   ├── fetch.rs                # HTTP/HTTPS async
│   │   ├── socks_proxy.rs          # Tor-mode SOCKS5 chain
│   │   ├── doh_resolver.rs         # DNS-over-HTTPS
│   │   └── circuit.rs              # Circuit rotation logic
│   └── utils/                      # Utilidades compartidas
│       ├── mod.rs                  # Re-exports
│       ├── ipc.rs                  # Tipos de mensajes MPSC
│       ├── process_model.rs        # Auto-scaling logic
│       └── memory.rs               # Ephemeral buffers + zeroize
└── tests/
    ├── wpt/                        # Web Platform Tests
    ├── privacy/                    # Fingerprint tests
    └── performance/                # Frame time benchmarks
```

---

## ⚡ Características Principales

### 🔒 Privacidad (Herencia Tor)

- **First-Party Isolation (FPI)**: Cookies y localStorage aislados por `(domain, first_party_domain)`
- **Anti-Fingerprinting**: Jitter imperceptible en Canvas/WebGL (±1 en canales RGBA)
- **Disk Avoidance**: Cache en `mmap` anónimo, `zeroize()` automático al cerrar

```rust
// Ejemplo: Obtener cookies respetando FPI
let cookies = fpi.get_cookies("tracker.com", "example.com");
// Retorna vacío si first_party no coincide
```

### 🚀 Velocidad (Herencia Chrome + Vulkan)

- **Pipeline Zero-Copy**: HTML/CSS → Parser → Layout → Vulkan sin allocaciones intermedias
- **Bindless Descriptors**: Acceso directo a recursos GPU sin rebinding
- **Triple Buffering**: <5ms por frame en hardware compatible

### 🧠 Auto-Scaling Inteligente

```rust
// Determina modelo según RAM disponible
pub fn determine_process_model(available_ram_mb: u64) -> ProcessModel {
    match available_ram_mb {
        0..=2048   => ProcessModel::SingleProcess,     // Todo en 1 task
        2049..=4096 => ProcessModel::Aggregated,       // Browser + 1 renderer
        4097..=8192 => ProcessModel::ModerateIsolation, // Browser + renderer por tab
        _          => ProcessModel::FullIsolation,     // Todos separados
    }
}
```

---

## 🔧 Configuración y Build

### Features Disponibles

| Feature | Descripción | Default |
|---------|-------------|---------|
| `ultrafast` | Vulkan 1.3 zero-copy rendering | ✅ |
| `privacy` | First-Party Isolation + anti-fingerprint | ❌ |
| `tor_mode` | SOCKS5 proxy + circuit rotation | ❌ |
| `msdf_fonts` | Multi-channel Signed Distance Field fonts | ❌ |
| `debug_vulkan` | Vulkan validation layers | ❌ |
| `fallback_vulkano` | Usar Vulkano como fallback | ❌ |
| `video_decode` | Aceleración hardware de video | ❌ |
| `local_search` | Búsqueda local con SQLite | ❌ |

### Build Commands

```bash
# Build estándar (ultrafast habilitado)
cargo build --release

# Build con privacidad completa
cargo build --release --features "privacy,tor_mode"

# Build con debug de Vulkan
cargo build --features "debug_vulkan"

# Run con flags de línea de comandos
cargo run -- --tor-only --debug-vulkan

# Tests
cargo test --all-features
```

### Variables de Entorno

```bash
# Nivel de logging
RUST_LOG=noir=debug cargo run

# Forzar modelo de proceso
NOIR_PROCESS_MODEL=full_isolation cargo run

# Límite de memoria para cache
NOIR_CACHE_MB=1024 cargo run
```

---

## 🗺️ Roadmap de Desarrollo (7 Fases)

| Fase | Duración | Objetivo | Criterio de Éxito |
|------|----------|----------|------------------|
| **0** | 2 sem | Vulkan 1.3 ultra-fast base | <8ms frame, triple buffering |
| **1** | 3 sem | Pipeline zero-copy parser → GPU | Parse + render en <15ms |
| **2** | 3 sem | Motor JS Boa + Bridge DOM mínimo | `document.getElementById()` funciona |
| **3** | 2 sem | Privacidad Tor: FPI + anti-fingerprint | Canvas fingerprint ≠ real |
| **4** | 3 sem | Multi-proceso + IPC | Crash en tab no afecta otros |
| **5** | 2 sem | Red: SOCKS5 + DoH + circuit rotation | Navegación anónima funcional |
| **6** | 3 sem | Flexbox/Grid + MSDF fonts | 90% WPT CSS pass |
| **7** | 2 sem | Búsqueda nativa + overlay | Autocomplete <50ms, index local |

**Total**: ~20 semanas (~5 meses) hasta producto usable.

---

## 🚀 Primeros Pasos

### 1. Clonar y Configurar

```bash
git clone https://github.com/tu-usuario/noir-browser.git
cd noir-browser/No-Chromium
```

### 2. Instalar Dependencias del Sistema

**Windows:**
```powershell
# Vulkan SDK: https://vulkan.lunarg.com/sdk/home
# Rust: https://rustup.rs
```

**Linux (Ubuntu/Debian):**
```bash
sudo apt install libvulkan-dev mesa-vulkan-drivers vulkan-tools
```

**macOS:**
```bash
# Vulkan via MoltenVK
brew install vulkan-loader vulkan-headers
```

### 3. Build y Ejecutar

```bash
# Build de desarrollo
cargo build

# Ejecutar con configuración por defecto
cargo run

# Ejecutar con modo Tor y debug
cargo run --features "tor_mode,debug_vulkan" -- --tor-only --debug-vulkan
```

### 4. Verificar Funcionamiento

```bash
# Test mínimo de compilación
cargo test test_build

# Benchmark de frame time
cargo bench --bench frame_time
```

---

## 🤝 Contribuir

1. Leer `ARCHITECTURE.md` en la raíz del proyecto
2. Seguir el roadmap de fases en `Fases.md`
3. Usar `cargo fmt` y `cargo clippy` antes de commit
4. Añadir tests para nuevas funcionalidades
5. Documentar APIs públicas en `lib.rs`

---

## 📄 Licencia

Este proyecto está bajo la licencia especificada en `LICENSE`.

> ⚠️ **Nota**: Esta arquitectura elimina la dependencia de C++ legacy, V8, y cualquier componente de Chromium. Todo es **Rust nativo + Vulkan directo + patrones de privacidad Tor**.

---

<div align="center">

**🌌 Noir Browser** · *Navegación rápida. Privacidad real. Código abierto.*

[📋 ARCHITECTURE.md](../ARCHITECTURE.md) · [🗺️ Fases.md](Fases.md) · [🚀 QUICKSTART.md](QUICKSTART.md)

</div>
