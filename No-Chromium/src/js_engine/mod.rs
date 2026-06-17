pub mod runtime;
pub mod web_apis;
pub mod dom_bridge;
pub mod dom_sync;
pub mod events;
pub mod sandbox;
pub mod modules;
pub mod bindings;

use boa_engine::JsValue;
use std::collections::HashMap;
use anyhow::Result;

pub type TabId = u64;

use runtime::JsRuntime;
use web_apis::WebApis;
use dom_bridge::DomBridge;
use events::EventSystem;
use sandbox::TabSandbox;
use modules::ModuleSystem;
use bindings::BrowserBindings;

pub struct JsEngine {
    runtime: JsRuntime,
    web_apis: WebApis,
    dom_bridge: DomBridge,
    event_system: EventSystem,
    sandbox: TabSandbox,
    module_system: ModuleSystem,
    active_tabs: HashMap<TabId, bool>,
}

impl JsEngine {
    pub fn new() -> Self {
        Self {
            runtime: JsRuntime::new(),
            web_apis: WebApis::new(),
            dom_bridge: DomBridge::new(),
            event_system: EventSystem::new(),
            sandbox: TabSandbox::new(),
            module_system: ModuleSystem::new(),
            active_tabs: HashMap::new(),
        }
    }

    pub fn init_tab(&mut self, tab_id: TabId) -> Result<()> {
        self.runtime.create_context(tab_id);
        self.sandbox.create_sandbox(tab_id);
        self.active_tabs.insert(tab_id, true);

        if let Some(context) = self.runtime.get_context(tab_id) {
            self.web_apis.register_all(context);
            self.dom_bridge.register_all(context);
            self.event_system.register_all(context);
            BrowserBindings::new().register_all(context);
        }

        tracing::info!("JS engine initialized for tab {}", tab_id);
        Ok(())
    }

    pub fn destroy_tab(&mut self, tab_id: TabId) {
        self.runtime.remove_context(tab_id);
        self.sandbox.remove_sandbox(tab_id);
        self.active_tabs.remove(&tab_id);
        tracing::info!("JS engine destroyed for tab {}", tab_id);
    }

    pub fn eval_script(&mut self, tab_id: TabId, code: &str) -> Result<String> {
        if !self.sandbox.start_script_execution(tab_id) {
            return Err(anyhow::anyhow!("Script already executing in tab {}", tab_id));
        }

        let result = self.runtime.eval(tab_id, code);
        self.sandbox.end_script_execution(tab_id);
        result
    }

    pub fn eval_script_value(&mut self, tab_id: TabId, code: &str) -> Result<JsValue> {
        if !self.sandbox.start_script_execution(tab_id) {
            return Err(anyhow::anyhow!("Script already executing in tab {}", tab_id));
        }

        let result = self.runtime.eval_with_result(tab_id, code);
        self.sandbox.end_script_execution(tab_id);
        result
    }

    pub fn call_function(&mut self, tab_id: TabId, name: &str, args: &[JsValue]) -> Result<JsValue> {
        self.runtime.call_function(tab_id, name, args)
    }

    pub fn load_module(&mut self, tab_id: TabId, specifier: &str) -> Result<()> {
        if let Some(context) = self.runtime.get_context(tab_id) {
            self.module_system.load_module(tab_id, specifier, context).map_err(|e| anyhow::anyhow!(e))?;
        }
        Ok(())
    }

    pub fn register_module(&mut self, specifier: &str, source: &str) {
        self.module_system.register_module(specifier, source);
    }

    pub fn process_events(&mut self, tab_id: TabId) -> Result<()> {
        let events = self.event_system.process_events();
        for event in events {
            let listeners = self.event_system.get_listeners_for_target(&event.target_id, &event.event_type);
            for listener in listeners {
                let callback_name = format!("__callback_{}", listener.callback_id);
                let _ = self.runtime.eval(tab_id, &format!("if (typeof {} === 'function') {}();", callback_name, callback_name));
            }
        }

        let dom_events = self.dom_bridge.get_pending_events();
        for event in dom_events {
            let listeners = self.event_system.get_listeners_for_target(&event.target, &event.event_type);
            for listener in listeners {
                let callback_name = format!("__callback_{}", listener.callback_id);
                let _ = self.runtime.eval(tab_id, &format!("if (typeof {} === 'function') {}();", callback_name, callback_name));
            }
        }

        Ok(())
    }

    pub fn schedule_timeout(&mut self, tab_id: TabId, callback_id: u32, delay_ms: u64) -> u64 {
        self.runtime.schedule_task(tab_id, callback_id, Some(std::time::Duration::from_millis(delay_ms)), false)
    }

    pub fn schedule_interval(&mut self, tab_id: TabId, callback_id: u32, interval_ms: u64) -> u64 {
        self.runtime.schedule_task(tab_id, callback_id, Some(std::time::Duration::from_millis(interval_ms)), true)
    }

    pub fn cancel_timer(&mut self, timer_id: u64) {
        self.runtime.cancel_task(timer_id);
    }

    pub fn update_url(url: &str) {
        BrowserBindings::update_url(url);
    }

    pub fn update_title(title: &str) {
        BrowserBindings::update_title(title);
    }

    pub fn has_context(&self, tab_id: TabId) -> bool {
        self.runtime.has_context(tab_id)
    }

    pub fn get_console_logs(&self) -> Vec<web_apis::ConsoleMessage> {
        self.web_apis.get_console_logs()
    }

    pub fn is_active(&self, tab_id: TabId) -> bool {
        self.active_tabs.get(&tab_id).copied().unwrap_or(false)
    }

    pub fn active_tab_count(&self) -> usize {
        self.active_tabs.len()
    }
}

impl Default for JsEngine {
    fn default() -> Self {
        Self::new()
    }
}
