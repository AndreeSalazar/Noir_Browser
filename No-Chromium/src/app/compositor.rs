//! Compositor por layers + damage tracking (FASE D3)
//!
//! Sistema de compositing estilo Chrome Blink + Firefox WebRender:
//! - Layers independientes con su propio contenido
//! - Damage tracking: solo re-pintar las areas modificadas
//! - Z-order para compositing correcto
//! - Hit testing por layer
//!
//! Inspirado en el pipeline de Chrome:
//! https://developer.chrome.com/blog/inside-browser-part3

use std::collections::HashSet;

/// Tipo de compositing layer (FASE D2 extension)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LayerType {
    /// Layer raiz (todo el documento)
    Document,
    /// Layer con transform propio
    Transformed,
    /// Layer con opacity < 1
    Transparent,
    /// Layer con overflow: scroll
    ScrollContainer,
    /// Layer fijo (position: fixed)
    Fixed,
    /// Layer de video (deco'd en GPU)
    Video,
    /// Layer de canvas
    Canvas,
    /// Layer de iframe
    Iframe,
}

impl LayerType {
    pub fn name(&self) -> &'static str {
        match self {
            LayerType::Document => "document",
            LayerType::Transformed => "transformed",
            LayerType::Transparent => "transparent",
            LayerType::ScrollContainer => "scroll",
            LayerType::Fixed => "fixed",
            LayerType::Video => "video",
            LayerType::Canvas => "canvas",
            LayerType::Iframe => "iframe",
        }
    }

    /// Es compositor-only? (no necesita re-paint al cambiar)
    pub fn is_compositor_only(&self) -> bool {
        matches!(self,
            LayerType::Transformed |
            LayerType::Transparent |
            LayerType::ScrollContainer |
            LayerType::Fixed
        )
    }
}

/// Una compositing layer con contenido
#[derive(Debug, Clone)]
pub struct Layer {
    pub id: u32,
    pub layer_type: LayerType,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    /// Transform 2D (translate, scale, rotate)
    pub transform: [f32; 6],  // a, b, c, d, e, f
    pub opacity: f32,
    pub z_index: i32,
    /// Region sucia (x, y, w, h) - None si no dirty
    pub damage: Option<(f32, f32, f32, f32)>,
    /// Bitmap del contenido (placeholder)
    pub has_bitmap: bool,
    /// Layer esta visible
    pub visible: bool,
    /// Layer padre (None para root)
    pub parent_id: Option<u32>,
}

impl Layer {
    pub fn new(id: u32, layer_type: LayerType, x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            id,
            layer_type,
            x, y, w, h,
            transform: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],  // identity
            opacity: 1.0,
            z_index: 0,
            damage: Some((x, y, w, h)),  // inicialmente dirty
            has_bitmap: false,
            visible: true,
            parent_id: None,
        }
    }

    /// Marcar una region como dirty
    pub fn invalidate(&mut self, x: f32, y: f32, w: f32, h: f32) {
        self.damage = Some(match self.damage {
            Some((dx, dy, dw, dh)) => union_rects((dx, dy, dw, dh), (x, y, w, h)),
            None => (x, y, w, h),
        });
    }

    /// Limpiar dirty (despues de pintar)
    pub fn clear_damage(&mut self) {
        self.damage = None;
    }

    /// Verificar si intersecta con un punto
    pub fn hit_test(&self, px: f32, py: f32) -> bool {
        if !self.visible { return false; }
        if self.opacity < 0.01 { return false; }
        px >= self.x && px <= self.x + self.w &&
        py >= self.y && py <= self.y + self.h
    }

    /// Es area dirty?
    pub fn is_dirty(&self) -> bool {
        self.damage.is_some()
    }
}

/// Union de dos rectangulos
fn union_rects(a: (f32, f32, f32, f32), b: (f32, f32, f32, f32)) -> (f32, f32, f32, f32) {
    let min_x = a.0.min(b.0);
    let min_y = a.1.min(b.1);
    let max_x = (a.0 + a.2).max(b.0 + b.2);
    let max_y = (a.1 + a.3).max(b.1 + b.3);
    (min_x, min_y, max_x - min_x, max_y - min_y)
}

/// Verifica si dos rectangulos se intersectan
pub fn rects_intersect(a: &(f32, f32, f32, f32), b: &(f32, f32, f32, f32)) -> bool {
    a.0 < b.0 + b.2 && a.0 + a.2 > b.0 && a.1 < b.1 + b.3 && a.1 + a.3 > b.1
}

/// Compositor de layers
#[derive(Debug, Default)]
pub struct Compositor {
    pub layers: Vec<Layer>,
    pub next_id: u32,
    /// Set de layers que necesitan re-compositing
    pub dirty_layers: HashSet<u32>,
    /// Tamaño del viewport
    pub viewport_w: f32,
    pub viewport_h: f32,
    /// Damage total del frame actual
    pub frame_damage: Option<(f32, f32, f32, f32)>,
    /// Frames pintados
    pub frames_painted: u64,
    /// Layers saltados (cache hits)
    pub layers_skipped: u64,
}

impl Compositor {
    pub fn new(viewport_w: f32, viewport_h: f32) -> Self {
        let mut me = Self {
            viewport_w,
            viewport_h,
            ..Default::default()
        };
        // Crear root layer
        let root = Layer::new(0, LayerType::Document, 0.0, 0.0, viewport_w, viewport_h);
        me.layers.push(root);
        me.next_id = 1;
        me
    }

    /// Crear una nueva layer
    pub fn create_layer(&mut self, layer_type: LayerType, x: f32, y: f32, w: f32, h: f32) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.layers.push(Layer::new(id, layer_type, x, y, w, h));
        self.invalidate(id, x, y, w, h);
        id
    }

    /// Eliminar una layer
    pub fn remove_layer(&mut self, id: u32) -> bool {
        if id == 0 { return false; }  // No eliminar root
        if let Some(pos) = self.layers.iter().position(|l| l.id == id) {
            let layer = self.layers.remove(pos);
            // Invalidar donde estaba
            self.invalidate_frame(layer.x, layer.y, layer.w, layer.h);
            true
        } else {
            false
        }
    }

    /// Obtener una layer
    pub fn get(&self, id: u32) -> Option<&Layer> {
        self.layers.iter().find(|l| l.id == id)
    }

    /// Obtener layer mutable
    pub fn get_mut(&mut self, id: u32) -> Option<&mut Layer> {
        self.layers.iter_mut().find(|l| l.id == id)
    }

    /// Mover una layer (transform)
    pub fn set_transform(&mut self, id: u32, dx: f32, dy: f32) {
        let info = self.get(id).map(|l| (l.x, l.y, l.w, l.h, l.layer_type.is_compositor_only()));
        if let Some((x, y, w, h, is_comp)) = info {
            if let Some(layer) = self.get_mut(id) {
                layer.transform[4] = dx;
                layer.transform[5] = dy;
            }
            if !is_comp {
                self.invalidate(id, x, y, w, h);
            }
        }
    }

    /// Cambiar opacidad (compositor-only)
    pub fn set_opacity(&mut self, id: u32, opacity: f32) {
        if let Some(layer) = self.get_mut(id) {
            layer.opacity = opacity.clamp(0.0, 1.0);
        }
    }

    /// Cambiar z-index
    pub fn set_z_index(&mut self, id: u32, z: i32) {
        if let Some(layer) = self.get_mut(id) {
            layer.z_index = z;
        }
        // Re-compositar
        self.frame_damage = Some((0.0, 0.0, self.viewport_w, self.viewport_h));
    }

    /// Invalidar una region de una layer
    pub fn invalidate(&mut self, id: u32, x: f32, y: f32, w: f32, h: f32) {
        if let Some(layer) = self.get_mut(id) {
            layer.invalidate(x, y, w, h);
        }
        self.dirty_layers.insert(id);
        self.invalidate_frame(x, y, w, h);
    }

    pub fn invalidate_frame(&mut self, x: f32, y: f32, w: f32, h: f32) {
        self.frame_damage = Some(match self.frame_damage {
            Some(existing) => union_rects(existing, (x, y, w, h)),
            None => (x, y, w, h),
        });
    }

    /// Aplicar scroll a una layer
    pub fn scroll(&mut self, id: u32, dx: f32, dy: f32) {
        let info = self.get(id).map(|l| (l.x, l.y, l.w, l.h, l.layer_type));
        if let Some((x, y, w, h, layer_type)) = info {
            if layer_type == LayerType::ScrollContainer {
                if let Some(layer) = self.get_mut(id) {
                    layer.transform[4] += dx;
                    layer.transform[5] += dy;
                }
            } else {
                self.invalidate(id, x, y, w, h);
            }
        }
    }

    /// Hit test: encontrar la layer superior en un punto
    pub fn hit_test(&self, px: f32, py: f32) -> Option<u32> {
        // Buscar de mayor z-index a menor
        let mut candidates: Vec<&Layer> = self.layers.iter()
            .filter(|l| l.hit_test(px, py))
            .collect();
        candidates.sort_by_key(|l| -l.z_index);
        candidates.first().map(|l| l.id)
    }

    /// Compositar un frame
    /// Retorna: (layers painted, layers skipped, damage total)
    pub fn composite(&mut self) -> (u32, u32, Option<(f32, f32, f32, f32)>) {
        let frame_damage = self.frame_damage.take();
        let mut painted = 0;
        let mut skipped = 0;
        let mut total_damage = frame_damage;

        // Ordenar layers por z-index (menor primero, pero compositor las pinta al reves)
        let mut sorted_layers: Vec<u32> = self.layers.iter().map(|l| l.id).collect();
        sorted_layers.sort_by_key(|&id| {
            self.layers.iter().find(|l| l.id == id).map(|l| l.z_index).unwrap_or(0)
        });

        for id in sorted_layers {
            let layer = match self.layers.iter().find(|l| l.id == id) {
                Some(l) => l.clone(),
                None => continue,
            };
            if !layer.visible { continue; }
            if layer.opacity < 0.01 { continue; }

            // Verificar si la layer intersecta con el damage del frame
            let layer_rect = (layer.x, layer.y, layer.w, layer.h);
            let should_paint = match (layer.damage, total_damage) {
                (Some(_), Some(d)) => rects_intersect(&layer_rect, &d),
                (Some(_), None) => true,
                (None, _) => false,  // layer no dirty
            };

            if should_paint {
                painted += 1;
                // Clear layer damage despues de pintar
                if let Some(l) = self.get_mut(id) {
                    l.clear_damage();
                }
            } else {
                skipped += 1;
            }
        }

        self.dirty_layers.clear();
        self.frames_painted += 1;
        self.layers_skipped += skipped as u64;

        (painted, skipped, total_damage)
    }

    /// Limpiar damage de todo (despues de pintar)
    pub fn clear_all_damage(&mut self) {
        for layer in &mut self.layers {
            layer.clear_damage();
        }
        self.frame_damage = None;
        self.dirty_layers.clear();
    }

    /// Total de layers
    pub fn count(&self) -> usize {
        self.layers.len()
    }

    /// Total de layers dirty
    pub fn dirty_count(&self) -> usize {
        self.dirty_layers.len()
    }

    /// Layers visibles
    pub fn visible_count(&self) -> usize {
        self.layers.iter().filter(|l| l.visible && l.opacity > 0.01).count()
    }

    /// Resize
    pub fn resize(&mut self, w: f32, h: f32) {
        self.viewport_w = w;
        self.viewport_h = h;
        if let Some(root) = self.get_mut(0) {
            root.w = w;
            root.h = h;
            root.damage = Some((0.0, 0.0, w, h));
        }
        self.invalidate_frame(0.0, 0.0, w, h);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer_type_name() {
        assert_eq!(LayerType::Document.name(), "document");
        assert_eq!(LayerType::Video.name(), "video");
    }

    #[test]
    fn test_compositor_only() {
        assert!(LayerType::Transformed.is_compositor_only());
        assert!(LayerType::Fixed.is_compositor_only());
        assert!(!LayerType::Document.is_compositor_only());
    }

    #[test]
    fn test_layer_creation() {
        let l = Layer::new(1, LayerType::Document, 0.0, 0.0, 100.0, 100.0);
        assert_eq!(l.id, 1);
        assert!(l.is_dirty());
    }

    #[test]
    fn test_layer_invalidate() {
        let mut l = Layer::new(1, LayerType::Document, 0.0, 0.0, 100.0, 100.0);
        l.clear_damage();
        assert!(!l.is_dirty());
        l.invalidate(10.0, 10.0, 20.0, 20.0);
        assert!(l.is_dirty());
    }

    #[test]
    fn test_layer_invalidate_union() {
        let mut l = Layer::new(1, LayerType::Document, 0.0, 0.0, 100.0, 100.0);
        l.invalidate(0.0, 0.0, 50.0, 50.0);
        l.invalidate(75.0, 75.0, 25.0, 25.0);
        // El damage debe cubrir ambos
        let (x, y, w, h) = l.damage.unwrap();
        assert_eq!(x, 0.0);
        assert_eq!(y, 0.0);
        assert_eq!(w, 100.0);
        assert_eq!(h, 100.0);
    }

    #[test]
    fn test_layer_hit_test() {
        let l = Layer::new(1, LayerType::Document, 10.0, 10.0, 100.0, 100.0);
        assert!(l.hit_test(50.0, 50.0));
        assert!(!l.hit_test(0.0, 0.0));
        assert!(!l.hit_test(200.0, 200.0));
    }

    #[test]
    fn test_layer_hit_test_invisible() {
        let mut l = Layer::new(1, LayerType::Document, 0.0, 0.0, 100.0, 100.0);
        l.visible = false;
        assert!(!l.hit_test(50.0, 50.0));
    }

    #[test]
    fn test_layer_hit_test_opacity() {
        let mut l = Layer::new(1, LayerType::Document, 0.0, 0.0, 100.0, 100.0);
        l.opacity = 0.0;
        assert!(!l.hit_test(50.0, 50.0));
    }

    #[test]
    fn test_compositor_creation() {
        let c = Compositor::new(800.0, 600.0);
        assert_eq!(c.count(), 1);
        assert_eq!(c.viewport_w, 800.0);
    }

    #[test]
    fn test_compositor_create_layer() {
        let mut c = Compositor::new(800.0, 600.0);
        let id = c.create_layer(LayerType::Transformed, 10.0, 10.0, 100.0, 100.0);
        assert_eq!(c.count(), 2);
        assert!(c.dirty_count() > 0);
    }

    #[test]
    fn test_compositor_remove_layer() {
        let mut c = Compositor::new(800.0, 600.0);
        let id = c.create_layer(LayerType::Transformed, 10.0, 10.0, 100.0, 100.0);
        assert!(c.remove_layer(id));
        assert_eq!(c.count(), 1);
    }

    #[test]
    fn test_compositor_cannot_remove_root() {
        let mut c = Compositor::new(800.0, 600.0);
        assert!(!c.remove_layer(0));
    }

    #[test]
    fn test_compositor_set_transform_compositor_only() {
        let mut c = Compositor::new(800.0, 600.0);
        let id = c.create_layer(LayerType::Transformed, 10.0, 10.0, 100.0, 100.0);
        // Clear dirty
        c.clear_all_damage();
        c.set_transform(id, 50.0, 50.0);
        // Compositor-only: NO debe invalidar
        let layer = c.get(id).unwrap();
        // El layer.transform cambio pero la layer NO esta dirty
        assert_eq!(layer.transform[4], 50.0);
    }

    #[test]
    fn test_compositor_set_transform_non_compositor() {
        let mut c = Compositor::new(800.0, 600.0);
        let id = c.create_layer(LayerType::Document, 10.0, 10.0, 100.0, 100.0);
        c.clear_all_damage();
        c.set_transform(id, 50.0, 50.0);
        // Non-compositor: DEBE invalidar
        assert!(c.frame_damage.is_some());
    }

    #[test]
    fn test_compositor_set_opacity() {
        let mut c = Compositor::new(800.0, 600.0);
        let id = c.create_layer(LayerType::Transparent, 10.0, 10.0, 100.0, 100.0);
        c.set_opacity(id, 0.5);
        let layer = c.get(id).unwrap();
        assert_eq!(layer.opacity, 0.5);
    }

    #[test]
    fn test_compositor_opacity_clamp() {
        let mut c = Compositor::new(800.0, 600.0);
        let id = c.create_layer(LayerType::Transparent, 10.0, 10.0, 100.0, 100.0);
        c.set_opacity(id, 2.0);
        assert_eq!(c.get(id).unwrap().opacity, 1.0);
        c.set_opacity(id, -1.0);
        assert_eq!(c.get(id).unwrap().opacity, 0.0);
    }

    #[test]
    fn test_compositor_hit_test() {
        let mut c = Compositor::new(800.0, 600.0);
        c.create_layer(LayerType::Document, 0.0, 0.0, 100.0, 100.0);
        c.create_layer(LayerType::Document, 50.0, 50.0, 100.0, 100.0);
        let hit = c.hit_test(60.0, 60.0);
        assert!(hit.is_some());
    }

    #[test]
    fn test_compositor_hit_test_no_hit() {
        let mut c = Compositor::new(800.0, 600.0);
        c.create_layer(LayerType::Document, 0.0, 0.0, 50.0, 50.0);
        // (500, 500) no esta en la layer pequena, pero el root (800x600) si
        // El hit test retorna la layer superior: root cubre
        let hit = c.hit_test(500.0, 500.0);
        // Solo verificamos que la layer pequena no es la hit
        if let Some(id) = hit {
            assert_ne!(id, 1);  // No es la layer pequena
        }
    }

    #[test]
    fn test_compositor_composite() {
        let mut c = Compositor::new(800.0, 600.0);
        c.create_layer(LayerType::Document, 0.0, 0.0, 100.0, 100.0);
        c.create_layer(LayerType::Document, 100.0, 100.0, 100.0, 100.0);
        let (painted, _skipped, damage) = c.composite();
        assert!(painted > 0);
        assert!(damage.is_some());
    }

    #[test]
    fn test_compositor_clear_all_damage() {
        let mut c = Compositor::new(800.0, 600.0);
        c.create_layer(LayerType::Document, 0.0, 0.0, 100.0, 100.0);
        c.clear_all_damage();
        assert_eq!(c.dirty_count(), 0);
        assert!(c.frame_damage.is_none());
    }

    #[test]
    fn test_compositor_resize() {
        let mut c = Compositor::new(800.0, 600.0);
        c.resize(1024.0, 768.0);
        assert_eq!(c.viewport_w, 1024.0);
        assert_eq!(c.get(0).unwrap().w, 1024.0);
    }

    #[test]
    fn test_compositor_visible_count() {
        let mut c = Compositor::new(800.0, 600.0);
        c.create_layer(LayerType::Document, 0.0, 0.0, 100.0, 100.0);
        let id = c.create_layer(LayerType::Document, 0.0, 0.0, 100.0, 100.0);
        c.set_opacity(id, 0.0);  // invisible
        // root + layer con opacity > 0
        assert!(c.visible_count() >= 1);
    }

    #[test]
    fn test_rects_intersect() {
        assert!(rects_intersect(&(0.0, 0.0, 10.0, 10.0), &(5.0, 5.0, 10.0, 10.0)));
        assert!(!rects_intersect(&(0.0, 0.0, 10.0, 10.0), &(20.0, 20.0, 10.0, 10.0)));
    }

    #[test]
    fn test_union_rects() {
        let u = union_rects((0.0, 0.0, 10.0, 10.0), (5.0, 5.0, 10.0, 10.0));
        assert_eq!(u, (0.0, 0.0, 15.0, 15.0));
    }

    #[test]
    fn test_scroll() {
        let mut c = Compositor::new(800.0, 600.0);
        let id = c.create_layer(LayerType::ScrollContainer, 0.0, 0.0, 100.0, 100.0);
        c.scroll(id, 50.0, 100.0);
        let layer = c.get(id).unwrap();
        assert_eq!(layer.transform[4], 50.0);
        assert_eq!(layer.transform[5], 100.0);
    }
}
