# 🌌 Noir Browser - Arquitectura Fusionada (Chrome × Tor)

> **ADN Híbrido:** Velocidad de Chrome + Privacidad de Tor + Vulkan Ultra-Fast

---

## 🧬 Filosofía de Diseño

| Principio | Fuente | Implementación en Noir |
|-----------|--------|------------------------|
| **Aislamiento por proceso** | Chrome Site Isolation | `tokio::task` con memoria separada por dominio |
| **Privacidad por defecto** | Tor Browser | Sin telemetría, cookies partitioned, fingerprint jitter |
| **Renderizado GPU puro** | Innovación Noir | Vulkan 1.3, zero-copy, bindless, triple buffering |
| **Servicios adaptativos** | Chrome Servicification | Auto-escala: 1 proceso en 4GB RAM, procesos separados en 16GB |
| **Anonimato de red** | Tor Onion Routing | SOCKS5 chain opcional, circuit rotation, DNS over HTTPS |
| **Evitar escritura a disco** | Tor Disk Avoidance | Cache efímera en `mmap` anónimo, `zeroize` al cerrar |

---

## 🏗️ Arquitectura Multi-Proceso (Rust Native)

### Estructura de Procesos

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

```rust
// Mensajes entre Browser Process ↔ Renderer Process
pub enum BrowserMessage {
    Navigate { url: String, tab_id: TabId },
    StopLoading { tab_id: TabId },
    GetTitle { tab_id: TabId, reply: oneshot::Sender<String> },
    CloseTab { tab_id: TabId },
}

// Mensajes entre Renderer Process ↔ GPU Process
pub enum RenderMessage {
    SubmitFrame { commands: Vec<CommandBuffer>, sem: Semaphore },
    SwapChainInvalid,
    Resize { width: u32, height: u32 },
}

// Mensajes entre Renderer Process ↔ Network Process
pub enum NetworkMessage {
    FetchUrl { url: Url, headers: Headers, reply: oneshot::Sender<Response> },
    WebSocketConnect { url: Url, reply: oneshot::Sender<WsStream> },
    DnsResolve { hostname: String, reply: oneshot::Sender<IpAddr> },
}
```

---

## 🔒 Privacidad (Herencia Tor)

### First-Party Isolation (FPI)

```rust
// Cada dominio tiene su propio contenedor de estado
pub struct FirstPartyIsolation {
    // Cookies aisladas por (domain, first_party_domain)
    cookie_jar: HashMap<(String, String), Vec<Cookie>>,
    
    // localStorage aislado por origen
    local_storage: HashMap<String, StorageMap>,
    
    // Historial de sesión (no persiste a disco)
    session_history: Vec<HistoryEntry>,
}

impl FirstPartyIsolation {
    pub fn get_cookies(&self, request_domain: &str, page_domain: &str) -> Vec<Cookie> {
        // Solo devuelve cookies si el first-party coincide
        self.cookie_jar
            .get(&(request_domain.to_string(), page_domain.to_string()))
            .cloned()
            .unwrap_or_default()
    }
}
```

### Anti-Fingerprinting (Canvas + WebGL Jitter)

```rust
// Agrega ruido imperceptible al renderizado de canvas
pub fn apply_canvas_jitter(&self, pixel_data: &mut [u8], tab_id: TabId) {
    let seed = self.fingerprint_seed(tab_id); // Semilla por sesión
    let mut rng = StdRng::seed_from_u64(seed);
    
    for chunk in pixel_data.chunks_mut(4) {
        // ±1 en cada canal RGBA (imperceptible al ojo humano)
        chunk[0] = chunk[0].wrapping_add(rng.gen_range(0..=2) as u8);
        chunk[1] = chunk[1].wrapping_add(rng.gen_range(0..=2) as u8);
        chunk[2] = chunk[2].wrapping_add(rng.gen_range(0..=2) as u8);
    }
}
```

### Disk Avoidance (Zero-Write Cache)

```rust
// Cache en memoria anónima (mmap sin archivo de respaldo)
pub struct EphemeralCache {
    map: MemMap, // mmap anónimo
    used: AtomicUsize,
}

impl EphemeralCache {
    pub fn new(size: usize) -> Self {
        // mmap sin archivo: datos existen solo en RAM
        let map = MemMap::anonymous(size).expect("Failed to create anonymous mmap");
        Self { map, used: AtomicUsize::new(0) }
    }
    
    // Al cerrar tab, zeroize toda la memoria
    pub fn purge(&mut self) {
        self.map.zeroize();
        self.used.store(0, Ordering::Relaxed);
    }
}
```

---

## ⚡ Velocidad (Herencia Chrome + Vulkan)

### Pipeline Zero-Copy

```
┌────────────┐    ┌─────────────┐    ┌──────────────┐    ┌─────────────┐
│ HTML/CSS   │───►│ Parser      │───►│ Layout       │───►│ Vulkan GPU  │
│ Raw Bytes  │    │ (no alloc)  │    │ (GPU assist) │    │ (bindless)  │
└────────────┘    └─────────────┘    └──────────────┘    └─────────────┘
                                                                │
                                                                ▼
                                                         ┌─────────────┐
                                                         │  Display    │
                                                         │  <5ms/frame │
                                                         └─────────────┘
```

### Auto-Scaling de Procesos (Servicification)

```rust
// Decide cuántos procesos lanzar según RAM disponible
pub fn determine_process_model(available_ram_mb: u64) -> ProcessModel {
    match available_ram_mb {
        0..=2048   => ProcessModel::SingleProcess,     // Todo en 1 proceso
        2049..=4096 => ProcessModel::Aggregated,       // Browser + 1 renderer
        4097..=8192 => ProcessModel::ModerateIsolation, // Browser + renderer por tab
        _          => ProcessModel::FullIsolation,     // Browser + renderer + GPU + network
    }
}
```

---

## 🗺️ Roadmap de Reconstrucción (7 Fases)

| Fase | Duración | Objetivo | Criterio de Éxito |
|------|----------|----------|-------------------|
| **0** | 2 sem | Vulkan 1.3 ultra-fast base | <8ms frame, triple buffering |
| **1** | 3 sem | Pipeline zero-copy parser → GPU | Parse + render en <15ms |
| **2** | 3 sem | Motor JS Boa + Bridge DOM mínimo | `document.getElementById()` funciona |
| **3** | 2 sem | Privacidad Tor: FPI + anti-fingerprint | Canvas fingerprint ≠ real |
| **4** | 3 sem | Multi-proceso + IPC | Crash en tab no afecta otros |
| **5** | 2 sem | Red: SOCKS5 + DoH + circuit rotation | Navegación anónima funcional |
| **6** | 3 sem | Flexbox/Grid + MSDF fonts | 90% WPT CSS pass |
| **7** | 2 sem | Búsqueda nativa + overlay | Autocomplete <50ms, index local |

**Total: 20 semanas (~5 meses)** hasta producto usable.

---

## 📁 Estructura Final del Proyecto

```
Noir_Browser/
├── Cargo.toml                      # Workspace con features
├── Fases.md                        # Roadmap completo
├── ARCHITECTURE.md                 # Este documento
├── src/
│   ├── main.rs                     # Entry point, decide process model
│   ├── app.rs                      # UI loop (winit)
│   ├── browser/
│   │   ├── mod.rs                  # Browser process coordinator
│   │   ├── tab_manager.rs          # Gestión de tabs + IPC
│   │   ├── navigation.rs           # Navigation flow (Chrome-style)
│   │   └── privacy/
│   │       ├── mod.rs              # First-party isolation
│   │       ├── fingerprint.rs      # Canvas/WebGL jitter
│   │       └── disk_avoidance.rs   # Ephemeral cache
│   ├── renderer/
│   │   ├── mod.rs                  # Renderer process
│   │   ├── html_parser.rs          # Zero-copy parser
│   │   ├── css_cascade.rs          # CSS specificity + inheritance
│   │   ├── layout_engine.rs        # Block/inline → Flexbox/Grid
│   │   └── js_engine/
│   │       ├── mod.rs              # Boa integration
│   │       └── dom_bridge.rs       # document/window bindings
│   ├── vulkan_engine/
│   │   ├── mod.rs                  # GPU process
│   │   ├── core.rs                 # UltraFastVulkanEngine
│   │   ├── shaders/
│   │   │   ├── ui.comp             # UI compositing
│   │   │   ├── text_msdf.frag      # MSDF text rendering
│   │   │   └── image.frag          # Image sampling
│   │   └── bindless.rs             # Descriptor indexing
│   ├── network/
│   │   ├── mod.rs                  # Network process
│   │   ├── fetch.rs                # HTTP/HTTPS async
│   │   ├── socks_proxy.rs          # Tor-mode SOCKS5 chain
│   │   ├── doh_resolver.rs         # DNS-over-HTTPS
│   │   └── circuit.rs              # Circuit rotation
│   └── utils/
│       ├── ipc.rs                  # MPSC channels + oneshot
│       └── process_model.rs        # Auto-scaling logic
└── tests/
    ├── wpt/                        # Web Platform Tests
    ├── privacy/                    # Fingerprint tests
    └── performance/                # Frame time benchmarks
```

---

## 🚀 Próximos Pasos Inmediatos

1. **Eliminar archivos legacy** que no encajan con la nueva arquitectura
2. **Crear `Cargo.toml` workspace** con features: `privacy`, `tor_mode`, `ultrafast`
3. **Implementar Fase 0**: Vulkan 1.3 engine con triple buffering
4. **Implementar Fase 1**: Zero-copy parser → GPU pipeline
5. **Implementar Fase 3**: First-party isolation + anti-fingerprint

---

> **Nota:** Esta arquitectura elimina la dependencia de C++ legacy, V8, y cualquier componente de Chromium. Todo es **Rust nativo + Vulkan directo + patrones de privacidad Tor**.
