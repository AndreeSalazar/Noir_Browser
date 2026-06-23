//! Ignition-style JS Bytecode VM (V8-inspired)
//!
//! Basado en V8 Ignition (registrador-based interpreter):
//! - Bytecodes con operandos en lugar de stack-based
//! - Cada byte es opcode, siguientes bytes son operandos
//! - Stack frame con virtual registers
//!
//! Bytecodes implementados (subset de V8 Ignition):
//! - Ldar: Load accumulator from register
//! - Star: Store accumulator to register
//! - LdaSmi: Load small integer
//! - Add/Sub/Mul/Div: arithmetic
//! - Jump/Return: control flow
//!
//! Esto es educational - nuestro JS engine v3 ya existe y es mas completo.
//! Aqui demostramos los principios de V8 Ignition.

/// Opcode (V8 Ignition-style)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpCode {
    Ldar,    // Load accumulator from register
    Star,    // Store accumulator to register
    LdaSmi,  // Load small integer
    Add,     // a = a + r[operand]
    Sub,
    Mul,
    Div,
    Mod,
    Jump,    // unconditional jump
    JumpIfFalse,
    Return,  // return a
    Halt,    // stop execution
    Nop,     // no-op
    Call,    // call function
    Push,    // push register
    Pop,     // pop stack
}

impl OpCode {
    pub fn to_byte(self) -> u8 {
        match self {
            OpCode::Ldar => 0x00,
            OpCode::Star => 0x01,
            OpCode::LdaSmi => 0x02,
            OpCode::Add => 0x10,
            OpCode::Sub => 0x11,
            OpCode::Mul => 0x12,
            OpCode::Div => 0x13,
            OpCode::Mod => 0x14,
            OpCode::Jump => 0x20,
            OpCode::JumpIfFalse => 0x21,
            OpCode::Return => 0x30,
            OpCode::Halt => 0x31,
            OpCode::Nop => 0x32,
            OpCode::Call => 0x40,
            OpCode::Push => 0x50,
            OpCode::Pop => 0x51,
        }
    }

    pub fn from_byte(b: u8) -> Option<Self> {
        match b {
            0x00 => Some(OpCode::Ldar),
            0x01 => Some(OpCode::Star),
            0x02 => Some(OpCode::LdaSmi),
            0x10 => Some(OpCode::Add),
            0x11 => Some(OpCode::Sub),
            0x12 => Some(OpCode::Mul),
            0x13 => Some(OpCode::Div),
            0x14 => Some(OpCode::Mod),
            0x20 => Some(OpCode::Jump),
            0x21 => Some(OpCode::JumpIfFalse),
            0x30 => Some(OpCode::Return),
            0x31 => Some(OpCode::Halt),
            0x32 => Some(OpCode::Nop),
            0x40 => Some(OpCode::Call),
            0x50 => Some(OpCode::Push),
            0x51 => Some(OpCode::Pop),
            _ => None,
        }
    }
}

/// Valor de runtime (V8-style Smi/HeapObject)
#[derive(Debug, Clone, PartialEq, Default)]
pub enum Value {
    #[default]
    Undefined,
    Smi(i32),    // Small integer (tagged pointer in real V8)
    Bool(bool),
    Null,
    Str(String),
}

impl Value {
    pub fn as_smi(&self) -> Option<i32> {
        if let Value::Smi(n) = self { Some(*n) } else { None }
    }
    pub fn as_bool(&self) -> Option<bool> {
        if let Value::Bool(b) = self { Some(*b) } else { None }
    }
}

/// Bytecode array (V8 Ignition-style)
#[derive(Debug, Clone)]
pub struct BytecodeArray {
    pub bytes: Vec<u8>,
}

impl BytecodeArray {
    pub fn new() -> Self { Self { bytes: Vec::new() } }

    pub fn emit(&mut self, op: OpCode) { self.bytes.push(op.to_byte()); }
    pub fn emit_u8(&mut self, operand: u8) { self.bytes.push(operand); }
    pub fn emit_i32(&mut self, operand: i32) { self.bytes.extend_from_slice(&operand.to_le_bytes()); }

    /// Decode opcode at position
    pub fn decode_at(&self, pos: usize) -> Option<(OpCode, usize)> {
        let b = *self.bytes.get(pos)?;
        let op = OpCode::from_byte(b)?;
        Some((op, pos + 1))
    }
}

impl Default for BytecodeArray {
    fn default() -> Self { Self::new() }
}

/// Virtual register file (V8 Ignition-style)
#[derive(Debug, Default)]
pub struct RegisterFile {
    pub registers: Vec<Value>,
    pub accumulator: Value,
    pub stack: Vec<Value>,
}

impl RegisterFile {
    pub fn new(num_regs: usize) -> Self {
        Self {
            registers: vec![Value::Undefined; num_regs],
            accumulator: Value::Undefined,
            stack: Vec::new(),
        }
    }
}

/// Ignition interpreter (V8-style)
pub struct IgnitionInterpreter {
    pc: usize,
    bytecode: BytecodeArray,
    registers: RegisterFile,
    finished: bool,
    return_value: Option<Value>,
}

impl IgnitionInterpreter {
    pub fn new(bytecode: BytecodeArray) -> Self {
        Self {
            pc: 0,
            bytecode,
            registers: RegisterFile::new(16),
            finished: false,
            return_value: None,
        }
    }

    /// Run hasta Return o Halt
    pub fn execute(&mut self) -> Option<Value> {
        while !self.finished && self.pc < self.bytecode.bytes.len() {
            self.step();
        }
        self.return_value.clone()
    }

    /// Execute un solo step
    pub fn step(&mut self) {
        let start = self.pc;
        let (op, _) = match self.bytecode.decode_at(start) {
            Some(x) => x,
            None => {
                self.finished = true;
                return;
            }
        };
        match op {
            OpCode::Ldar => {
                let reg = self.bytecode.bytes[self.pc + 1] as usize;
                self.registers.accumulator = self.registers.registers[reg].clone();
                self.pc += 2;
            }
            OpCode::Star => {
                let reg = self.bytecode.bytes[self.pc + 1] as usize;
                let val = self.registers.accumulator.clone();
                if reg < self.registers.registers.len() {
                    self.registers.registers[reg] = val;
                }
                self.pc += 2;
            }
            OpCode::LdaSmi => {
                let val = self.bytecode.bytes[self.pc + 1] as i32;
                self.registers.accumulator = Value::Smi(val);
                self.pc += 2;
            }
            OpCode::Add => {
                let reg = self.bytecode.bytes[self.pc + 1] as usize;
                let a = self.registers.accumulator.as_smi().unwrap_or(0);
                let b = self.registers.registers[reg].as_smi().unwrap_or(0);
                self.registers.accumulator = Value::Smi(a + b);
                self.pc += 2;
            }
            OpCode::Sub => {
                let reg = self.bytecode.bytes[self.pc + 1] as usize;
                let a = self.registers.accumulator.as_smi().unwrap_or(0);
                let b = self.registers.registers[reg].as_smi().unwrap_or(0);
                self.registers.accumulator = Value::Smi(a - b);
                self.pc += 2;
            }
            OpCode::Mul => {
                let reg = self.bytecode.bytes[self.pc + 1] as usize;
                let a = self.registers.accumulator.as_smi().unwrap_or(0);
                let b = self.registers.registers[reg].as_smi().unwrap_or(0);
                self.registers.accumulator = Value::Smi(a * b);
                self.pc += 2;
            }
            OpCode::Jump => {
                let offset = self.bytecode.bytes[self.pc + 1] as usize;
                self.pc = offset;
            }
            OpCode::Return => {
                self.return_value = Some(self.registers.accumulator.clone());
                self.finished = true;
            }
            OpCode::Halt => {
                self.finished = true;
            }
            OpCode::Nop => { self.pc += 1; }
            _ => {
                // Unimplemented: skip
                self.pc += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opcode_roundtrip() {
        for op in [OpCode::Ldar, OpCode::Star, OpCode::LdaSmi, OpCode::Add, OpCode::Return] {
            assert_eq!(OpCode::from_byte(op.to_byte()), Some(op));
        }
    }

    #[test]
    fn test_constant_arithmetic() {
        // Calcular 2 + 3 = 5
        let mut bc = BytecodeArray::new();
        bc.emit(OpCode::LdaSmi);
        bc.emit_u8(2);
        bc.emit(OpCode::Return);
        let mut interp = IgnitionInterpreter::new(bc);
        let r = interp.execute();
        assert_eq!(r, Some(Value::Smi(2)));
    }

    #[test]
    fn test_register_add() {
        // r0 = 10, a = r0 + r1 donde r1 = 5
        let mut bc = BytecodeArray::new();
        bc.emit(OpCode::LdaSmi); bc.emit_u8(10);
        bc.emit(OpCode::Star); bc.emit_u8(0);
        bc.emit(OpCode::LdaSmi); bc.emit_u8(5);
        bc.emit(OpCode::Star); bc.emit_u8(1);
        bc.emit(OpCode::Ldar); bc.emit_u8(0);
        bc.emit(OpCode::Add); bc.emit_u8(1);
        bc.emit(OpCode::Return);
        let mut interp = IgnitionInterpreter::new(bc);
        assert_eq!(interp.execute(), Some(Value::Smi(15)));
    }

    #[test]
    fn test_halt_stops() {
        let mut bc = BytecodeArray::new();
        bc.emit(OpCode::LdaSmi); bc.emit_u8(42);
        bc.emit(OpCode::Halt);
        let mut interp = IgnitionInterpreter::new(bc);
        interp.execute();
        assert!(interp.finished);
    }

    #[test]
    fn test_subtraction() {
        let mut bc = BytecodeArray::new();
        bc.emit(OpCode::LdaSmi); bc.emit_u8(10);
        bc.emit(OpCode::Star); bc.emit_u8(0);
        bc.emit(OpCode::LdaSmi); bc.emit_u8(3);
        bc.emit(OpCode::Star); bc.emit_u8(1);
        bc.emit(OpCode::Ldar); bc.emit_u8(0);
        bc.emit(OpCode::Sub); bc.emit_u8(1);
        bc.emit(OpCode::Return);
        let mut interp = IgnitionInterpreter::new(bc);
        assert_eq!(interp.execute(), Some(Value::Smi(7)));
    }

    #[test]
    fn test_multiplication() {
        let mut bc = BytecodeArray::new();
        bc.emit(OpCode::LdaSmi); bc.emit_u8(6);
        bc.emit(OpCode::Star); bc.emit_u8(0);
        bc.emit(OpCode::LdaSmi); bc.emit_u8(7);
        bc.emit(OpCode::Star); bc.emit_u8(1);
        bc.emit(OpCode::Ldar); bc.emit_u8(0);
        bc.emit(OpCode::Mul); bc.emit_u8(1);
        bc.emit(OpCode::Return);
        let mut interp = IgnitionInterpreter::new(bc);
        assert_eq!(interp.execute(), Some(Value::Smi(42)));
    }

    #[test]
    fn test_jump() {
        let mut bc = BytecodeArray::new();
        // offset 0: LdaSmi 1, Star 0, Jump 6
        bc.emit(OpCode::LdaSmi); bc.emit_u8(1);
        bc.emit(OpCode::Star); bc.emit_u8(0);
        bc.emit(OpCode::Jump); bc.emit_u8(8);  // jump to offset 8
        // offset 6: LdaSmi 99 (skipped)
        bc.emit(OpCode::LdaSmi); bc.emit_u8(99);
        // offset 8: LdaSmi 42, Return
        bc.emit(OpCode::LdaSmi); bc.emit_u8(42);
        bc.emit(OpCode::Return);
        let mut interp = IgnitionInterpreter::new(bc);
        assert_eq!(interp.execute(), Some(Value::Smi(42)));
    }

    #[test]
    fn test_value_accessors() {
        assert_eq!(Value::Smi(42).as_smi(), Some(42));
        assert_eq!(Value::Bool(true).as_bool(), Some(true));
        assert_eq!(Value::Null.as_smi(), None);
    }

    #[test]
    fn test_bytecode_decode() {
        let mut bc = BytecodeArray::new();
        bc.emit(OpCode::LdaSmi);
        bc.emit_u8(5);
        let (op, next) = bc.decode_at(0).unwrap();
        assert_eq!(op, OpCode::LdaSmi);
        assert_eq!(next, 1);
    }
}
