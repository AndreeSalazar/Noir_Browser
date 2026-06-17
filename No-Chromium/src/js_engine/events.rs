use boa_engine::{Context, JsArgs, JsNativeError, JsResult, JsValue, NativeFunction};
use std::sync::{Arc, Mutex, OnceLock};

#[derive(Debug, Clone)]
pub struct DomEventListener {
    pub target_id: String,
    pub event_type: String,
    pub callback_id: u32,
}

#[derive(Debug, Clone)]
pub struct EventDispatch {
    pub target_id: String,
    pub event_type: String,
    pub data: Option<String>,
}

static EVENT_LISTENERS: OnceLock<Arc<Mutex<Vec<DomEventListener>>>> = OnceLock::new();
static EVENT_QUEUE: OnceLock<Arc<Mutex<Vec<EventDispatch>>>> = OnceLock::new();

fn get_listeners() -> &'static Arc<Mutex<Vec<DomEventListener>>> {
    EVENT_LISTENERS.get_or_init(|| Arc::new(Mutex::new(Vec::new())))
}

fn get_queue() -> &'static Arc<Mutex<Vec<EventDispatch>>> {
    EVENT_QUEUE.get_or_init(|| Arc::new(Mutex::new(Vec::new())))
}

fn js_add_event_listener(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let event_type = args.get_or_undefined(0)
        .to_string(ctx)
        .map(|s| s.to_std_string_escaped())
        .map_err(|e| JsNativeError::typ().with_message(format!("addEventListener: {}", e)))?;

    let callback_id = rand::random::<u32>();
    get_listeners().lock().unwrap().push(DomEventListener {
        target_id: "document".to_string(),
        event_type,
        callback_id,
    });
    Ok(JsValue::undefined())
}

fn js_remove_event_listener(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let event_type = args.get_or_undefined(0)
        .to_string(ctx)
        .map(|s| s.to_std_string_escaped())
        .map_err(|e| JsNativeError::typ().with_message(format!("removeEventListener: {}", e)))?;

    get_listeners().lock().unwrap().retain(|l| l.event_type != event_type);
    Ok(JsValue::undefined())
}

fn js_dispatch_event(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let event_type = args.get_or_undefined(0)
        .to_string(ctx)
        .map(|s| s.to_std_string_escaped())
        .map_err(|e| JsNativeError::typ().with_message(format!("dispatchEvent: {}", e)))?;

    get_queue().lock().unwrap().push(EventDispatch {
        target_id: "document".to_string(),
        event_type,
        data: None,
    });
    Ok(JsValue::undefined())
}

pub struct EventSystem;

impl EventSystem {
    pub fn new() -> Self {
        Self
    }

    pub fn register_all(&self, context: &mut Context) {
        let add_fn = NativeFunction::from_fn_ptr(js_add_event_listener).to_js_function(context.realm());
        let _ = context.register_global_property(
            boa_engine::js_string!("addEventListener"),
            add_fn,
            boa_engine::property::Attribute::all(),
        );

        let remove_fn = NativeFunction::from_fn_ptr(js_remove_event_listener).to_js_function(context.realm());
        let _ = context.register_global_property(
            boa_engine::js_string!("removeEventListener"),
            remove_fn,
            boa_engine::property::Attribute::all(),
        );

        let dispatch_fn = NativeFunction::from_fn_ptr(js_dispatch_event).to_js_function(context.realm());
        let _ = context.register_global_property(
            boa_engine::js_string!("dispatchEvent"),
            dispatch_fn,
            boa_engine::property::Attribute::all(),
        );
    }

    pub fn add_listener(&self, target_id: &str, event_type: &str) -> u32 {
        let callback_id = rand::random::<u32>();
        get_listeners().lock().unwrap().push(DomEventListener {
            target_id: target_id.to_string(),
            event_type: event_type.to_string(),
            callback_id,
        });
        callback_id
    }

    pub fn remove_listener(&self, callback_id: u32) {
        get_listeners().lock().unwrap().retain(|l| l.callback_id != callback_id);
    }

    pub fn dispatch_event(&self, target_id: &str, event_type: &str, data: Option<String>) {
        get_queue().lock().unwrap().push(EventDispatch {
            target_id: target_id.to_string(),
            event_type: event_type.to_string(),
            data,
        });
    }

    pub fn get_listeners_for_target(&self, target_id: &str, event_type: &str) -> Vec<DomEventListener> {
        get_listeners().lock().unwrap()
            .iter()
            .filter(|l| l.target_id == target_id && l.event_type == event_type)
            .cloned()
            .collect()
    }

    pub fn process_events(&self) -> Vec<EventDispatch> {
        get_queue().lock().unwrap().drain(..).collect()
    }
}
