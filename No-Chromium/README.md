# Noir Browser

Un navegador web minimalista, ultra-eficiente y **100% independiente de Chromium**,
escrito en Rust con motor JS, WASM, y WebGPU propios.

## Filosofía

> No imitar a Google, sino **perfeccionar** la experiencia web con:
> - Rust - Memory safety sin garbage collector
> - **WebGPU** (en lugar de Vulkan directo) - Multi-backend automático
> - 10MB RAM por pestaña (vs 100-200MB de Chrome)
> - 0 dependencias de Chromium/Google
> - 0 telemetría/tracking

## Estado Actual

- **FASE A-D**: CSS + JS + Forms + Network
- **FASE E**: Chrome-style UI
- **FASE G**: Image optimization (LRU cache, async)
- **FASE R1-R2**: Cleanup + Refactor
- **FASE W1-W5**: JS Engine v3 con DOM
- **FASE I**: Promises/async-await
- **FASE H**: HTML5 video/audio support
- **FASE J**: Flexbox + Grid CSS
- **FASE F prep**: Renderer traits
- **FASE W2**: WASM v2 profesional
- **FASE WGPU**: WebGPU multi-backend
- **FASE WGPU integration**: IntegratedRenderer

## Arquitectura

```
No-Chromium/src/
├── app/              (250 líneas, 4 archivos)
│   ├── mod.rs          (Entry + ApplicationHandler)
│   ├── state.rs        (NoirApp struct)
│   ├── config.rs       (AppConfig: debug_webgpu, etc.)
│   ├── draw.rs         (Rendering primitives)
│   ├── glyphs.rs       (Bitmap font)
│   ├── theme.rs        (Colors & dimensions)
│   ├── renderer.rs     (Chrome UI rendering)
│   ├── input.rs        (Click + keyboard handling)
│   └── navigation.rs   (URL resolution)
│
├── js_engine_v3/     (12 módulos + promise)
│   ├── mod.rs
│   ├── value.rs        (JsValue enum)
│   ├── env.rs          (Environment)
│   ├── ast.rs          (AST)
│   ├── lexer.rs        (Tokenizer)
│   ├── parser.rs       (Parser)
│   ├── interpreter.rs  (Tree-walking executor)
│   ├── dom.rs          (DOM API)
│   ├── console.rs
│   ├── timer.rs
│   ├── fetch.rs
│   ├── builtins.rs     (Math, JSON, window, etc.)
│   └── promise.rs      (Promises/async-await)
│
├── wasm_v2/          (13 módulos profesionales)
│   ├── mod.rs
│   ├── types.rs
│   ├── value.rs
│   ├── arena.rs        (Bump allocator)
│   ├── leb128.rs       (LEB128 codec)
│   ├── decoder.rs      (WASM binary parser)
│   ├── validator.rs
│   ├── opcodes.rs
│   ├── interpreter.rs
│   ├── jit.rs
│   ├── compiler.rs
│   ├── runtime.rs
│   └── wasi.rs
│
├── webgpu/           (11 módulos GPU) ⭐
│   ├── mod.rs
│   ├── device.rs       (Multi-backend GPU)
│   ├── shaders.rs      (WGSL shaders)
│   ├── buffer.rs
│   ├── texture.rs
│   ├── pipeline.rs
│   ├── renderer.rs
│   ├── compute.rs
│   ├── bridge.rs       (JS <-> WebGPU)
│   ├── pwa.rs          (PWA support)
│   └── integration.rs  (Integrated renderer)
│
├── bridge/           (JS <-> WASM bridge)
├── parsers/          (HTML/CSS parsers)
├── media/            (Image cache)
├── network/          (HTTP/DNS)
├── utils/            (Process model)
├── tests/            (Auto-generated)
├── archive/          (Experimental code)
└── docs/             (Documentation)
```

## Métricas

- **95+ tests** pasando
- **0 errores** de compilación
- **~10MB RAM** por pestaña
- **0 dependencias** de Chromium/Google
- **Build time**: 30 segundos

## GPU Acceleration (WebGPU)

Noir Browser usa **WebGPU** como su API de GPU principal. WebGPU es un estándar
web (W3C) que abstrae múltiples backends de GPU:

- **Windows**: DirectX 12
- **Linux**: Vulkan
- **macOS**: Metal
- **Android**: Vulkan

### Por qué WebGPU en lugar de Vulkan directo

- **Multi-backend automático**: Un solo código, múltiples OS
- **API web estándar**: Compatible con JS engine directamente
- **Compute shaders**: Para PWA, crypto, ML
- **5x menos código**: Que escribir Vulkan/Metal/DX12 directo
- **Mejor debugging**: Integración con DevTools

Ver `docs/WebGPU_Architecture.md` para más detalles.

## Compilar y Ejecutar

```bash
# Compilar
cargo build

# Ejecutar
cargo run

# Tests
cargo test
```

## Scripts Python

El proyecto usa Python para generar código automáticamente:

- `scripts/generate_js_engine_v3.py` - Genera el JS engine
- `scripts/generate_wasm_v2.py` - Genera el WASM engine
- `scripts/generate_webgpu.py` - Genera el módulo WebGPU
- `scripts/generate_promises.py` - Genera el módulo de Promises
- `scripts/generate_tests.py` - Genera tests automáticamente
- `scripts/generate_bridge.py` - Genera el bridge JS <-> WASM

Ver `docs/Python_Code_Generation.md` para más detalles.

## Licencia

MIT
