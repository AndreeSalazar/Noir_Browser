//! Compiler - High-level API for compiling and running WASM
use super::types::*;
use super::value::Value;
use super::decoder::{Decoder, Module};
use super::validator::Validator;
use super::jit::{JitCompiler, OptimizationLevel, CompiledModule};
use super::interpreter::Interpreter;
use super::runtime::Runtime;

pub struct Compiler {
    pub optimization_level: OptimizationLevel,
    pub validate: bool,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            optimization_level: OptimizationLevel::Standard,
            validate: true,
        }
    }

    pub fn with_optimization(mut self, level: OptimizationLevel) -> Self {
        self.optimization_level = level;
        self
    }

    pub fn without_validation(mut self) -> Self {
        self.validate = false;
        self
    }

    /// Compile WASM bytes to a module
    pub fn compile(&self, bytes: &[u8]) -> WasmResult<(Module, CompiledModule)> {
        // Decode
        let module = Decoder::new(bytes).decode()?;

        // Validate
        if self.validate {
            let mut validator = Validator::new();
            validator.validate(&module)?;
        }

        // Compile (JIT)
        let opt_level = self.optimization_level.clone();
        let jit = JitCompiler::new(opt_level);
        let compiled = jit.compile(&module);

        Ok((module, compiled))
    }

    /// Compile and instantiate
    pub fn compile_and_instantiate(&self, bytes: &[u8], runtime: &mut Runtime) -> WasmResult<()> {
        let (module, _compiled) = self.compile(bytes)?;
        runtime.instantiate(&module)?;
        Ok(())
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}
