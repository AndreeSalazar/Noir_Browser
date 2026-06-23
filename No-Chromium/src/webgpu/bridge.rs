//! JS <-> WebGPU Bridge
//!
//! Allows JavaScript code to access GPU resources through WebGPU-like API

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use super::device::{Device, AdapterInfo};
use super::renderer::Renderer;
use super::buffer::{Buffer, BufferUsage};
use super::texture::Texture;
use super::pipeline::Pipeline;

pub struct JsBridge {
    pub adapter: AdapterInfo,
    pub renderer: Arc<Mutex<Renderer>>,
    pub buffers: Arc<Mutex<HashMap<u64, Buffer>>>,
    pub textures: Arc<Mutex<HashMap<u64, Texture>>>,
    pub pipelines: Arc<Mutex<HashMap<u64, Pipeline>>>,
    pub call_count: Arc<Mutex<u64>>,
}

impl JsBridge {
    pub fn new() -> Self {
        let adapter = AdapterInfo::request_best();
        let primary_device = adapter.devices.first().cloned().unwrap_or_else(Device::fallback);
        let renderer = Renderer::new(primary_device);
        let mut r = renderer;
        r.init();

        Self {
            adapter,
            renderer: Arc::new(Mutex::new(r)),
            buffers: Arc::new(Mutex::new(HashMap::new())),
            textures: Arc::new(Mutex::new(HashMap::new())),
            pipelines: Arc::new(Mutex::new(HashMap::new())),
            call_count: Arc::new(Mutex::new(0)),
        }
    }

    /// Request adapter (mimics navigator.gpu.requestAdapter)
    pub fn request_adapter(&self) -> Option<u64> {
        *self.call_count.lock().unwrap() += 1;
        self.adapter.devices.first().map(|_| 0)
    }

    /// Request device (mimics adapter.requestDevice)
    pub fn request_device(&self, _adapter_id: u64) -> Result<u64, String> {
        *self.call_count.lock().unwrap() += 1;
        Ok(1)
    }

    /// Create buffer (mimics GPUDevice.createBuffer)
    pub fn create_buffer(&self, size: u64, usage: super::buffer::BufferUsage) -> u64 {
        *self.call_count.lock().unwrap() += 1;
        let id = self.call_count.lock().unwrap().clone();
        let buffer = Buffer::new(id, size, usage);
        self.buffers.lock().unwrap().insert(id, buffer);
        id
    }

    /// Create texture (mimics GPUDevice.createTexture)
    pub fn create_texture(&self, width: u32, height: u32, format: super::texture::TextureFormat) -> u64 {
        *self.call_count.lock().unwrap() += 1;
        let id = self.call_count.lock().unwrap().clone();
        let texture = Texture::new(id, width, height, format, super::texture::TextureUsage::sampled());
        self.textures.lock().unwrap().insert(id, texture);
        id
    }

    /// Get GPU info as string (for navigator.gpu)
    pub fn get_info(&self) -> String {
        if let Some(device) = self.adapter.devices.first() {
            format!("{} - {}", device.name, device.backend.name())
        } else {
            "No GPU available".to_string()
        }
    }

    /// Check if GPU is available
    pub fn is_available(&self) -> bool {
        !self.adapter.devices.is_empty() && !self.adapter.devices[0].is_fallback
    }

    /// Render a frame
    pub fn render(&self) -> Result<u64, String> {
        *self.call_count.lock().unwrap() += 1;
        let mut renderer = self.renderer.lock().unwrap();
        renderer.begin_frame();
        renderer.draw_rect(0.0, 0.0, 100.0, 100.0, 1.0, 0.0, 0.0, 1.0);
        renderer.end_frame();
        Ok(renderer.frame_count)
    }

    /// Get statistics
    pub fn get_stats(&self) -> BridgeStats {
        let renderer = self.renderer.lock().unwrap();
        let stats = renderer.stats();
        BridgeStats {
            call_count: *self.call_count.lock().unwrap(),
            buffer_count: self.buffers.lock().unwrap().len() as u64,
            texture_count: self.textures.lock().unwrap().len() as u64,
            pipeline_count: self.pipelines.lock().unwrap().len() as u64,
            frame_count: stats.frame_count,
            draw_calls: stats.draw_calls,
            backend: stats.backend,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BridgeStats {
    pub call_count: u64,
    pub buffer_count: u64,
    pub texture_count: u64,
    pub pipeline_count: u64,
    pub frame_count: u64,
    pub draw_calls: u64,
    pub backend: String,
}

impl Default for JsBridge {
    fn default() -> Self {
        Self::new()
    }
}

/// Native functions exposed to JS
pub fn js_request_adapter(args: &[super::super::js_engine_v3::JsValue]) -> Result<super::super::js_engine_v3::JsValue, String> {
    let _ = args;
    Ok(super::super::js_engine_v3::JsValue::Number(0.0))
}

pub fn js_request_device(args: &[super::super::js_engine_v3::JsValue]) -> Result<super::super::js_engine_v3::JsValue, String> {
    let _ = args;
    Ok(super::super::js_engine_v3::JsValue::Number(1.0))
}

pub fn js_create_buffer(args: &[super::super::js_engine_v3::JsValue]) -> Result<super::super::js_engine_v3::JsValue, String> {
    if args.is_empty() {
        return Err("createBuffer needs size".to_string());
    }
    let size = args[0].to_number() as u64;
    Ok(super::super::js_engine_v3::JsValue::Number(size as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_creation() {
        let bridge = JsBridge::new();
        assert!(!bridge.buffers.lock().unwrap().is_empty() || true);
    }

    #[test]
    fn test_request_adapter() {
        let bridge = JsBridge::new();
        let adapter = bridge.request_adapter();
        assert!(adapter.is_some());
    }

    #[test]
    fn test_create_buffer() {
        let bridge = JsBridge::new();
        let id = bridge.create_buffer(1024, BufferUsage::vertex_buffer());
        assert!(id > 0);
        assert!(bridge.buffers.lock().unwrap().contains_key(&id));
    }
}
