# Noir Browser - Plan de Mejoras v2

## Filosofía
- **Mantener la base actual** que ya funciona (87 tests, YouTube carga)
- **Mejorar incrementalmente** con ideas del plan v4
- **Seguir estándares web abiertos** (HTML5, CSS3, ES2024, WebGPU)
- **NO imitar Chrome** - crear nuestra propia identidad visual
- **Funcional > Imitación** - que sitios reales funcionen bien

## Estado actual (✅)
- 87 tests pasando
- Carga YouTube, GitHub, Wikipedia
- Custom Chrome-like UI
- JS engine v3 funcional
- WASM engine v2
- WebGPU module creado (no integrado)
- Tokio runtime para async
- Sin panics en runtime

## Mejoras propuestas (ordenadas por impacto)

### PRIORIDAD ALTA (arreglar bugs visibles)

#### M1: Arreglar render layout
**Problema**: El texto se renderiza en columna estrecha a la izquierda
**Causa probable**: `viewport_w` no se calcula bien o `content_x` no se aplica
**Solución**:
- Diagnosticar el bug exacto
- Asegurar que `effective_w = ctx.width` (no 0)
- Aplicar `content_x` consistentemente

#### M2: Click en links
**Problema**: Los links no son clickeables
**Solución**:
- Detectar click en coordenadas de `LayoutBlock` con `href`
- Llamar `navigation::navigate()` con la URL
- Cambiar cursor a "pointer" al hover

#### M3: Scroll con mouse wheel
**Problema**: La rueda del mouse no hace scroll
**Solución**:
- Capturar `WindowEvent::MouseWheel`
- Actualizar `tab.scroll_y`
- Redibujar

### PRIORIDAD MEDIA (features útiles)

#### M4: Image rendering
**Problema**: Las imágenes no se muestran
**Estado**: `media/` ya tiene cache, falta integrarlo bien
**Solución**:
- Verificar que `draw_image_to_buffer` se llama correctamente
- Soportar PNG, JPEG (básico)
- Placeholder mientras carga

#### M5: CSS improvements
**Estado**: Ya tiene Flexbox + Grid básico
**Mejoras**:
- `display: flex` con align-items, justify-content
- `display: grid` con grid-template
- `position: relative/absolute/fixed`
- `z-index`
- `overflow: hidden/scroll/auto`

#### M6: JS engine improvements
**Estado**: 36 integration tests pasan
**Mejoras**:
- async/await (Promises ya están)
- Better error messages
- Stack traces
- console.log funciona
- DOM API: getElementById, querySelector

### PRIORIDAD BAJA (nice to have)

#### M7: Bookmarks
- Guardar páginas favoritas
- Sidebar con bookmarks
- Import/export JSON

#### M8: History
- Guardar URLs visitadas
- Sidebar con historial
- Búsqueda en historial

#### M9: Downloads
- Detectar `<a download>` y headers Content-Disposition
- Guardar en carpeta Downloads
- Progress bar

#### M10: DevTools (F12)
- Panel de HTML inspector
- CSS inspector
- Console.log output
- Network tab
- Storage tab

#### M11: Multi-tab improvements
- Drag & drop tabs
- Cerrar tab con click en X
- New tab con Ctrl+T
- Cerrar tab con Ctrl+W

#### M12: WebGPU rendering
**Estado**: `webgpu/` module existe
**Mejora**:
- Integrar `IntegratedRenderer` con la ventana
- Reemplazar `softbuffer` por `wgpu`
- Shaders WGSL
- Compute pipelines para layout

#### M13: Performance
- Dirty rectangles (solo redibujar áreas cambiadas)
- Lazy loading de imágenes
- RequestAnimationFrame
- OffscreenCanvas
- Service workers

## Plan de implementación inmediato

### Paso 1: Diagnosticar el bug del layout (HOY)
- Agregar `eprintln!` en `layout_page` para ver `effective_w` y `content_x`
- Agregar `eprintln!` en `render_layout_blocks` para ver `block.x`, `block.y`
- Identificar el problema exacto
- Arreglarlo

### Paso 2: Click en links
- Detectar `MouseInput` con `ElementState::Pressed`
- Comparar coordenadas con `LayoutBlock.x/y/w/h`
- Si `is_link`, navegar a `block.href`

### Paso 3: Scroll con mouse
- Detectar `MouseScroll` event
- Actualizar `tab.scroll_y` con delta
- Marcar `dirty = true` para redibujar

### Paso 4: Mejorar el look
- Background gradient en title bar
- Animaciones suaves
- Iconos SVG
- Color theme configurable

## Métricas de éxito

- [ ] YouTube se ve correctamente (video, sidebar, search bar)
- [ ] GitHub se ve correctamente (repos, code, issues)
- [ ] Click en links navega
- [ ] Scroll con mouse funciona
- [ ] 60 FPS en hardware modesto
- [ ] < 100 MB RAM con 3 tabs abiertos

## Diferenciadores (NO Chrome)

- **Tema dark nativo** (no opcional)
- **Búsqueda integrada** (yt, gh, wiki, etc. como prefixes)
- **Privacy by default** (tracking protection)
- **Customizable** (themes via CSS)
- **Open source** (código en Rust limpio)
- **Multi-backend GPU** (Vulkan/Metal/DX12)

## Cómo NO imitar a Chrome

1. **NO usar Material Design** - crear nuestro propio design system
2. **NO tener omnibox** - tener search bar + URL bar separados
3. **NO tener Google account** - local-only
4. **NO enviar telemetría** - privacy first
5. **NO tener sync** - datos locales
6. **NO tener extensions store** - themable via CSS

## Próximas semanas

- Semana 1: Arreglar layout + click + scroll
- Semana 2: Mejorar image rendering
- Semana 3: CSS improvements
- Semana 4: DevTools básico
