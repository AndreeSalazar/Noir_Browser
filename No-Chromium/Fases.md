# 🌌 Noir Browser - Reconstrucción Total (Ultra-Fast + Web Search)

> **Objetivo:** Reconstruir la base completa del motor para lograr:
> - ⚡ **Máxima velocidad** explotando Vulkan 1.3 al 100% (zero-copy, async compute, descriptor indexing)
> - 🔍 **Búsqueda web nativa integrada** sin depender de APIs externas lentas
> - 🚀 **Pipeline renderizado re-architected** para <8ms frame time (120fps+)
> - 💾 **Memoria <50MB** por tab en uso normal

---

## 📋 Índice de Fases

| Fase | Nombre | Duración Estimada | Impacto | Prioridad |
|------|--------|-------------------|---------|-----------|
| 🔥 0 | **Core Vulkan Ultra-Fast** | 3 semanas | Crítico | 🔴 Alta |
| 🔥 1 | **Pipeline Zero-Copy** | 2 semanas | Crítico | 🔴 Alta |
| 🔥 2 | **Web Search Engine Nativo** | 4 semanas | Crítico | 🔴 Alta |
| 🔥 3 | **JS Engine Integrado (Boa)** | 3 semanas | Crítico | 🔴 Alta |
| 🔥 4 | **Layout Avanzado (Flex/Grid)** | 3 semanas | Alto | 🟡 Media |
| 🔥 5 | **Font Rendering MSDF GPU** | 2 semanas | Alto | 🟡 Media |
| 🔥 6 | **Video Hardware Decode** | 3 semanas | Medio | 🟢 Baja |
| 🔥 7 | **Polishing & WPT Compliance** | 4 semanas | Alto | 🟡 Media |

---

## 🔥 FASE 0: Core Vulkan Ultra-Fast (Semana 1-3)

### 🎯 Objetivo
Reconstruir el motor Vulkan desde cero para eliminar todo overhead y lograr **zero-copy rendering**.

### 🏗️ Arquitectura Nueva

```rust
// src/vulkan_engine/core.rs
pub struct UltraFastVulkanEngine {
    // 🆕 Device con features 1.3 habilitadas
    device: Arc<ash::Device>,
    
    // 🆕 Async Compute Queue separado (para layout + decode en paralelo)
    compute_queue: Queue,
    graphics_queue: Queue,
    transfer_queue: Queue,
    
    // 🆕 Descriptor Indexing (bindless resources)
    descriptor_pool: DescriptorPool,
    storage_buffers: Vec<StorageBuffer>,
    
    // 🆕 Memory Allocator con VMA (Vulkan Memory Allocator)
    vma_allocator: Allocator,
    
    // 🆕 Render Passes múltiples (UI, Content, Compositing)
    ui_render_pass: RenderPass,
    content_render_pass: RenderPass,
    composite_render_pass: RenderPass,
    
    // 🆕 Pipeline Cache pre-warm
    pipeline_cache: PipelineCache,
    shader_modules: HashMap<ShaderKey, ShaderModule>,
    
    // 🆕 Framebuffer swapchain con triple buffering
    swapchain: Swapchain,
    frame_resources: [FrameResources; 3],
    
    // 🆕 Sincronización con timelines semaphores
    timeline_semaphore: Semaphore,
    frame_counter: u64,
}
```

### ⚡ Optimizaciones Críticas

| Optimización | Implementación | Ganancia Esperada |
|--------------|----------------|-------------------|
| **Bindless Resources** | `VK_EXT_descriptor_indexing` + storage buffers | -60% draw calls |
| **Async Compute** | Layout + image decode en compute queue paralelo | -40% CPU idle time |
| **Triple Buffering** | 3 frames in-flight con timeline semaphores | +50% GPU utilization |
| **Pipeline Caching** | Pre-compile shaders + PSO cache en disco | -80% stutter en primer load |
| **Zero-Copy Staging** | Mapped buffers con `VK_MEMORY_PROPERTY_HOST_COHERENT_BIT` | -70% memcpy overhead |
| **Render Pass Tiling** | Subpasses con input attachments para UI overlay | -30% bandwidth GPU |

### 📝 Implementación Step-by-Step

#### Paso 1: Inicialización Vulkan 1.3
```rust
// src/vulkan_engine/init.rs
pub fn create_ultra_fast_instance() -> Result<UltraFastVulkanEngine, Error> {
    let entry = Entry::load()?;
    
    // 🆕 Habilitar features 1.3 obligatorias
    let features = vk::PhysicalDeviceFeatures::builder()
        .geometry_shader(false) // 2D no necesita
        .tessellation_shader(false)
        .multi_draw_indirect(true) // 🆕 Draw calls agrupados
        .draw_indirect_first_instance(true)
        .build();
    
    let vulkan_13_features = vk::PhysicalDeviceVulkan13Features::builder()
        .synchronization2(true) // 🆕 Sincronización moderna
        .dynamic_rendering(true) // 🆕 Sin render passes fijos
        .build();
    
    // 🆕 Extensiones críticas
    let extensions = [
        ash::khr::swapchain::NAME,
        ash::khr::timeline_semaphore::NAME,
        ash::ext::descriptor_indexing::NAME,
        ash::ext::memory_budget::NAME,
    ];
    
    // ... crear device, queues, allocator
}
```

#### Paso 2: Sistema de Memoria VMA
```rust
// src/vulkan_engine/memory.rs
pub struct MemoryManager {
    allocator: Arc<Mutex<Allocator>>,
    
    // 🆕 Pools por tipo de uso
    gpu_only_pool: AllocationPool,    // Texturas, buffers de GPU
    cpu_to_gpu_pool: AllocationPool,  // Staging (upload)
    gpu_to_cpu_pool: AllocationPool,  // Readback (screenshot)
    
    // 🆕 Fragmentación tracking
    fragmentation_threshold: f32,
}

impl MemoryManager {
    pub fn alloc_buffer(
        &self,
        size: usize,
        usage: vk::BufferUsageFlags,
        memory_type: MemoryType
    ) -> Result<BufferAllocation, Error> {
        let buffer_info = vk::BufferCreateInfo::builder()
            .size(size as u64)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .build();
        
        let alloc_info = AllocationCreateInfo {
            usage: match memory_type {
                MemoryType::GpuOnly => MemoryUsage::GpuOnly,
                MemoryType::CpuToGpu => MemoryUsage::CpuToGpu,
                MemoryType::GpuToCpu => MemoryUsage::GpuToCpu,
            },
            required_flags: vk::MemoryPropertyFlags::DEVICE_LOCAL,
            ..Default::default()
        };
        
        self.allocator.lock().unwrap().create_buffer(
            &buffer_info,
            &alloc_info
        )
    }
}
```

#### Paso 3: Render Pipeline Ultra-Fast
```rust
// src/vulkan_engine/render_pipeline.rs
pub struct RenderPipeline {
    // 🆕 Dynamic rendering (VK_KHR_dynamic_rendering)
    rendering_info: vk::RenderingInfoKHR,
    
    // 🆕 Multi-draw indirect para batch de geometría
    draw_commands: Vec<DrawIndirectCommand>,
    
    // 🆕 Descriptor sets bindless
    bindless_descriptor_set: DescriptorSet,
    
    // 🆕 Push constants para parámetros por-frame
    push_constants: PushConstants,
}

impl RenderPipeline {
    pub fn render_frame(&mut self, frame_idx: usize) -> Result<(), Error> {
        let frame = &self.frame_resources[frame_idx];
        
        // 🆕 Wait con timeline semaphore (no block)
        self.wait_for_frame(frame.frame_number);
        
        // 🆕 Begin dynamic rendering
        let rendering_info = vk::RenderingInfoKHR::builder()
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.swapchain.extent,
            })
            .layer_count(1)
            .color_attachments(&[vk::RenderingAttachmentInfoKHR::builder()
                .image_view(frame.color_image_view)
                .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::STORE)
                .clear_value(vk::ClearValue {
                    color: vk::ClearColorValue { float32: [0.12, 0.13, 0.14, 1.0] }
                })
                .build()])
            .build();
        
        unsafe {
            self.device.cmd_begin_rendering(frame.command_buffer, &rendering_info);
        }
        
        // 🆕 Bind pipeline + descriptor sets bindless
        self.bind_pipeline();
        self.bind_bindless_descriptors();
        
        // 🆕 Multi-draw indirect (un solo draw call para toda la UI)
        self.execute_multi_draw_indirect();
        
        unsafe {
            self.device.cmd_end_rendering(frame.command_buffer);
        }
        
        // 🆕 Submit con timeline semaphore
        self.submit_frame(frame);
        
        Ok(())
    }
    
    fn execute_multi_draw_indirect(&mut self) {
        // 🆕 Agrupar TODOS los draw calls en un solo buffer
        let draw_buffer = self.draw_commands.as_slice();
        
        unsafe {
            self.device.cmd_draw_indirect(
                self.frame_resources.current().command_buffer,
                self.draw_buffer_handle,
                0,
                draw_buffer.len() as u32,
                std::mem::size_of::<vk::DrawIndirectCommand>() as u32,
            );
        }
    }
}
```

### ✅ Criterios de Aceptación Fase 0
- [ ] Vulkan 1.3 con `dynamic_rendering` y `synchronization2`
- [ ] VMA allocator integrado (0 leaks de memoria)
- [ ] Triple buffering con timeline semaphores
- [ ] Multi-draw indirect implementado (>70% reducción draw calls)
- [ ] Pipeline cache en disco (0 stutter en segundo launch)
- [ ] Frame time < 8ms a 120fps en 1080p
- [ ] Consumo RAM < 50MB por tab en idle

---

## 🔥 FASE 1: Pipeline Zero-Copy (Semana 4-5)

### 🎯 Objetivo
Eliminar TODAS las copias de memoria entre CPU y GPU. Parser → Layout → GPU en un solo flujo.

### 🏗️ Arquitectura

```rust
// src/pipeline/zero_copy.rs
pub struct ZeroCopyPipeline {
    // 🆕 Parser produce directamente en GPU buffers
    dom_buffer: GpuBuffer,          // DOM binario en GPU
    css_buffer: GpuBuffer,          // Estilos computados en GPU
    layout_buffer: GpuBuffer,       // Cajas layout en GPU
    
    // 🆕 Compute shader para layout (CPU no toca)
    layout_compute_pipeline: ComputePipeline,
    
    // 🆕 Transferencias con DMA (direct memory access)
    dma_transfer: DmaEngine,
    
    // 🆕 Async entre parsing y renderizado
    async_bridge: AsyncBridge,
}

impl ZeroCopyPipeline {
    pub fn process_page(&mut self, html: &str, css: &str) -> Result<(), Error> {
        // 🆕 Parse HTML directo a buffer GPU
        self.parse_html_to_gpu(html)?;
        
        // 🆕 Parse CSS directo a buffer GPU
        self.parse_css_to_gpu(css)?;
        
        // 🆕 Compute shader resuelve layout en GPU
        self.run_layout_compute()?;
        
        // 🆕 Render directo sin copias intermedias
        self.render_from_gpu_buffers()?;
        
        Ok(())
    }
    
    fn parse_html_to_gpu(&mut self, html: &str) -> Result<(), Error> {
        // 🆕 Parser produce nodos DOM directamente en mapped buffer
        let mut writer = GpuBufferWriter::new(&mut self.dom_buffer);
        
        for token in HtmlLexer::new(html) {
            writer.write_node(&token.to_gpu_node());
        }
        
        // 🆕 Flush directo a GPU (zero-copy si coherent)
        writer.flush()?;
        
        Ok(())
    }
    
    fn run_layout_compute(&mut self) -> Result<(), Error> {
        // 🆕 Compute shader lee DOM + CSS, escribe layout
        let dispatch = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
            .build();
        
        unsafe {
            self.device.cmd_begin_compute(self.compute_buffer, &dispatch);
            
            // 🆕 Bind layout compute pipeline
            self.device.cmd_bind_pipeline(
                self.compute_buffer,
                vk::PipelineBindPoint::COMPUTE,
                self.layout_compute_pipeline.handle,
            );
            
            // 🆕 Dispatch con workgroups calculados
            let workgroups = (self.dom_buffer.size / 256) + 1;
            self.device.cmd_dispatch(self.compute_buffer, workgroups, 1, 1);
            
            self.device.cmd_end_compute(self.compute_buffer);
        }
        
        Ok(())
    }
}
```

### ⚡ Optimizaciones Zero-Copy

| Técnica | Antes | Después | Ganancia |
|---------|-------|---------|----------|
| **Parser → DOM** | CPU alloc → memcpy → GPU | CPU write directo a mapped buffer | -80% memcpy |
| **CSS → Layout** | CPU calcula → GPU upload | Compute shader GPU puro | -90% CPU usage |
| **Layout → Render** | CPU structs → GPU buffers | Buffers GPU leídos directo por shaders | -70% latency |
| **Image Decode** | CPU decode → staging → GPU | NVDEC/Vulkan Video → GPU directo | -95% CPU decode |

### ✅ Criterios de Aceptación Fase 1
- [ ] Parser escribe directo a GPU buffers (0 allocs intermedios)
- [ ] Compute shader resuelve layout básico (block + inline)
- [ ] Async bridge entre parsing y renderizado
- [ ] DMA engine para transferencias optimizadas
- [ ] Frame time < 5ms en páginas simples

---

## 🔥 FASE 2: Web Search Engine Nativo (Semana 6-9)

### 🎯 Objetivo
Implementar búsqueda web integrada **sin depender de APIs externas**, con indexación local y resultados instantáneos.

### 🏗️ Arquitectura

```rust
// src/search/web_engine.rs
pub struct WebSearchEngine {
    // 🆕 Crawler asíncrono multi-sitio
    crawler: AsyncCrawler,
    
    // 🆕 Indexador local (tipo SQLite FTS5)
    indexer: LocalIndexer,
    
    // 🆕 Cache de resultados con TTL
    result_cache: LruCache<SearchQuery, SearchResult>,
    
    // 🆕 Ranking algorithm (tipo BM25)
    ranker: Bm25Ranker,
    
    // 🆕 Sugerencias en tiempo real
    suggestion_engine: TrieAutocomplete,
    
    // 🆕 Filtros por tipo (web, imágenes, noticias)
    filters: SearchFilters,
}

pub struct AsyncCrawler {
    // 🆕 Tokio runtime dedicado
    runtime: Runtime,
    
    // 🆕 Connection pool optimizado
    connection_pool: ConnectionPool,
    
    // 🆕 Robots.txt respetado
    robots_cache: HashMap<Domain, RobotsRules>,
    
    // 🆕 Rate limiting inteligente
    rate_limiter: RateLimiter,
    
    // 🆕 Parser de resultados (meta tags, structured data)
    result_parser: StructuredDataParser,
}

impl WebSearchEngine {
    pub async fn search(&mut self, query: &str) -> Result<SearchResults, Error> {
        // 🆕 Check cache primero
        if let Some(cached) = self.result_cache.get(query) {
            return Ok(cached.clone());
        }
        
        // 🆕 Buscar en índice local
        let local_results = self.indexer.search(query).await?;
        
        // 🆕 Si hay pocos resultados, crawl en vivo
        if local_results.len() < 5 {
            let live_results = self.crawler.search_live(query).await?;
            self.indexer.add_results(&live_results).await?;
        }
        
        // 🆕 Rankear y retornar
        let ranked = self.ranker.rank(query, &local_results)?;
        self.result_cache.put(query.into(), ranked.clone());
        
        Ok(ranked)
    }
    
    pub async fn suggest(&self, partial: &str) -> Vec<String> {
        // 🆕 Autocomplete con trie (O(len) tiempo)
        self.suggestion_engine.complete(partial)
    }
}
```

### 🌐 Integración con Navegador

```rust
// src/browser/search_integration.rs
pub struct SearchIntegration {
    search_engine: WebSearchEngine,
    ui_overlay: SearchOverlay,
    history: SearchHistory,
}

impl SearchIntegration {
    pub async fn handle_omnibox_input(&mut self, input: &str) {
        // 🆕 Detectar si es URL o búsqueda
        if is_url(input) {
            self.navigate_to(input).await;
        } else {
            // 🆕 Mostrar sugerencias en overlay
            let suggestions = self.search_engine.suggest(input).await;
            self.ui_overlay.show_suggestions(suggestions);
            
            // 🆕 Buscar en background
            let results = self.search_engine.search(input).await;
            self.ui_overlay.show_results(results);
        }
    }
}
```

### ⚡ Optimizaciones de Búsqueda

| Técnica | Implementación | Ganancia |
|---------|----------------|----------|
| **Autocomplete Trie** | Estructura prefix tree en memoria | <1ms sugerencias |
| **BM25 Ranking** | Algoritmo estándar de search engines | Relevancia >85% |
| **Cache LRU** | 1000 queries cacheadas con TTL | -70% requests externos |
| **Indexación Local** | SQLite FTS5 + custom tokenizer | Búsqueda offline |
| **Crawler Async** | Tokio + connection pooling | 100+ páginas/seg |

### ✅ Criterios de Aceptación Fase 2
- [ ] Búsqueda web funcional desde omnibox
- [ ] Sugerencias en tiempo real (<50ms)
- [ ] Indexación local persistente
- [ ] Cache inteligente con TTL
- [ ] Soporte filtros (web, imágenes, noticias)
- [ ] Respeto robots.txt y rate limiting

---

## 🔥 FASE 3: JS Engine Integrado (Boa) (Semana 10-12)

### 🎯 Objetivo
Integrar motor JavaScript nativo en Rust (Boa) con bridge DOM mínimo para interactividad web.

### 🏗️ Arquitectura

```rust
// src/js_engine/boa_integration.rs
pub struct JsRuntime {
    // 🆕 Contexto Boa con DOM bindings
    context: Context,
    
    // 🆕 Queue de microtasks
    microtask_queue: VecDeque<JsTask>,
    
    // 🆕 Event loop integrado con Tokio
    event_loop: JsEventLoop,
    
    // 🆕 Console handler para debug
    console_handler: ConsoleHandler,
    
    // 🆕 Sandbox de seguridad
    sandbox: JsSandbox,
}

impl JsRuntime {
    pub fn execute_script(&mut self, source: &str) -> Result<JsValue, Error> {
        // 🆕 Parse y ejecutar en contexto sandboxed
        let result = self.context.eval(source)?;
        
        // 🆕 Procesar microtasks pendientes
        self.run_microtasks()?;
        
        Ok(result)
    }
    
    pub fn register_dom_bindings(&mut self, dom: &DomRuntime) {
        // 🆕 Inyectar objetos globales
        let global = self.context.global_object();
        
        // document.getElementById
        global.set(
            "document",
            JsObject::new(DomDocumentBridge::new(dom)),
            &mut self.context,
        );
        
        // window.alert, console.log, etc.
        global.set("console", JsObject::new(ConsoleBridge::new()), &mut self.context);
        global.set("alert", JsFunction::new(alert_bridge), &mut self.context);
    }
    
    fn run_microtasks(&mut self) -> Result<(), Error> {
        while let Some(task) = self.microtask_queue.pop_front() {
            self.execute_script(&task.source)?;
        }
        Ok(())
    }
}
```

### 🌉 Bridge DOM Mínimo

```rust
// src/js_engine/dom_bridge.rs
pub struct DomDocumentBridge {
    dom: Arc<DomRuntime>,
}

impl DomDocumentBridge {
    pub fn get_element_by_id(&self, id: &str) -> Option<DomElementBridge> {
        self.dom.find_by_id(id).map(|el| DomElementBridge::new(el))
    }
    
    pub fn query_selector(&self, selector: &str) -> Option<DomElementBridge> {
        self.dom.query_selector(selector).map(|el| DomElementBridge::new(el))
    }
    
    pub fn create_element(&self, tag: &str) -> DomElementBridge {
        let el = self.dom.create_element(tag);
        DomElementBridge::new(el)
    }
}

pub struct DomElementBridge {
    element: Arc<DomElement>,
}

impl DomElementBridge {
    pub fn get_inner_text(&self) -> String {
        self.element.text_content()
    }
    
    pub fn set_inner_text(&self, text: &str) {
        self.element.set_text_content(text);
    }
    
    pub fn add_event_listener(&self, event: &str, callback: JsFunction) {
        self.element.register_handler(event, callback);
    }
}
```

### ✅ Criterios de Aceptación Fase 3
- [ ] Boa integrado y ejecutando scripts inline/externos
- [ ] Bridge DOM mínimo: `getElementById`, `querySelector`, `addEventListener`
- [ ] `console.log` capturado y mostrado en devtools
- [ ] Event loop JS integrado con Tokio
- [ ] Sandbox de seguridad (sin acceso a filesystem)
- [ ] Scripts básicos funcionan: `alert()`, `setTimeout`, manipulación DOM

---

## 🔥 FASE 4: Layout Avanzado (Flex/Grid) (Semana 13-15)

### 🎯 Objetivo
Implementar CSS Flexbox y Grid nativos para compatibilidad con >90% de sitios web modernos.

### 🏗️ Arquitectura

```rust
// src/layout/flexbox.rs
pub struct FlexboxLayoutEngine {
    // 🆕 Resolver flex containers
    containers: Vec<FlexContainer>,
    
    // 🆕 Algoritmo de layout flex (W3C spec)
    flex_algorithm: FlexAlgorithm,
    
    // 🆕 Compute en GPU para paralelo
    compute_pipeline: ComputePipeline,
}

impl FlexboxLayoutEngine {
    pub fn resolve(&mut self, container: &FlexContainer) -> LayoutResult {
        // 🆕 Paso 1: Determinar main/cross axis
        let axis = container.flex_direction;
        
        // 🆕 Paso 2: Calcular tamaños flexibles
        let items = self.flex_algorithm.calculate_flex_items(
            &container.children,
            container.main_size,
        );
        
        // 🆕 Paso 3: Resolver alineación
        let aligned = self.flex_algorithm.align_items(&items, container);
        
        LayoutResult { positions: aligned }
    }
}
```

### ✅ Criterios de Aceptación Fase 4
- [ ] Flexbox completo: `flex-direction`, `justify-content`, `align-items`, `flex-wrap`, `flex-grow/shrink/basis`
- [ ] Grid básico: `grid-template-columns/rows`, `gap`, `grid-column/row`
- [ ] Layout resuelto en GPU (compute shader)
- [ ] Compatible con >90% de sitios top 1000

---

## 🔥 FASE 5: Font Rendering MSDF GPU (Semana 16-17)

### 🎯 Objetivo
Renderizado de texto ultra-definido a cualquier zoom usando Multi-channel Signed Distance Fields.

### 🏗️ Arquitectura

```rust
// src/fonts/msdf_renderer.rs
pub struct MsdfFontRenderer {
    // 🆕 Atlas de fuentes MSDF en GPU
    font_atlas: GpuTexture,
    
    // 🆕 Shader MSDF custom
    msdf_shader: ShaderModule,
    
    // 🆕 Cache de glyphs renderizados
    glyph_cache: LruCache<(FontId, Char), GpuGlyph>,
    
    // 🆕 FreeType para generar MSDF inicial
    freetype_library: Library,
}

impl MsdfFontRenderer {
    pub fn render_text(&mut self, text: &str, font: &Font, size: f32) -> Result<GpuText, Error> {
        // 🆕 Para cada char, buscar en cache o generar MSDF
        let mut glyphs = Vec::new();
        for ch in text.chars() {
            let glyph = self.glyph_cache
                .get(&(font.id, ch))
                .cloned()
                .unwrap_or_else(|| self.generate_msdf_glyph(font, ch));
            
            glyphs.push(glyph);
        }
        
        // 🆕 Submit a GPU como instancias
        self.render_glyphs_instanced(&glyphs)
    }
}
```

### ✅ Criterios de Aceptación Fase 5
- [ ] MSDF atlas generado y cacheado en GPU
- [ ] Texto nítido a cualquier zoom (100% - 500%)
- [ ] Shader MSDF con anti-aliasing por píxel
- [ ] Soporte fonts web (WOFF2)

---

## 🔥 FASE 6: Video Hardware Decode (Semana 18-20)

### 🎯 Objetivo
Reproducción de video con decodificación por hardware (NVDEC / Vulkan Video).

### ✅ Criterios de Aceptación Fase 6
- [ ] Soporte H.264/VP9 decode por hardware
- [ ] Playback fluido 1080p60 sin CPU spike
- [ ] Integración con DOM `<video>` element
- [ ] Controls nativos overlay

---

## 🔥 FASE 7: Polishing & WPT Compliance (Semana 21-24)

### 🎯 Objetivo
Pulir UX, integrar Web Platform Tests, y preparar para lanzamiento.

### ✅ Criterios de Aceptación Fase 7
- [ ] >80% pass rate en Web Platform Tests
- [ ] DevTools básico (inspector, console, network)
- [ ] Memory profiler integrado
- [ ] Crash reporter automático
- [ ] Benchmarks públicos vs Chrome/Firefox

---

## 📊 Métricas de Rendimiento Esperadas

| Métrica | Chrome 120 | Firefox 120 | **Noir Browser** |
|---------|------------|-------------|------------------|
| RAM por tab | 300-500MB | 250-400MB | **<50MB** |
| CPU idle | 2-5% | 3-6% | **<0.5%** |
| Frame time | 8-16ms | 10-18ms | **<5ms** |
| Cold start | 2-4s | 1.5-3s | **<1s** |
| JS execution | 1.0x (base) | 0.9x | **0.8-0.9x** (Boa) |
| Render calls | 500-1000/frame | 600-1200/frame | **<100/frame** (bindless) |

---

## ⚠️ Mitigación de Riesgos

| Riesgo | Probabilidad | Impacto | Mitigación |
|--------|--------------|---------|------------|
| Complejidad Vulkan Ash | Alta | Alto | Usar `gpu-allocator` + wrappers si boilerplate excesivo |
| Boa JS engine limitado | Media | Alto | Fallback a V8 vía `deno_core` si Boa no soporta features críticas |
| Compatibilidad web baja | Alta | Alto | WPT desde día 1; fallback rendering para sitios rotos |
| Memory leaks GPU | Media | Alto | VMA con tracking + validation layers en debug |
| Crawler bloqueado | Media | Medio | Respetar robots.txt; usar múltiples fuentes (DuckDuckGo, SearXNG) |

---

## 🚀 Plan de Ejecución Inmediata (Primeras 2 Semanas)

### Semana 1: Setup Core Vulkan
1. [ ] Crear `UltraFastVulkanEngine` con Vulkan 1.3
2. [ ] Integrar VMA allocator
3. [ ] Implementar triple buffering con timeline semaphores
4. [ ] Dynamic rendering setup
5. [ ] Pipeline cache en disco

### Semana 2: Zero-Copy Pipeline
1. [ ] Parser HTML → GPU buffer directo
2. [ ] Parser CSS → GPU buffer directo
3. [ ] Compute shader layout básico (block/inline)
4. [ ] Async bridge parser → render
5. [ ] Benchmark frame time y RAM

### Semana 3: Web Search Engine (Inicio)
1. [ ] Setup `AsyncCrawler` con Tokio
2. [ ] Implementar `LocalIndexer` (SQLite FTS5)
3. [ ] Crear `TrieAutocomplete` para sugerencias
4. [ ] Integrar con omnibox
5. [ ] Cache LRU con TTL

---

## 📁 Estructura de Carpetas Reconstruida

```
No-Chromium/
├── src/
│   ├── main.rs                    # Entry point
│   ├── app.rs                     # Application loop (winit)
│   ├── vulkan_engine/
│   │   ├── core.rs                # UltraFastVulkanEngine
│   │   ├── init.rs                # Vulkan 1.3 setup
│   │   ├── memory.rs              # VMA allocator
│   │   ├── render_pipeline.rs     # Multi-draw indirect + dynamic rendering
│   │   ├── swapchain.rs           # Triple buffering
│   │   ├── shaders/
│   │   │   ├── ui.vert            # Vertex shader UI
│   │   │   ├── ui.frag            # Fragment shader UI
│   │   │   ├── layout.comp        # Compute shader layout
│   │   │   └── msdf.frag          # MSDF font shader
│   │   └── sync.rs                # Timeline semaphores
│   ├── pipeline/
│   │   ├── zero_copy.rs           # Zero-copy pipeline
│   │   ├── async_bridge.rs        # Async parser → render
│   │   └── dma.rs                 # DMA transfers
│   ├── parsers/
│   │   ├── html.rs                # HTML lexer/parser → GPU
│   │   ├── css.rs                 # CSS parser → GPU
│   │   └── dom_native.rs          # DOM tree binario
│   ├── layout/
│   │   ├── box_model.rs           # CSS box model
│   │   ├── flexbox.rs             # Flexbox engine
│   │   ├── grid.rs                # Grid engine
│   │   └── compute.rs             # GPU layout compute
│   ├── js_engine/
│   │   ├── boa_integration.rs     # Boa JS runtime
│   │   ├── dom_bridge.rs          # DOM bindings
│   │   ├── event_loop.rs          # JS event loop
│   │   └── sandbox.rs             # Security sandbox
│   ├── search/
│   │   ├── web_engine.rs          # WebSearchEngine
│   │   ├── crawler.rs             # AsyncCrawler
│   │   ├── indexer.rs             # LocalIndexer
│   │   ├── ranker.rs              # BM25Ranker
│   │   ├── autocomplete.rs        # TrieAutocomplete
│   │   └── cache.rs               # LRU cache
│   ├── browser/
│   │   ├── page_modular/
│   │   │   ├── mod.rs             # Layout + hitboxes
│   │   │   └── dark_theme.rs      # Noir Dark Theme
│   │   ├── page.rs                # Tab handler
│   │   ├── navigation.rs          # History + navigation
│   │   └── search_integration.rs  # Omnibox search
│   ├── media/
│   │   ├── image_manager.rs       # Image decode + pre-cache
│   │   ├── video_decode.rs        # NVDEC/Vulkan Video
│   │   └── fonts/
│   │       └── msdf.rs            # MSDF font renderer
│   └── utils/
│       ├── benchmark.rs           # Performance metrics
│       ├── memory_tracker.rs      # GPU/CPU memory tracking
│       └── wpt_runner.rs          # Web Platform Tests
├── shaders/                       # GLSL shaders source
├── assets/
│   └── pre_cache/                 # Pre-cached resources
├── Cargo.toml
└── README.md
```

---

## 🎯 Próximos Pasos Inmediatos

1. **HOY**: Revisar este `Fases.md` y confirmar arquitectura
2. **Día 1-2**: Crear estructura de carpetas nueva + `Cargo.toml` actualizado
3. **Día 3-7**: Implementar `UltraFastVulkanEngine` (Fase 0)
4. **Día 8-14**: Pipeline zero-copy + benchmarks
5. **Día 15-21**: Web search engine básico
6. **Día 22-30**: Integrar Boa JS engine

---

## 💡 Notas Finales

- **Prioridad #1**: Vulkan ultra-fast (sin esto, nada más importa)
- **Prioridad #2**: Búsqueda web nativa (diferenciador clave vs Chrome)
- **Prioridad #3**: JS engine (necesario para web moderna)
- **Métrica clave**: Frame time < 5ms, RAM < 50MB/tab
- **Filosofía**: Zero-copy todo lo posible, GPU primero, CPU mínimo

¿Listo para reconstruir? 🚀
