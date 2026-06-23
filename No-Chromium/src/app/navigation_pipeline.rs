//! Navigation Pipeline (Chrome Blink-inspired)
//!
//! Basado en el flujo de navegacion de Chrome:
//! 1. UI thread: "is this URL or search query?"
//! 2. Start navigation: network thread hace DNS + TLS
//! 3. Read response: MIME type sniffing, SafeBrowsing check, CORB
//! 4. Find renderer process (parallel con network request)
//! 5. Commit navigation: IPC al renderer
//! 6. Initial load: parse, style, layout, paint
//!
//! Aplicado a Noir: implementar el state machine completo de navegacion

use std::collections::HashMap;

/// Estado del navigation pipeline (Chrome-style)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NavState {
    Idle,
    InputReceived,
    DnsLookup,
    TlsHandshake,
    SendingRequest,
    ReceivingResponse,
    SafeBrowsingCheck,
    CorbCheck,
    RendererProcessFound,
    Committing,
    Loading,
    Interactive,
    Complete,
    Error,
}

impl NavState {
    pub fn name(&self) -> &'static str {
        match self {
            NavState::Idle => "Idle",
            NavState::InputReceived => "Input received",
            NavState::DnsLookup => "DNS lookup",
            NavState::TlsHandshake => "TLS handshake",
            NavState::SendingRequest => "Sending request",
            NavState::ReceivingResponse => "Receiving response",
            NavState::SafeBrowsingCheck => "Safe Browsing check",
            NavState::CorbCheck => "CORB check",
            NavState::RendererProcessFound => "Renderer process found",
            NavState::Committing => "Committing navigation",
            NavState::Loading => "Loading",
            NavState::Interactive => "Interactive (DOM ready)",
            NavState::Complete => "Complete (all resources)",
            NavState::Error => "Error",
        }
    }
}

/// Tipo de input
#[derive(Debug, Clone, PartialEq)]
pub enum InputType {
    Url(String),
    SearchQuery(String),
    BackNavigation,
    ForwardNavigation,
    Reload,
}

/// Resultado de un check
#[derive(Debug, Clone, PartialEq)]
pub enum CheckResult {
    Pass,
    Fail(String),
    Redirect(String),
}

/// Frame Tree (Chrome-style)
#[derive(Debug, Clone)]
pub struct FrameTree {
    pub root_frame: Frame,
    pub frames: HashMap<u32, Frame>,
    pub next_frame_id: u32,
}

#[derive(Debug, Clone)]
pub struct Frame {
    pub id: u32,
    pub parent_id: Option<u32>,
    pub url: String,
    pub title: String,
    pub is_main_frame: bool,
}

/// Navigation pipeline (Chrome-style)
#[derive(Debug)]
pub struct NavigationPipeline {
    pub state: NavState,
    pub current_url: String,
    pub input_type: Option<InputType>,
    pub frame_id: u32,
    pub frame_tree: FrameTree,
    pub redirects: Vec<String>,
    pub safe_browsing_result: Option<CheckResult>,
    pub corb_result: Option<CheckResult>,
    pub loading_start_ms: u64,
    pub dom_ready_ms: Option<u64>,
    pub complete_ms: Option<u64>,
    pub errors: Vec<String>,
    pub history: Vec<String>,
    pub history_index: usize,
}

impl NavigationPipeline {
    pub fn new() -> Self {
        let mut frame_tree = FrameTree {
            root_frame: Frame {
                id: 0, parent_id: None, url: String::new(),
                title: "New Tab".into(), is_main_frame: true,
            },
            frames: HashMap::new(),
            next_frame_id: 1,
        };
        frame_tree.frames.insert(0, frame_tree.root_frame.clone());
        Self {
            state: NavState::Idle,
            current_url: String::new(),
            input_type: None,
            frame_id: 0,
            frame_tree,
            redirects: Vec::new(),
            safe_browsing_result: None,
            corb_result: None,
            loading_start_ms: 0,
            dom_ready_ms: None,
            complete_ms: None,
            errors: Vec::new(),
            history: Vec::new(),
            history_index: 0,
        }
    }

    /// Determina si el input es URL o search query (Chrome omnibox)
    pub fn classify_input(&mut self, input: &str) -> InputType {
        if input.is_empty() {
            return InputType::SearchQuery(input.to_string());
        }
        let lower = input.to_lowercase();
        if lower.starts_with("http://") || lower.starts_with("https://")
            || lower.starts_with("file://") || lower.starts_with("about:")
            || lower.starts_with("data:") {
            InputType::Url(input.to_string())
        } else if input.contains(' ') || !input.contains('.') {
            // Es search query
            InputType::SearchQuery(input.to_string())
        } else if input.ends_with(".com") || input.ends_with(".org")
            || input.ends_with(".net") || input.ends_with(".io")
            || input.ends_with(".dev") {
            // Asumir URL con https://
            InputType::Url(format!("https://{}", input))
        } else {
            InputType::Url(format!("https://{}", input))
        }
    }

    /// Inicia navegacion
    pub fn navigate(&mut self, input: &str, current_time_ms: u64) {
        self.input_type = Some(self.classify_input(input));
        self.state = NavState::InputReceived;
        self.loading_start_ms = current_time_ms;
        self.dom_ready_ms = None;
        self.complete_ms = None;
        self.errors.clear();
        self.redirects.clear();

        match self.input_type.as_ref().unwrap() {
            InputType::Url(url) => {
                self.current_url = url.clone();
            }
            InputType::SearchQuery(q) => {
                self.current_url = format!("https://duckduckgo.com/?q={}", urlencoding_simple(q));
            }
            _ => {}
        }
    }

    /// Back navigation
    pub fn go_back(&mut self) -> bool {
        if self.history_index > 0 {
            self.history_index -= 1;
            self.current_url = self.history[self.history_index].clone();
            true
        } else {
            false
        }
    }

    /// Forward navigation
    pub fn go_forward(&mut self) -> bool {
        if self.history_index + 1 < self.history.len() {
            self.history_index += 1;
            self.current_url = self.history[self.history_index].clone();
            true
        } else {
            false
        }
    }

    /// Reload
    pub fn reload(&mut self) {
        // No cambia URL, solo re-carga
    }

    /// Marcar navegacion como completa
    pub fn mark_complete(&mut self, current_time_ms: u64) {
        self.complete_ms = Some(current_time_ms);
        self.state = NavState::Complete;
        // Agregar al history
        if self.history.is_empty() || self.history[self.history_index] != self.current_url {
            self.history.truncate(self.history_index + 1);
            self.history.push(self.current_url.clone());
            self.history_index = self.history.len() - 1;
        }
    }

    /// Marcar error
    pub fn mark_error(&mut self, error: &str) {
        self.errors.push(error.to_string());
        self.state = NavState::Error;
    }

    /// Tiempo total de carga en ms
    pub fn total_load_time_ms(&self) -> Option<u64> {
        match self.complete_ms {
            Some(end) => Some(end - self.loading_start_ms),
            None => None,
        }
    }

    /// Time to Interactive
    pub fn tti_ms(&self) -> Option<u64> {
        match self.dom_ready_ms {
            Some(dom) => Some(dom - self.loading_start_ms),
            None => None,
        }
    }
}

fn urlencoding_simple(s: &str) -> String {
    s.chars().map(|c| {
        if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' || c == '~' {
            c.to_string()
        } else {
            format!("%{:02X}", c as u32)
        }
    }).collect()
}

impl Default for NavigationPipeline {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_creation() {
        let p = NavigationPipeline::new();
        assert_eq!(p.state, NavState::Idle);
    }

    #[test]
    fn test_url_input() {
        let mut p = NavigationPipeline::new();
        let t = p.classify_input("https://example.com");
        assert_eq!(t, InputType::Url("https://example.com".to_string()));
    }

    #[test]
    fn test_search_query() {
        let mut p = NavigationPipeline::new();
        let t = p.classify_input("hello world");
        assert!(matches!(t, InputType::SearchQuery(_)));
    }

    #[test]
    fn test_no_tld_assumes_https() {
        let mut p = NavigationPipeline::new();
        let t = p.classify_input("example.com");
        if let InputType::Url(u) = t {
            assert!(u.starts_with("https://"));
        } else {
            panic!("Expected URL");
        }
    }

    #[test]
    fn test_navigate_to_url() {
        let mut p = NavigationPipeline::new();
        p.navigate("https://example.com", 0);
        assert_eq!(p.current_url, "https://example.com");
        assert_eq!(p.state, NavState::InputReceived);
    }

    #[test]
    fn test_navigate_search() {
        let mut p = NavigationPipeline::new();
        p.navigate("hello world", 0);
        assert!(p.current_url.contains("duckduckgo"));
        assert!(p.current_url.contains("hello"));
    }

    #[test]
    fn test_back_navigation() {
        let mut p = NavigationPipeline::new();
        p.navigate("https://a.com", 0);
        p.mark_complete(100);
        p.navigate("https://b.com", 200);
        p.mark_complete(300);
        assert!(p.go_back());
        assert_eq!(p.current_url, "https://a.com");
    }

    #[test]
    fn test_forward_navigation() {
        let mut p = NavigationPipeline::new();
        p.navigate("https://a.com", 0);
        p.mark_complete(100);
        p.navigate("https://b.com", 200);
        p.mark_complete(300);
        p.go_back();
        assert!(p.go_forward());
        assert_eq!(p.current_url, "https://b.com");
    }

    #[test]
    fn test_back_at_start_fails() {
        let mut p = NavigationPipeline::new();
        p.navigate("https://a.com", 0);
        p.mark_complete(100);
        assert!(!p.go_back());
    }

    #[test]
    fn test_total_load_time() {
        let mut p = NavigationPipeline::new();
        p.navigate("https://a.com", 0);
        p.mark_complete(500);
        assert_eq!(p.total_load_time_ms(), Some(500));
    }

    #[test]
    fn test_state_names() {
        assert_eq!(NavState::Idle.name(), "Idle");
        assert_eq!(NavState::Loading.name(), "Loading");
        assert_eq!(NavState::Complete.name(), "Complete (all resources)");
    }

    #[test]
    fn test_frame_tree() {
        let p = NavigationPipeline::new();
        assert_eq!(p.frame_tree.frames.len(), 1);
        assert!(p.frame_tree.frames.contains_key(&0));
    }

    #[test]
    fn test_mark_error() {
        let mut p = NavigationPipeline::new();
        p.navigate("https://a.com", 0);
        p.mark_error("DNS failed");
        assert_eq!(p.state, NavState::Error);
        assert_eq!(p.errors.len(), 1);
    }
}
