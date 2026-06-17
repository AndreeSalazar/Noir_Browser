# Noir Browser (No-Chromium Engine)

[![Rust Version](https://img.shields.io/badge/rust-1.75%2B-blue.svg)](https://www.rust-lang.org/)
[![Graphics API](https://img.shields.io/badge/Graphics-Vulkan%20%2F%20Ash-red.svg)](https://vulkan.org/)
[![License](https://img.shields.io/badge/License-MIT%2FApache--2.0-green.svg)](#)

Un motor de navegacion web **independiente, ultrarrapido y seguro** desarrollado desde cero en **Rust y Vulkan (Ash)**, sin depender de Chromium, WebKit ni Gecko.

---

## Mision

Noir Browser es un motor de navegacion 100% independiente:
- **Sin Codigo de Chrome/Chromium** - Toda la base es Rust puro
- **Renderizado GPU con Vulkan** - Pipeline grafico nativo con Ash
- **Motor JavaScript propio** - Boa Engine (ECMAScript nativo en Rust)
- **Privacidad por defecto** - Sin telemetria, anti-fingerprint, aislamiento por tab

---

## Arquitectura del Motor

### Pipeline de Renderizado
```
Red (reqwest) -> Parser HTML (html5ever) -> Parser CSS -> Layout -> Vulkan GPU -> Framebuffer
```

### Modulos Principales

| Modulo | Descripcion |
|--------|-------------|
| `app/` | Ventana winit, UI Chrome-like, softbuffer para Phase 0 |
| `browser/` | Coordinador de tabs, navegacion, historial |
| `js_engine/` | Motor JavaScript completo (Boa 0.18) |
| `network/` | Stack de red con reqwest + DNS-over-HTTPS |
| `parsers/` | HTML5, CSS, DOM nativos |
| `vulkan_engine/` | Renderizador Vulkan 2D con Ash |
| `renderer/` | Pipeline de renderizado GPU |
| `utils/` | IPC, process model, utilidades |

---

## Motor JavaScript (js_engine/)

100% independiente, basado en **Boa Engine 0.18** (ECMAScript en Rust puro).

### Modulos del Motor JS

| Archivo | Funcion |
|---------|---------|
| `runtime.rs` | Contexto Boa por tab, evaluacion de scripts, task queue (setTimeout/setInterval) |
| `web_apis.rs` | `console.log/warn/error/info`, `JSON.parse/stringify` |
| `dom_bridge.rs` | `document.getElementById()`, `querySelector()`, `createElement()`, `addEventListener()` |
| `events.rs` | Sistema de eventos: `addEventListener`, `removeEventListener`, `dispatchEvent` |
| `sandbox.rs` | Aislamiento por tab, permisos, CSP, timeouts de scripts |
| `modules.rs` | Sistema de modulos ES con import/export y resolucion de specifiers |
| `bindings.rs` | `navigator.userAgent/platform/language`, `location.href/assign/reload`, `window.alert/confirm/prompt` |

### API Publica (JsEngine)

```rust
use no_chromium::js_engine::JsEngine;

let mut engine = JsEngine::new();

// Inicializar contexto JS para un tab
engine.init_tab(1)?;

// Evaluar JavaScript
let result = engine.eval_script(1, "console.log('Hello from Noir!')")?;

// Cargar modulo ES
engine.register_module("./utils.js", "export function foo() {}");
engine.load_module(1, "./utils.js")?;

// Procesar eventos pendientes
engine.process_events(1)?;

// Cleanup
engine.destroy_tab(1);
```

### Web APIs Soportadas

- **console**: `log()`, `warn()`, `error()`, `info()`, `clear()`
- **JSON**: `parse()`, `stringify()`
- **document**: `getElementById()`, `querySelector()`, `createElement()`, `addEventListener()`
- **navigator**: `userAgent`, `platform`, `language`, `onLine`, `cookieEnabled`
- **location**: `href`, `assign()`, `reload()`
- **window**: `title`, `close()`, `alert()`, `confirm()`, `prompt()`
- **Eventos**: `addEventListener()`, `removeEventListener()`, `dispatchEvent()`
- **Modulos**: ES modules con `import`/`export`

---

## Dependencias Clave (Sin Chromium)

| Dependencia | Uso |
|-------------|-----|
| `boa_engine` 0.18 | Motor JavaScript ECMAScript nativo en Rust |
| `ash` 0.37 | Bindings Vulkan de bajo nivel |
| `softbuffer` 0.4 | Framebuffer por software (Phase 0) |
| `winit` 0.30 | Ventana y eventos cross-platform |
| `reqwest` 0.12 | HTTP/2 client con TLS |
| `html5ever` 0.26 | Parser HTML5 |
| `cssparser` 0.31 | Parser CSS |
| `tokio` 1.35 | Runtime async multi-thread |

---

## Estructura del Proyecto

```
Noir_Browser/
├── Cargo.toml                    # Workspace config
├── No-Chromium/
│   ├── Cargo.toml                # Dependencias del navegador
│   └── src/
│       ├── main.rs               # Entry point, CLI, process model
│       ├── lib.rs                # Lib publica
│       ├── app/                  # UI y ventana
│       │   ├── mod.rs            # ApplicationHandler, draw_frame
│       │   ├── config.rs         # AppConfig
│       │   ├── state.rs          # NoirApp state
│       │   ├── draw.rs           # Primitivas de dibujo
│       │   ├── glyphs.rs         # Bitmap font (95 caracteres)
│       │   └── theme.rs          # Colores y layout
│       ├── js_engine/            # Motor JavaScript
│       │   ├── mod.rs            # JsEngine API publica
│       │   ├── runtime.rs        # Boa context + task queue
│       │   ├── web_apis.rs       # console, JSON
│       │   ├── dom_bridge.rs     # DOM API bridge
│       │   ├── events.rs         # Event system
│       │   ├── sandbox.rs        # Per-tab isolation
│       │   ├── modules.rs        # ES modules
│       │   └── bindings.rs       # navigator, location, window
│       ├── browser/              # Navegacion y tabs
│       ├── network/              # Stack de red
│       ├── parsers/              # HTML, CSS, DOM
│       ├── renderer/             # Pipeline de renderizado
│       ├── vulkan_engine/        # Vulkan GPU engine
│       └── utils/                # IPC, process model
└── README.md
```

---

## Roadmap

### Fase 0 (Actual) - UI y Motor JS
- [x] Ventana con winit + softbuffer
- [x] UI Chrome-like con tema oscuro
- [x] Bitmap font renderer (95 glyphs)
- [x] Controles de ventana (minimizar/maximizar/cerrar)
- [x] Dragging de ventana
- [x] Motor JavaScript Boa integrado
- [x] Web APIs: console, JSON, document, navigator, location, window
- [x] Sandbox por tab
- [x] Sistema de modulos ES

### Fase 1 - Renderizado y Layout
- [ ] Pipeline Vulkan completo (shaders, textures)
- [ ] Layout engine: CSS Flexbox & Grid
- [ ] MSDF font rendering (texto GPU)
- [ ] Decodificacion de imagenes (PNG, JPEG, WebP)

### Fase 2 - Stack de Red
- [ ] HTTP/2 completo con reqwest
- [ ] DNS-over-HTTPS (hickory-resolver)
- [ ] Cache de recursos (LRU)
- [ ] Soporte CORS

### Fase 3 - Features Avanzadas
- [ ] Decodificacion de video acelerada por hardware
- [ ] Service Workers
- [ ] WebAssembly (wasm-bindgen)
- [ ] Extensions/Plugins system

---

## Construccion

```bash
# Build release
cargo build --release

# Run
cargo run

# Tests
cargo test
```

---

## Licencia

MIT / Apache-2.0
