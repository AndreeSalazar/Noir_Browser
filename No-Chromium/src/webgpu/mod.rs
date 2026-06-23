//! WebGPU Module - GPU acceleration for Noir Browser
//!
//! Multi-backend support (managed by WebGPU):
//! - Windows: DirectX 12 (via WebGPU)
//! - Linux: Vulkan (via WebGPU)
//! - macOS: Metal (via WebGPU)
//! - Android: Vulkan (via WebGPU)
//!
//! Features:
//! - 2D rendering pipeline
//! - Compute shaders
//! - Texture management
//! - Bridge to JS engine v3
//!
//! Note: WebGPU abstracts all GPU APIs. We use WebGPU as the primary
//! GPU interface. Raw Vulkan/Metal/DX12 access is not needed.

#![allow(dead_code)]

pub mod device;
pub mod pipeline;
pub mod shaders;
pub mod buffer;
pub mod texture;
pub mod renderer;
pub mod bridge;
pub mod compute;
pub mod pwa;
pub mod integration;
pub mod gpu_renderer;

pub use device::{Device, DeviceFeatures, DeviceLimits, GpuBackend};
pub use pipeline::{Pipeline, PipelineConfig, PrimitiveTopology};
pub use shaders::{ShaderModule, ShaderStage};
pub use buffer::{Buffer, BufferUsage, VertexFormat};
pub use texture::{Texture, TextureFormat, TextureUsage};
pub use renderer::Renderer;
pub use bridge::JsBridge;
pub use compute::ComputePipeline;
pub use pwa::{PwaManager, ServiceWorker, Manifest};
pub use integration::{IntegratedRenderer, RendererStats};
