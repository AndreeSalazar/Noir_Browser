//! Vulkan Engine Module - Fase 0: Ultra-Fast Base
//! 
//! Arquitectura: Ash (bindings directos de Vulkan) + zero-copy + bindless descriptors
//! Objetivos: <8ms frame time, triple buffering, timeline semaphores

pub mod core;

pub use core::UltraFastVulkanEngine;
