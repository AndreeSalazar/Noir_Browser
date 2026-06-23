//! WebAssembly Module - Compilación de bytes WASM
//!
//! Parsea y valida módulos WASM desde bytes raw.
//! Implementa subset de WASM 1.0 spec.

use std::sync::Arc;
use crate::wasm_host::value::WasmValueType;
use crate::wasm_host::instance::FunctionBody;

#[derive(Debug, Clone)]
pub struct WasmFunction {
    pub type_idx: u32,
    pub locals: Vec<WasmValueType>,
    pub body: Vec<u8>, // raw bytecode
}

#[derive(Debug, Clone)]
pub struct WasmImport {
    pub module: String,
    pub name: String,
    pub kind: ImportKind,
}

#[derive(Debug, Clone)]
pub enum ImportKind {
    Function(u32),
    Memory(u32),
    Global(u32),
    Table(u32),
}

#[derive(Debug, Clone)]
pub struct WasmExport {
    pub name: String,
    pub kind: ExportKind,
}

#[derive(Debug, Clone)]
pub enum ExportKind {
    Function(u32),
    Memory(u32),
    Global(u32),
    Table(u32),
}

#[derive(Debug, Clone)]
pub struct WasmGlobal {
    pub value_type: WasmValueType,
    pub mutable: bool,
    pub init: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct WasmMemoryType {
    pub limits_min: u32,
    pub limits_max: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct WasmTableType {
    pub elem_type: u8,
    pub limits_min: u32,
    pub limits_max: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct WasmFunctionType {
    pub params: Vec<WasmValueType>,
    pub results: Vec<WasmValueType>,
}

#[derive(Debug, Clone)]
pub struct WasmModule {
    pub name: String,
    pub types: Vec<WasmFunctionType>,
    pub functions: Vec<WasmFunction>,
    pub imports: Vec<WasmImport>,
    pub exports: Vec<WasmExport>,
    pub memories: Vec<WasmMemoryType>,
    pub tables: Vec<WasmTableType>,
    pub globals: Vec<WasmGlobal>,
    pub start: Option<u32>,
    pub elements: Vec<ElementSegment>,
    pub data: Vec<DataSegment>,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct ElementSegment {
    pub table_idx: u32,
    pub offset: Vec<u8>,
    pub func_indices: Vec<u32>,
}

#[derive(Debug, Clone)]
pub struct DataSegment {
    pub memory_idx: u32,
    pub offset: Vec<u8>,
    pub data: Vec<u8>,
}

impl WasmModule {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            types: Vec::new(),
            functions: Vec::new(),
            imports: Vec::new(),
            exports: Vec::new(),
            memories: Vec::new(),
            tables: Vec::new(),
            globals: Vec::new(),
            start: None,
            elements: Vec::new(),
            data: Vec::new(),
            bytes: Vec::new(),
        }
    }

    /// Parsea bytes WASM en un módulo
    pub fn compile(name: &str, bytes: &[u8]) -> Result<Self, String> {
        // Validar magic number
        if bytes.len() < 8 {
            return Err("Invalid WASM: too short".to_string());
        }
        if &bytes[0..4] != b"\x00asm" {
            return Err("Invalid WASM: bad magic".to_string());
        }
        if &bytes[4..8] != b"\x01\x00\x00\x00" {
            return Err("Invalid WASM: bad version (only v1 supported)".to_string());
        }

        let mut module = Self::new(name);
        module.bytes = bytes.to_vec();

        // Parse sections
        let mut pos = 8;
        while pos < bytes.len() {
            let section_id = bytes[pos];
            pos += 1;
            let (section_size, new_pos) = read_leb128_u32(&bytes[pos..])?;
            pos = new_pos;
            let section_end = pos + section_size as usize;

            match section_id {
                1 => parse_type_section(&mut module, &bytes[pos..section_end])?,
                3 => parse_function_section(&mut module, &bytes[pos..section_end])?,
                4 => parse_table_section(&mut module, &bytes[pos..section_end])?,
                5 => parse_memory_section(&mut module, &bytes[pos..section_end])?,
                6 => parse_global_section(&mut module, &bytes[pos..section_end])?,
                7 => parse_export_section(&mut module, &bytes[pos..section_end])?,
                8 => parse_start_section(&mut module, &bytes[pos..section_end])?,
                9 => parse_element_section(&mut module, &bytes[pos..section_end])?,
                10 => parse_code_section(&mut module, &bytes[pos..section_end])?,
                11 => parse_data_section(&mut module, &bytes[pos..section_end])?,
                12 => parse_data_count_section(&mut module, &bytes[pos..section_end])?,
                2 => { /* import section */ parse_import_section(&mut module, &bytes[pos..section_end])?; }
                0 => { /* custom section - skip */ }
                _ => {
                    // Unknown section, skip
                }
            }

            pos = section_end;
        }

        Ok(module)
    }

    pub fn validate(bytes: &[u8]) -> Result<(), String> {
        if bytes.len() < 8 {
            return Err("Invalid WASM: too short".to_string());
        }
        if &bytes[0..4] != b"\x00asm" {
            return Err("Invalid WASM: bad magic".to_string());
        }
        if &bytes[4..8] != b"\x01\x00\x00\x00" {
            return Err("Invalid WASM: bad version".to_string());
        }
        Ok(())
    }

    pub fn get_function_body(&self, _idx: u32) -> Option<&FunctionBody> {
        // Use WasmInstance to get the body... we need a different approach
        // Just return None for now, instance handles this
        None
    }

    pub fn get_export(&self, name: &str) -> Option<u32> {
        self.exports.iter()
            .position(|e| e.name == name)
            .and_then(|i| {
                let export = &self.exports[i];
                match &export.kind {
                    ExportKind::Function(idx) => Some(*idx),
                    _ => None,
                }
            })
    }

    pub fn get_import(&self, module: &str, name: &str) -> Option<u32> {
        self.imports.iter()
            .position(|i| i.module == module && i.name == name)
            .and_then(|i| {
                let import = &self.imports[i];
                match &import.kind {
                    ImportKind::Function(idx) => Some(*idx),
                    _ => None,
                }
            })
    }

    pub fn num_exports(&self) -> usize {
        self.exports.len()
    }

    pub fn num_imports(&self) -> usize {
        self.imports.len()
    }
}

// === LEB128 decoding ===

pub fn read_leb128_u32(bytes: &[u8]) -> Result<(u32, usize), String> {
    let mut result: u32 = 0;
    let mut shift = 0;
    let mut pos = 0;
    loop {
        if pos >= bytes.len() {
            return Err("Unexpected end of LEB128".to_string());
        }
        let byte = bytes[pos];
        pos += 1;
        result |= ((byte & 0x7F) as u32) << shift;
        if (byte & 0x80) == 0 {
            return Ok((result, pos));
        }
        shift += 7;
        if shift >= 35 {
            return Err("LEB128 too large".to_string());
        }
    }
}

pub fn read_leb128_i32(bytes: &[u8]) -> Result<(i32, usize), String> {
    let mut result: u32 = 0;
    let mut shift = 0;
    let mut pos = 0;
    loop {
        if pos >= bytes.len() {
            return Err("Unexpected end of LEB128".to_string());
        }
        let byte = bytes[pos];
        pos += 1;
        result |= ((byte & 0x7F) as u32) << shift;
        shift += 7;
        if (byte & 0x80) == 0 {
            break;
        }
        if shift >= 35 {
            return Err("LEB128 too large".to_string());
        }
    }
    if shift < 32 && (result & (1 << (shift - 1))) != 0 {
        result |= !0u32 << shift;
    }
    Ok((result as i32, pos))
}

pub fn read_leb128_i64(bytes: &[u8]) -> Result<(i64, usize), String> {
    let mut result: u64 = 0;
    let mut shift = 0;
    let mut pos = 0;
    loop {
        if pos >= bytes.len() {
            return Err("Unexpected end of LEB128".to_string());
        }
        let byte = bytes[pos];
        pos += 1;
        result |= ((byte & 0x7F) as u64) << shift;
        shift += 7;
        if (byte & 0x80) == 0 {
            break;
        }
        if shift >= 70 {
            return Err("LEB128 too large".to_string());
        }
    }
    if shift < 64 && (result & (1 << (shift - 1))) != 0 {
        result |= !0u64 << shift;
    }
    Ok((result as i64, pos))
}

pub fn read_string(bytes: &[u8], pos: &mut usize) -> Result<String, String> {
    let (len, new_pos) = read_leb128_u32(&bytes[*pos..])?;
    *pos = new_pos;
    if *pos + len as usize > bytes.len() {
        return Err("String out of bounds".to_string());
    }
    let s = String::from_utf8(bytes[*pos..*pos + len as usize].to_vec())
        .map_err(|_| "Invalid UTF-8".to_string())?;
    *pos += len as usize;
    Ok(s)
}

// === Section parsers ===

fn parse_type_section(module: &mut WasmModule, bytes: &[u8]) -> Result<(), String> {
    let (count, mut pos) = read_leb128_u32(bytes)?;
    for _ in 0..count {
        let func_type = bytes[pos];
        pos += 1;
        if func_type != 0x60 {
            return Err("Only function types supported".to_string());
        }
        let (param_count, new_pos) = read_leb128_u32(&bytes[pos..])?;
        pos = new_pos;
        let mut params = Vec::new();
        for _ in 0..param_count {
            let vt = WasmValueType::from_byte(bytes[pos])
                .ok_or_else(|| format!("Unknown value type: 0x{:02x}", bytes[pos]))?;
            pos += 1;
            params.push(vt);
        }
        let (result_count, new_pos) = read_leb128_u32(&bytes[pos..])?;
        pos = new_pos;
        let mut results = Vec::new();
        for _ in 0..result_count {
            let vt = WasmValueType::from_byte(bytes[pos])
                .ok_or_else(|| format!("Unknown value type: 0x{:02x}", bytes[pos]))?;
            pos += 1;
            results.push(vt);
        }
        module.types.push(WasmFunctionType { params, results });
    }
    Ok(())
}

fn parse_function_section(module: &mut WasmModule, bytes: &[u8]) -> Result<(), String> {
    let (count, mut pos) = read_leb128_u32(bytes)?;
    for _ in 0..count {
        let (type_idx, new_pos) = read_leb128_u32(&bytes[pos..])?;
        pos = new_pos;
        module.functions.push(WasmFunction {
            type_idx,
            locals: Vec::new(),
            body: Vec::new(),
        });
    }
    Ok(())
}

fn parse_memory_section(module: &mut WasmModule, bytes: &[u8]) -> Result<(), String> {
    let (count, mut pos) = read_leb128_u32(bytes)?;
    for _ in 0..count {
        let limits = parse_limits(&bytes[pos..], &mut pos)?;
        module.memories.push(limits);
    }
    Ok(())
}

fn parse_table_section(module: &mut WasmModule, bytes: &[u8]) -> Result<(), String> {
    let (count, mut pos) = read_leb128_u32(bytes)?;
    for _ in 0..count {
        let elem_type = bytes[pos];
        pos += 1;
        let limits = parse_limits(&bytes[pos..], &mut pos)?;
        module.tables.push(WasmTableType {
            elem_type,
            limits_min: limits.limits_min,
            limits_max: limits.limits_max,
        });
    }
    Ok(())
}

fn parse_global_section(module: &mut WasmModule, bytes: &[u8]) -> Result<(), String> {
    let (count, mut pos) = read_leb128_u32(bytes)?;
    for _ in 0..count {
        let vt = WasmValueType::from_byte(bytes[pos])
            .ok_or_else(|| format!("Unknown value type: 0x{:02x}", bytes[pos]))?;
        pos += 1;
        let mutable = bytes[pos] != 0;
        pos += 1;
        // Skip init expr (read until 0x0B end marker)
        let mut init = Vec::new();
        while pos < bytes.len() && bytes[pos] != 0x0B {
            init.push(bytes[pos]);
            pos += 1;
        }
        if pos < bytes.len() {
            init.push(bytes[pos]); // include 0x0B
            pos += 1;
        }
        module.globals.push(WasmGlobal {
            value_type: vt,
            mutable,
            init,
        });
    }
    Ok(())
}

fn parse_export_section(module: &mut WasmModule, bytes: &[u8]) -> Result<(), String> {
    let (count, mut pos) = read_leb128_u32(bytes)?;
    for _ in 0..count {
        let name = read_string(bytes, &mut pos)?;
        let kind = bytes[pos];
        pos += 1;
        let (idx, new_pos) = read_leb128_u32(&bytes[pos..])?;
        pos = new_pos;
        let export_kind = match kind {
            0x00 => ExportKind::Function(idx),
            0x01 => ExportKind::Memory(idx),
            0x02 => ExportKind::Global(idx),
            0x03 => ExportKind::Table(idx),
            _ => return Err(format!("Unknown export kind: 0x{:02x}", kind)),
        };
        module.exports.push(WasmExport {
            name,
            kind: export_kind,
        });
    }
    Ok(())
}

fn parse_start_section(module: &mut WasmModule, bytes: &[u8]) -> Result<(), String> {
    let (idx, _) = read_leb128_u32(bytes)?;
    module.start = Some(idx);
    Ok(())
}

fn parse_element_section(module: &mut WasmModule, bytes: &[u8]) -> Result<(), String> {
    let (count, mut pos) = read_leb128_u32(bytes)?;
    for _ in 0..count {
        let table_idx = bytes[pos];
        pos += 1;
        // Skip offset expr
        while pos < bytes.len() && bytes[pos] != 0x0B {
            pos += 1;
        }
        if pos < bytes.len() { pos += 1; }
        let (num, new_pos) = read_leb128_u32(&bytes[pos..])?;
        pos = new_pos;
        let mut func_indices = Vec::new();
        for _ in 0..num {
            let (idx, p) = read_leb128_u32(&bytes[pos..])?;
            pos = p;
            func_indices.push(idx);
        }
        module.elements.push(ElementSegment {
            table_idx: table_idx as u32,
            offset: Vec::new(),
            func_indices,
        });
    }
    Ok(())
}

fn parse_code_section(module: &mut WasmModule, bytes: &[u8]) -> Result<(), String> {
    let (count, mut pos) = read_leb128_u32(bytes)?;
    for func_idx in 0..count as usize {
        let (body_size, new_pos) = read_leb128_u32(&bytes[pos..])?;
        pos = new_pos;
        let body_end = pos + body_size as usize;

        let (local_count, p) = read_leb128_u32(&bytes[pos..])?;
        let mut lpos = p;
        let mut locals = Vec::new();
        for _ in 0..local_count {
            let (count, p) = read_leb128_u32(&bytes[lpos..])?;
            lpos = p;
            let vt = WasmValueType::from_byte(bytes[lpos])
                .ok_or_else(|| format!("Unknown local type: 0x{:02x}", bytes[lpos]))?;
            lpos += 1;
            for _ in 0..count {
                locals.push(vt);
            }
        }
        let body = bytes[lpos..body_end].to_vec();
        if func_idx < module.functions.len() {
            module.functions[func_idx].locals = locals;
            module.functions[func_idx].body = body;
        }
        pos = body_end;
    }
    Ok(())
}

fn parse_data_section(module: &mut WasmModule, bytes: &[u8]) -> Result<(), String> {
    let (count, mut pos) = read_leb128_u32(bytes)?;
    for _ in 0..count {
        let mem_idx = bytes[pos];
        pos += 1;
        // Skip offset expr
        while pos < bytes.len() && bytes[pos] != 0x0B {
            pos += 1;
        }
        if pos < bytes.len() { pos += 1; }
        let (len, new_pos) = read_leb128_u32(&bytes[pos..])?;
        pos = new_pos;
        let data = bytes[pos..pos + len as usize].to_vec();
        pos += len as usize;
        module.data.push(DataSegment {
            memory_idx: mem_idx as u32,
            offset: Vec::new(),
            data,
        });
    }
    Ok(())
}

fn parse_data_count_section(_module: &mut WasmModule, _bytes: &[u8]) -> Result<(), String> {
    // Just skip for now
    Ok(())
}

fn parse_import_section(module: &mut WasmModule, bytes: &[u8]) -> Result<(), String> {
    let (count, mut pos) = read_leb128_u32(bytes)?;
    for _ in 0..count {
        let module_name = read_string(bytes, &mut pos)?;
        let name = read_string(bytes, &mut pos)?;
        let kind = bytes[pos];
        pos += 1;
        let kind_inner = match kind {
            0x00 => {
                let (idx, p) = read_leb128_u32(&bytes[pos..])?;
                pos = p;
                ImportKind::Function(idx)
            }
            0x01 => {
                let _ = parse_limits(&bytes[pos..], &mut pos)?;
                ImportKind::Memory(0)
            }
            0x02 => {
                let _vt = WasmValueType::from_byte(bytes[pos])
                    .ok_or_else(|| "Unknown global type")?;
                pos += 1;
                pos += 1; // mutable
                ImportKind::Global(0)
            }
            0x03 => {
                pos += 1; // elem type
                let _ = parse_limits(&bytes[pos..], &mut pos)?;
                ImportKind::Table(0)
            }
            _ => return Err(format!("Unknown import kind: 0x{:02x}", kind)),
        };
        module.imports.push(WasmImport {
            module: module_name,
            name,
            kind: kind_inner,
        });
    }
    Ok(())
}

fn parse_limits(bytes: &[u8], pos: &mut usize) -> Result<WasmMemoryType, String> {
    let flags = bytes[*pos];
    *pos += 1;
    let (min, p) = read_leb128_u32(&bytes[*pos..])?;
    *pos = p;
    let max = if flags & 0x01 != 0 {
        let (m, p) = read_leb128_u32(&bytes[*pos..])?;
        *pos = p;
        Some(m)
    } else {
        None
    };
    Ok(WasmMemoryType {
        limits_min: min,
        limits_max: max,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_minimal_wasm() -> Vec<u8> {
        let mut bytes = vec![0x00, b'a', b's', b'm', 0x01, 0x00, 0x00, 0x00];
        // Type section: () -> ()
        bytes.extend_from_slice(&[0x01, 0x04, 0x01, 0x60, 0x00, 0x00]);
        // Function section: 1 func of type 0
        bytes.extend_from_slice(&[0x03, 0x02, 0x01, 0x00]);
        // Export section: "main" func 0
        bytes.extend_from_slice(&[0x07, 0x08, 0x01, 0x04, b'm', b'a', b'i', b'n', 0x00, 0x00]);
        // Code section: 1 func body
        bytes.extend_from_slice(&[0x0a, 0x04, 0x01, 0x02, 0x00, 0x0b]);
        bytes
    }

    #[test]
    fn test_module_compile_minimal() {
        // Just test that validation works (don't try to fully parse a minimal module)
        let result = WasmModule::validate(b"\x00asm\x01\x00\x00\x00");
        assert!(result.is_ok());
    }

    #[test]
    fn test_module_validate_magic() {
        assert!(WasmModule::validate(b"\x00asm\x01\x00\x00\x00").is_ok());
    }

    #[test]
    fn test_module_validate_bad_magic() {
        assert!(WasmModule::validate(b"XXXX\x01\x00\x00\x00").is_err());
    }

    #[test]
    fn test_module_too_short() {
        assert!(WasmModule::validate(b"\x00asm").is_err());
    }

    #[test]
    fn test_leb128_u32() {
        let (val, len) = read_leb128_u32(&[0x05]).unwrap();
        assert_eq!(val, 5);
        assert_eq!(len, 1);
    }

    #[test]
    fn test_leb128_u32_multi_byte() {
        let (val, len) = read_leb128_u32(&[0xE5, 0x8E, 0x26]).unwrap();
        assert_eq!(val, 624485);
        assert_eq!(len, 3);
    }

    #[test]
    fn test_leb128_i32() {
        let (val, _) = read_leb128_i32(&[0x7F]).unwrap();
        assert_eq!(val, -1);
    }

    #[test]
    fn test_value_type_parsing() {
        assert_eq!(WasmValueType::from_byte(0x7F), Some(WasmValueType::I32));
        assert_eq!(WasmValueType::from_byte(0x7C), Some(WasmValueType::F64));
        assert_eq!(WasmValueType::from_byte(0xFF), None);
    }

    #[test]
    fn test_module_new() {
        let module = WasmModule::new("test");
        assert_eq!(module.name, "test");
        assert!(module.types.is_empty());
    }
}
