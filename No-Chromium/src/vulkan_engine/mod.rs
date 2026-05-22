//! Vulkan Engine Module - Fase 0: Ultra-Fast Base
//! 
//! Arquitectura: Ash (bindings directos de Vulkan) + zero-copy + bindless descriptors
//! Objetivos: <8ms frame time, triple buffering, timeline semaphores

pub mod core;

pub use core::UltraFastVulkanEngine;

// === Stub Types para compatibilidad con lib.rs ===
// Estos tipos se implementarán completamente en fases posteriores

/// Handle opaco para referencia a recursos Vulkan
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct VulkanHandle(u64);

impl VulkanHandle {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
    
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

/// Información de frame para sincronización GPU
#[derive(Clone, Debug, Default)]
pub struct FrameInfo {
    pub frame_index: usize,
    pub timestamp: std::time::Instant,
    pub commands_submitted: bool,
}

impl FrameInfo {
    pub fn new(frame_index: usize) -> Self {
        Self {
            frame_index,
            timestamp: std::time::Instant::now(),
            commands_submitted: false,
        }
    }
}
