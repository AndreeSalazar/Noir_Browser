use boa_engine::{Context, Source, JsValue};
use std::error::Error;

pub mod challenge;

pub struct JsEngine {
    context: Context,
}

impl JsEngine {
    pub fn new() -> Self {
        let context = Context::default();
        Self { context }
    }

    pub fn run_sandboxed(&mut self, script: &str) -> Result<JsValue, String> {
        let source = Source::from_bytes(script.as_bytes());
        match self.context.eval(source) {
            Ok(value) => Ok(value),
            Err(err) => {
                Err(format!("JS execution error: {}", err))
            }
        }
    }
}
