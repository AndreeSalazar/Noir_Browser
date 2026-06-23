//! Composite Layers (Chrome Blink-inspired)
//!
//! Basado en el sistema de layers de Chrome:
//! - Cada layer se rasteriza independientemente
//! - Compositor thread los combina (no bloquea main thread)
//! - Transform/opacity son "compositor-only" (no requieren re-paint)
//! - will-change: transform crea una layer automaticamente
//!
//! Aplicado a Noir: el scroll y los cambios de zoom usan layers separados
//! para evitar re-paint del main thread.

use std::collections::HashMap;

/// Una compositing layer (Chrome-style)
#[derive(Debug, Clone)]
pub struct Layer {
    pub id: u32,
    pub name: String,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub transform: (f32, f32),  // translate(x, y)
    pub opacity: f32,
    pub z_index: i32,
    /// Es una layer "compositor-only"? (solo transform/opacity)
    pub compositor_only: bool,
    /// Bitmap rasterizado
    pub dirty: bool,
}

/// Gestor de layers
#[derive(Debug, Default)]
pub struct LayerTree {
    layers: Vec<Layer>,
    next_id: u32,
    /// Layer principal (root)
    root: u32,
}

impl LayerTree {
    pub fn new() -> Self {
        let mut tree = Self::default();
        let root_id = tree.create_layer("root".to_string(), 0.0, 0.0, 0.0, 0.0);
        tree.root = root_id;
        tree
    }

    /// Crear una nueva layer
    pub fn create_layer(&mut self, name: String, x: f32, y: f32, w: f32, h: f32) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.layers.push(Layer {
            id,
            name,
            x, y, w, h,
            transform: (0.0, 0.0),
            opacity: 1.0,
            z_index: 0,
            compositor_only: false,
            dirty: true,
        });
        id
    }

    /// Crea una layer para un elemento con will-change: transform/opacity
    pub fn create_compositor_layer(&mut self, name: String, x: f32, y: f32, w: f32, h: f32) -> u32 {
        let id = self.create_layer(name, x, y, w, h);
        if let Some(layer) = self.layers.iter_mut().find(|l| l.id == id) {
            layer.compositor_only = true;
        }
        id
    }

    /// Aplicar transform (compositor-only, no requiere re-paint)
    pub fn set_transform(&mut self, id: u32, dx: f32, dy: f32) {
        if let Some(layer) = self.layers.iter_mut().find(|l| l.id == id) {
            layer.transform = (dx, dy);
            // Solo cambia transform, no requiere re-raster si es compositor-only
        }
    }

    /// Aplicar opacidad (compositor-only)
    pub fn set_opacity(&mut self, id: u32, opacity: f32) {
        if let Some(layer) = self.layers.iter_mut().find(|l| l.id == id) {
            layer.opacity = opacity.clamp(0.0, 1.0);
        }
    }

    /// Marcar layer como sucia (necesita re-raster)
    pub fn invalidate(&mut self, id: u32) {
        if let Some(layer) = self.layers.iter_mut().find(|l| l.id == id) {
            layer.dirty = true;
        }
    }

    /// Obtener layer por id
    pub fn get(&self, id: u32) -> Option<&Layer> {
        self.layers.iter().find(|l| l.id == id)
    }

    /// Obtener layer por id (mut)
    pub fn get_mut(&mut self, id: u32) -> Option<&mut Layer> {
        self.layers.iter_mut().find(|l| l.id == id)
    }

    /// Listar todas las layers ordenadas por z_index
    pub fn ordered_layers(&self) -> Vec<&Layer> {
        let mut sorted: Vec<&Layer> = self.layers.iter().collect();
        sorted.sort_by_key(|l| l.z_index);
        sorted
    }

    /// Limpiar dirty flags
    pub fn clear_dirty(&mut self) {
        for layer in &mut self.layers {
            layer.dirty = false;
        }
    }

    /// Contar layers dirty
    pub fn dirty_count(&self) -> usize {
        self.layers.iter().filter(|l| l.dirty).count()
    }

    /// Total de layers
    pub fn len(&self) -> usize {
        self.layers.len()
    }

    pub fn is_empty(&self) -> bool {
        self.layers.is_empty()
    }

    /// Root layer id
    pub fn root_id(&self) -> u32 {
        self.root
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer_tree_creation() {
        let tree = LayerTree::new();
        assert_eq!(tree.len(), 1);
        assert_eq!(tree.root_id(), 0);
    }

    #[test]
    fn test_create_layer() {
        let mut tree = LayerTree::new();
        let id = tree.create_layer("navbar".into(), 0.0, 0.0, 1280.0, 50.0);
        assert_eq!(tree.len(), 2);
        let layer = tree.get(id).unwrap();
        assert_eq!(layer.name, "navbar");
        assert!(layer.dirty);
    }

    #[test]
    fn test_compositor_layer() {
        let mut tree = LayerTree::new();
        let id = tree.create_compositor_layer("scroll".into(), 0.0, 0.0, 100.0, 100.0);
        let layer = tree.get(id).unwrap();
        assert!(layer.compositor_only);
    }

    #[test]
    fn test_transform_no_repaint() {
        let mut tree = LayerTree::new();
        let id = tree.create_compositor_layer("scroll".into(), 0.0, 0.0, 100.0, 100.0);
        // Limpiar dirty
        tree.clear_dirty();
        assert_eq!(tree.dirty_count(), 0);
        // Cambiar transform - NO debe marcar dirty
        tree.set_transform(id, 50.0, 0.0);
        assert_eq!(tree.dirty_count(), 0);
    }

    #[test]
    fn test_opacity() {
        let mut tree = LayerTree::new();
        let id = tree.create_layer("overlay".into(), 0.0, 0.0, 100.0, 100.0);
        tree.set_opacity(id, 0.5);
        let layer = tree.get(id).unwrap();
        assert_eq!(layer.opacity, 0.5);
    }

    #[test]
    fn test_invalidate() {
        let mut tree = LayerTree::new();
        let id = tree.create_layer("a".into(), 0.0, 0.0, 100.0, 100.0);
        tree.clear_dirty();
        tree.invalidate(id);
        assert_eq!(tree.dirty_count(), 1);
    }

    #[test]
    fn test_ordered_layers() {
        let mut tree = LayerTree::new();
        let _ = tree.create_layer("back".into(), 0.0, 0.0, 100.0, 100.0);
        let _ = tree.create_layer("front".into(), 0.0, 0.0, 100.0, 100.0);
        let _ = tree.create_layer("middle".into(), 0.0, 0.0, 100.0, 100.0);
        if let Some(l) = tree.get_mut(1) { l.z_index = -1; }
        if let Some(l) = tree.get_mut(2) { l.z_index = 10; }
        if let Some(l) = tree.get_mut(3) { l.z_index = 5; }
        let ordered = tree.ordered_layers();
        assert_eq!(ordered[0].z_index, -1);
    }

    #[test]
    fn test_opacity_clamp() {
        let mut tree = LayerTree::new();
        let id = tree.create_layer("a".into(), 0.0, 0.0, 100.0, 100.0);
        tree.set_opacity(id, 5.0);  // > 1.0
        assert_eq!(tree.get(id).unwrap().opacity, 1.0);
        tree.set_opacity(id, -1.0);  // < 0
        assert_eq!(tree.get(id).unwrap().opacity, 0.0);
    }
}
