# 🌌 Noir Browser v2.0 (No-Chromium Engine) - GPU-First Architecture

![Rust](https://img.shields.io/badge/rust-1.75%2B-blue.svg)
![Vulkan](https://img.shields.io/badge/Graphics-Vulkan%20%2F%20Ash-red.svg)
![License](https://img.shields.io/badge/License-MIT%2FApache--2.0-green.svg)
![Status](https://img.shields.io/badge/Status-Reconstrucci%C3%B3n%20v2.0-orange.svg)

> ⚡ **RECONSTRUCCIÓN EN PROGRESO**: Estamos reescribiendo Noir Browser desde cero con arquitectura **GPU-First** para ser el navegador más rápido del mundo. Ver [RECONSTRUCCION_v2.md](./RECONSTRUCCION_v2.md) para detalles.

Un motor de navegación web ultrarrápido, moderno e independiente desarrollado desde cero en Rust y Vulkan (Ash), diseñado para romper la hegemonía y la pesadez de los motores basados en Chromium, WebKit y Gecko.

---

## 🚀 ¿Por qué No-Chromium? (El Potencial y la Misión)

Hoy en día, casi todos los navegadores web modernos (Chrome, Edge, Brave, Opera, Vivaldi) son clones con diferentes pieles que corren bajo el mismo motor masivo y sediento de recursos: Chromium. Esto ha creado un monopolio tecnológico de facto, exponiendo a los usuarios a telemetría invasiva, sobrecarga de memoria y un ecosistema web uniforme.

**Noir Browser (No-Chromium) nace con cuatro propósitos clave:**

✨ **Independencia Tecnológica**: Crear un renderizador web nativo 2D totalmente escrito en Rust, eliminando el C++ heredado de 30 años y reduciendo la superficie de vulnerabilidad.

🎮 **Rendimiento de GPU Puro**: Dibujar cada elemento del DOM, imagen, botón y texto usando llamadas a la GPU con Vulkan, logrando renderizados pixel-perfect estables a más de 60fps sin sobrecargar la CPU.

💾 **Consumo Eficiente y Seguro**: Menos del 10% del consumo de memoria RAM de un proceso tradicional de Chrome, gracias a una arquitectura concurrente y libre de recolección de basura (Garbage Collector).

🔥 **v2.0 - GPU-First**: Todo el pipeline (parsing, layout, composición) ejecutándose en paralelo en la GPU mediante Compute Shaders. Zero-copy memory management. Búsqueda nativa integrada.

---

## 🛠️ Arquitectura v2.0: GPU-First Pipeline

```text
┌─────────────────┐
│   NETWORK       │ HTTP/3 + QUIC + DoH + Pre-fetch predictivo
└────────┬────────┘
         ↓
┌─────────────────┐
│   PARSING (GPU) │ HTML/CSS tokenization en Compute Shaders
└────────┬────────┘
         ↓
┌─────────────────┐
│   LAYOUT (GPU)  │ Flexbox/Grid paralelizado + Text shaping
└────────┬────────┘
         ↓
┌─────────────────┐
│   COMPOSITION   │ Vulkan render passes optimizados + Dirty rect
└────────┬────────┘
         ↓
┌─────────────────┐
│   PRESENT       │ Swapchain + V-Sync + Frame pacing
└─────────────────┘
```

### Principios de Diseño:
- **Zero-Copy**: Datos nunca se copian innecesariamente entre CPU↔GPU
- **Async Everything**: Tokio + async/await en cada capa, sin bloqueos
- **Compute-First**: Parsing y layout ejecutándose en paralelo en GPU
- **Search-Native**: Búsqueda integrada en el kernel, no como addon

> 📋 Para detalles técnicos completos: [RECONSTRUCCION_v2.md](./RECONSTRUCCION_v2.md)

---

## 📁 Estructura del Proyecto (v2.0)

```text
Noir_Browser/
├── No-Chromium/
│   ├── src/
│   │   ├── core/           # Kernel: gpu/, memory/, async/
│   │   ├── parsing/        # HTML/CSS/DOM parsers en GPU
│   │   ├── layout/         # Flexbox/Grid + text shaping paralelo
│   │   ├── search/         # OmniSearch nativo + autocomplete GPU
│   │   ├── network/        # HTTP/3 + QUIC + pre-fetch ML
│   │   └── vulkan_engine/  # Render core optimizado
│   ├── shaders/            # Compute/Graphics shaders fuente
│   ├── scripts/            # Setup, build, benchmark
│   └── tests/              # Perf, WPT, search tests
├── RECONSTRUCCION_v2.md    # 🚀 Roadmap completo de reconstrucción
├── Fases.md                # 📋 Plan de implementación por fases
└── README.md               # Este documento
```

---

## 🎯 Roadmap de Reconstrucción

| Fase | Objetivo | Duración | Estado |
|------|----------|----------|--------|
| 🔴 0 | Foundation: Allocator + Vulkan setup | 2 sem | 🔄 En progreso |
| 🔴 1 | GPU Parsing Engine (HTML/CSS en shaders) | 3 sem | ⏳ Pendiente |
| 🔴 2 | Layout paralelo (Flexbox/Grid en GPU) | 3 sem | ⏳ Pendiente |
| 🔴 3 | OmniSearch nativo + autocomplete VRAM | 2 sem | ⏳ Pendiente |
| 🔴 4 | Network: HTTP/3 + pre-fetch inteligente | 2 sem | ⏳ Pendiente |
| 🔴 5 | Polish: Profiling, benchmarks, docs | 2 sem | ⏳ Pendiente |

> ✅ **Meta final**: Startup <200ms, <30MB RAM/tab, 120fps locked, búsqueda <100ms

---

## 🚀 Inicio Rápido (v2.0)

```bash
# 1. Clonar y entrar al proyecto
git clone https://github.com/tu-user/noir-browser.git
cd Noir_Browser/No-Chromium

# 2. Ejecutar setup de Fase 0 (allocator + Vulkan optimizado)
bash scripts/setup_phase0.sh

# 3. Compilar en modo release (máximas optimizaciones)
cargo build --release

# 4. Ejecutar con profiling opcional
# TRACY=1 ./target/release/noir-browser

# 5. (Opcional) Ejecutar benchmarks
cargo bench
```

### Requisitos:
- Rust 1.75+ con nightly (para algunas features)
- Vulkan 1.2+ con soporte para compute shaders
- glslc (para compilación de shaders, opcional en dev)
- Windows 10/11, Linux con Vulkan drivers actualizados

---

## 🌓 Características Especiales (v2.0)

✨ **Tema Oscuro Inteligente (Noir Dark Theme)**: Analiza en tiempo real la luminancia de los fondos y textos CSS. Los fondos claros se convierten en un gris oscuro premium (`#1f2023`), y los textos oscuros se iluminan suavemente para evitar destellos oculares, conservando el diseño original.

✨ **Pre-Cache Off-line con GPU**: Los recursos más comunes se pre-cargan y decodifican de forma asíncrona en VRAM mediante Tokio + compute shaders, eliminando stalls de CPU.

✨ **OmniSearch Nativo**: Barra de dirección = barra de búsqueda. Multi-engine paralelo (DuckDuckGo, SearXNG), resultados pre-renderizados en GPU, autocomplete con trie en VRAM (<1ms).

✨ **Zero-GC Architecture**: Sin recolección de basura, sin pauses, sin overhead. Arena allocators + ring buffers para memory management determinista.

---

## 📈 Métricas de Rendimiento (Objetivos v2.0)

| Métrica | Noir v2.0 (objetivo) | Chrome (referencia) |
|---------|---------------------|---------------------|
| Startup time | **< 200ms** | ~800ms |
| RAM por tab | **< 30MB** | ~150-300MB |
| FPS en scroll | **120fps locked** | 60fps variable |
| Búsqueda → resultados | **< 100ms** | ~300-500ms |
| Parse HTML 1MB | **< 5ms (GPU)** | ~50ms (CPU) |
| Layout complejo | **< 10ms (paralelo)** | ~100ms (secuencial) |

---

## 🤝 Contribuir

¡Estamos reconstruyendo desde cero y necesitamos ayuda!

1. Lee [RECONSTRUCCION_v2.md](./RECONSTRUCCION_v2.md) para entender la arquitectura
2. Revisa los issues etiquetados con `good first issue` o `fase-0`
3. Ejecuta `bash scripts/setup_phase0.sh` para configurar tu entorno
4. ¡Envía tu PR! Usamos `cargo fmt`, `cargo clippy` y tests obligatorios

### Áreas que necesitan ayuda:
- 🔹 Implementación de compute shaders para HTML/CSS parsing
- 🔹 Bridge DOM mínimo para integración con Boa (JS engine)
- 🔹 Optimización de Vulkan pipeline caching
- 🔹 Tests de Web Platform Tests (WPT)

---

## 📄 Licencia

Noir Browser está licenciado bajo **MIT** o **Apache 2.0**, a tu elección.

---

> 💡 **¿Listo para contribuir?** Empieza con la [Fase 0](./RECONSTRUCCION_v2.md#-fase-0-foundation-2-semanas) o abre un issue para discutir ideas. ¡Hagamos el navegador más rápido del mundo! 🚀
