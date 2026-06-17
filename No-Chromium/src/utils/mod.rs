//! Módulo de Utilidades Compartidas
//!
//! Provee canales IPC, helpers de memoria, y funciones utilitarias
//! usadas por múltiples componentes del navegador.

pub mod ipc;
pub mod process_model;
pub mod memory;

// Re-exportar tipos comunes
#[allow(unused_imports)]
pub use ipc::{BrowserMessage, RendererMessage, NetworkMessage};
#[allow(unused_imports)]
pub use process_model::{ProcessModel, determine_process_model, detect_available_ram};
#[allow(unused_imports)]
pub use memory::EphemeralCache;
