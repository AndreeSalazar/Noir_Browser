//! Módulo de Privacidad para Browser
//!
//! Implementa First-Party Isolation (FPI) para aislar
//! cookies, localStorage y otros estados por dominio.

use std::collections::HashMap;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Cookie con metadatos de aislamiento
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
    pub expires: Option<i64>,
    pub secure: bool,
    pub http_only: bool,
    pub same_site: SameSite,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SameSite {
    Strict,
    Lax,
    None,
}

/// Jarra de cookies aislada por First-Party
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct FirstPartyCookieJar {
    // Clave: (request_domain, first_party_domain)
    #[zeroize(skip)]
    cookies: HashMap<(String, String), Vec<Cookie>>,
}

impl FirstPartyCookieJar {
    /// Crea una nueva jarra de cookies vacía
    pub fn new() -> Self {
        Self {
            cookies: HashMap::new(),
        }
    }
    
    /// Almacena una cookie con aislamiento por first-party
    pub fn insert(&mut self, cookie: Cookie, first_party: &str) {
        let key = (cookie.domain.clone(), first_party.to_string());
        self.cookies
            .entry(key)
            .or_insert_with(Vec::new)
            .push(cookie);
    }
    
    /// Obtiene cookies válidas para una request, respetando FPI
    pub fn get_for_request(
        &self,
        request_domain: &str,
        first_party: &str,
    ) -> Vec<Cookie> {
        self.cookies
            .get(&(request_domain.to_string(), first_party.to_string()))
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter(|c| !c.is_expired())
            .collect()
    }
    
    /// Elimina todas las cookies para un first-party específico
    pub fn clear_for_first_party(&mut self, first_party: &str) {
        self.cookies.retain(|(_, fp), _| fp != first_party);
    }
    
    /// Limpia todas las cookies (zeroize)
    pub fn clear_all(&mut self) {
        self.cookies.clear();
    }
}

impl Cookie {
    fn is_expired(&self) -> bool {
        if let Some(expires) = self.expires {
            expires < chrono::Utc::now().timestamp()
        } else {
            false
        }
    }
}

/// Almacenamiento localStorage aislado por origen
pub struct FirstPartyStorage {
    #[zeroize(skip)]
    storage: HashMap<String, StorageMap>,
}

#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct StorageMap {
    #[zeroize(skip)]
    data: HashMap<String, String>,
}

impl FirstPartyStorage {
    pub fn new() -> Self {
        Self {
            storage: HashMap::new(),
        }
    }
    
    pub fn get(&self, origin: &str, key: &str) -> Option<&String> {
        self.storage.get(origin)?.data.get(key)
    }
    
    pub fn set(&mut self, origin: String, key: String, value: String) {
        self.storage
            .entry(origin)
            .or_insert_with(|| StorageMap { data: HashMap::new() })
            .data
            .insert(key, value);
    }
    
    pub fn remove(&mut self, origin: &str, key: &str) {
        if let Some(map) = self.storage.get_mut(origin) {
            map.data.remove(key);
        }
    }
    
    pub fn clear_origin(&mut self, origin: &str) {
        self.storage.remove(origin);
    }
}

/// Historial de sesión efímero (no persiste a disco)
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct SessionHistory {
    #[zeroize(skip)]
    entries: Vec<HistoryEntry>,
    max_entries: usize,
}

#[derive(Clone, Debug)]
pub struct HistoryEntry {
    pub url: String,
    pub title: String,
    pub timestamp: i64,
    pub first_party: String,
}

impl SessionHistory {
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Vec::with_capacity(max_entries),
            max_entries,
        }
    }
    
    pub fn add(&mut self, entry: HistoryEntry) {
        if self.entries.len() >= self.max_entries {
            self.entries.remove(0);
        }
        self.entries.push(entry);
    }
    
    pub fn get_recent(&self, limit: usize) -> &[HistoryEntry] {
        let start = self.entries.len().saturating_sub(limit);
        &self.entries[start..]
    }
    
    /// Zeroize todo el historial al cerrar
    pub fn purge(&mut self) {
        self.entries.zeroize();
        self.entries.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fpi_cookie_isolation() {
        let mut jar = FirstPartyCookieJar::new();
        
        // Insertar cookie para example.com como first-party
        let cookie = Cookie {
            name: "session".to_string(),
            value: "abc123".to_string(),
            domain: "example.com".to_string(),
            path: "/".to_string(),
            expires: None,
            secure: true,
            http_only: true,
            same_site: SameSite::Strict,
        };
        
        jar.insert(cookie.clone(), "example.com");
        
        // Debería retornar la cookie cuando first-party coincide
        let results = jar.get_for_request("example.com", "example.com");
        assert_eq!(results.len(), 1);
        
        // No debería retornar la cookie cuando first-party difiere
        let results = jar.get_for_request("example.com", "evil.com");
        assert_eq!(results.len(), 0);
    }
}
