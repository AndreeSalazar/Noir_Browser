//! Network Monitor - Ver requests HTTP

use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RequestStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone)]
pub struct NetworkRequest {
    pub id: u64,
    pub method: String,
    pub url: String,
    pub status: u16,
    pub content_type: Option<String>,
    pub size: u64,
    pub duration_ms: u64,
    pub started: u64,
    pub finished: Option<u64>,
    pub status_kind: RequestStatus,
}

pub struct NetworkMonitor {
    requests: Arc<Mutex<Vec<NetworkRequest>>>,
    next_id: Arc<Mutex<u64>>,
    enabled: Arc<Mutex<bool>>,
}

impl NetworkMonitor {
    pub fn new() -> Self {
        Self {
            requests: Arc::new(Mutex::new(Vec::new())),
            next_id: Arc::new(Mutex::new(1)),
            enabled: Arc::new(Mutex::new(true)),
        }
    }

    /// Inicia tracking de un request
    pub fn track_request(&self, method: &str, url: &str) -> u64 {
        let mut next_id = self.next_id.lock().unwrap();
        let id = *next_id;
        *next_id += 1;

        let request = NetworkRequest {
            id,
            method: method.to_string(),
            url: url.to_string(),
            status: 0,
            content_type: None,
            size: 0,
            duration_ms: 0,
            started: current_time_millis(),
            finished: None,
            status_kind: RequestStatus::Pending,
        };

        self.requests.lock().unwrap().push(request);
        id
    }

    /// Actualiza el estado de un request
    pub fn update_request(
        &self,
        id: u64,
        status: Option<u16>,
        content_type: Option<String>,
        size: Option<u64>,
    ) {
        let mut requests = self.requests.lock().unwrap();
        if let Some(req) = requests.iter_mut().find(|r| r.id == id) {
            if let Some(s) = status { req.status = s; }
            if let Some(ct) = content_type { req.content_type = Some(ct); }
            if let Some(sz) = size { req.size = sz; }
            if let Some(s) = status {
                req.status_kind = if s < 400 { RequestStatus::Completed } else { RequestStatus::Failed };
            }
        }
    }

    /// Marca un request como completado
    pub fn complete_request(&self, id: u64, status: u16, size: u64) {
        let now = current_time_millis();
        let mut requests = self.requests.lock().unwrap();
        if let Some(req) = requests.iter_mut().find(|r| r.id == id) {
            req.status = status;
            req.size = size;
            req.duration_ms = now - req.started;
            req.finished = Some(now);
            req.status_kind = if status < 400 { RequestStatus::Completed } else { RequestStatus::Failed };
        }
    }

    /// Marca como fallido
    pub fn fail_request(&self, id: u64, error: &str) {
        let _ = error;
        let mut requests = self.requests.lock().unwrap();
        if let Some(req) = requests.iter_mut().find(|r| r.id == id) {
            req.status_kind = RequestStatus::Failed;
            req.finished = Some(current_time_millis());
        }
    }

    /// Limpia todos los requests
    pub fn clear(&self) {
        self.requests.lock().unwrap().clear();
    }

    /// Obtiene todos los requests
    pub fn all(&self) -> Vec<NetworkRequest> {
        self.requests.lock().unwrap().clone()
    }

    /// Cuenta total
    pub fn count(&self) -> usize {
        self.requests.lock().unwrap().len()
    }

    /// Filtra por tipo
    pub fn by_type(&self, content_type: &str) -> Vec<NetworkRequest> {
        self.requests.lock().unwrap().iter()
            .filter(|r| r.content_type.as_deref().map(|t| t.contains(content_type)).unwrap_or(false))
            .cloned()
            .collect()
    }

    /// Filtra por status code
    pub fn by_status(&self, status: u16) -> Vec<NetworkRequest> {
        self.requests.lock().unwrap().iter()
            .filter(|r| r.status == status)
            .cloned()
            .collect()
    }

    /// Total bytes transferidos
    pub fn total_bytes(&self) -> u64 {
        self.requests.lock().unwrap().iter().map(|r| r.size).sum()
    }

    /// Habilita/deshabilita
    pub fn set_enabled(&self, enabled: bool) {
        *self.enabled.lock().unwrap() = enabled;
    }

    pub fn is_enabled(&self) -> bool {
        *self.enabled.lock().unwrap()
    }
}

impl Default for NetworkMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for NetworkMonitor {
    fn clone(&self) -> Self {
        Self {
            requests: Arc::clone(&self.requests),
            next_id: Arc::clone(&self.next_id),
            enabled: Arc::clone(&self.enabled),
        }
    }
}

fn current_time_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_monitor_creation() {
        let m = NetworkMonitor::new();
        assert_eq!(m.count(), 0);
        assert!(m.is_enabled());
    }

    #[test]
    fn test_track_request() {
        let m = NetworkMonitor::new();
        let id = m.track_request("GET", "https://test.com/");
        assert_eq!(m.count(), 1);
        assert!(id > 0);
    }

    #[test]
    fn test_complete_request() {
        let m = NetworkMonitor::new();
        let id = m.track_request("GET", "https://test.com/");
        m.complete_request(id, 200, 1024);
        let requests = m.all();
        assert_eq!(requests[0].status, 200);
        assert_eq!(requests[0].size, 1024);
    }

    #[test]
    fn test_fail_request() {
        let m = NetworkMonitor::new();
        let id = m.track_request("GET", "https://test.com/");
        m.fail_request(id, "Connection error");
        let requests = m.all();
        assert_eq!(requests[0].status_kind, RequestStatus::Failed);
    }

    #[test]
    fn test_by_type() {
        let m = NetworkMonitor::new();
        m.track_request("GET", "https://test.com/");
        m.complete_request(1, 200, 1024);
        m.update_request(1, Some(200), Some("text/html".to_string()), None);
        let htmls = m.by_type("text/html");
        assert_eq!(htmls.len(), 1);
    }

    #[test]
    fn test_by_status() {
        let m = NetworkMonitor::new();
        m.track_request("GET", "https://test.com/");
        m.complete_request(1, 404, 0);
        let not_found = m.by_status(404);
        assert_eq!(not_found.len(), 1);
    }

    #[test]
    fn test_total_bytes() {
        let m = NetworkMonitor::new();
        m.track_request("GET", "https://test.com/a");
        m.complete_request(1, 200, 1000);
        m.track_request("GET", "https://test.com/b");
        m.complete_request(2, 200, 2000);
        assert_eq!(m.total_bytes(), 3000);
    }

    #[test]
    fn test_enable_disable() {
        let m = NetworkMonitor::new();
        m.set_enabled(false);
        assert!(!m.is_enabled());
        m.set_enabled(true);
        assert!(m.is_enabled());
    }

    #[test]
    fn test_clear() {
        let m = NetworkMonitor::new();
        m.track_request("GET", "https://test.com/");
        m.track_request("POST", "https://test.com/api");
        assert_eq!(m.count(), 2);
        m.clear();
        assert_eq!(m.count(), 0);
    }
}
