//! Import Resolver - Resuelve imports WASM a funciones JS/valores

use std::collections::HashMap;
use std::sync::Arc;
use crate::js_engine_v3::JsValue;
use crate::wasm_host::WasmValue;

pub type NativeFunction = Arc<dyn Fn(&[WasmValue]) -> Result<Vec<WasmValue>, String> + Send + Sync>;

/// Valor de import (función nativa, memoria, global, etc.)
pub enum ImportValue {
    Function(NativeFunction),
    Memory(Arc<crate::wasm_host::memory::WasmMemory>),
    Global(WasmValue),
    Table(Arc<crate::wasm_host::table::WasmTable>),
}

/// Resolver de imports
pub struct ImportResolver {
    imports: HashMap<(String, String), ImportValue>,
}

impl ImportResolver {
    pub fn new() -> Self {
        Self {
            imports: HashMap::new(),
        }
    }

    /// Registra un import
    pub fn register(&mut self, module: &str, name: &str, value: ImportValue) {
        self.imports.insert((module.to_string(), name.to_string()), value);
    }

    /// Registra una función nativa
    pub fn register_fn<F>(&mut self, module: &str, name: &str, f: F)
    where
        F: Fn(&[WasmValue]) -> Result<Vec<WasmValue>, String> + Send + Sync + 'static,
    {
        self.register(module, name, ImportValue::Function(Arc::new(f)));
    }

    /// Resuelve un import
    pub fn resolve(&self, module: &str, name: &str) -> Option<&ImportValue> {
        self.imports.get(&(module.to_string(), name.to_string()))
    }

    /// Verifica que todos los imports están registrados
    pub fn validate(&self, required: &[(String, String)]) -> Result<(), String> {
        for (m, n) in required {
            if self.resolve(m, n).is_none() {
                return Err(format!("Missing import: {}.{}", m, n));
            }
        }
        Ok(())
    }

    pub fn count(&self) -> usize {
        self.imports.len()
    }
}

impl Default for ImportResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for ImportValue {
    fn clone(&self) -> Self {
        match self {
            ImportValue::Function(f) => ImportValue::Function(Arc::clone(f)),
            ImportValue::Memory(m) => ImportValue::Memory(Arc::clone(m)),
            ImportValue::Global(v) => ImportValue::Global(*v),
            ImportValue::Table(t) => ImportValue::Table(Arc::clone(t)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolver_creation() {
        let r = ImportResolver::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_resolver_register_fn() {
        let mut r = ImportResolver::new();
        r.register_fn("env", "log", |_args| Ok(vec![]));
        assert_eq!(r.count(), 1);
        assert!(r.resolve("env", "log").is_some());
    }

    #[test]
    fn test_resolver_validate() {
        let mut r = ImportResolver::new();
        r.register_fn("env", "log", |_| Ok(vec![]));
        let required = vec![("env".to_string(), "log".to_string())];
        assert!(r.validate(&required).is_ok());
    }

    #[test]
    fn test_resolver_validate_missing() {
        let r = ImportResolver::new();
        let required = vec![("env".to_string(), "missing".to_string())];
        assert!(r.validate(&required).is_err());
    }

    #[test]
    fn test_resolver_call_function() {
        let mut r = ImportResolver::new();
        r.register_fn("env", "add", |args| {
            if let (Some(WasmValue::I32(a)), Some(WasmValue::I32(b))) = (args.get(0), args.get(1)) {
                Ok(vec![WasmValue::I32(a + b)])
            } else {
                Ok(vec![])
            }
        });
        let import = r.resolve("env", "add").unwrap();
        if let ImportValue::Function(f) = import {
            let result = f(&[WasmValue::I32(2), WasmValue::I32(3)]).unwrap();
            assert_eq!(result, vec![WasmValue::I32(5)]);
        }
    }
}
