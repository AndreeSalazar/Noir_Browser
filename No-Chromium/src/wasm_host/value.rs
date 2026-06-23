//! WASM Value Types
//!
//! Tipos de valores que WASM puede manejar.

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WasmValue {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
    V128(u128),
}

impl WasmValue {
    pub fn default_for_type(value_type: &WasmValueType) -> Self {
        match value_type {
            WasmValueType::I32 => WasmValue::I32(0),
            WasmValueType::I64 => WasmValue::I64(0),
            WasmValueType::F32 => WasmValue::F32(0.0),
            WasmValueType::F64 => WasmValue::F64(0.0),
            WasmValueType::V128 => WasmValue::V128(0),
        }
    }

    pub fn as_i32(&self) -> i32 {
        match self {
            WasmValue::I32(n) => *n,
            WasmValue::I64(n) => *n as i32,
            WasmValue::F32(n) => *n as i32,
            WasmValue::F64(n) => *n as i32,
            WasmValue::V128(n) => *n as i32,
        }
    }

    pub fn as_i64(&self) -> i64 {
        match self {
            WasmValue::I32(n) => *n as i64,
            WasmValue::I64(n) => *n,
            WasmValue::F32(n) => *n as i64,
            WasmValue::F64(n) => *n as i64,
            WasmValue::V128(n) => *n as i64,
        }
    }

    pub fn as_f32(&self) -> f32 {
        match self {
            WasmValue::I32(n) => *n as f32,
            WasmValue::I64(n) => *n as f32,
            WasmValue::F32(n) => *n,
            WasmValue::F64(n) => *n as f32,
            WasmValue::V128(n) => *n as f32,
        }
    }

    pub fn as_f64(&self) -> f64 {
        match self {
            WasmValue::I32(n) => *n as f64,
            WasmValue::I64(n) => *n as f64,
            WasmValue::F32(n) => *n as f64,
            WasmValue::F64(n) => *n,
            WasmValue::V128(n) => *n as f64,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WasmValueType {
    I32,
    I64,
    F32,
    F64,
    V128,
}

impl WasmValueType {
    pub fn from_byte(b: u8) -> Option<Self> {
        match b {
            0x7F => Some(WasmValueType::I32),
            0x7E => Some(WasmValueType::I64),
            0x7D => Some(WasmValueType::F32),
            0x7C => Some(WasmValueType::F64),
            0x7B => Some(WasmValueType::V128),
            _ => None,
        }
    }

    pub fn size(&self) -> u32 {
        match self {
            WasmValueType::I32 | WasmValueType::F32 => 4,
            WasmValueType::I64 | WasmValueType::F64 => 8,
            WasmValueType::V128 => 16,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_type_from_byte() {
        assert_eq!(WasmValueType::from_byte(0x7F), Some(WasmValueType::I32));
        assert_eq!(WasmValueType::from_byte(0x7E), Some(WasmValueType::I64));
        assert_eq!(WasmValueType::from_byte(0x7D), Some(WasmValueType::F32));
        assert_eq!(WasmValueType::from_byte(0x7C), Some(WasmValueType::F64));
        assert_eq!(WasmValueType::from_byte(0xFF), None);
    }

    #[test]
    fn test_value_type_size() {
        assert_eq!(WasmValueType::I32.size(), 4);
        assert_eq!(WasmValueType::I64.size(), 8);
        assert_eq!(WasmValueType::V128.size(), 16);
    }

    #[test]
    fn test_default_values() {
        assert_eq!(WasmValue::default_for_type(&WasmValueType::I32), WasmValue::I32(0));
        assert_eq!(WasmValue::default_for_type(&WasmValueType::F64), WasmValue::F64(0.0));
    }

    #[test]
    fn test_value_conversions() {
        assert_eq!(WasmValue::I32(42).as_i64(), 42);
        assert_eq!(WasmValue::F64(3.14).as_f64(), 3.14);
    }
}
