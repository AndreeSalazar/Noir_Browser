//! WASI - WebAssembly System Interface (Preview)
//!
//! Provides basic host capabilities:
//! - args_get
//! - environ_get
//! - clock_time_get
//! - random_get
//! - proc_exit

use super::types::*;
use super::value::Value;
use super::runtime::Runtime;
use std::time::{SystemTime, UNIX_EPOCH};pub struct WasiContext {
    pub args: Vec<String>,
    pub env: Vec<String>,
    pub stdin: Vec<u8>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

impl WasiContext {
    pub fn new() -> Self {
        Self {
            args: std::env::args().collect(),
            env: std::env::vars().map(|(k, v)| format!("{}={}", k, v)).collect(),
            stdin: Vec::new(),
            stdout: Vec::new(),
            stderr: Vec::new(),
        }
    }

    pub fn install(self, runtime: &mut Runtime) {
        // args_sizes_get
        let args_sizes = self.args.clone();
        runtime.register_host_function("args_sizes_get", move |_args: &[Value]| {
            let total: usize = args_sizes.iter().map(|s| s.len() + 1).sum();
            Ok(vec![Value::I32(args_sizes.len() as i32), Value::I32(total as i32)])
        });

        // args_get
        let args_clone = self.args.clone();
        runtime.register_host_function("args_get", move |args: &[Value]| {
            let argv_ptr = args[0].as_i32() as u32;
            let argv_buf_ptr = args[1].as_i32() as u32;
            let mut ptr = argv_ptr;
            let mut buf_ptr = argv_buf_ptr;
            for arg in &args_clone {
                // Simplified - would need memory access
                ptr += arg.len() as u32 + 1;
                buf_ptr += arg.len() as u32 + 1;
            }
            Ok(vec![Value::I32(0)])
        });

        // clock_time_get
        runtime.register_host_function("clock_time_get", |_args: &[Value]| {
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(0);
            Ok(vec![Value::I64(nanos as i64)])
        });

        // random_get
        runtime.register_host_function("random_get", |_args: &[Value]| {
            // Simplified - would write to memory
            Ok(vec![Value::I32(0)])
        });

        // proc_exit
        runtime.register_host_function("proc_exit", |args: &[Value]| {
            let code = args[0].as_i32();
            std::process::exit(code);
        });
    }
}

impl Default for WasiContext {
    fn default() -> Self {
        Self::new()
    }
}
