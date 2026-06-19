use std::time::Instant;

pub mod core;

#[derive(Clone, Debug)]
pub struct FrameInfo {
    pub timestamp: Instant,
    pub frame_id: u64,
}

impl Default for FrameInfo {
    fn default() -> Self {
        Self { timestamp: Instant::now(), frame_id: 0 }
    }
}

// Re-export the main engine type
pub use core::UltraFastVulkanEngine;
