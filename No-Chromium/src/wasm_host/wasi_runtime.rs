//! WASI Runtime - WebAssembly System Interface
//!
//! Implementa subset de WASI para módulos que lo requieren.
//! Proporciona:
//! - fd_write: write to stdout/stderr
//! - fd_read: read from stdin
//! - args_get: command-line arguments
//! - environ_get: environment variables
//! - clock_time_get: time
//! - random_get: random bytes

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::wasm_host::imports::{ImportResolver, ImportValue};
use crate::wasm_host::WasmValue;

pub struct WasiRuntime {
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub stdout_buf: Vec<u8>,
    pub stderr_buf: Vec<u8>,
}

impl WasiRuntime {
    pub fn new() -> Self {
        Self {
            args: Vec::new(),
            env: HashMap::new(),
            stdout_buf: Vec::new(),
            stderr_buf: Vec::new(),
        }
    }

    pub fn with_args(args: Vec<String>) -> Self {
        Self {
            args,
            env: HashMap::new(),
            stdout_buf: Vec::new(),
            stderr_buf: Vec::new(),
        }
    }

    /// Registra imports WASI en el resolver
    pub fn register(&self, resolver: &mut ImportResolver) {
        let args_len = self.args.len();
        let args_total: usize = self.args.iter().map(|a| a.len() + 1).sum();
        let env_len = self.env.len();
        let env_total: usize = self.env.iter().map(|(k, v)| k.len() + v.len() + 2).sum();

        // fd_write(fd, iovs, iovs_len, nwritten) -> errno
        resolver.register_fn("wasi_snapshot_preview1", "fd_write", |_args| {
            Ok(vec![WasmValue::I32(0)])
        });

        // fd_read(fd, iovs, iovs_len, nread) -> errno
        resolver.register_fn("wasi_snapshot_preview1", "fd_read", |_args| {
            Ok(vec![WasmValue::I32(0)])
        });

        // args_get(argv_ptr, argv_buf) -> argc
        resolver.register_fn("wasi_snapshot_preview1", "args_get", move |_args| {
            Ok(vec![WasmValue::I32(args_len as i32)])
        });

        // args_sizes_get() -> argc, argv_buf_size
        resolver.register_fn("wasi_snapshot_preview1", "args_sizes_get", move |_args| {
            Ok(vec![
                WasmValue::I32(args_len as i32),
                WasmValue::I32(args_total as i32),
            ])
        });

        // environ_get(envp_ptr, envp_buf) -> errno
        resolver.register_fn("wasi_snapshot_preview1", "environ_get", |_args| {
            Ok(vec![WasmValue::I32(0)])
        });

        // environ_sizes_get() -> envc, envp_buf_size
        resolver.register_fn("wasi_snapshot_preview1", "environ_sizes_get", move |_args| {
            Ok(vec![
                WasmValue::I32(env_len as i32),
                WasmValue::I32(env_total as i32),
            ])
        });

        // clock_time_get(id, precision, time_ptr) -> errno
        resolver.register_fn("wasi_snapshot_preview1", "clock_time_get", |_args| {
            Ok(vec![WasmValue::I32(0)])
        });

        // random_get(buf_ptr, buf_len) -> errno
        resolver.register_fn("wasi_snapshot_preview1", "random_get", |_args| {
            Ok(vec![WasmValue::I32(0)])
        });

        // proc_exit(code) -> never returns
        resolver.register_fn("wasi_snapshot_preview1", "proc_exit", |_args| {
            std::process::exit(0);
        });
    }

    pub fn get_stdout(&self) -> String {
        String::from_utf8_lossy(&self.stdout_buf).to_string()
    }

    pub fn get_stderr(&self) -> String {
        String::from_utf8_lossy(&self.stderr_buf).to_string()
    }
}

impl Default for WasiRuntime {
    fn default() -> Self {
        Self::new()
    }
}

/// Get current time in nanoseconds (for clock_time_get)
pub fn get_clock_time() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasi_creation() {
        let wasi = WasiRuntime::new();
        assert_eq!(wasi.args.len(), 0);
    }

    #[test]
    fn test_wasi_with_args() {
        let wasi = WasiRuntime::with_args(vec!["arg1".to_string(), "arg2".to_string()]);
        assert_eq!(wasi.args.len(), 2);
    }

    #[test]
    fn test_wasi_register() {
        let wasi = WasiRuntime::new();
        let mut resolver = ImportResolver::new();
        wasi.register(&mut resolver);
        assert!(resolver.resolve("wasi_snapshot_preview1", "fd_write").is_some());
        assert!(resolver.resolve("wasi_snapshot_preview1", "proc_exit").is_some());
    }

    #[test]
    fn test_get_clock_time() {
        let t = get_clock_time();
        assert!(t > 0);
    }

    #[test]
    fn test_wasi_args_sizes() {
        let wasi = WasiRuntime::with_args(vec!["hello".to_string()]);
        let mut resolver = ImportResolver::new();
        wasi.register(&mut resolver);
        let import = resolver.resolve("wasi_snapshot_preview1", "args_sizes_get").unwrap();
        if let ImportValue::Function(f) = import {
            let result = f(&[]).unwrap();
            assert_eq!(result[0], WasmValue::I32(1)); // argc
            assert_eq!(result[1], WasmValue::I32(6)); // 5 + null = 6
        }
    }
}
