//! JS <-> WASM Bridge
//!
//! Comunicación bidireccional entre JS engine y WASM instances.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::js_engine_v3::interpreter::Interpreter;
use crate::js_engine_v3::JsValue;
use crate::wasm_host::{WasmInstance, WasmValue};

/// Bridge entre JS y WASM
pub struct Bridge {
    #[allow(dead_code)]
    js_interp: Arc<Mutex<Interpreter>>,
    wasm_instances: Arc<Mutex<HashMap<String, Arc<Mutex<WasmInstance>>>>>,
    call_count: Arc<Mutex<u64>>,
}

impl Bridge {
    pub fn new(js_interp: Arc<Mutex<Interpreter>>) -> Self {
        Self {
            js_interp,
            wasm_instances: Arc::new(Mutex::new(HashMap::new())),
            call_count: Arc::new(Mutex::new(0)),
        }
    }

    pub fn register_wasm(&self, name: &str, instance: Arc<Mutex<WasmInstance>>) {
        self.wasm_instances.lock().unwrap().insert(name.to_string(), instance);
    }

    /// JS -> WASM: Llamar función WASM desde JS
    pub fn js_call_wasm(&self, module: &str, func: &str, args: Vec<JsValue>) -> Result<Vec<JsValue>, String> {
        *self.call_count.lock().unwrap() += 1;

        let wasm_args: Vec<WasmValue> = args.iter().map(|v| js_to_wasm(v)).collect();

        let instance = {
            let instances = self.wasm_instances.lock().unwrap();
            instances.get(module).cloned()
        };
        let instance = instance.ok_or_else(|| format!("Module not found: {}", module))?;

        let results = instance.lock().unwrap().call(func, &wasm_args)?;
        Ok(results.iter().map(|v| wasm_to_js(v)).collect())
    }

    /// WASM -> JS: Llamar función JS desde WASM (imports)
    pub fn wasm_call_js(&self, _func: &str, args: Vec<WasmValue>) -> Result<WasmValue, String> {
        *self.call_count.lock().unwrap() += 1;

        let js_args: Vec<JsValue> = args.iter().map(|v| wasm_to_js(v)).collect();
        let _ = js_args;

        // Simplified: would look up in JS env and call
        Ok(WasmValue::I32(0))
    }

    pub fn get_stats(&self) -> BridgeStats {
        BridgeStats {
            call_count: *self.call_count.lock().unwrap(),
            module_count: self.wasm_instances.lock().unwrap().len() as u32,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BridgeStats {
    pub call_count: u64,
    pub module_count: u32,
}

pub fn js_to_wasm(value: &JsValue) -> WasmValue {
    match value {
        JsValue::Number(n) => {
            if n.fract() == 0.0 && *n >= i32::MIN as f64 && *n <= i32::MAX as f64 {
                WasmValue::I32(*n as i32)
            } else {
                WasmValue::F64(*n)
            }
        }
        JsValue::Boolean(b) => WasmValue::I32(if *b { 1 } else { 0 }),
        JsValue::Null | JsValue::Undefined => WasmValue::I32(0),
        _ => WasmValue::I32(0),
    }
}

pub fn wasm_to_js(value: &WasmValue) -> JsValue {
    match value {
        WasmValue::I32(n) => JsValue::Number(*n as f64),
        WasmValue::I64(n) => JsValue::Number(*n as f64),
        WasmValue::F32(n) => JsValue::Number(*n as f64),
        WasmValue::F64(n) => JsValue::Number(*n),
        WasmValue::V128(n) => JsValue::Number(*n as f64),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_creation() {
        let interp = Arc::new(Mutex::new(Interpreter::new()));
        let bridge = Bridge::new(interp);
        let stats = bridge.get_stats();
        assert_eq!(stats.call_count, 0);
    }

    #[test]
    fn test_js_to_wasm_conversion() {
        assert_eq!(js_to_wasm(&JsValue::Number(42.0)), WasmValue::I32(42));
        assert_eq!(js_to_wasm(&JsValue::Boolean(true)), WasmValue::I32(1));
        assert_eq!(js_to_wasm(&JsValue::Boolean(false)), WasmValue::I32(0));
    }

    #[test]
    fn test_wasm_to_js_conversion() {
        assert_eq!(wasm_to_js(&WasmValue::I32(42)), JsValue::Number(42.0));
        assert_eq!(wasm_to_js(&WasmValue::F64(3.14)), JsValue::Number(3.14));
    }

    #[test]
    fn test_js_to_wasm_float() {
        assert_eq!(js_to_wasm(&JsValue::Number(3.14)), WasmValue::F64(3.14));
    }
}
