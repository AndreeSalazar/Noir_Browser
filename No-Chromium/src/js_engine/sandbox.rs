use std::collections::{HashMap, HashSet};

pub type TabId = u64;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Permission {
    Fetch,
    Dom,
    Storage,
    Console,
    Timers,
    Modules,
    Workers,
    All,
}

#[derive(Debug, Clone)]
pub struct SandboxConfig {
    pub tab_id: TabId,
    pub permissions: HashSet<Permission>,
    pub max_memory_bytes: usize,
    pub max_script_time_ms: u64,
    pub allowed_origins: Vec<String>,
    pub blocked_patterns: Vec<String>,
    pub csp_policy: Option<String>,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        let mut permissions = HashSet::new();
        permissions.insert(Permission::Fetch);
        permissions.insert(Permission::Dom);
        permissions.insert(Permission::Storage);
        permissions.insert(Permission::Console);
        permissions.insert(Permission::Timers);

        Self {
            tab_id: 0,
            permissions,
            max_memory_bytes: 64 * 1024 * 1024,
            max_script_time_ms: 5000,
            allowed_origins: Vec::new(),
            blocked_patterns: Vec::new(),
            csp_policy: None,
        }
    }
}

pub struct TabSandbox {
    configs: HashMap<TabId, SandboxConfig>,
    script_start_times: HashMap<TabId, std::time::Instant>,
    execution_counts: HashMap<TabId, u64>,
}

impl TabSandbox {
    pub fn new() -> Self {
        Self {
            configs: HashMap::new(),
            script_start_times: HashMap::new(),
            execution_counts: HashMap::new(),
        }
    }

    pub fn create_sandbox(&mut self, tab_id: TabId) {
        let config = SandboxConfig { tab_id, ..Default::default() };
        self.configs.insert(tab_id, config);
        self.execution_counts.insert(tab_id, 0);
        tracing::info!("Sandbox created for tab {}", tab_id);
    }

    pub fn remove_sandbox(&mut self, tab_id: TabId) {
        self.configs.remove(&tab_id);
        self.script_start_times.remove(&tab_id);
        self.execution_counts.remove(&tab_id);
        tracing::info!("Sandbox removed for tab {}", tab_id);
    }

    pub fn has_permission(&self, tab_id: TabId, permission: &Permission) -> bool {
        if let Some(config) = self.configs.get(&tab_id) {
            config.permissions.contains(permission) || config.permissions.contains(&Permission::All)
        } else {
            false
        }
    }

    pub fn check_fetch_allowed(&self, tab_id: TabId, url: &str) -> bool {
        if !self.has_permission(tab_id, &Permission::Fetch) {
            return false;
        }
        if let Some(config) = self.configs.get(&tab_id) {
            for blocked in &config.blocked_patterns {
                if url.contains(blocked) {
                    return false;
                }
            }
            if !config.allowed_origins.is_empty() {
                return config.allowed_origins.iter().any(|origin| url.starts_with(origin));
            }
        }
        true
    }

    pub fn start_script_execution(&mut self, tab_id: TabId) -> bool {
        if self.script_start_times.contains_key(&tab_id) {
            return false;
        }
        self.script_start_times.insert(tab_id, std::time::Instant::now());
        true
    }

    pub fn check_script_timeout(&self, tab_id: TabId) -> bool {
        if let Some(start) = self.script_start_times.get(&tab_id) {
            if let Some(config) = self.configs.get(&tab_id) {
                return start.elapsed().as_millis() as u64 > config.max_script_time_ms;
            }
        }
        false
    }

    pub fn end_script_execution(&mut self, tab_id: TabId) {
        self.script_start_times.remove(&tab_id);
        if let Some(count) = self.execution_counts.get_mut(&tab_id) {
            *count += 1;
        }
    }

    pub fn get_execution_count(&self, tab_id: TabId) -> u64 {
        self.execution_counts.get(&tab_id).copied().unwrap_or(0)
    }
}
