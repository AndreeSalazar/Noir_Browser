//! WASM Host - Integración completa de WebAssembly
//!
//! Proporciona:
//! - WebAssembly.Module/Instance/Memory/Table/Function en JS engine
//! - Compilación de bytes WASM
//! - Linking con imports (console, fetch, DOM)
//! - Streaming compilation (futuro)
//! - DOM bindings para WASM modules

#![allow(dead_code)]

pub mod value;
pub mod module;
pub mod instance;
pub mod memory;
pub mod table;
pub mod imports;
pub mod dom_bindings;
pub mod wasi_runtime;

pub use value::{WasmValue, WasmValueType};
pub use module::WasmModule;
pub use instance::{WasmInstance, FunctionBody};
pub use memory::WasmMemory;
pub use table::WasmTable;
pub use imports::{ImportResolver, ImportValue};
pub use dom_bindings::DomBindings;
