#[cfg(test)]
mod debug_tests {
    use crate::wasm_host::module::read_leb128_i32;

    #[test]
    fn test_read_leb128_42() {
        let bytes = [0x2A];
        let (val, pos) = read_leb128_i32(&bytes).unwrap();
        assert_eq!(val, 42, "Expected 42, got {}", val);
        assert_eq!(pos, 1, "Expected pos=1, got {}", pos);
    }
}
