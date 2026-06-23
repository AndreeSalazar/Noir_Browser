//! Built-in objects: Math, JSON, console, document, window
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use super::value::JsValue;
use super::env::Env;
use super::dom::Dom;
use super::console::Console;
use super::console::{console_log, console_warn, console_error, console_info};

pub fn register_builtins(env: &Rc<RefCell<Env>>, dom: &Rc<RefCell<Dom>>, _console: &Rc<RefCell<Console>>) {
    let mut global = env.borrow_mut();

    // console
    let mut console_obj = HashMap::new();
    console_obj.insert("log".to_string(), JsValue::NativeFunction { name: "log".to_string(), func: console_log });
    console_obj.insert("info".to_string(), JsValue::NativeFunction { name: "info".to_string(), func: console_info });
    console_obj.insert("warn".to_string(), JsValue::NativeFunction { name: "warn".to_string(), func: console_warn });
    console_obj.insert("error".to_string(), JsValue::NativeFunction { name: "error".to_string(), func: console_error });
    global.set("console".to_string(), JsValue::Object(Rc::new(RefCell::new(console_obj))));

    // Math
    let mut math_obj = HashMap::new();
    math_obj.insert("PI".to_string(), JsValue::Number(std::f64::consts::PI));
    math_obj.insert("E".to_string(), JsValue::Number(std::f64::consts::E));
    math_obj.insert("sqrt".to_string(), JsValue::NativeFunction { name: "sqrt".to_string(), func: |args: &[JsValue]| {
        if args.is_empty() { return Ok(JsValue::Number(f64::NAN)); }
        Ok(JsValue::Number(args[0].to_number().sqrt()))
    }});
    math_obj.insert("pow".to_string(), JsValue::NativeFunction { name: "pow".to_string(), func: |args: &[JsValue]| {
        if args.len() < 2 { return Ok(JsValue::Number(f64::NAN)); }
        Ok(JsValue::Number(args[0].to_number().powf(args[1].to_number())))
    }});
    math_obj.insert("abs".to_string(), JsValue::NativeFunction { name: "abs".to_string(), func: |args: &[JsValue]| {
        if args.is_empty() { return Ok(JsValue::Number(f64::NAN)); }
        Ok(JsValue::Number(args[0].to_number().abs()))
    }});
    math_obj.insert("floor".to_string(), JsValue::NativeFunction { name: "floor".to_string(), func: |args: &[JsValue]| {
        if args.is_empty() { return Ok(JsValue::Number(f64::NAN)); }
        Ok(JsValue::Number(args[0].to_number().floor()))
    }});
    math_obj.insert("ceil".to_string(), JsValue::NativeFunction { name: "ceil".to_string(), func: |args: &[JsValue]| {
        if args.is_empty() { return Ok(JsValue::Number(f64::NAN)); }
        Ok(JsValue::Number(args[0].to_number().ceil()))
    }});
    math_obj.insert("round".to_string(), JsValue::NativeFunction { name: "round".to_string(), func: |args: &[JsValue]| {
        if args.is_empty() { return Ok(JsValue::Number(f64::NAN)); }
        Ok(JsValue::Number(args[0].to_number().round()))
    }});
    math_obj.insert("max".to_string(), JsValue::NativeFunction { name: "max".to_string(), func: |args: &[JsValue]| {
        if args.is_empty() { return Ok(JsValue::Number(f64::NEG_INFINITY)); }
        Ok(JsValue::Number(args.iter().map(|a| a.to_number()).fold(f64::NEG_INFINITY, f64::max)))
    }});
    math_obj.insert("min".to_string(), JsValue::NativeFunction { name: "min".to_string(), func: |args: &[JsValue]| {
        if args.is_empty() { return Ok(JsValue::Number(f64::INFINITY)); }
        Ok(JsValue::Number(args.iter().map(|a| a.to_number()).fold(f64::INFINITY, f64::min)))
    }});
    math_obj.insert("random".to_string(), JsValue::NativeFunction { name: "random".to_string(), func: |_args: &[JsValue]| {
        use std::time::SystemTime;
        let nanos = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos();
        Ok(JsValue::Number((nanos % 1000) as f64 / 1000.0))
    }});
    global.set("Math".to_string(), JsValue::Object(Rc::new(RefCell::new(math_obj))));

    // JSON
    let json_obj: HashMap<String, JsValue> = vec![
        ("stringify", JsValue::NativeFunction { name: "stringify".to_string(), func: |args: &[JsValue]| {
            if args.is_empty() { return Ok(JsValue::String("undefined".to_string())); }
            Ok(JsValue::String(args[0].to_string()))
        }}),
        ("parse", JsValue::NativeFunction { name: "parse".to_string(), func: |args: &[JsValue]| {
            // Simplified - no real JSON parsing
            if args.is_empty() { return Ok(JsValue::Null); }
            Ok(JsValue::String(args[0].to_string()))
        }}),
    ].into_iter().map(|(k, v)| (k.to_string(), v)).collect();
    global.set("JSON".to_string(), JsValue::Object(Rc::new(RefCell::new(json_obj))));

    // document
    let mut doc_obj = HashMap::new();
    doc_obj.insert("title".to_string(), JsValue::String("Noir Browser".to_string()));
    doc_obj.insert("body".to_string(), JsValue::Object(Rc::new(RefCell::new(HashMap::new()))));
    doc_obj.insert("getElementById".to_string(), JsValue::NativeFunction { name: "getElementById".to_string(), func: |args: &[JsValue]| {
        // Would access dom
        if args.is_empty() { return Ok(JsValue::Null); }
        Ok(JsValue::Null)
    }});
    doc_obj.insert("querySelector".to_string(), JsValue::NativeFunction { name: "querySelector".to_string(), func: |_args: &[JsValue]| {
        Ok(JsValue::Null)
    }});
    doc_obj.insert("querySelectorAll".to_string(), JsValue::NativeFunction { name: "querySelectorAll".to_string(), func: |_args: &[JsValue]| {
        Ok(JsValue::Array(Rc::new(RefCell::new(Vec::new()))))
    }});
    doc_obj.insert("createElement".to_string(), JsValue::NativeFunction { name: "createElement".to_string(), func: |args: &[JsValue]| {
        if args.is_empty() { return Ok(JsValue::Null); }
        Ok(Dom::create_element(&args[0].to_string()))
    }});
    global.set("document".to_string(), JsValue::Object(Rc::new(RefCell::new(doc_obj))));

    // window
    let mut win_obj = HashMap::new();
    win_obj.insert("innerWidth".to_string(), JsValue::Number(1280.0));
    win_obj.insert("innerHeight".to_string(), JsValue::Number(720.0));
    win_obj.insert("location".to_string(), JsValue::Object(Rc::new(RefCell::new({
        let mut loc = HashMap::new();
        loc.insert("href".to_string(), JsValue::String(String::new()));
        loc
    }))));
    win_obj.insert("alert".to_string(), JsValue::NativeFunction { name: "alert".to_string(), func: |args: &[JsValue]| {
        let msg = args.iter().map(|a| a.to_string()).collect::<Vec<_>>().join(" ");
        tracing::info!("[alert] {}", msg);
        Ok(JsValue::Undefined)
    }});
    win_obj.insert("setTimeout".to_string(), JsValue::NativeFunction { name: "setTimeout".to_string(), func: |_args: &[JsValue]| {
        Ok(JsValue::Number(0.0))
    }});
    win_obj.insert("setInterval".to_string(), JsValue::NativeFunction { name: "setInterval".to_string(), func: |_args: &[JsValue]| {
        Ok(JsValue::Number(0.0))
    }});
    global.set("window".to_string(), JsValue::Object(Rc::new(RefCell::new(win_obj))));

    // parseInt, parseFloat
    global.set("parseInt".to_string(), JsValue::NativeFunction { name: "parseInt".to_string(), func: |args: &[JsValue]| {
        if args.is_empty() { return Ok(JsValue::Number(f64::NAN)); }
        let s = args[0].to_string();
        let radix = if args.len() > 1 { args[1].to_number() as u32 } else { 10 };
        Ok(JsValue::Number(s.parse::<i64>().map(|n| n as f64).unwrap_or(f64::NAN)))
    }});
    global.set("parseFloat".to_string(), JsValue::NativeFunction { name: "parseFloat".to_string(), func: |args: &[JsValue]| {
        if args.is_empty() { return Ok(JsValue::Number(f64::NAN)); }
        Ok(JsValue::Number(args[0].to_string().parse::<f64>().unwrap_or(f64::NAN)))
    }});

    // isNaN, isFinite
    global.set("isNaN".to_string(), JsValue::NativeFunction { name: "isNaN".to_string(), func: |args: &[JsValue]| {
        if args.is_empty() { return Ok(JsValue::Boolean(false)); }
        Ok(JsValue::Boolean(args[0].to_number().is_nan()))
    }});
    global.set("isFinite".to_string(), JsValue::NativeFunction { name: "isFinite".to_string(), func: |args: &[JsValue]| {
        if args.is_empty() { return Ok(JsValue::Boolean(false)); }
        Ok(JsValue::Boolean(args[0].to_number().is_finite()))
    }});

    // Global NaN, Infinity, undefined
    global.set("NaN".to_string(), JsValue::Number(f64::NAN));
    global.set("Infinity".to_string(), JsValue::Number(f64::INFINITY));
    global.set("undefined".to_string(), JsValue::Undefined);

    // Encode/decode
    global.set("encodeURIComponent".to_string(), JsValue::NativeFunction { name: "encodeURIComponent".to_string(), func: |args: &[JsValue]| {
        if args.is_empty() { return Ok(JsValue::String(String::new())); }
        let s = args[0].to_string();
        let mut result = String::new();
        for c in s.chars() {
            if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '~') {
                result.push(c);
            } else {
                for b in c.to_string().as_bytes() {
                    result.push_str(&format!("%{:02X}", b));
                }
            }
        }
        Ok(JsValue::String(result))
    }});
    global.set("decodeURIComponent".to_string(), JsValue::NativeFunction { name: "decodeURIComponent".to_string(), func: |args: &[JsValue]| {
        if args.is_empty() { return Ok(JsValue::String(String::new())); }
        // Simplified - just return as-is
        Ok(JsValue::String(args[0].to_string()))
    }});

    let _ = dom; // Suppress unused warning

    // Promise (basic)
    global.set("Promise".to_string(), JsValue::NativeFunction {
        name: "Promise".to_string(),
        func: super::promise::js_promise_constructor,
    });
}

