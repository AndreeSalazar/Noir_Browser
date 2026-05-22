# 🗓️ Fases.md - Roadmap de Desarrollo para Noir Browser

> **Objetivo:** Llevar Noir Browser de un prototipo funcional a un navegador web utilizable para navegación básica, con arquitectura escalable hacia características avanzadas.

---

## 🎯 Estado Actual del Proyecto (v0.7.0)

### ✅ Componentes Implementados
| Módulo | Estado | Observaciones |
|--------|--------|--------------|
| **Pipeline de Renderizado** | 🟢 Funcional | HTML → CSS → Layout → Vulkan 2D |
| **Parser HTML Nativo** | 🟢 Funcional | Genera DOM binario optimizado |
| **Parser CSS Básico** | 🟡 Parcial | Soporta selectores simples, cascada básica |
| **Motor de Layout** | 🟡 Parcial | Block/inline flow, alineación por línea |
| **Vulkan Engine (Ash)** | 🟢 Funcional | Renderizado 2D con antialiasing |
| **Gestión de Recursos** | 🟢 Funcional | Pre-caching asíncrono con Tokio |
| **Noir Dark Theme** | 🟢 Funcional | Análisis de luminancia en tiempo real |
| **Navegación Básica** | 🟢 Funcional | Tabs, historial, address bar |
| **Motor JS (Boa)** | 🔴 No integrado | Dependencia presente, sin ejecución activa |
| **Flexbox/Grid** | 🔴 No implementado | En roadmap |
| **Video Hardware** | 🔴 No implementado | En roadmap |
| **MSDF Fonts** | 🔴 No implementado | En roadmap |

---

## 📋 Fases de Desarrollo Priorizadas

### 🔹 FASE 0: Estabilización del Core (Semanas 1-2)
**Objetivo:** Garantizar que el pipeline básico renderice páginas estáticas sin crashes.

```markdown
## Tareas Críticas
- [ ] **Validar sincronización Vulkan**: Implementar fences/semáforos explícitos en `vulkan_engine/renderer/swapchain.rs`
- [ ] **Manejo de errores en parsers**: Añadir fallbacks cuando HTML/CSS malformados causen panic
- [ ] **Memory safety en layout**: Revisar ownership en `page_modular/mod.rs` para evitar use-after-free
- [ ] **Logging estructurado**: Integrar `tracing` para debug de render pipeline
- [ ] **Benchmark básico**: Medir FPS y RAM en páginas de prueba (google.com, example.com)

## Criterios de Aceptación
- ✅ Renderizado estable de 10 páginas estáticas sin crashes
- ✅ Consumo de RAM < 150MB para páginas simples
- ✅ FPS sostenido > 30 en viewport 1280x720
- ✅ Logs de error legibles en consola
```

---

### 🔹 FASE 1: Integración del Motor JavaScript (Semanas 3-5) ⚡ **PRIORIDAD CRÍTICA**
**Objetivo:** Ejecutar scripts básicos para habilitar interactividad mínima.

```markdown
## Tareas Técnicas
- [ ] **Inicializar Boa en el runtime**: 
  - Modificar `src/runtime/mini_js.rs` para crear contexto Boa por pestaña
  - Implementar sandboxing básico (sin acceso a filesystem/sockets)
  
- [ ] **Conectar parser → engine**:
  - En `apply_runtime_scripts()`, ejecutar scripts inline con `JsEngine::run_sandboxed()`
  - Implementar fetch asíncrono para scripts externos vía `resource_loader`
  
- [ ] **Bridge DOM mínimo**:
  - Crear bindings en `src/parsers/webidl_bridge.rs` para:
    - `document.getElementById()`
    - `element.addEventListener('click', ...)`
    - `element.innerHTML` (solo lectura)
    
- [ ] **Event loop de JS**:
  - Integrar microtasks de Boa con el event loop de Tokio
  - Manejar `setTimeout` básico (sin precisión de tiempo real)

## Pruebas de Validación
- [ ] Página con `<script>alert("OK")</script>` muestra alerta en consola
- [ ] Botón con `onclick="console.log('click')"` registra evento
- [ ] Fetch de script externo desde CDN se ejecuta sin bloquear UI

## Riesgos y Mitigación
⚠️ **Riesgo**: Boa puede ser lento en páginas con mucho JS  
✅ **Mitigación**: Implementar timeout de ejecución (500ms por script) y fallback a modo "JS desactivado"
```

---

### 🔹 FASE 2: Layout Avanzado - Flexbox Básico (Semanas 6-8)
**Objetivo:** Soportar diseños modernos sin romper compatibilidad con layout actual.

```markdown
## Alcance Inicial (MVP de Flexbox)
- [ ] **Parser CSS ampliado**: Reconocer `display: flex`, `flex-direction`, `justify-content`, `align-items`
- [ ] **Algoritmo de layout flex**:
  - Implementar en `src/layout/layout_gen.rs`
  - Soporte para `flex-wrap: nowrap` inicialmente
  - Cálculo de `flex-basis`, `flex-grow`, `flex-shrink` básico
  
- [ ] **Integración con render pipeline**:
  - Modificar `resolve_fragment_layout()` para manejar contenedores flex
  - Actualizar hitboxes para elementos flex children

- [ ] **Fallback automático**: Si un contenedor flex tiene propiedades no soportadas, usar layout block tradicional

## Pruebas de Validación
- [ ] Renderizar navbar con `display: flex; justify-content: space-between`
- [ ] Tarjetas con `flex-wrap: wrap` se reorganizan en viewport estrecho
- [ ] Sitios como GitHub/GitLab cargan sin distorsión visual

## Notas Técnicas
> Evitar implementar Grid en esta fase. Priorizar compatibilidad con ~70% de sitios que usan Flexbox básico.
```

---

### 🔹 FASE 3: Compatibilidad Web y Testing (Semanas 9-11)
**Objetivo:** Medir y mejorar el soporte de estándares web.

```markdown
## Infraestructura de Testing
- [ ] **Integrar Web Platform Tests (WPT)**:
  - Clonar subconjunto de tests de `html/dom`, `css/cssom`, `css/flexbox`
  - Crear runner en `tools/web_platform_export.py` para ejecutar tests en Noir
  - Generar reporte de compliance (% tests pasados)

- [ ] **Suite de regresión visual**:
  - Capturar screenshots de páginas de referencia (Google, Wikipedia, MDN)
  - Comparar píxel-a-píxel con baseline usando `image` crate
  - Alertar en CI si diff > 2%

- [ ] **Fuzzing de parsers**:
  - Usar `cargo fuzz` para probar HTML/CSS maliciosos o corruptos
  - Garantizar que ningún input cause panic o memory leak

## Métricas de Calidad
| Categoría | Meta Fase 3 | Herramienta de Medición |
|-----------|-------------|------------------------|
| HTML5 Parsing | 85% WPT pass | `html5ever` test suite |
| CSS2.1 Selectors | 75% pass | WPT `css/selectors` |
| Layout Stability | 0 crashes en 100 páginas | Stress test automatizado |
| Memory Safety | 0 leaks en valgrind | `cargo-valgrind` |
```

---

### 🔹 FASE 4: Características Avanzadas (Semanas 12-16)
**Objetivo:** Habilitar funcionalidades que diferencian a Noir de otros navegadores.

```markdown
## 4.1 MSDF Font Rendering (GPU)
- [ ] Integrar shader MSDF en `vulkan_engine/shaders/`
- [ ] Pre-rasterizar fuentes comunes (Roboto, Inter) a atlas GPU
- [ ] Implementar zoom de texto sin pérdida de calidad

## 4.2 Video Hardware Acceleration
- [ ] Detectar codecs soportados vía `vkGetPhysicalDeviceVideoFormatPropertiesKHR`
- [ ] Integrar NVDEC/Vulkan Video para decodificación H.264/VP9
- [ ] Pipeline: `demux → decode → Vulkan texture → compositor`

## 4.3 Seguridad y Privacidad
- [ ] Implementar Content Security Policy (CSP) básico
- [ ] Bloquear trackers conocidos vía lista de filtros (EasyList compatible)
- [ ] Modo "Incógnito" con aislamiento de cookies por sesión

## 4.4 Extensibilidad
- [ ] API de plugins en Rust (WASM sandboxed)
- [ ] Sistema de temas CSS inyectables
- [ ] WebExtensions API mínima (`browser.tabs`, `browser.storage`)
```

---

## 🛠️ Herramientas y Dependencias Recomendadas

```toml
# Añadir a Cargo.toml para fases futuras
[dev-dependencies]
criterion = "0.5"          # Benchmarking
proptest = "1.4"           # Property-based testing
insta = "1.34"             # Snapshot testing visual

[dependencies]
tracing = "0.1"            # Logging estructurado
tracing-subscriber = "0.3"
wgpu = "0.19"              # Alternativa a Ash si la complejidad crece
font-kit = "0.13"          # Fallback para font rendering
```

---

## 📊 Métricas de Progreso (Dashboard)

Crear `docs/progress.md` con:

```markdown
## Semanal
- [ ] % WPT pass (HTML/CSS/JS)
- [ ] FPS promedio en benchmark suite
- [ ] RAM peak en carga de 10 tabs
- [ ] Bugs críticos reportados/resueltos

## Por Fase
- [ ] Criterios de aceptación cumplidos (✅/❌)
- [ ] Retrasos identificados y plan de mitigación
- [ ] Decisiones de arquitectura documentadas (ADR)
```

---

## 🚨 Riesgos Globales y Plan de Contingencia

| Riesgo | Impacto | Probabilidad | Mitigación |
|--------|---------|--------------|------------|
| **Complejidad de Ash/Vulkan** | Alto | Media | Evaluar migración a `wgpu` si el boilerplate supera 30% del código |
| **Boa no soporta APIs web críticas** | Alto | Alta | Implementar polyfills en Rust para `fetch`, `DOMParser`, etc. |
| **Falta de contributors** | Medio | Alta | Documentar "good first issues" y crear guía de contribución en `CONTRIBUTING.md` |
| **Rendimiento inferior a Chromium en benchmarks** | Medio | Media | Enfocar en nichos donde importa: embedded, low-RAM, privacidad |

---

## ✅ Checklist de "Mínimo Producto Utilizable" (Post-Fase 2)

Un usuario debería poder:
- [ ] Navegar a `https://example.com` y ver contenido renderizado
- [ ] Hacer clic en enlaces y navegar entre páginas
- [ ] Escribir en inputs y enviar formularios básicos
- [ ] Ver imágenes y texto con tema oscuro aplicado
- [ ] Abrir/cerrar tabs sin memory leaks
- [ ] Ejecutar scripts simples (alert, console.log, manipulación básica del DOM)
- [ ] Usar atajos de teclado: `Ctrl+T`, `Ctrl+W`, `Ctrl+L`, `F5`

---

> 💡 **Recomendación estratégica**: Priorizar **Fase 1 (JS)** sobre Flexbox. Sin JavaScript, la web moderna es inutilizable. Flexbox puede esperar; un botón que no funciona por falta de JS hace que el navegador sea descartado inmediatamente.

---

*Última actualización: Mayo 2026*  
*Responsable: Equipo Noir Browser*  
*Próxima revisión: Al completar Fase 0*
