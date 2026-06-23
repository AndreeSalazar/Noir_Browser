//! LEB128 decoder/encoder
use super::types::WasmResult;
use super::types::WasmError;

pub struct Reader<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, pos: 0 }
    }

    pub fn pos(&self) -> usize { self.pos }
    pub fn remaining(&self) -> usize { self.bytes.len().saturating_sub(self.pos) }

    pub fn read_u8(&mut self) -> WasmResult<u8> {
        if self.remaining() < 1 {
            return Err(WasmError::UnexpectedEof);
        }
        let b = self.bytes[self.pos];
        self.pos += 1;
        Ok(b)
    }

    pub fn read_bytes(&mut self, n: usize) -> WasmResult<&'a [u8]> {
        if self.remaining() < n {
            return Err(WasmError::UnexpectedEof);
        }
        let bytes = &self.bytes[self.pos..self.pos + n];
        self.pos += n;
        Ok(bytes)
    }

    pub fn read_name(&mut self) -> WasmResult<String> {
        let len = self.read_u32()? as usize;
        let bytes = self.read_bytes(len)?;
        String::from_utf8(bytes.to_vec()).map_err(|_| WasmError::Validation("Invalid UTF-8".to_string()))
    }

    pub fn read_u32(&mut self) -> WasmResult<u32> {
        self.read_leb128_u32()
    }

    pub fn read_u64(&mut self) -> WasmResult<u64> {
        self.read_leb128_u64()
    }

    pub fn read_i32(&mut self) -> WasmResult<i32> {
        self.read_leb128_i32()
    }

    pub fn read_i64(&mut self) -> WasmResult<i64> {
        self.read_leb128_i64()
    }

    pub fn read_leb128_u32(&mut self) -> WasmResult<u32> {
        let mut result: u32 = 0;
        let mut shift = 0;
        loop {
            let byte = self.read_u8()?;
            result |= ((byte & 0x7F) as u32) << shift;
            if byte & 0x80 == 0 {
                return Ok(result);
            }
            shift += 7;
            if shift >= 35 {
                return Err(WasmError::Validation("LEB128 overflow".to_string()));
            }
        }
    }

    pub fn read_leb128_u64(&mut self) -> WasmResult<u64> {
        let mut result: u64 = 0;
        let mut shift = 0;
        loop {
            let byte = self.read_u8()?;
            result |= ((byte & 0x7F) as u64) << shift;
            if byte & 0x80 == 0 {
                return Ok(result);
            }
            shift += 7;
            if shift >= 70 {
                return Err(WasmError::Validation("LEB128 overflow".to_string()));
            }
        }
    }

    pub fn read_leb128_i32(&mut self) -> WasmResult<i32> {
        let mut result: i32 = 0;
        let mut shift = 0;
        loop {
            let byte = self.read_u8()?;
            result |= ((byte & 0x7F) as i32) << shift;
            shift += 7;
            if byte & 0x80 == 0 {
                if shift < 32 && (byte & 0x40) != 0 {
                    result |= -(1 << shift);
                }
                return Ok(result);
            }
        }
    }

    pub fn read_leb128_i64(&mut self) -> WasmResult<i64> {
        let mut result: i64 = 0;
        let mut shift = 0;
        loop {
            let byte = self.read_u8()?;
            result |= ((byte & 0x7F) as i64) << shift;
            shift += 7;
            if byte & 0x80 == 0 {
                if shift < 64 && (byte & 0x40) != 0 {
                    result |= -(1 << shift);
                }
                return Ok(result);
            }
        }
    }

    /// Read a vector of items
    pub fn read_vector<T, F>(&mut self, f: F) -> WasmResult<Vec<T>>
    where F: Fn(&mut Self) -> WasmResult<T>
    {
        let len = self.read_leb128_u32()? as usize;
        let mut vec = Vec::with_capacity(len);
        for _ in 0..len {
            vec.push(f(self)?);
        }
        Ok(vec)
    }
}

/// LEB128 encoder
pub struct Writer {
    bytes: Vec<u8>,
}

impl Writer {
    pub fn new() -> Self { Self { bytes: Vec::new() } }
    pub fn into_bytes(self) -> Vec<u8> { self.bytes }

    pub fn write_u8(&mut self, byte: u8) { self.bytes.push(byte); }
    pub fn write_bytes(&mut self, bytes: &[u8]) { self.bytes.extend_from_slice(bytes); }

    pub fn write_u32(&mut self, value: u32) { self.write_leb128_u32(value); }
    pub fn write_i32(&mut self, value: i32) { self.write_leb128_i32(value); }

    pub fn write_leb128_u32(&mut self, mut value: u32) {
        loop {
            let byte = (value & 0x7F) as u8;
            value >>= 7;
            if value == 0 {
                self.bytes.push(byte);
                break;
            } else {
                self.bytes.push(byte | 0x80);
            }
        }
    }

    pub fn write_leb128_i32(&mut self, value: i32) {
        let mut value = value as u32;
        let more = true;
        while more {
            let byte = (value & 0x7F) as u8;
            value >>= 7;
            let sign_bit = byte & 0x40;
            if (value == 0 && sign_bit == 0) || (value == 0xFFFFFFFF && sign_bit != 0) {
                self.bytes.push(byte);
                break;
            } else {
                self.bytes.push(byte | 0x80);
            }
        }
    }
}
