//! Ejemplos de uso del bridge JS <-> WASM

use crate::bridge::Bridge;
use crate::js_engine_v3::lexer::Lexer;
use crate::js_engine_v3::parser::Parser;
use crate::js_engine_v3::interpreter::Interpreter;
use crate::wasm_engine::{Decoder, Instance};
use std::sync::{Arc, Mutex};

pub fn example_js_calls_wasm() {
    // Setup JS interpreter
    let interp = Arc::new(Mutex::new(Interpreter::new()));
    let bridge = Bridge::new(interp.clone());

    // Load WASM module
    let wasm_bytes = include_bytes!("../../test_data/example.wasm");
    let module = Decoder::new(wasm_bytes).decode().unwrap();
    let instance = Arc::new(Mutex::new(Instance::new(module)));

    bridge.register_wasm("example", instance.clone());

    // JS code that calls WASM
    let js_code = r#"
        let result = bridge.js_call_wasm("example", "add", [2.0, 3.0]);
        print("Result: " + result);
    "#;

    // Execute JS
    let mut lexer = Lexer::new(js_code);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().unwrap();
    interp.lock().unwrap().interpret(&program).unwrap();
}

pub fn example_wasm_calls_js() {
    // WASM module that imports JS function
    // The bridge handles the import automatically
    let wasm_bytes = include_bytes!("../../test_data/calls_js.wasm");
    let module = Decoder::new(wasm_bytes).decode().unwrap();
    let mut instance = Instance::new(module);

    // Setup bridge for imports
    let interp = Arc::new(Mutex::new(Interpreter::new()));
    let bridge = Bridge::new(interp.clone());

    // When WASM calls "env.print", bridge calls JS console.log
    // This is automatic through the import system

    instance.call("main", &[]).unwrap();
}
