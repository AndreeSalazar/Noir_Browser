use boa_engine::{Context, JsValue, Source};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use anyhow::Result;

pub type TabId = u64;

#[derive(Clone)]
pub struct JsTask {
    pub id: u64,
    pub tab_id: TabId,
    pub callback_id: u32,
    pub delay: Option<Duration>,
    pub created_at: Instant,
    pub repeating: bool,
    pub interval: Option<Duration>,
}

pub struct JsRuntime {
    contexts: HashMap<TabId, Context>,
    task_queue: Arc<Mutex<Vec<JsTask>>>,
    next_task_id: u64,
}

impl JsRuntime {
    pub fn new() -> Self {
        Self {
            contexts: HashMap::new(),
            task_queue: Arc::new(Mutex::new(Vec::new())),
            next_task_id: 1,
        }
    }

    pub fn create_context(&mut self, tab_id: TabId) {
        let context = Context::default();

        self.contexts.insert(tab_id, context);
        tracing::info!("JS context created for tab {}", tab_id);
    }

    pub fn remove_context(&mut self, tab_id: TabId) {
        self.contexts.remove(&tab_id);

        self.task_queue.lock().unwrap().retain(|t| t.tab_id != tab_id);
        tracing::info!("JS context removed for tab {}", tab_id);
    }

    pub fn eval(&mut self, tab_id: TabId, code: &str) -> Result<String> {
        let context = self.contexts.get_mut(&tab_id)
            .ok_or_else(|| anyhow::anyhow!("No JS context for tab {}", tab_id))?;

        match context.eval(Source::from_bytes(code)) {
            Ok(val) => {
                let result = val.to_string(context)
                    .map(|s| s.to_std_string_escaped())
                    .unwrap_or_else(|_| "undefined".to_string());
                Ok(result)
            }
            Err(e) => {
                let msg = e.to_string();
                tracing::error!("JS error in tab {}: {}", tab_id, msg);
                Err(anyhow::anyhow!("JS error: {}", msg))
            }
        }
    }

    pub fn eval_with_result(&mut self, tab_id: TabId, code: &str) -> Result<JsValue> {
        let context = self.contexts.get_mut(&tab_id)
            .ok_or_else(|| anyhow::anyhow!("No JS context for tab {}", tab_id))?;

        context.eval(Source::from_bytes(code))
            .map_err(|e| anyhow::anyhow!("JS error: {}", e))
    }

    pub fn call_function(
        &mut self,
        tab_id: TabId,
        function_name: &str,
        args: &[JsValue],
    ) -> Result<JsValue> {
        let context = self.contexts.get_mut(&tab_id)
            .ok_or_else(|| anyhow::anyhow!("No JS context for tab {}", tab_id))?;

        let global = context.global_object();
        let name = boa_engine::js_string!(function_name);
        let func = global.get(name, context)
            .map_err(|e| anyhow::anyhow!("Failed to get function {}: {}", function_name, e))?;

        func.as_callable()
            .ok_or_else(|| anyhow::anyhow!("{} is not a function", function_name))?
            .call(&global.into(), args, context)
            .map_err(|e| anyhow::anyhow!("Failed to call {}: {}", function_name, e))
    }

    pub fn schedule_task(
        &mut self,
        tab_id: TabId,
        callback_id: u32,
        delay: Option<Duration>,
        repeating: bool,
    ) -> u64 {
        let task_id = self.next_task_id;
        self.next_task_id += 1;

        let task = JsTask {
            id: task_id,
            tab_id,
            callback_id,
            delay,
            created_at: Instant::now(),
            repeating,
            interval: delay,
        };

        self.task_queue.lock().unwrap().push(task);
        task_id
    }

    pub fn cancel_task(&mut self, task_id: u64) {
        self.task_queue.lock().unwrap().retain(|t| t.id != task_id);
    }

    pub fn get_pending_tasks(&self, tab_id: TabId) -> Vec<JsTask> {
        let now = Instant::now();
        self.task_queue.lock().unwrap()
            .iter()
            .filter(|t| t.tab_id == tab_id)
            .filter(|t| {
                match t.delay {
                    Some(delay) => now.duration_since(t.created_at) >= delay,
                    None => true,
                }
            })
            .cloned()
            .collect()
    }

    pub fn remove_task(&mut self, task_id: u64) {
        self.task_queue.lock().unwrap().retain(|t| t.id != task_id);
    }

    pub fn has_context(&self, tab_id: TabId) -> bool {
        self.contexts.contains_key(&tab_id)
    }

    pub fn get_context(&mut self, tab_id: TabId) -> Option<&mut Context> {
        self.contexts.get_mut(&tab_id)
    }
}
