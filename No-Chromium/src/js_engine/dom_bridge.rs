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
    pub children_ids: Vec<ElementId>,
    pub parent_id: Option<ElementId>,
}

#[derive(Debug, Clone)]
pub struct DomEvent {
    pub target: ElementId,
    pub event_type: String,
    pub callback_id: u32,
}

static DOM_ELEMENTS: OnceLock<Arc<Mutex<HashMap<ElementId, DomElement>>>> = OnceLock::new();
static DOM_EVENTS: OnceLock<Arc<Mutex<Vec<DomEvent>>>> = OnceLock::new();
static DOM_MUTATED: OnceLock<Arc<Mutex<bool>>> = OnceLock::new();

fn get_elements() -> &'static Arc<Mutex<HashMap<ElementId, DomElement>>> {
    DOM_ELEMENTS.get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
}

pub fn get_elements_static() -> Option<&'static Arc<Mutex<HashMap<ElementId, DomElement>>>> {
    DOM_ELEMENTS.get()
}

fn get_events() -> &'static Arc<Mutex<Vec<DomEvent>>> {
    DOM_EVENTS.get_or_init(|| Arc::new(Mutex::new(Vec::new())))
}

fn get_mutated_flag() -> &'static Arc<Mutex<bool>> {
    DOM_MUTATED.get_or_init(|| Arc::new(Mutex::new(false)))
}

pub fn mark_mutated() {
    *get_mutated_flag().lock().unwrap() = true;
}

pub fn take_mutated_flag() -> bool {
    let flag = *get_mutated_flag().lock().unwrap();
    if flag {
        *get_mutated_flag().lock().unwrap() = false;
    }
    flag
}

fn js_get_element_by_id(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let id = args.get_or_undefined(0)
        .to_string(ctx)
        .map(|s| s.to_std_string_escaped())
        .map_err(|e| JsNativeError::typ().with_message(format!("getElementById: {}", e)))?;

    let elements = get_elements().lock().unwrap();
    if let Some(elem) = elements.get(&id) {
        Ok(make_element_object(elem, ctx))
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
            return Ok(make_element_object(elem, ctx));
        }
    }
    Ok(JsValue::Null)
}

fn js_query_selector_all(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let selector = args.get_or_undefined(0)
        .to_string(ctx)
        .map(|s| s.to_std_string_escaped())
        .map_err(|e| JsNativeError::typ().with_message(format!("querySelectorAll: {}", e)))?;

    let elements = get_elements().lock().unwrap();
    let arr = boa_engine::JsObject::with_null_proto();
    let mut idx = 0u32;
    for (_id, elem) in elements.iter() {
        let matched = if selector.starts_with('#') {
            elem.id == selector[1..]
        } else if selector.starts_with('.') {
            elem.attributes.get("class").map_or(false, |c| c.contains(&selector[1..]))
        } else {
            elem.tag_name == selector
        };

        if matched {
            let _ = arr.set(boa_engine::js_string!((idx).to_string()), make_element_object(elem, ctx), false, ctx);
            idx += 1;
        }
    }
    let _ = arr.set(boa_engine::js_string!("length"), JsValue::from(idx), false, ctx);
    Ok(JsValue::Object(arr))
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
        children_ids: Vec::new(),
        parent_id: None,
    };
    get_elements().lock().unwrap().insert(id.clone(), elem);
    mark_mutated();

    Ok(make_element_obj_raw(&id, &tag, ctx))
}

fn js_add_event_listener(this: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let event_type = args.get_or_undefined(0)
        .to_string(ctx)
        .map(|s| s.to_std_string_escaped())
        .map_err(|e| JsNativeError::typ().with_message(format!("addEventListener: {}", e)))?;

    let target = if let Some(obj) = this.as_object() {
        obj.get(boa_engine::js_string!("id"), ctx)
            .ok()
            .and_then(|v| v.to_string(ctx).ok())
            .map(|s| s.to_std_string_escaped())
            .unwrap_or_else(|| "document".to_string())
    } else {
        "document".to_string()
    };

    let callback_id = rand::random::<u32>();
    get_events().lock().unwrap().push(DomEvent {
        target,
        event_type,
        callback_id,
    });
    Ok(JsValue::undefined())
}

fn js_set_text_content(this: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let text = args.get_or_undefined(0)
        .to_string(ctx)
        .map(|s| s.to_std_string_escaped())
        .map_err(|e| JsNativeError::typ().with_message(format!("textContent: {}", e)))?;

    if let Some(obj) = this.as_object() {
        if let Ok(id_val) = obj.get(boa_engine::js_string!("id"), ctx) {
            if let Ok(id_str) = id_val.to_string(ctx) {
                let id = id_str.to_std_string_escaped();
                let mut elements = get_elements().lock().unwrap();
                if let Some(elem) = elements.get_mut(&id) {
                    elem.text_content = text.clone();
                    elem.inner_html = text;
                }
            }
        }
    }
    mark_mutated();
    Ok(JsValue::undefined())
}

fn js_get_text_content(this: &JsValue, _args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    if let Some(obj) = this.as_object() {
        if let Ok(id_val) = obj.get(boa_engine::js_string!("id"), ctx) {
            if let Ok(id_str) = id_val.to_string(ctx) {
                let id = id_str.to_std_string_escaped();
                let elements = get_elements().lock().unwrap();
                if let Some(elem) = elements.get(&id) {
                    return Ok(JsValue::from(boa_engine::JsString::from(elem.text_content.as_str())));
                }
            }
        }
    }
    Ok(JsValue::from(boa_engine::JsString::from("")))
}

fn js_set_inner_html(this: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let html = args.get_or_undefined(0)
        .to_string(ctx)
        .map(|s| s.to_std_string_escaped())
        .map_err(|e| JsNativeError::typ().with_message(format!("innerHTML: {}", e)))?;

    if let Some(obj) = this.as_object() {
        if let Ok(id_val) = obj.get(boa_engine::js_string!("id"), ctx) {
            if let Ok(id_str) = id_val.to_string(ctx) {
                let id = id_str.to_std_string_escaped();
                let mut elements = get_elements().lock().unwrap();
                if let Some(elem) = elements.get_mut(&id) {
                    elem.inner_html = html.clone();
                    elem.text_content = strip_html_tags(&html);
                    elem.children_ids.clear();
                }
            }
        }
    }
    mark_mutated();
    Ok(JsValue::undefined())
}

fn js_get_inner_html(this: &JsValue, _args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    if let Some(obj) = this.as_object() {
        if let Ok(id_val) = obj.get(boa_engine::js_string!("id"), ctx) {
            if let Ok(id_str) = id_val.to_string(ctx) {
                let id = id_str.to_std_string_escaped();
                let elements = get_elements().lock().unwrap();
                if let Some(elem) = elements.get(&id) {
                    return Ok(JsValue::from(boa_engine::JsString::from(elem.inner_html.as_str())));
                }
            }
        }
    }
    Ok(JsValue::from(boa_engine::JsString::from("")))
}

fn js_append_child(this: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let child_val = args.get_or_undefined(0);

    if let (Some(parent_obj), Some(child_obj)) = (this.as_object(), child_val.as_object()) {
        let parent_id = parent_obj.get(boa_engine::js_string!("id"), ctx)
            .ok().and_then(|v| v.to_string(ctx).ok())
            .map(|s| s.to_std_string_escaped());
        let child_id = child_obj.get(boa_engine::js_string!("id"), ctx)
            .ok().and_then(|v| v.to_string(ctx).ok())
            .map(|s| s.to_std_string_escaped());

        if let (Some(pid), Some(cid)) = (parent_id, child_id) {
            let mut elements = get_elements().lock().unwrap();
            if let Some(child) = elements.get_mut(&cid) {
                child.parent_id = Some(pid.clone());
            }
            if let Some(parent) = elements.get_mut(&pid) {
                if !parent.children_ids.contains(&cid) {
                    parent.children_ids.push(cid);
                }
            }
        }
    }
    mark_mutated();
    Ok(child_val.clone())
}

fn js_remove_child(this: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let child_val = args.get_or_undefined(0);

    if let (Some(parent_obj), Some(child_obj)) = (this.as_object(), child_val.as_object()) {
        let parent_id = parent_obj.get(boa_engine::js_string!("id"), ctx)
            .ok().and_then(|v| v.to_string(ctx).ok())
            .map(|s| s.to_std_string_escaped());
        let child_id = child_obj.get(boa_engine::js_string!("id"), ctx)
            .ok().and_then(|v| v.to_string(ctx).ok())
            .map(|s| s.to_std_string_escaped());

        if let (Some(pid), Some(cid)) = (parent_id, child_id) {
            let mut elements = get_elements().lock().unwrap();
            if let Some(parent) = elements.get_mut(&pid) {
                parent.children_ids.retain(|c| c != &cid);
            }
            if let Some(child) = elements.get_mut(&cid) {
                child.parent_id = None;
            }
        }
    }
    mark_mutated();
    Ok(child_val.clone())
}

fn js_set_attribute(this: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let attr_name = args.get_or_undefined(0)
        .to_string(ctx)
        .map(|s| s.to_std_string_escaped())
        .map_err(|e| JsNativeError::typ().with_message(format!("setAttribute: {}", e)))?;
    let attr_val = args.get_or_undefined(1)
        .to_string(ctx)
        .map(|s| s.to_std_string_escaped())
        .map_err(|e| JsNativeError::typ().with_message(format!("setAttribute: {}", e)))?;

    if let Some(obj) = this.as_object() {
        if let Ok(id_val) = obj.get(boa_engine::js_string!("id"), ctx) {
            if let Ok(id_str) = id_val.to_string(ctx) {
                let id = id_str.to_std_string_escaped();
                let mut elements = get_elements().lock().unwrap();
                if let Some(elem) = elements.get_mut(&id) {
                    elem.attributes.insert(attr_name, attr_val);
                }
            }
        }
    }
    mark_mutated();
    Ok(JsValue::undefined())
}

fn js_get_attribute(this: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let attr_name = args.get_or_undefined(0)
        .to_string(ctx)
        .map(|s| s.to_std_string_escaped())
        .map_err(|e| JsNativeError::typ().with_message(format!("getAttribute: {}", e)))?;

    if let Some(obj) = this.as_object() {
        if let Ok(id_val) = obj.get(boa_engine::js_string!("id"), ctx) {
            if let Ok(id_str) = id_val.to_string(ctx) {
                let id = id_str.to_std_string_escaped();
                let elements = get_elements().lock().unwrap();
                if let Some(elem) = elements.get(&id) {
                    if let Some(val) = elem.attributes.get(&attr_name) {
                        return Ok(JsValue::from(boa_engine::JsString::from(val.as_str())));
                    }
                }
            }
        }
    }
    Ok(JsValue::Null)
}

fn js_get_elements_by_tag_name(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let tag = args.get_or_undefined(0)
        .to_string(ctx)
        .map(|s| s.to_std_string_escaped())
        .map_err(|e| JsNativeError::typ().with_message(format!("getElementsByTagName: {}", e)))?;

    let tag_lower = tag.to_lowercase();
    let elements = get_elements().lock().unwrap();
    let arr = boa_engine::JsObject::with_null_proto();
    let mut idx = 0u32;
    for (_id, elem) in elements.iter() {
        if elem.tag_name == tag_lower {
            let _ = arr.set(boa_engine::js_string!((idx).to_string()), make_element_object(elem, ctx), false, ctx);
            idx += 1;
        }
    }
    let _ = arr.set(boa_engine::js_string!("length"), JsValue::from(idx), false, ctx);
    Ok(JsValue::Object(arr))
}

fn js_get_elements_by_class_name(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let class_name = args.get_or_undefined(0)
        .to_string(ctx)
        .map(|s| s.to_std_string_escaped())
        .map_err(|e| JsNativeError::typ().with_message(format!("getElementsByClassName: {}", e)))?;

    let elements = get_elements().lock().unwrap();
    let arr = boa_engine::JsObject::with_null_proto();
    let mut idx = 0u32;
    for (_id, elem) in elements.iter() {
        if let Some(class) = elem.attributes.get("class") {
            if class.split_whitespace().any(|c| c == class_name) {
                let _ = arr.set(boa_engine::js_string!((idx).to_string()), make_element_object(elem, ctx), false, ctx);
                idx += 1;
            }
        }
    }
    let _ = arr.set(boa_engine::js_string!("length"), JsValue::from(idx), false, ctx);
    Ok(JsValue::Object(arr))
}

fn js_set_style(this: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let css_text = args.get_or_undefined(0)
        .to_string(ctx)
        .map(|s| s.to_std_string_escaped())
        .map_err(|e| JsNativeError::typ().with_message(format!("setAttribute: {}", e)))?;

    if let Some(obj) = this.as_object() {
        if let Ok(id_val) = obj.get(boa_engine::js_string!("id"), ctx) {
            if let Ok(id_str) = id_val.to_string(ctx) {
                let id = id_str.to_std_string_escaped();
                let mut elements = get_elements().lock().unwrap();
                if let Some(elem) = elements.get_mut(&id) {
                    elem.attributes.insert("style".to_string(), css_text);
                }
            }
        }
    }
    mark_mutated();
    Ok(JsValue::undefined())
}

fn make_element_object(elem: &DomElement, ctx: &mut Context) -> JsValue {
    let obj = boa_engine::JsObject::with_null_proto();
    let _ = obj.set(boa_engine::js_string!("id"), JsValue::from(boa_engine::JsString::from(elem.id.as_str())), false, ctx);
    let _ = obj.set(boa_engine::js_string!("tagName"), JsValue::from(boa_engine::JsString::from(elem.tag_name.as_str())), false, ctx);
    let _ = obj.set(boa_engine::js_string!("textContent"), JsValue::from(boa_engine::JsString::from(elem.text_content.as_str())), false, ctx);
    let _ = obj.set(boa_engine::js_string!("innerHTML"), JsValue::from(boa_engine::JsString::from(elem.inner_html.as_str())), false, ctx);
    let _ = obj.set(boa_engine::js_string!("className"), JsValue::from(boa_engine::JsString::from(
        elem.attributes.get("class").map_or("", |s| s.as_str())
    )), false, ctx);

    let set_text_fn = NativeFunction::from_fn_ptr(js_set_text_content).to_js_function(ctx.realm());
    let _ = obj.set(boa_engine::js_string!("textContent"), set_text_fn, false, ctx);

    let set_inner_html_fn = NativeFunction::from_fn_ptr(js_set_inner_html).to_js_function(ctx.realm());
    let _ = obj.set(boa_engine::js_string!("innerHTML"), set_inner_html_fn, false, ctx);

    let get_inner_html_fn = NativeFunction::from_fn_ptr(js_get_inner_html).to_js_function(ctx.realm());
    let _ = obj.set(boa_engine::js_string!("getInnerHTML"), get_inner_html_fn, false, ctx);

    let append_fn = NativeFunction::from_fn_ptr(js_append_child).to_js_function(ctx.realm());
    let _ = obj.set(boa_engine::js_string!("appendChild"), append_fn, false, ctx);

    let remove_fn = NativeFunction::from_fn_ptr(js_remove_child).to_js_function(ctx.realm());
    let _ = obj.set(boa_engine::js_string!("removeChild"), remove_fn, false, ctx);

    let set_attr_fn = NativeFunction::from_fn_ptr(js_set_attribute).to_js_function(ctx.realm());
    let _ = obj.set(boa_engine::js_string!("setAttribute"), set_attr_fn, false, ctx);

    let get_attr_fn = NativeFunction::from_fn_ptr(js_get_attribute).to_js_function(ctx.realm());
    let _ = obj.set(boa_engine::js_string!("getAttribute"), get_attr_fn, false, ctx);

    let add_listener_fn = NativeFunction::from_fn_ptr(js_add_event_listener).to_js_function(ctx.realm());
    let _ = obj.set(boa_engine::js_string!("addEventListener"), add_listener_fn, false, ctx);

    let style_obj = boa_engine::JsObject::with_null_proto();
    let set_style_fn2 = NativeFunction::from_fn_ptr(js_set_style).to_js_function(ctx.realm());
    let _ = style_obj.set(boa_engine::js_string!("cssText"), set_style_fn2, false, ctx);
    let _ = obj.set(boa_engine::js_string!("style"), boa_engine::JsValue::Object(style_obj), false, ctx);

    JsValue::Object(obj)
}

fn make_element_obj_raw(id: &str, tag: &str, ctx: &mut Context) -> JsValue {
    let obj = boa_engine::JsObject::with_null_proto();
    let _ = obj.set(boa_engine::js_string!("id"), JsValue::from(boa_engine::JsString::from(id)), false, ctx);
    let _ = obj.set(boa_engine::js_string!("tagName"), JsValue::from(boa_engine::JsString::from(tag)), false, ctx);
    let _ = obj.set(boa_engine::js_string!("textContent"), JsValue::from(boa_engine::JsString::from("")), false, ctx);
    let _ = obj.set(boa_engine::js_string!("innerHTML"), JsValue::from(boa_engine::JsString::from("")), false, ctx);
    let _ = obj.set(boa_engine::js_string!("className"), JsValue::from(boa_engine::JsString::from("")), false, ctx);

    let set_text_fn = NativeFunction::from_fn_ptr(js_set_text_content).to_js_function(ctx.realm());
    let _ = obj.set(boa_engine::js_string!("textContent"), set_text_fn, false, ctx);

    let set_inner_html_fn = NativeFunction::from_fn_ptr(js_set_inner_html).to_js_function(ctx.realm());
    let _ = obj.set(boa_engine::js_string!("innerHTML"), set_inner_html_fn, false, ctx);

    let get_inner_html_fn = NativeFunction::from_fn_ptr(js_get_inner_html).to_js_function(ctx.realm());
    let _ = obj.set(boa_engine::js_string!("getInnerHTML"), get_inner_html_fn, false, ctx);

    let append_fn = NativeFunction::from_fn_ptr(js_append_child).to_js_function(ctx.realm());
    let _ = obj.set(boa_engine::js_string!("appendChild"), append_fn, false, ctx);

    let remove_fn = NativeFunction::from_fn_ptr(js_remove_child).to_js_function(ctx.realm());
    let _ = obj.set(boa_engine::js_string!("removeChild"), remove_fn, false, ctx);

    let set_attr_fn = NativeFunction::from_fn_ptr(js_set_attribute).to_js_function(ctx.realm());
    let _ = obj.set(boa_engine::js_string!("setAttribute"), set_attr_fn, false, ctx);

    let get_attr_fn = NativeFunction::from_fn_ptr(js_get_attribute).to_js_function(ctx.realm());
    let _ = obj.set(boa_engine::js_string!("getAttribute"), get_attr_fn, false, ctx);

    let add_listener_fn = NativeFunction::from_fn_ptr(js_add_event_listener).to_js_function(ctx.realm());
    let _ = obj.set(boa_engine::js_string!("addEventListener"), add_listener_fn, false, ctx);

    let style_obj = boa_engine::JsObject::with_null_proto();
    let set_style_fn2 = NativeFunction::from_fn_ptr(js_set_style).to_js_function(ctx.realm());
    let _ = style_obj.set(boa_engine::js_string!("cssText"), set_style_fn2, false, ctx);
    let _ = obj.set(boa_engine::js_string!("style"), boa_engine::JsValue::Object(style_obj), false, ctx);

    JsValue::Object(obj)
}

fn strip_html_tags(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }
    result.trim().to_string()
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
            children_ids: Vec::new(),
            parent_id: None,
        });
        Self
    }

    pub fn register_all(&self, context: &mut Context) {
        let doc_obj = boa_engine::JsObject::with_null_proto();

        let get_by_id_fn = NativeFunction::from_fn_ptr(js_get_element_by_id).to_js_function(context.realm());
        let _ = doc_obj.set(boa_engine::js_string!("getElementById"), get_by_id_fn, false, context);

        let query_selector_fn = NativeFunction::from_fn_ptr(js_query_selector).to_js_function(context.realm());
        let _ = doc_obj.set(boa_engine::js_string!("querySelector"), query_selector_fn, false, context);

        let query_selector_all_fn = NativeFunction::from_fn_ptr(js_query_selector_all).to_js_function(context.realm());
        let _ = doc_obj.set(boa_engine::js_string!("querySelectorAll"), query_selector_all_fn, false, context);

        let create_element_fn = NativeFunction::from_fn_ptr(js_create_element).to_js_function(context.realm());
        let _ = doc_obj.set(boa_engine::js_string!("createElement"), create_element_fn, false, context);

        let add_event_listener_fn = NativeFunction::from_fn_ptr(js_add_event_listener).to_js_function(context.realm());
        let _ = doc_obj.set(boa_engine::js_string!("addEventListener"), add_event_listener_fn, false, context);

        let get_by_tag_fn = NativeFunction::from_fn_ptr(js_get_elements_by_tag_name).to_js_function(context.realm());
        let _ = doc_obj.set(boa_engine::js_string!("getElementsByTagName"), get_by_tag_fn, false, context);

        let get_by_class_fn = NativeFunction::from_fn_ptr(js_get_elements_by_class_name).to_js_function(context.realm());
        let _ = doc_obj.set(boa_engine::js_string!("getElementsByClassName"), get_by_class_fn, false, context);

        let body_obj = boa_engine::JsObject::with_null_proto();
        let _ = body_obj.set(boa_engine::js_string!("id"), JsValue::from(boa_engine::JsString::from("body")), false, context);
        let _ = body_obj.set(boa_engine::js_string!("tagName"), JsValue::from(boa_engine::JsString::from("body")), false, context);
        let _ = body_obj.set(boa_engine::js_string!("textContent"), JsValue::from(boa_engine::JsString::from("")), false, context);
        let _ = body_obj.set(boa_engine::js_string!("innerHTML"), JsValue::from(boa_engine::JsString::from("")), false, context);

        let body_set_text = NativeFunction::from_fn_ptr(js_set_text_content).to_js_function(context.realm());
        let _ = body_obj.set(boa_engine::js_string!("textContent"), body_set_text, false, context);
        let body_set_html = NativeFunction::from_fn_ptr(js_set_inner_html).to_js_function(context.realm());
        let _ = body_obj.set(boa_engine::js_string!("innerHTML"), body_set_html, false, context);
        let body_append = NativeFunction::from_fn_ptr(js_append_child).to_js_function(context.realm());
        let _ = body_obj.set(boa_engine::js_string!("appendChild"), body_append, false, context);
        let body_remove = NativeFunction::from_fn_ptr(js_remove_child).to_js_function(context.realm());
        let _ = body_obj.set(boa_engine::js_string!("removeChild"), body_remove, false, context);
        let body_set_attr = NativeFunction::from_fn_ptr(js_set_attribute).to_js_function(context.realm());
        let _ = body_obj.set(boa_engine::js_string!("setAttribute"), body_set_attr, false, context);
        let body_get_attr = NativeFunction::from_fn_ptr(js_get_attribute).to_js_function(context.realm());
        let _ = body_obj.set(boa_engine::js_string!("getAttribute"), body_get_attr, false, context);
        let body_add_listener = NativeFunction::from_fn_ptr(js_add_event_listener).to_js_function(context.realm());
        let _ = body_obj.set(boa_engine::js_string!("addEventListener"), body_add_listener, false, context);

        let _ = doc_obj.set(boa_engine::js_string!("body"), boa_engine::JsValue::Object(body_obj.clone()), false, context);
        let _ = doc_obj.set(boa_engine::js_string!("documentElement"), boa_engine::JsValue::Object(body_obj), false, context);

        let _ = context.register_global_property(boa_engine::js_string!("document"), boa_engine::JsValue::Object(doc_obj), boa_engine::property::Attribute::all());
    }

    pub fn set_element_text(&self, id: &str, text: &str) {
        if let Some(elem) = get_elements().lock().unwrap().get_mut(id) {
            elem.text_content = text.to_string();
        }
        mark_mutated();
    }

    pub fn get_element(&self, id: &str) -> Option<DomElement> {
        get_elements().lock().unwrap().get(id).cloned()
    }

    pub fn get_pending_events(&self) -> Vec<DomEvent> {
        get_events().lock().unwrap().drain(..).collect()
    }

    pub fn get_all_elements() -> Vec<DomElement> {
        get_elements().lock().unwrap().values().cloned().collect()
    }

    pub fn get_root_elements() -> Vec<DomElement> {
        let elements = get_elements().lock().unwrap();
        elements.values()
            .filter(|e| e.parent_id.is_none() || e.parent_id.as_deref() == Some("document"))
            .cloned()
            .collect()
    }
}
