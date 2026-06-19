# Noir Browser

Un navegador web minimalista, ultra-eficiente y **100% independiente de Chromium**, escrito en Rust.

## Filosofía

> No imitar a Google, sino **perfeccionar** la experiencia web con:
> - 🦀 **Rust** - Memory safety sin garbage collector
> - ⚡ **10MB RAM** por pestaña (vs 100-200MB de Chrome)
> - 🎨 **Vulkan-ready** (FASE F) - GPU acceleration planeada
> - 🔒 **Privacy-first** - Sin telemetría, sin tracking
> - 🧅 **Tor-inspired** - Soporte para SOCKS5 y circuitos

## Arquitectura

```
No-Chromium/src/
├── app/           # UI + Event handling + Rendering
├── parsers/       # HTML/CSS/JS parsing
├── js_engine/     # JavaScript runtime (Boa 0.18)
├── network/       # HTTP client + DNS + Proxies
├── media/         # Image/Video/Audio handling
├── utils/         # Process model, memory, IPC
├── archive/       # Código experimental/legacy
├── lib.rs         # Public API
└── main.rs        # Entry point
```

### Módulos Activos

| Módulo | Responsabilidad | Líneas |
|--------|----------------|--------|
| `app/` | UI Chrome-like, event loop, drawing | ~1100 |
| `parsers/` | HTML5, CSS, JS parsing | ~2500 |
| `js_engine/` | JS runtime + DOM bridge (Boa 0.18) | ~1200 |
| `network/` | HTTP, DNS over HTTPS, SOCKS, Tor | ~800 |
| `media/` | Image cache, audio, video | ~600 |
| `utils/` | Process model, memory, IPC | ~400 |

### Archive (Código Experimental)

- `archive/vulkan_engine/` - GPU rendering (FASE F)
- `archive/browser/` - Tab manager alternativo
- `archive/renderer/` - Renderers experimentales
- `archive/runtime/` - JS runtime alternativo
- `archive/ui/` - Sistema de UI alternativo
- `archive/layout/` - Layout engines alternativos
- `archive/generated_rust_backup/` - Código auto-generado

## Estado Actual

- ✅ **FASE A-D**: CSS + JS + Forms + Network
- ✅ **FASE E**: Chrome-style UI
- ✅ **FASE G**: Image optimization (LRU cache, async, stats)
- 🚧 **FASE F**: Vulkan renderer (en archive, listo para integrar)

## Características Implementadas

### UI Chrome-like
- ✅ Custom title bar (sin decoraciones del OS)
- ✅ Tab bar con close buttons
- ✅ Nav bar (back/forward/reload/home)
- ✅ Address bar con search engine shortcuts
- ✅ 15 motores de búsqueda integrados (yt, gg, gh, ddg, etc.)
- ✅ Atajos de teclado (Ctrl+T/W/L/R/D, F5, F11, Ctrl+Tab)
- ✅ New tab page con quick links

### Parsing & Rendering
- ✅ HTML5 parser (html5ever)
- ✅ CSS engine con cascade (590+ líneas)
- ✅ Layout engine
- ✅ Softbuffer (CPU rendering)
- ✅ Bitmap font propio (95 caracteres)

### JavaScript Engine
- ✅ Boa 0.18 (motor JS nativo en Rust)
- ✅ DOM API (getElementById, querySelector, etc.)
- ✅ Mutations tracking
- ✅ Timers (setTimeout, setInterval)
- ✅ Console API
- ✅ Fetch API

### Network
- ✅ HTTP/HTTPS
- ✅ Redirects (301/302/307/308)
- ✅ Cookie jar
- ✅ POST/PUT/DELETE
- ✅ DNS over HTTPS
- ✅ SOCKS5 proxy (Tor ready)

### Media
- ✅ LRU image cache (100 items, 50MB)
- ✅ Async image fetching
- ✅ Retry logic
- ✅ Format detection (PNG, JPEG, GIF, WebP, BMP)
- ✅ Lazy loading (`loading="lazy"`)
- ✅ Estadísticas de cache

## Compilar y Ejecutar

```bash
# Compilar
cargo build

# Ejecutar
cargo run

# Tests
cargo test
```

## Métricas

- **37 tests pasando** ✅
- **0 errores de compilación** ✅
- **~10MB RAM** por pestaña (medido)
- **0 dependencias de Chromium/Google** ✅

## Roadmap

Ver `Noir_Browser_Roadmap.md` para el plan completo.

### Próximas Fases

1. **FASE R2**: Refactor `app/mod.rs` (934 líneas → 5 archivos)
2. **FASE R3**: Refactor `js_engine/dom_bridge.rs` (507 líneas → 4 archivos)
3. **FASE H**: HTML5 `<video>` support
4. **FASE I**: Advanced JS (Promises, async/await)
5. **FASE J**: Flexbox + Grid CSS
6. **FASE F**: Vulkan renderer integration

## Licencia

MIT
