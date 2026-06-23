# Rutas y estado real de Noir Browser

Este archivo sirve como mapa rápido del proyecto y como evaluación honesta del estado actual. Noir Browser no usa Chromium/WebKit/Gecko: el código activo es Rust propio con `winit`, `softbuffer`, `reqwest`, parsers propios y módulos experimentales de WebGPU/WASM/JS.

## Resumen ejecutivo

**Estado actual:** prototipo funcional de navegador/visor web básico.

Puede abrir ventana, aceptar URLs/búsquedas, descargar HTML, parsear contenido, hacer layout simple, renderizar texto/imágenes básicas y mostrar una UI tipo navegador. Eso ya es bastante para un navegador reconstruido desde cero.

**Todavía no es equivalente a Chrome/Firefox/Safari.** Le faltan piezas grandes de plataforma web: DOM vivo completo, CSS moderno, JavaScript compatible con sitios reales, ejecución de scripts conectada al DOM, navegación robusta, almacenamiento, seguridad web, multimedia real, accesibilidad, fuentes modernas, compositing GPU real y Web APIs amplias.

## Ruta principal de ejecución

```text
Cargo workspace
└─ No-Chromium/src/main.rs
   └─ no_chromium::create_browser(AppConfig::default())
      └─ BrowserInstance::run()
         └─ bootstrap::run(config)
            └─ app::execute(config)
               ├─ AppContext::new(config)
               ├─ AppContext::initialize()
               └─ app::event_loop::run(context)
```

Archivos clave:

| Ruta | Rol |
|---|---|
| `Cargo.toml` | Workspace principal. |
| `No-Chromium/Cargo.toml` | Dependencias del navegador. |
| `No-Chromium/src/main.rs` | Entrada del binario. Crea runtime Tokio y lanza la app. |
| `No-Chromium/src/lib.rs` | API pública del crate `no_chromium`. |
| `No-Chromium/src/bootstrap.rs` | Puente entre configuración, logging y app. |
| `No-Chromium/src/app/mod.rs` | Entrada del módulo de aplicación. |
| `No-Chromium/src/app/event_loop.rs` | Loop de ventana con `winit`. |
| `No-Chromium/src/app/context.rs` | Estado central: ventana, tabs, URL bar, fetch, consola, layout. |

## Pipeline real al navegar

```text
Usuario escribe URL
└─ app/input.rs
   └─ AppContext::navigate(url)
      └─ app/navigation.rs
         ├─ network/fetch.rs descarga HTML con reqwest
         ├─ parsers/page_document.rs extrae título, texto, links, imágenes, videos
         ├─ parsers/dom_tree.rs genera nodos DOM internos
         ├─ js_engine_v3 evalúa JS básico, pero la integración DOM real aún es parcial
         ├─ parsers/layout.rs calcula bloques visuales simples
         ├─ media/image_support.rs descarga/decodifica/cachea imágenes
         └─ app/renderer.rs pinta todo en softbuffer
```

## Módulos activos importantes

### `app/` — ventana, UI y navegación visible

| Ruta | Qué hace hoy |
|---|---|
| `app/event_loop.rs` | Crea ventana sin decoraciones, procesa mouse/teclado/scroll/redraw. |
| `app/context.rs` | Mantiene tabs, URL bar, estado de carga, historial básico, consola y búsqueda. |
| `app/input.rs` | Atajos, clicks, foco de barra, tabs y navegación. |
| `app/navigation.rs` | Resuelve URLs/búsquedas, lanza fetch async, parsea HTML y construye layout. |
| `app/renderer.rs` | Dibuja la UI tipo navegador, página nueva, loading, errores, texto, imágenes y videos placeholder. |
| `app/draw.rs` + `app/glyphs.rs` | Primitivas de dibujo y fuente bitmap. |

**Valoración:** esta es la parte más real y conectada del navegador.

### `network/` — descarga HTTP básica

| Ruta | Qué hace hoy |
|---|---|
| `network/fetch.rs` | Cliente HTTP con `reqwest`, redirects manuales, cookies globales simples, lectura de texto. |
| `network/mod.rs` | Reexports y `NetworkCoordinator` stub. |
| `network/socks_proxy.rs`, `doh_resolver.rs`, `circuit.rs` | Stubs/pendiente. |

**Valoración:** suficiente para cargar HTML y recursos simples. Tor/DNS privado/circuitos aún no están implementados realmente en la ruta activa.

### `parsers/` — HTML/CSS/layout propio

| Ruta | Qué hace hoy |
|---|---|
| `parsers/dom_tree.rs` | Parser DOM interno. |
| `parsers/page_document.rs` | Extrae título, textos, links, imágenes, videos/iframes, estilos inline y viewport. |
| `parsers/css_simple.rs`, `css_engine.rs`, `css_lexer.rs` | CSS básico/cascada parcial. |
| `parsers/layout.rs` | Layout vertical y grid simple para muchas imágenes; hit-test de links. |
| `parsers/resource_loader.rs`, `style_collector.rs` | Soporte auxiliar de recursos/estilos. |

**Valoración:** muy buen inicio para motor propio, pero todavía lejos de CSS real: falta flexbox/grid completo, posicionamiento, inline layout real, overflow, stacking contexts, pseudo-clases, media queries completas, fuentes, etc.

### `media/` — imágenes reales, multimedia experimental

| Ruta | Qué hace hoy |
|---|---|
| `media/mod.rs` | Solo conecta `image_support` e `image_manager`. |
| `media/image_support.rs` | Cache LRU simple, detección/decodificación PNG/JPEG/WebP/etc con `image`, dibujo al framebuffer. |
| `media/image_manager.rs` | Cache auxiliar de imágenes. |
| `media/player.rs`, `audio.rs`, `discovery.rs` | No parecen formar parte del árbol activo actual; son prototipo/pendiente. |

**Valoración:** imágenes sí; reproducción real de YouTube/video aún no.

### `js_engine_v3/` — JS propio básico

| Ruta | Qué hace hoy |
|---|---|
| `js_engine_v3/lexer.rs` | Lexer JS propio. |
| `js_engine_v3/parser.rs` | Parser AST propio. |
| `js_engine_v3/interpreter.rs` | Intérprete tree-walking para variables, expresiones, funciones, loops, objetos básicos. |
| `js_engine_v3/builtins.rs` | Builtins parciales. |
| `js_engine_v3/dom.rs` | DOM interno del motor JS. |
| `js_engine_v3/mod.rs` | API de compatibilidad usada por `app/navigation.rs`. Varias funciones clave son placeholder. |

**Punto crítico:** `extract_inline_scripts()` devuelve vacío, `sync_dom_to_js_engine()` es placeholder, `rebuild_page_from_dom()` es placeholder y `take_mutated_flag()` devuelve `false`. O sea: aunque hay un intérprete JS propio, la integración real HTML → JS → DOM → layout todavía está incompleta.

### `webgpu/` — diseño experimental de GPU

| Ruta | Qué hace hoy |
|---|---|
| `webgpu/mod.rs` | Módulo WebGPU experimental. |
| `webgpu/renderer.rs` | Renderer lógico con estadísticas, buffers/pipelines simulados. |
| `webgpu/integration.rs` | Integración opcional conceptual. |
| `webgpu/bridge.rs` | Puente JS/WebGPU experimental. |

**Valoración:** existe arquitectura WebGPU, pero el render visible actual está en `softbuffer` CPU (`app/renderer.rs`). Si dices “WGPU/WebGPU”, en el estado actual conviene aclarar que está diseñado/prototipado, no es todavía el backend principal real de pantalla.

### `wasm_v2/` — WebAssembly experimental

Hay decoder/interpreter/runtime/validator/JIT conceptual. Compila, pero no está conectado a la navegación web real.

### `utils/` — proceso/IPC/memoria

Incluye modelo de proceso adaptativo y stubs de IPC/memoria. Útil como base, pero no es aislamiento tipo Chrome real todavía.

### `archive/` — código archivado/no activo

`No-Chromium/src/archive/` contiene mucho código antiguo, generado o prototipo. No debe contarse como funcionalidad activa salvo que se reactive explícitamente.

## Qué tan bueno es ahora mismo

### Puntos fuertes

- Proyecto compila actualmente con `cargo test --workspace --no-run`.
- Arquitectura modular bastante clara: app, network, parsers, media, JS, WebGPU, WASM.
- UI real con ventana propia, tabs, URL bar, loading, consola, búsqueda y atajos.
- Navegación HTTP funcional para HTML básico.
- Parser/layout/render propio, sin Chromium.
- Soporte real de imágenes decodificadas y cacheadas.
- Tests de integración para URL resolver, config, PageDocument, JS básico y UI state.

### Riesgos/debilidades

- Mucho código tiene comentarios tipo “Generated by Python” o “stub”; hay que separar lo real de lo aspiracional.
- La documentación actual promete más de lo que la ruta activa implementa.
- JS no está realmente conectado a scripts inline ni mutaciones DOM en la navegación actual.
- WebGPU no es todavía el render backend principal visible.
- YouTube puede cargar/mostrar partes porque se descarga HTML y se dibujan links/imágenes/placeholders, pero no porque soporte toda la plataforma que YouTube necesita.
- Cookies, seguridad, sandbox, CORS, storage, service workers, media source extensions, codecs y APIs modernas aún son muy básicos o ausentes.
- Hay warnings de compilación y archivos no conectados que podrían confundir el mantenimiento.

## Prioridad recomendada

### Fase 1 — hacer honesto y sólido lo que ya funciona

1. Limpiar warnings fáciles y código muerto activo.
2. Separar claramente `archive/` y prototipos no compilados de módulos reales.
3. Ajustar README/ARCHITECTURE para distinguir “implementado”, “experimental” y “roadmap”.
4. Añadir tests de navegación/parsing/layout con HTML real pequeño.
5. Mejorar errores de red y estado de carga por tab.

### Fase 2 — DOM + JS mínimo útil

1. Hacer que `extract_inline_scripts()` extraiga scripts reales.
2. Sincronizar `PageDocument`/`dom_tree` con `js_engine_v3::Dom`.
3. Implementar mutaciones básicas: `document.createElement`, `appendChild`, `textContent`, `querySelector`.
4. Recalcular layout cuando JS cambie DOM.
5. Conectar `console.log` del JS a la consola visible de la app.

### Fase 3 — CSS/layout web básico serio

1. Mejorar inline layout de texto y links.
2. Soportar `display:block/inline/none`, margins/padding/borders de forma consistente.
3. Implementar flexbox básico.
4. Cargar CSS externo de forma async real.
5. Mejorar fuentes: rasterización TTF/MSDF en vez de bitmap simple.

### Fase 4 — GPU real

1. Decidir backend real: `wgpu` crate, Vulkan/Ash directo o mantener softbuffer.
2. Conectar el renderer WebGPU a la ventana real.
3. Pasar rectángulos/texturas/texto a buffers GPU reales.
4. Medir FPS, uso de CPU y latencia.

### Fase 5 — compatibilidad con sitios complejos

1. Fetch de subrecursos: CSS, JS externo, imágenes relativas, fuentes.
2. CORS, CSP, cookies correctas, storage.
3. Event loop JS, timers y promesas conectadas al navegador.
4. Formularios, inputs editables, navegación por links/form submit.
5. Multimedia real: audio/video, MSE/streams si apuntas a YouTube.

## Conclusión

Noir Browser es un proyecto muy ambicioso y ya tiene una base real. Lo mejor ahora es no venderlo todavía como “Chrome reconstruido completo”, sino como:

> **motor web experimental en Rust, sin Chromium, con UI propia, networking, parser/layout/render básico, imágenes y módulos experimentales de JS/WebGPU/WASM.**

Si sigues por fases y haces que cada promesa esté conectada a código activo + tests, puede convertirse en una base muy seria.
