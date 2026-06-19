#![allow(dead_code)]

#[derive(Debug, Clone, Copy)]
pub enum RenderLayerKind {
    Chrome,
    PageBackground,
    PageContent2D,
    PageContent3D,
    Overlay,
}

#[derive(Debug, Clone, Copy)]
pub struct RenderLayer {
    pub kind: RenderLayerKind,
    pub z_index: i32,
    pub opacity: f32,
}

pub struct Compositor {
    layers: Vec<RenderLayer>,
}

impl Compositor {
    pub fn new() -> Self {
        Self { layers: Vec::new() }
    }

    pub fn push_layer(&mut self, layer: RenderLayer) {
        self.layers.push(layer);
        self.layers.sort_by_key(|layer| layer.z_index);
    }

    pub fn layers(&self) -> &[RenderLayer] {
        &self.layers
    }
}
