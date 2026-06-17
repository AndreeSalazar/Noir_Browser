use boa_engine::{Context, Source};
use std::collections::HashMap;

pub type TabId = u64;

#[derive(Debug, Clone)]
pub struct ModuleRecord {
    pub specifier: String,
    pub source: String,
    pub imports: Vec<String>,
    pub exports: Vec<String>,
    pub loaded: bool,
}

pub struct ModuleSystem {
    modules: HashMap<String, ModuleRecord>,
    tab_modules: HashMap<TabId, Vec<String>>,
}

impl ModuleSystem {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
            tab_modules: HashMap::new(),
        }
    }

    pub fn register_module(&mut self, specifier: &str, source: &str) {
        let record = ModuleRecord {
            specifier: specifier.to_string(),
            source: source.to_string(),
            imports: Self::extract_imports(source),
            exports: Self::extract_exports(source),
            loaded: false,
        };
        self.modules.insert(specifier.to_string(), record);
        tracing::debug!("Registered module: {}", specifier);
    }

    pub fn load_module(&mut self, tab_id: TabId, specifier: &str, context: &mut Context) -> Result<(), String> {
        let source = self.modules.get(specifier)
            .map(|m| m.source.clone())
            .ok_or_else(|| format!("Module not found: {}", specifier))?;

        let result = context.eval(Source::from_bytes(&source));
        match result {
            Ok(_) => {
                if let Some(m) = self.modules.get_mut(specifier) {
                    m.loaded = true;
                }
                self.tab_modules.entry(tab_id).or_insert_with(Vec::new).push(specifier.to_string());
                tracing::info!("Loaded module '{}' for tab {}", specifier, tab_id);
                Ok(())
            }
            Err(e) => Err(format!("Failed to load module '{}': {}", specifier, e)),
        }
    }

    pub fn get_module(&self, specifier: &str) -> Option<ModuleRecord> {
        self.modules.get(specifier).cloned()
    }

    pub fn get_tab_modules(&self, tab_id: TabId) -> Vec<String> {
        self.tab_modules.get(&tab_id).cloned().unwrap_or_default()
    }

    pub fn is_module_loaded(&self, specifier: &str) -> bool {
        self.modules.get(specifier).map(|m| m.loaded).unwrap_or(false)
    }

    pub fn resolve_specifier(&self, base: &str, specifier: &str) -> String {
        if specifier.starts_with("http://") || specifier.starts_with("https://") {
            return specifier.to_string();
        }
        if specifier.starts_with("./") || specifier.starts_with("../") {
            let base_dir = std::path::Path::new(base)
                .parent()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();
            return format!("{}/{}", base_dir, specifier);
        }
        specifier.to_string()
    }

    fn extract_imports(source: &str) -> Vec<String> {
        let mut imports = Vec::new();
        for line in source.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("import ") {
                if let Some(specifier) = trimmed.strip_prefix("import ").and_then(|s| {
                    s.split(" from ").last().or_else(|| s.split_whitespace().nth(1))
                }) {
                    let cleaned = specifier.trim().trim_matches('\'').trim_matches('"');
                    if !cleaned.is_empty() {
                        imports.push(cleaned.to_string());
                    }
                }
            }
        }
        imports
    }

    fn extract_exports(source: &str) -> Vec<String> {
        let mut exports = Vec::new();
        for line in source.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("export ") {
                if let Some(name) = trimmed.strip_prefix("export ").and_then(|s| {
                    s.split_whitespace().nth(1).or_else(|| s.split('(').next())
                }) {
                    let cleaned = name.trim().trim_matches('{').trim_matches('}').trim();
                    if !cleaned.is_empty() && cleaned != "default" {
                        exports.push(cleaned.to_string());
                    }
                }
            }
        }
        exports
    }
}
