# Noir Browser - Roadmap Detallado

## 🎯 VISIÓN GENERAL

**Objetivo**: Convertir Noir Browser en un navegador capaz de ver YouTube, memes, imágenes, y contenido web moderno, manteniendo la eficiencia de RAM (10MB actual) y la independencia de Chromium.

---

## 🤔 VULKAN vs WASM: ¿TIENE SENTIDO?

### **Pregunta del usuario**: ¿Por qué Vulkan y no WASM?

### **Respuesta corta**: SÍ, tiene TODO el sentido. Son cosas COMPLETAMENTE diferentes.

### **Explicación detallada**:

| Aspecto | Vulkan | WebAssembly (WASM) |
|---------|--------|-------------------|
| **¿Qué es?** | API de gráficos 3D de bajo nivel | Formato binario para código ejecutable en navegadores |
| **Propósito** | Renderizar gráficos con GPU | Ejecutar código (C++, Rust, Go) en el navegador |
| **Nivel** | Hardware (GPU) | Software (CPU) |
| **Velocidad** | Extremadamente rápido (GPU) | Rápido (casi nativo) |
| **Uso en navegadores** | Renderizado de páginas, animaciones, video | Código de páginas web, juegos, apps |
| **Reemplaza** | OpenGL, DirectX | JavaScript (parcialmente) |
| **En Chrome** | ✅ Sí, usa Vulkan/Metal/DirectX | ✅ Sí, ejecuta WASM |

### **Analogía simple**:
- **Vulkan** = El motor de un carro (potencia, velocidad)
- **WASM** = El combustible que pones en el motor (código que se ejecuta)
- **JavaScript** = Otro tipo de combustible (más lento que WASM)

### **¿Por qué Noir usa Vulkan?**

✅ **Ventajas de Vulkan en Noir**:
1. **Rendimiento extremo**: 10-100x más rápido que software rendering
2. **Control total**: Acceso directo a GPU
3. **Multiplataforma**: Windows, Linux, Android, etc.
4. **Moderno**: Diseñado para hardware actual
5. **Eficiencia energética**: Mejor uso de batería en laptops

✅ **¿Y WASM?**
- **También es importante**, pero es una capa diferente
- Noir DEBE soportar WASM para páginas modernas
- YouTube USA WASM para decodificación de video
- Chrome/Firefox usan AMBOS: Vulkan para gráficos + WASM para código

### **Arquitectura correcta de Noir**:

```
┌─────────────────────────────────────┐
│   PÁGINA WEB (HTML/CSS/JS)          │
├─────────────────────────────────────┤
│   JavaScript Engine (Boa)           │
│   + WebAssembly Support (futuro)    │
├─────────────────────────────────────┤
│   Layout Engine (CSS + HTML)        │
├─────────────────────────────────────┤
│   Rendering:                        │
│   - Vulkan (GPU) ← FASE F           │
│   - Softbuffer (CPU) ← actual       │
└─────────────────────────────────────┘
```

---

## 📋 PLAN DE IMPLEMENTACIÓN

### **FASE G: Image Support Enhancement** (1-2 semanas)

**Objetivo**: Que las imágenes funcionen perfectamente (memes, fotos, etc.)

#### **G1: Async Image Loading** (2-3 días)
- [ ] Mejorar `media/image_support.rs` con `tokio::spawn`
- [ ] Implementar image cache con LRU eviction
- [ ] Añadir loading states (skeleton, spinner)
- [ ] Retry logic para imágenes que fallan

#### **G2: More Image Formats** (3-4 días)
- [ ] **WebP** support (usado por Google, YouTube)
- [ ] **GIF** support (animaciones)
- [ ] **SVG** support (iconos, logos)
- [ ] **AVIF** support (formato moderno)

#### **G3: Image Optimization** (2-3 días)
- [ ] Lazy loading (`<img loading="lazy">`)
- [ ] Responsive images (`<img srcset>`)
- [ ] Image preloading (`<link rel="preload">`)
- [ ] Background images (`background-image: url()`)

---

### **FASE H: HTML5 Video Support** (2-3 semanas)

**Objetivo**: Soporte básico para `<video>` tag (necesario para YouTube)

#### **H1: Video Element Parsing** (2-3 días)
- [ ] Parse `<video>` tag en `dom_tree.rs`
- [ ] Extraer atributos: `src`, `controls`, `autoplay`, `loop`
- [ ] Soporte para `<source>` tags
- [ ] Soporte para subtítulos (`<track>`)

#### **H2: Video Rendering** (5-7 días)
- [ ] Integrar `ffmpeg-next` o `gstreamer` para decodificación
- [ ] Renderizar primer frame como poster
- [ ] Controles básicos (play/pause/volume)
- [ ] Fullscreen support

#### **H3: Codec Support** (5-7 días)
- [ ] **H.264** (más común, usado por YouTube)
- [ ] **VP9** (open source, usado por YouTube)
- [ ] **AV1** (moderno, eficiente)
- [ ] **WebM** container

---

### **FASE I: Advanced JavaScript** (3-4 semanas)

**Objetivo**: Que el JS engine pueda ejecutar código moderno (necesario para YouTube, redes sociales)

#### **I1: Promises & async/await** (1 semana)
- [ ] Implementar `Promise` en Boa bindings
- [ ] Soporte para `async`/`await` syntax
- [ ] Microtask queue
- [ ] Error propagation

#### **I2: Fetch API completa** (3-4 días)
- [ ] `fetch()` con Promises (no sync)
- [ ] Request/Response objects
- [ ] Headers API
- [ ] Body streams

#### **I3: Web APIs adicionales** (1-2 semanas)
- [ ] `localStorage` / `sessionStorage`
- [ ] `IndexedDB` (básico)
- [ ] `WebSocket` (necesario para chat, live)
- [ ] `Service Worker` (básico)
- [ ] `IntersectionObserver` (lazy loading)
- [ ] `MutationObserver` (DOM changes)

---

### **FASE J: CSS Modern Features** (2-3 semanas)

**Objetivo**: Soporte para CSS moderno (necesario para YouTube, redes sociales)

#### **J1: Flexbox** (1 semana)
- [ ] `display: flex` parsing
- [ ] Flex container properties
- [ ] Flex item properties
- [ ] Alignment y justification

#### **J2: Grid Layout** (1 semana)
- [ ] `display: grid` parsing
- [ ] Grid template columns/rows
- [ ] Grid areas
- [ ] Auto-placement

#### **J3: Advanced CSS** (3-4 días)
- [ ] Media queries (`@media`)
- [ ] Animations (`@keyframes`)
- [ ] Transforms (`transform: rotate/scale/translate`)
- [ ] Pseudo-classes (`:hover`, `:focus`, `:nth-child`)

---

### **FASE K: FASE F - Vulkan Renderer** (4-6 semanas)

**Objetivo**: Reemplazar softbuffer con Vulkan para rendimiento extremo

#### **K1: Vulkan Integration** (1-2 semanas)
- [ ] Activar `vulkan_engine/` module
- [ ] Conectar con winit window
- [ ] Swapchain management
- [ ] Render pass setup

#### **K2: GPU-Accelerated Rendering** (2-3 semanas)
- [ ] Vertex/Fragment shaders para texto
- [ ] Texture rendering para imágenes
- [ ] Compositor para layers
- [ ] Hardware-accelerated CSS effects

#### **K3: Performance Optimization** (1 semana)
- [ ] Frame timing optimization
- [ ] Memory pool para GPU resources
- [ ] Command buffer recycling
- [ ] Multi-threading rendering

---

### **FASE L: Real-World Site Support** (ongoing)

**Objetivo**: Hacer que Noir funcione con sitios reales

#### **L1: YouTube** (2-3 semanas)
- [ ] Player UI rendering
- [ ] Video controls
- [ ] Comments loading
- [ ] Search functionality
- [ ] Related videos

#### **L2: Social Media** (2-3 semanas)
- [ ] Twitter/X (text + images)
- [ ] Reddit (posts + comments)
- [ ] GitHub (code rendering)
- [ ] Discord (chat básico)

#### **L3: News & Blogs** (1 semana)
- [ ] Article rendering
- [ ] Image galleries
- [ ] Comment systems
- [ ] Infinite scroll

---

## 🎯 PRIORIDADES

### **CRÍTICO (Hacer YA)** 🔴
1. ✅ Image async loading (ya implementado parcialmente)
2. ✅ More image formats (WebP, GIF)
3. ✅ Promises/async-await en JS

### **IMPORTANTE (1 mes)** 🟡
1. HTML5 video support
2. Flexbox en CSS
3. Fetch API con Promises

### **DESEABLE (3 meses)** 🟢
1. Vulkan renderer
2. Grid layout
3. WebSocket

### **FUTURO (6+ meses)** 🌟
1. Service Workers
2. IndexedDB
3. PWA support

---

## 📊 MÉTRICAS DE ÉXITO

### **Técnicas**
- ✅ **RAM**: Mantener <50MB con 10 tabs (actualmente 10MB con 1 tab)
- ✅ **CPU**: <5% en idle
- ✅ **Tests**: 100+ tests pasando
- ✅ **Sitios**: Soporte para top 100 sitios web

### **Funcionales**
- ✅ Ver YouTube (búsqueda + video)
- ✅ Ver memes (imágenes)
- ✅ Leer noticias (texto + imágenes)
- ✅ Usar redes sociales (básico)
- ✅ Navegación fluida

---

## 🚀 EMPEZANDO AHORA

Voy a aplicar la **FASE G1: Async Image Loading** primero porque:
1. Es la base para ver memes
2. Ya tienes código parcial
3. Es rápido de implementar
4. Da resultados inmediatos

¿Quieres que empiece con eso?
