#![allow(dead_code)]

#[derive(Debug, Clone, Copy)]
pub struct Scene3DConfig {
    pub depth_enabled: bool,
    pub hdr_enabled: bool,
}

impl Default for Scene3DConfig {
    fn default() -> Self {
        Self {
            depth_enabled: true,
            hdr_enabled: false,
        }
    }
}

pub struct Renderer3D {
    config: Scene3DConfig,
}

impl Renderer3D {
    pub fn new(config: Scene3DConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> Scene3DConfig {
        self.config
    }
}
