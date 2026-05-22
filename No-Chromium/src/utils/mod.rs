//! Módulo de Utilidades Compartidas
//!
//! Provee canales IPC, helpers de memoria, y funciones utilitarias
//! usadas por múltiples componentes del navegador.

pub mod ipc;
pub mod process_model;
pub mod memory;

// Re-exportar tipos comunes
pub use ipc::{BrowserMessage, RendererMessage, NetworkMessage, RenderMessage};
pub use process_model::{ProcessModel, determine_process_model};
pub use memory::{EphemeralBuffer, zeroize_slice};
