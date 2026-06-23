//! Core types for WASM
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum WasmError {
    InvalidMagic,
    InvalidVersion,
    UnexpectedEof,
    InvalidSection(u8),
    InvalidOpcode(u8),
    InvalidValueType(u8),
    InvalidFunction(u32),
    InvalidLocalIndex(u32),
    InvalidGlobalIndex(u32),
    InvalidMemoryIndex(u32),
    InvalidTypeIndex(u32),
    InvalidTableIndex(u32),
    Validation(String),
    Trap(String),
    OutOfMemory,
    Compile(String),
    Link(String),
    StackOverflow,
}

impl fmt::Display for WasmError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            WasmError::InvalidMagic => write!(f, "Invalid magic number"),
            WasmError::InvalidVersion => write!(f, "Invalid version"),
            WasmError::UnexpectedEof => write!(f, "Unexpected end of file"),
            WasmError::InvalidSection(id) => write!(f, "Invalid section ID: {}", id),
            WasmError::InvalidOpcode(op) => write!(f, "Invalid opcode: 0x{:02x}", op),
            WasmError::InvalidValueType(t) => write!(f, "Invalid value type: 0x{:02x}", t),
            WasmError::InvalidFunction(idx) => write!(f, "Invalid function index: {}", idx),
            WasmError::InvalidLocalIndex(idx) => write!(f, "Invalid local index: {}", idx),
            WasmError::InvalidGlobalIndex(idx) => write!(f, "Invalid global index: {}", idx),
            WasmError::InvalidMemoryIndex(idx) => write!(f, "Invalid memory index: {}", idx),
            WasmError::InvalidTypeIndex(idx) => write!(f, "Invalid type index: {}", idx),
            WasmError::InvalidTableIndex(idx) => write!(f, "Invalid table index: {}", idx),
            WasmError::Validation(msg) => write!(f, "Validation error: {}", msg),
            WasmError::Trap(msg) => write!(f, "Trap: {}", msg),
            WasmError::OutOfMemory => write!(f, "Out of memory"),
            WasmError::Compile(msg) => write!(f, "Compile error: {}", msg),
            WasmError::Link(msg) => write!(f, "Link error: {}", msg),
            WasmError::StackOverflow => write!(f, "Stack overflow"),
        }
    }
}

impl std::error::Error for WasmError {}

pub type WasmResult<T> = Result<T, WasmError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValueType {
    I32,
    I64,
    F32,
    F64,
    V128,    // SIMD
    FuncRef,
    ExternRef,
}

impl ValueType {
    pub fn from_byte(b: u8) -> WasmResult<Self> {
        match b {
            0x7F => Ok(ValueType::I32),
            0x7E => Ok(ValueType::I64),
            0x7D => Ok(ValueType::F32),
            0x7C => Ok(ValueType::F64),
            0x7B => Ok(ValueType::V128),
            0x70 => Ok(ValueType::FuncRef),
            0x6F => Ok(ValueType::ExternRef),
            t => Err(WasmError::InvalidValueType(t)),
        }
    }

    pub fn to_byte(&self) -> u8 {
        match self {
            ValueType::I32 => 0x7F,
            ValueType::I64 => 0x7E,
            ValueType::F32 => 0x7D,
            ValueType::F64 => 0x7C,
            ValueType::V128 => 0x7B,
            ValueType::FuncRef => 0x70,
            ValueType::ExternRef => 0x6F,
        }
    }

    pub fn is_numeric(&self) -> bool {
        matches!(self, ValueType::I32 | ValueType::I64 | ValueType::F32 | ValueType::F64)
    }

    pub fn is_reference(&self) -> bool {
        matches!(self, ValueType::FuncRef | ValueType::ExternRef)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FuncType {
    pub params: Vec<ValueType>,
    pub results: Vec<ValueType>,
}

impl FuncType {
    pub fn new(params: Vec<ValueType>, results: Vec<ValueType>) -> Self {
        Self { params, results }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Limits {
    pub min: u32,
    pub max: Option<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MemoryType {
    pub limits: Limits,
    pub shared: bool, // For threads
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalType {
    pub value_type: ValueType,
    pub mutable: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TableType {
    pub element_type: ValueType,
    pub limits: Limits,
}
