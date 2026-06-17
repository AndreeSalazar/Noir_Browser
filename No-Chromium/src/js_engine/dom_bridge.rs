use boa_engine::{Context, JsArgs, JsNativeError, JsResult, JsValue, NativeFunction};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

pub type ElementId = String;

#[derive(Debug, Clone)]
pub struct DomElement {
    pub id: ElementId,
    pub tag_name: String,
    pub text_content: String,
    pub inner_html: String,
    pub attributes: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct DomEvent {
    pub target: ElementId,
    pub event_type: String,
    pub callback_id: u32,
}

static DOM_ELEMENTS: OnceLock<Arc<Mutex<HashMap<ElementId, DomElement>>>> = OnceLock::new();
static DOM_EVENTS: OnceLock<Arc<Mutex<Vec<DomEvent>>>> = OnceLock::new();

fn get_elements() -> &'static Arc<Mutex<HashMap<ElementId, DomElement>>> {
    DOM_ELEMENTS.get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
}

fn get_events() -> &'static Arc<Mutex<Vec<DomEvent>>> {
    DOM_EVENTS.get_or_init(|| Arc::new(Mutex::new(Vec::new())))
}

fn js_get_element_by_id(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let id = args.get_or_undefined(0)
        .to_string(ctx)
        .map(|s| s.to_std_string_escaped())
        .map_err(|e| JsNativeError::typ().with_message(format!("getElementById: {}", e)))?;

    let elements = get_elements().lock().unwrap();
    if let Some(elem) = elements.get(&id) {
        let obj = boa_engine::JsObject::with_null_proto();
        let _ = obj.set(boa_engine::js_string!("id"), JsValue::from(boa_engine::JsString::from(elem.id.as_str())), false, ctx);
        let _ = obj.set(boa_engine::js_string!("tagName"), JsValue::from(boa_engine::JsString::from(elem.tag_name.as_str())), false, ctx);
        let _ = obj.set(boa_engine::js_string!("textContent"), JsValue::from(boa_engine::JsString::from(elem.text_content.as_str())), false, ctx);
        let _ = obj.set(boa_engine::js_string!("innerHTML"), JsValue::from(boa_engine::JsString::from(elem.inner_html.as_str())), false, ctx);
        Ok(JsValue::Object(obj))
    } else {
        Ok(JsValue::Null)
    }
}

fn js_query_selector(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let selector = args.get_or_undefined(0)
        .to_string(ctx)
        .map(|s| s.to_std_string_escaped())
        .map_err(|e| JsNativeError::typ().with_message(format!("querySelector: {}", e)))?;

    let elements = get_elements().lock().unwrap();
    for (_id, elem) in elements.iter() {
        let matched = if selector.starts_with('#') {
            elem.id == selector[1..]
        } else if selector.starts_with('.') {
            elem.attributes.get("class").map_or(false, |c| c.contains(&selector[1..]))
        } else {
            elem.tag_name == selector
        };

        if matched {
            let obj = boa_engine::JsObject::with_null_proto();
            let _ = obj.set(boa_engine::js_string!("id"), JsValue::from(boa_engine::JsString::from(elem.id.as_str())), false, ctx);
            let _ = obj.set(boa_engine::js_string!("tagName"), JsValue::from(boa_engine::JsString::from(elem.tag_name.as_str())), false, ctx);
            return Ok(JsValue::Object(obj));
        }
    }
    Ok(JsValue::Null)
}

fn js_create_element(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let tag = args.get_or_undefined(0)
        .to_string(ctx)
        .map(|s| s.to_std_string_escaped())
        .map_err(|e| JsNativeError::typ().with_message(format!("createElement: {}", e)))?;

    let id = format!("elem_{}", rand::random::<u32>());
    let elem = DomElement {
        id: id.clone(),
        tag_name: tag.clone(),
        text_content: String::new(),
        inner_html: String::new(),
        attributes: HashMap::new(),
    };
    get_elements().lock().unwrap().insert(id.clone(), elem);

    let obj = boa_engine::JsObject::with_null_proto();
    let _ = obj.set(boa_engine::js_string!("id"), JsValue::from(boa_engine::JsString::from(id.as_str())), false, ctx);
    let _ = obj.set(boa_engine::js_string!("tagName"), JsValue::from(boa_engine::JsString::from(tag.as_str())), false, ctx);
    let _ = obj.set(boa_engine::js_string!("textContent"), JsValue::from(boa_engine::JsString::from("")), false, ctx);
    let _ = obj.set(boa_engine::js_string!("innerHTML"), JsValue::from(boa_engine::JsString::from("")), false, ctx);
    Ok(JsValue::Object(obj))
}

fn js_add_event_listener(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let event_type = args.get_or_undefined(0)
        .to_string(ctx)
        .map(|s| s.to_std_string_escaped())
        .map_err(|e| JsNativeError::typ().with_message(format!("addEventListener: {}", e)))?;

    let callback_id = rand::random::<u32>();
    get_events().lock().unwrap().push(DomEvent {
        target: "document".to_string(),
        event_type,
        callback_id,
    });
    Ok(JsValue::undefined())
}

pub struct DomBridge;

impl DomBridge {
    pub fn new() -> Self {
        let elements = get_elements();
        let mut map = elements.lock().unwrap();
        map.insert("document".to_string(), DomElement {
            id: "document".to_string(),
            tag_name: "document".to_string(),
            text_content: String::new(),
            inner_html: String::new(),
            attributes: HashMap::new(),
        });
        Self
    }

    pub fn register_all(&self, context: &mut Context) {
        let doc_obj = boa_engine::JsObject::with_null_proto();

        let get_by_id_fn = NativeFunction::from_fn_ptr(js_get_element_by_id).to_js_function(context.realm());
        let _ = doc_obj.set(boa_engine::js_string!("getElementById"), get_by_id_fn, false, context);

        let query_selector_fn = NativeFunction::from_fn_ptr(js_query_selector).to_js_function(context.realm());
        let _ = doc_obj.set(boa_engine::js_string!("querySelector"), query_selector_fn, false, context);

        let create_element_fn = NativeFunction::from_fn_ptr(js_create_element).to_js_function(context.realm());
        let _ = doc_obj.set(boa_engine::js_string!("createElement"), create_element_fn, false, context);

        let add_event_listener_fn = NativeFunction::from_fn_ptr(js_add_event_listener).to_js_function(context.realm());
        let _ = doc_obj.set(boa_engine::js_string!("addEventListener"), add_event_listener_fn, false, context);

        let _ = context.register_global_property(boa_engine::js_string!("document"), boa_engine::JsValue::Object(doc_obj), boa_engine::property::Attribute::all());
    }

    pub fn set_element_text(&self, id: &str, text: &str) {
        if let Some(elem) = get_elements().lock().unwrap().get_mut(id) {
            elem.text_content = text.to_string();
        }
    }

    pub fn get_element(&self, id: &str) -> Option<DomElement> {
        get_elements().lock().unwrap().get(id).cloned()
    }

    pub fn get_pending_events(&self) -> Vec<DomEvent> {
        get_events().lock().unwrap().drain(..).collect()
    }
}
