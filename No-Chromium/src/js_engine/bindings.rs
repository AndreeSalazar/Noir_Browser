use boa_engine::{Context, JsArgs, JsNativeError, JsResult, JsValue, NativeFunction};
use std::sync::{Arc, Mutex, OnceLock};

pub struct BrowserBindings;

static BINDINGS_DATA: OnceLock<Arc<Mutex<BindingsData>>> = OnceLock::new();

struct BindingsData {
    user_agent: String,
    platform: String,
    language: String,
    current_url: String,
    title: String,
}

fn get_data() -> &'static Arc<Mutex<BindingsData>> {
    BINDINGS_DATA.get_or_init(|| Arc::new(Mutex::new(BindingsData {
        user_agent: "NoirBrowser/0.2.0 (Rust; Noir)".to_string(),
        platform: std::env::consts::OS.to_string(),
        language: "en-US".to_string(),
        current_url: "about:blank".to_string(),
        title: "Noir Browser".to_string(),
    })))
}

fn js_navigator_user_agent(_this: &JsValue, _args: &[JsValue], _ctx: &mut Context) -> JsResult<JsValue> {
    let ua = get_data().lock().unwrap().user_agent.clone();
    Ok(JsValue::from(boa_engine::JsString::from(ua.as_str())))
}

fn js_navigator_platform(_this: &JsValue, _args: &[JsValue], _ctx: &mut Context) -> JsResult<JsValue> {
    let p = get_data().lock().unwrap().platform.clone();
    Ok(JsValue::from(boa_engine::JsString::from(p.as_str())))
}

fn js_navigator_language(_this: &JsValue, _args: &[JsValue], _ctx: &mut Context) -> JsResult<JsValue> {
    let l = get_data().lock().unwrap().language.clone();
    Ok(JsValue::from(boa_engine::JsString::from(l.as_str())))
}

fn js_navigator_online(_this: &JsValue, _args: &[JsValue], _ctx: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::from(true))
}

fn js_navigator_cookie_enabled(_this: &JsValue, _args: &[JsValue], _ctx: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::from(true))
}

fn js_location_href(_this: &JsValue, _args: &[JsValue], _ctx: &mut Context) -> JsResult<JsValue> {
    let url = get_data().lock().unwrap().current_url.clone();
    Ok(JsValue::from(boa_engine::JsString::from(url.as_str())))
}

fn js_location_assign(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let new_url = args.get_or_undefined(0)
        .to_string(ctx)
        .map(|s| s.to_std_string_escaped())
        .map_err(|e| JsNativeError::typ().with_message(format!("location.assign: {}", e)))?;
    tracing::info!("location.assign: {}", new_url);
    Ok(JsValue::undefined())
}

fn js_location_reload(_this: &JsValue, _args: &[JsValue], _ctx: &mut Context) -> JsResult<JsValue> {
    tracing::info!("location.reload()");
    Ok(JsValue::undefined())
}

fn js_window_title(_this: &JsValue, _args: &[JsValue], _ctx: &mut Context) -> JsResult<JsValue> {
    let t = get_data().lock().unwrap().title.clone();
    Ok(JsValue::from(boa_engine::JsString::from(t.as_str())))
}

fn js_window_close(_this: &JsValue, _args: &[JsValue], _ctx: &mut Context) -> JsResult<JsValue> {
    tracing::info!("window.close()");
    Ok(JsValue::undefined())
}

fn js_window_alert(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let msg = args.get_or_undefined(0)
        .to_string(ctx)
        .map(|s| s.to_std_string_escaped())
        .unwrap_or_default();
    tracing::warn!("[JS alert] {}", msg);
    Ok(JsValue::undefined())
}

fn js_window_confirm(_this: &JsValue, _args: &[JsValue], _ctx: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::from(true))
}

fn js_window_prompt(_this: &JsValue, _args: &[JsValue], _ctx: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::Null)
}

impl BrowserBindings {
    pub fn new() -> Self {
        Self
    }

    pub fn register_all(&self, context: &mut Context) {
        self.register_navigator(context);
        self.register_location(context);
        self.register_window(context);
    }

    fn register_navigator(&self, context: &mut Context) {
        let nav_obj = boa_engine::JsObject::with_null_proto();

        let ua_fn = NativeFunction::from_fn_ptr(js_navigator_user_agent).to_js_function(context.realm());
        let _ = nav_obj.set(boa_engine::js_string!("userAgent"), ua_fn, false, context);

        let platform_fn = NativeFunction::from_fn_ptr(js_navigator_platform).to_js_function(context.realm());
        let _ = nav_obj.set(boa_engine::js_string!("platform"), platform_fn, false, context);

        let lang_fn = NativeFunction::from_fn_ptr(js_navigator_language).to_js_function(context.realm());
        let _ = nav_obj.set(boa_engine::js_string!("language"), lang_fn, false, context);

        let online_fn = NativeFunction::from_fn_ptr(js_navigator_online).to_js_function(context.realm());
        let _ = nav_obj.set(boa_engine::js_string!("onLine"), online_fn, false, context);

        let cookie_fn = NativeFunction::from_fn_ptr(js_navigator_cookie_enabled).to_js_function(context.realm());
        let _ = nav_obj.set(boa_engine::js_string!("cookieEnabled"), cookie_fn, false, context);

        let _ = context.register_global_property(boa_engine::js_string!("navigator"), boa_engine::JsValue::Object(nav_obj), boa_engine::property::Attribute::all());
    }

    fn register_location(&self, context: &mut Context) {
        let loc_obj = boa_engine::JsObject::with_null_proto();

        let href_fn = NativeFunction::from_fn_ptr(js_location_href).to_js_function(context.realm());
        let _ = loc_obj.set(boa_engine::js_string!("href"), href_fn, false, context);

        let assign_fn = NativeFunction::from_fn_ptr(js_location_assign).to_js_function(context.realm());
        let _ = loc_obj.set(boa_engine::js_string!("assign"), assign_fn, false, context);

        let reload_fn = NativeFunction::from_fn_ptr(js_location_reload).to_js_function(context.realm());
        let _ = loc_obj.set(boa_engine::js_string!("reload"), reload_fn, false, context);

        let _ = context.register_global_property(boa_engine::js_string!("location"), boa_engine::JsValue::Object(loc_obj), boa_engine::property::Attribute::all());
    }

    fn register_window(&self, context: &mut Context) {
        let window_obj = boa_engine::JsObject::with_null_proto();

        let title_fn = NativeFunction::from_fn_ptr(js_window_title).to_js_function(context.realm());
        let _ = window_obj.set(boa_engine::js_string!("title"), title_fn, false, context);

        let close_fn = NativeFunction::from_fn_ptr(js_window_close).to_js_function(context.realm());
        let _ = window_obj.set(boa_engine::js_string!("close"), close_fn, false, context);

        let alert_fn = NativeFunction::from_fn_ptr(js_window_alert).to_js_function(context.realm());
        let _ = window_obj.set(boa_engine::js_string!("alert"), alert_fn, false, context);

        let confirm_fn = NativeFunction::from_fn_ptr(js_window_confirm).to_js_function(context.realm());
        let _ = window_obj.set(boa_engine::js_string!("confirm"), confirm_fn, false, context);

        let prompt_fn = NativeFunction::from_fn_ptr(js_window_prompt).to_js_function(context.realm());
        let _ = window_obj.set(boa_engine::js_string!("prompt"), prompt_fn, false, context);

        let _ = context.register_global_property(boa_engine::js_string!("window"), boa_engine::JsValue::Object(window_obj), boa_engine::property::Attribute::all());
    }

    pub fn update_url(url: &str) {
        get_data().lock().unwrap().current_url = url.to_string();
    }

    pub fn update_title(title: &str) {
        get_data().lock().unwrap().title = title.to_string();
    }
}
