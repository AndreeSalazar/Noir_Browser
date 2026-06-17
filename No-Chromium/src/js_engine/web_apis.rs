use boa_engine::{Context, JsArgs, JsNativeError, JsResult, JsValue, NativeFunction};
use std::sync::{Arc, Mutex, OnceLock};

pub struct WebApis {
    console_logs: Arc<Mutex<Vec<ConsoleMessage>>>,
}

#[derive(Debug, Clone)]
pub struct ConsoleMessage {
    pub level: ConsoleLevel,
    pub message: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConsoleLevel {
    Log,
    Warn,
    Error,
    Info,
    Debug,
}

static CONSOLE_LOGS: OnceLock<Arc<Mutex<Vec<ConsoleMessage>>>> = OnceLock::new();

fn get_logs() -> &'static Arc<Mutex<Vec<ConsoleMessage>>> {
    CONSOLE_LOGS.get_or_init(|| Arc::new(Mutex::new(Vec::new())))
}

fn js_console_log(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let msg = args_to_string(args, ctx);
    get_logs().lock().unwrap().push(ConsoleMessage {
        level: ConsoleLevel::Log,
        message: msg.clone(),
        timestamp: timestamp_ms(),
    });
    tracing::info!("[JS] {}", msg);
    Ok(JsValue::undefined())
}

fn js_console_warn(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let msg = args_to_string(args, ctx);
    get_logs().lock().unwrap().push(ConsoleMessage {
        level: ConsoleLevel::Warn,
        message: msg.clone(),
        timestamp: timestamp_ms(),
    });
    tracing::warn!("[JS] {}", msg);
    Ok(JsValue::undefined())
}

fn js_console_error(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let msg = args_to_string(args, ctx);
    get_logs().lock().unwrap().push(ConsoleMessage {
        level: ConsoleLevel::Error,
        message: msg.clone(),
        timestamp: timestamp_ms(),
    });
    tracing::error!("[JS] {}", msg);
    Ok(JsValue::undefined())
}

fn js_console_info(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let msg = args_to_string(args, ctx);
    get_logs().lock().unwrap().push(ConsoleMessage {
        level: ConsoleLevel::Info,
        message: msg.clone(),
        timestamp: timestamp_ms(),
    });
    tracing::info!("[JS] {}", msg);
    Ok(JsValue::undefined())
}

fn js_console_clear(_this: &JsValue, _args: &[JsValue], _ctx: &mut Context) -> JsResult<JsValue> {
    get_logs().lock().unwrap().clear();
    Ok(JsValue::undefined())
}

fn js_json_parse(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let json_str = args.get_or_undefined(0)
        .to_string(ctx)
        .map(|s| s.to_std_string_escaped())
        .map_err(|e| JsNativeError::typ().with_message(format!("JSON.parse: {}", e)))?;

    match serde_json::from_str::<serde_json::Value>(&json_str) {
        Ok(val) => json_value_to_js(&val, ctx),
        Err(e) => Err(JsNativeError::syntax().with_message(format!("JSON.parse error: {}", e)).into()),
    }
}

fn js_json_stringify(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let val = args.get_or_undefined(0);
    let json_str = val.to_json(ctx)?;
    Ok(JsValue::from(boa_engine::JsString::from(json_str.to_string())))
}

impl WebApis {
    pub fn new() -> Self {
        let logs = CONSOLE_LOGS.get_or_init(|| Arc::new(Mutex::new(Vec::new()))).clone();
        Self { console_logs: logs }
    }

    pub fn register_all(&self, context: &mut Context) {
        self.register_console(context);
        self.register_json(context);
    }

    fn register_console(&self, context: &mut Context) {
        let console_obj = boa_engine::JsObject::with_null_proto();

        let log_fn = NativeFunction::from_fn_ptr(js_console_log).to_js_function(context.realm());
        let _ = console_obj.set(boa_engine::js_string!("log"), log_fn, false, context);

        let warn_fn = NativeFunction::from_fn_ptr(js_console_warn).to_js_function(context.realm());
        let _ = console_obj.set(boa_engine::js_string!("warn"), warn_fn, false, context);

        let error_fn = NativeFunction::from_fn_ptr(js_console_error).to_js_function(context.realm());
        let _ = console_obj.set(boa_engine::js_string!("error"), error_fn, false, context);

        let info_fn = NativeFunction::from_fn_ptr(js_console_info).to_js_function(context.realm());
        let _ = console_obj.set(boa_engine::js_string!("info"), info_fn, false, context);

        let clear_fn = NativeFunction::from_fn_ptr(js_console_clear).to_js_function(context.realm());
        let _ = console_obj.set(boa_engine::js_string!("clear"), clear_fn, false, context);

        let _ = context.register_global_property(boa_engine::js_string!("console"), boa_engine::JsValue::Object(console_obj), boa_engine::property::Attribute::all());
    }

    fn register_json(&self, context: &mut Context) {
        let json_obj = boa_engine::JsObject::with_null_proto();

        let parse_fn = NativeFunction::from_fn_ptr(js_json_parse).to_js_function(context.realm());
        let _ = json_obj.set(boa_engine::js_string!("parse"), parse_fn, false, context);

        let stringify_fn = NativeFunction::from_fn_ptr(js_json_stringify).to_js_function(context.realm());
        let _ = json_obj.set(boa_engine::js_string!("stringify"), stringify_fn, false, context);

        let _ = context.register_global_property(boa_engine::js_string!("JSON"), boa_engine::JsValue::Object(json_obj), boa_engine::property::Attribute::all());
    }

    pub fn get_console_logs(&self) -> Vec<ConsoleMessage> {
        self.console_logs.lock().unwrap().clone()
    }

    pub fn clear_console_logs(&self) {
        self.console_logs.lock().unwrap().clear();
    }
}

fn args_to_string(args: &[JsValue], ctx: &mut Context) -> String {
    args.iter()
        .map(|a| a.to_string(ctx).map(|s| s.to_std_string_escaped()).unwrap_or_else(|_| "undefined".to_string()))
        .collect::<Vec<_>>()
        .join(" ")
}

fn timestamp_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn json_value_to_js(val: &serde_json::Value, ctx: &mut Context) -> JsResult<JsValue> {
    match val {
        serde_json::Value::Null => Ok(JsValue::Null),
        serde_json::Value::Bool(b) => Ok(JsValue::from(*b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(JsValue::from(i))
            } else if let Some(f) = n.as_f64() {
                Ok(JsValue::from(f))
            } else {
                Ok(JsValue::from(0.0))
            }
        }
        serde_json::Value::String(s) => Ok(JsValue::from(boa_engine::JsString::from(s.as_str()))),
        serde_json::Value::Array(arr) => {
            let js_arr = boa_engine::JsObject::with_null_proto();
            for (i, item) in arr.iter().enumerate() {
                let js_val = json_value_to_js(item, ctx)?;
                let _ = js_arr.set(i as u32, js_val, false, ctx);
            }
            let _ = js_arr.set(boa_engine::js_string!("length"), JsValue::from(arr.len() as u32), false, ctx);
            Ok(JsValue::Object(js_arr))
        }
        serde_json::Value::Object(map) => {
            let js_obj = boa_engine::JsObject::with_null_proto();
            for (key, val) in map {
                let js_val = json_value_to_js(val, ctx)?;
                let _ = js_obj.set(boa_engine::js_string!(key.as_str()), js_val, false, ctx);
            }
            Ok(JsValue::Object(js_obj))
        }
    }
}
