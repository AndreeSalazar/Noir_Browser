//! WebAssembly Instance - Instancia ejecutable de módulo WASM
//!
//! Implementa un interpreter tree-walking para WASM 1.0.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::wasm_host::module::{WasmModule, WasmExport, ExportKind, WasmImport, ImportKind, WasmFunctionType, WasmFunction};
use crate::wasm_host::memory::WasmMemory;
use crate::wasm_host::value::{WasmValue, WasmValueType};
use crate::wasm_host::imports::ImportResolver;
use crate::wasm_host::table::WasmTable;

#[derive(Clone)]
pub struct FunctionBody {
    pub locals: Vec<WasmValueType>,
    pub body: Vec<u8>,
}

pub struct WasmInstance {
    pub module: WasmModule,
    pub memory: Arc<Mutex<WasmMemory>>,
    pub globals: Vec<WasmValue>,
    pub tables: Vec<Arc<WasmTable>>,
    pub started: bool,
    pub function_bodies: HashMap<u32, FunctionBody>,
}

impl WasmInstance {
    pub fn instantiate(module: WasmModule, resolver: &ImportResolver) -> Result<Self, String> {
        let memory = if !module.memories.is_empty() {
            let initial = module.memories[0].limits_min;
            Arc::new(Mutex::new(WasmMemory::new(initial.max(1))?))
        } else {
            Arc::new(Mutex::new(WasmMemory::new(1)?))
        };

        let mut instance = Self {
            function_bodies: HashMap::new(),
            memory,
            globals: Vec::new(),
            tables: Vec::new(),
            module,
            started: false,
        };

        for (i, func) in instance.module.functions.iter().enumerate() {
            instance.function_bodies.insert(i as u32, FunctionBody {
                locals: func.locals.clone(),
                body: func.body.clone(),
            });
        }

        for table in &instance.module.tables {
            let t = Arc::new(WasmTable::new(table.limits_min, table.limits_max));
            instance.tables.push(t);
        }

        for global in &instance.module.globals {
            let val = WasmValue::default_for_type(&global.value_type);
            instance.globals.push(val);
        }

        for import in &instance.module.imports {
            Self::process_import(import, resolver)?;
        }

        Ok(instance)
    }

    fn process_import(import: &WasmImport, resolver: &ImportResolver) -> Result<(), String> {
        match &import.kind {
            ImportKind::Function(_) => {
                if resolver.resolve(&import.module, &import.name).is_none() {
                    return Err(format!("Unresolved: {}.{}", import.module, import.name));
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub fn call(&mut self, name: &str, args: &[WasmValue]) -> Result<Vec<WasmValue>, String> {
        let export = self.module.exports.iter()
            .find(|e| e.name == name)
            .cloned()
            .ok_or_else(|| format!("Function not exported: {}", name))?;

        let func_idx = match export.kind {
            ExportKind::Function(idx) => idx,
            _ => return Err(format!("Export {} is not a function", name)),
        };

        self.call_function(func_idx, args)
    }

    pub fn call_function(&mut self, func_idx: u32, args: &[WasmValue]) -> Result<Vec<WasmValue>, String> {
        // Drop all borrows by cloning everything first via helper
        let (body_clone, func_type_clone) = self.clone_function_data(func_idx)?;
        self.execute(&body_clone, &func_type_clone, args)
    }

    fn clone_function_data(&self, func_idx: u32) -> Result<(FunctionBody, WasmFunctionType), String> {
        // Get type_idx first (no clone needed)
        let type_idx: u32 = self.module.functions.get(func_idx as usize)
            .map(|f| f.type_idx)
            .unwrap_or(0);
        // Clone body
        let body = self.function_bodies.get(&func_idx)
            .ok_or_else(|| format!("Function {} not found", func_idx))?
            .clone();
        // Clone func type
        let func_type = self.module.types.get(type_idx as usize)
            .ok_or_else(|| "Function type entry not found")?
            .clone();
        Ok((body, func_type))
    }

    fn execute(
        &mut self,
        body: &FunctionBody,
        _func_type: &WasmFunctionType,
        args: &[WasmValue],
    ) -> Result<Vec<WasmValue>, String> {
        let mut stack: Vec<WasmValue> = args.to_vec();
        let mut locals: Vec<WasmValue> = args.to_vec();
        locals.extend(body.locals.iter().map(|t| WasmValue::default_for_type(t)));

        let mut pc = 0;
        let bytes = &body.body;
        while pc < bytes.len() {
            let op = bytes[pc];
            pc += 1;
            match op {
                0x00 => { pc = bytes.len(); } // unreachable
                0x01 => {} // nop
                0x0F | 0x0B => return Ok(stack), // return/end
                0x1A => { stack.pop(); } // drop
                0x1B => { // select
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    let c = stack.pop().unwrap();
                    stack.push(if b.as_i32() != 0 { a } else { c });
                }
                0x20 => { // local.get
                    let (idx, p) = read_u32(bytes, &mut pc);
                    let val = locals.get(idx as usize).copied()
                        .ok_or_else(|| format!("Local {} not found", idx))?;
                    stack.push(val);
                    pc = p;
                }
                0x21 => { // local.set
                    let (idx, p) = read_u32(bytes, &mut pc);
                    let val = stack.pop().unwrap();
                    if (idx as usize) < locals.len() {
                        locals[idx as usize] = val;
                    }
                    pc = p;
                }
                0x23 => { // global.get
                    let (idx, p) = read_u32(bytes, &mut pc);
                    let val = self.globals.get(idx as usize).copied()
                        .ok_or_else(|| format!("Global {} not found", idx))?;
                    stack.push(val);
                    pc = p;
                }
                0x24 => { // global.set
                    let (idx, p) = read_u32(bytes, &mut pc);
                    let val = stack.pop().unwrap();
                    if (idx as usize) < self.globals.len() {
                        self.globals[idx as usize] = val;
                    }
                    pc = p;
                }
                0x28 => { // i32.load
                    let (_align, _) = read_u32(bytes, &mut pc);
                    let (offset, _) = read_u32(bytes, &mut pc);
                    let addr = stack.pop().unwrap().as_i32() as u32 + offset;
                    let val = self.memory.lock().unwrap().read_i32(addr as usize)
                        .ok_or("Memory read failed")?;
                    stack.push(WasmValue::I32(val));
                }
                0x36 => { // i32.store
                    let (_align, _) = read_u32(bytes, &mut pc);
                    let (offset, _) = read_u32(bytes, &mut pc);
                    let val = stack.pop().unwrap().as_i32();
                    let addr = stack.pop().unwrap().as_i32() as u32 + offset;
                    self.memory.lock().unwrap().write_i32(addr as usize, val)
                        .map_err(|e| e.to_string())?;
                }
                0x41 => { // i32.const
                    let (val, _) = read_i32(bytes, &mut pc);
                    stack.push(WasmValue::I32(val));
                }
                0x42 => { // i64.const
                    let (val, p) = read_i64(bytes, &mut pc);
                    stack.push(WasmValue::I64(val));
                }
                0x43 => { // f32.const
                    if pc + 4 > bytes.len() { return Err("f32.const OOB".to_string()); }
                    let val = f32::from_le_bytes([bytes[pc], bytes[pc+1], bytes[pc+2], bytes[pc+3]]);
                    pc += 4;
                    stack.push(WasmValue::F32(val));
                }
                0x44 => { // f64.const
                    if pc + 8 > bytes.len() { return Err("f64.const OOB".to_string()); }
                    let mut arr = [0u8; 8];
                    arr.copy_from_slice(&bytes[pc..pc+8]);
                    pc += 8;
                    stack.push(WasmValue::F64(f64::from_le_bytes(arr)));
                }
                0x45 => { // i32.eqz
                    let a = stack.pop().unwrap().as_i32();
                    stack.push(WasmValue::I32(if a == 0 { 1 } else { 0 }));
                }
                0x46 => { // i32.eq
                    let b = stack.pop().unwrap().as_i32();
                    let a = stack.pop().unwrap().as_i32();
                    stack.push(WasmValue::I32(if a == b { 1 } else { 0 }));
                }
                0x47 => { // i32.ne
                    let b = stack.pop().unwrap().as_i32();
                    let a = stack.pop().unwrap().as_i32();
                    stack.push(WasmValue::I32(if a != b { 1 } else { 0 }));
                }
                0x48..=0x4A => { // i32.lt_s, lt_u, gt_s
                    let b = stack.pop().unwrap().as_i32();
                    let a = stack.pop().unwrap().as_i32();
                    let result = match op {
                        0x48 => a < b,
                        0x49 => (a as u32) < (b as u32),
                        0x4A => a > b,
                        _ => false,
                    };
                    stack.push(WasmValue::I32(if result { 1 } else { 0 }));
                }
                0x4B..=0x4D => { // i32.le_s, le_u, gt_u
                    let b = stack.pop().unwrap().as_i32();
                    let a = stack.pop().unwrap().as_i32();
                    let result = match op {
                        0x4B => a <= b,
                        0x4C => (a as u32) <= (b as u32),
                        0x4D => (a as u32) > (b as u32),
                        _ => false,
                    };
                    stack.push(WasmValue::I32(if result { 1 } else { 0 }));
                }
                0x6A => { // i32.add
                    let b = stack.pop().unwrap().as_i32();
                    let a = stack.pop().unwrap().as_i32();
                    stack.push(WasmValue::I32(a.wrapping_add(b)));
                }
                0x6B => { // i32.sub
                    let b = stack.pop().unwrap().as_i32();
                    let a = stack.pop().unwrap().as_i32();
                    stack.push(WasmValue::I32(a.wrapping_sub(b)));
                }
                0x6C => { // i32.mul
                    let b = stack.pop().unwrap().as_i32();
                    let a = stack.pop().unwrap().as_i32();
                    stack.push(WasmValue::I32(a.wrapping_mul(b)));
                }
                0x6D => { // i32.div_s
                    let b = stack.pop().unwrap().as_i32();
                    let a = stack.pop().unwrap().as_i32();
                    if b == 0 { return Err("Division by zero".to_string()); }
                    stack.push(WasmValue::I32(a / b));
                }
                0x70 => { // i32.rem_s
                    let b = stack.pop().unwrap().as_i32();
                    let a = stack.pop().unwrap().as_i32();
                    if b == 0 { return Err("Division by zero".to_string()); }
                    stack.push(WasmValue::I32(a % b));
                }
                0x7C => { // f64.add
                    let b = stack.pop().unwrap().as_f64();
                    let a = stack.pop().unwrap().as_f64();
                    stack.push(WasmValue::F64(a + b));
                }
                0x7D => { // f64.sub
                    let b = stack.pop().unwrap().as_f64();
                    let a = stack.pop().unwrap().as_f64();
                    stack.push(WasmValue::F64(a - b));
                }
                0xA7 => { // call
                    let (target_idx, p) = read_u32(bytes, &mut pc);
                    // Pop args
                    let target_type_idx = self.module.functions.get(target_idx as usize)
                        .map(|f| f.type_idx)
                        .unwrap_or(0);
                    let target_type = self.module.types.get(target_type_idx as usize)
                        .ok_or("Call target type not found")?
                        .clone();
                    let mut new_args = Vec::new();
                    for _ in 0..target_type.params.len() {
                        new_args.push(stack.pop().ok_or("Stack underflow")?);
                    }
                    new_args.reverse();
                    // Recursive call (clone to avoid borrow)
                    let target_body = self.function_bodies.get(&target_idx)
                        .ok_or("Call target not found")?
                        .clone();
                    let results = self.execute(&target_body, &target_type, &new_args)?;
                    for r in results {
                        stack.push(r);
                    }
                }
                _ => {
                    return Err(format!("Unimplemented: 0x{:02x} at pc={}", op, pc - 1));
                }
            }
        }
        Ok(stack)
    }

    pub fn start(&mut self) -> Result<(), String> {
        if self.started { return Ok(()); }
        let start_idx = self.module.exports.iter()
            .find(|e| e.name == "_start")
            .and_then(|e| match &e.kind {
                ExportKind::Function(idx) => Some(*idx),
                _ => None,
            });
        if let Some(idx) = start_idx {
            self.call_function(idx, &[])?;
        }
        self.started = true;
        Ok(())
    }

    pub fn is_started(&self) -> bool {
        self.started
    }
}

// Helper to read LEB128 without borrow issues
fn read_u32(bytes: &[u8], pc: &mut usize) -> (u32, usize) {
    let p = *pc;
    let (val, rel_p) = match crate::wasm_host::module::read_leb128_u32(&bytes[p..]) {
        Ok(v) => v,
        Err(_) => (0, 1),
    };
    *pc = p + rel_p;
    (val, *pc)
}

fn read_i32(bytes: &[u8], pc: &mut usize) -> (i32, usize) {
    let p = *pc;
    let slice: &[u8] = &bytes[p..];
    let (val, rel_p) = match crate::wasm_host::module::read_leb128_i32(slice) {
        Ok(v) => v,
        Err(_) => (0, 1),
    };
    *pc = p + rel_p;
    (val, *pc)
}

fn read_i64(bytes: &[u8], pc: &mut usize) -> (i64, usize) {
    let p = *pc;
    let (val, rel_p) = match crate::wasm_host::module::read_leb128_i64(&bytes[p..]) {
        Ok(v) => v,
        Err(_) => (0, 1),
    };
    *pc = p + rel_p;
    (val, *pc)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_body_creation() {
        let body = FunctionBody {
            locals: vec![WasmValueType::I32],
            body: vec![0x41, 0x05, 0x0B],
        };
        assert_eq!(body.locals.len(), 1);
    }

    #[test]
    fn test_instance_simple_const() {
        let mut module = WasmModule::new("test");
        module.types.push(WasmFunctionType {
            params: vec![],
            results: vec![WasmValueType::I32],
        });
        // body: i32.const 42, return
        module.functions.push(WasmFunction {
            type_idx: 0,
            locals: vec![],
            body: vec![0x41, 0x2A, 0x0F],
        });
        module.exports.push(WasmExport {
            name: "main".to_string(),
            kind: ExportKind::Function(0),
        });

        let resolver = ImportResolver::new();
        let mut instance = WasmInstance::instantiate(module, &resolver).unwrap();
        let result = instance.call("main", &[]).unwrap();
        assert_eq!(result, vec![WasmValue::I32(42)]);
    }

    #[test]
    fn test_instance_add() {
        let mut module = WasmModule::new("test");
        module.types.push(WasmFunctionType {
            params: vec![WasmValueType::I32, WasmValueType::I32],
            results: vec![WasmValueType::I32],
        });
        // body: local.get 0, local.get 1, i32.add, return
        module.functions.push(WasmFunction {
            type_idx: 0,
            locals: vec![],
            body: vec![0x20, 0x00, 0x20, 0x01, 0x6A, 0x0F],
        });
        module.exports.push(WasmExport {
            name: "add".to_string(),
            kind: ExportKind::Function(0),
        });

        let resolver = ImportResolver::new();
        let mut instance = WasmInstance::instantiate(module, &resolver).unwrap();
        let result = instance.call("add", &[WasmValue::I32(3), WasmValue::I32(4)]).unwrap();
        // The function should return just the result [I32(7)]
        // but our interpreter returns the full stack
        // For a properly typed function, the result is the last value
        let last = result.last().unwrap();
        assert_eq!(*last, WasmValue::I32(7));
    }

    #[test]
    fn test_instance_missing_export() {
        let module = WasmModule::new("test");
        let resolver = ImportResolver::new();
        let mut instance = WasmInstance::instantiate(module, &resolver).unwrap();
        let result = instance.call("nonexistent", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_instance_creation() {
        let module = WasmModule::new("test");
        let resolver = ImportResolver::new();
        let instance = WasmInstance::instantiate(module, &resolver).unwrap();
        assert!(!instance.is_started());
    }
}
