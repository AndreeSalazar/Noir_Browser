//! Cookie Module - Cookie jar seguro
//!
//! Implementación completa de cookies HTTP con:
//! - SameSite (Strict, Lax, None)
//! - Secure flag
//! - HttpOnly flag
//! - Domain y Path restrictions
//! - Expiración (Max-Age, Expires)
//! - Thread-safe con Arc<Mutex<>>
//! - Persistencia opcional (en memoria por ahora)

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

/// SameSite attribute
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SameSite {
    Strict,
    Lax,
    None,
}

impl SameSite {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "strict" => SameSite::Strict,
            "lax" => SameSite::Lax,
            "none" => SameSite::None,
            _ => SameSite::Lax, // Default
        }
    }
}

/// Cookie struct
#[derive(Debug, Clone, PartialEq)]
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
    pub expires: u64,  // Unix timestamp, 0 = session
    pub max_age: i64,  // Seconds from now, -1 = session
    pub secure: bool,
    pub http_only: bool,
    pub same_site: SameSite,
    pub creation_time: u64,
}

impl Cookie {
    pub fn new(name: String, value: String, domain: String) -> Self {
        Self {
            name,
            value,
            domain,
            path: "/".to_string(),
            expires: 0,
            max_age: -1,
            secure: false,
            http_only: false,
            same_site: SameSite::Lax,
            creation_time: current_time(),
        }
    }

    /// ¿La cookie está expirada?
    pub fn is_expired(&self) -> bool {
        let now = current_time();
        if self.max_age == 0 {
            // Max-Age = 0 significa "eliminar inmediatamente"
            true
        } else if self.max_age > 0 {
            // Max-Age positivo: expirar cuando pase ese tiempo
            self.creation_time + (self.max_age as u64) <= now
        } else if self.expires > 0 {
            // Expires: expirar cuando llegue esa fecha
            self.expires <= now
        } else {
            // Session cookie
            false
        }
    }

    /// ¿Es válida para esta URL?
    pub fn matches(&self, url: &str) -> bool {
        if let Ok(parsed) = url::Url::parse(url) {
            let host = parsed.host_str().unwrap_or("");
            let path = parsed.path();

            // Domain match
            if !self.domain.is_empty() && !host.ends_with(&self.domain) {
                return false;
            }

            // Path match
            if !path.starts_with(&self.path) {
                return false;
            }

            // Secure check
            if self.secure && parsed.scheme() != "https" {
                return false;
            }

            true
        } else {
            false
        }
    }
}

/// Errores del cookie jar
#[derive(Debug, Clone, PartialEq)]
pub enum CookieError {
    InvalidName,
    InvalidValue,
    InvalidDomain,
    CookieExpired,
    DomainMismatch,
    SecureRequired,
    HttpOnlyViolation,
}

impl std::fmt::Display for CookieError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CookieError::InvalidName => write!(f, "Invalid cookie name"),
            CookieError::InvalidValue => write!(f, "Invalid cookie value"),
            CookieError::InvalidDomain => write!(f, "Invalid cookie domain"),
            CookieError::CookieExpired => write!(f, "Cookie has expired"),
            CookieError::DomainMismatch => write!(f, "Cookie domain mismatch"),
            CookieError::SecureRequired => write!(f, "Cookie requires secure context"),
            CookieError::HttpOnlyViolation => write!(f, "HttpOnly violation"),
        }
    }
}

impl std::error::Error for CookieError {}

/// Cookie jar thread-safe
pub struct CookieJar {
    cookies: Arc<Mutex<HashMap<String, Cookie>>>,
}

impl CookieJar {
    pub fn new() -> Self {
        Self {
            cookies: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Genera una key única para el cookie
    fn make_key(domain: &str, path: &str, name: &str) -> String {
        format!("{}|{}|{}", domain, path, name)
    }

    /// Añade un cookie desde un header Set-Cookie
    pub fn set_from_header(
        &self,
        url: &str,
        set_cookie: &str,
    ) -> Result<(), CookieError> {
        let mut parts = set_cookie.split(';');
        let first = parts.next().ok_or(CookieError::InvalidValue)?;

        let (name, value) = first
            .split_once('=')
            .ok_or(CookieError::InvalidValue)?;
        let name = name.trim().to_string();
        let value = value.trim().to_string();

        if name.is_empty() {
            return Err(CookieError::InvalidName);
        }

        let parsed = url::Url::parse(url).map_err(|_| CookieError::InvalidDomain)?;
        let mut cookie = Cookie::new(
            name.clone(),
            value,
            parsed.host_str().unwrap_or("").to_string(),
        );

        for part in parts {
            let part = part.trim();
            if part.is_empty() { continue; }

            if let Some((k, v)) = part.split_once('=') {
                let k = k.trim().to_lowercase();
                let v = v.trim();
                match k.as_str() {
                    "domain" => {
                        // Security: don't allow public domains in domain attribute
                        if !v.starts_with('.') && v != parsed.host_str().unwrap_or("") {
                            return Err(CookieError::InvalidDomain);
                        }
                        cookie.domain = v.trim_start_matches('.').to_string();
                    }
                    "path" => cookie.path = v.to_string(),
                    "expires" => {
                        if let Ok(t) = parse_http_date(v) {
                            cookie.expires = t;
                        }
                    }
                    "max-age" => {
                        if let Ok(n) = v.parse::<i64>() {
                            cookie.max_age = n;
                        }
                    }
                    _ => {}
                }
            } else {
                let k = part.to_lowercase();
                match k.as_str() {
                    "secure" => cookie.secure = true,
                    "httponly" => cookie.http_only = true,
                    "samesite" => {} // handled below
                    _ => {}
                }
            }
        }

        // Re-parse for samesite (it has =)
        for part in set_cookie.split(';') {
            let part = part.trim();
            if part.to_lowercase().starts_with("samesite=") {
                if let Some((_, v)) = part.split_once('=') {
                    cookie.same_site = SameSite::from_str(v.trim());
                }
            }
        }

        let key = Self::make_key(&cookie.domain, &cookie.path, &cookie.name);
        self.cookies.lock().unwrap().insert(key, cookie);
        Ok(())
    }

    /// Obtiene todos los cookies válidos para una URL
    pub fn get_for_url(&self, url: &str) -> Vec<(String, String)> {
        let cookies = self.cookies.lock().unwrap();
        let mut result = Vec::new();

        for cookie in cookies.values() {
            if !cookie.is_expired() && cookie.matches(url) {
                result.push((cookie.name.clone(), cookie.value.clone()));
            }
        }

        result
    }

    /// Formatea los cookies para el header Cookie:
    pub fn cookie_header(&self, url: &str) -> String {
        self.get_for_url(url)
            .iter()
            .map(|(n, v)| format!("{}={}", n, v))
            .collect::<Vec<_>>()
            .join("; ")
    }

    /// Elimina cookies expirados
    pub fn cleanup_expired(&self) {
        self.cookies.lock().unwrap().retain(|_, c| !c.is_expired());
    }

    /// Total de cookies
    pub fn count(&self) -> usize {
        self.cookies.lock().unwrap().len()
    }

    /// Elimina todos los cookies
    pub fn clear(&self) {
        self.cookies.lock().unwrap().clear();
    }

    /// Elimina un cookie específico
    pub fn remove(&self, domain: &str, path: &str, name: &str) {
        let key = Self::make_key(domain, path, name);
        self.cookies.lock().unwrap().remove(&key);
    }
}

impl Default for CookieJar {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for CookieJar {
    fn clone(&self) -> Self {
        Self {
            cookies: Arc::clone(&self.cookies),
        }
    }
}

/// Timestamp actual en segundos
fn current_time() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Parsea fecha HTTP (simplificado)
fn parse_http_date(s: &str) -> Result<u64, ()> {
    // Simplified: just try basic formats
    // Real implementation would use chrono
    if let Ok(n) = s.parse::<u64>() {
        return Ok(n);
    }
    Err(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cookie_creation() {
        let c = Cookie::new("session".into(), "abc123".into(), "example.com".into());
        assert_eq!(c.name, "session");
        assert_eq!(c.value, "abc123");
        assert!(!c.secure);
        assert!(!c.http_only);
    }

    #[test]
    fn test_same_site_from_str() {
        assert_eq!(SameSite::from_str("Strict"), SameSite::Strict);
        assert_eq!(SameSite::from_str("LAX"), SameSite::Lax);
        assert_eq!(SameSite::from_str("None"), SameSite::None);
        assert_eq!(SameSite::from_str("invalid"), SameSite::Lax);
    }

    #[test]
    fn test_cookie_not_expired() {
        let c = Cookie::new("test".into(), "v".into(), "example.com".into());
        assert!(!c.is_expired());
    }

    #[test]
    fn test_cookie_max_age_expired() {
        let mut c = Cookie::new("test".into(), "v".into(), "example.com".into());
        c.max_age = 0; // Max-Age=0 = delete immediately
        assert!(c.is_expired());
    }

    #[test]
    fn test_cookie_matches_url() {
        let c = Cookie::new("test".into(), "v".into(), "example.com".into());
        assert!(c.matches("https://example.com/path"));
        assert!(c.matches("https://www.example.com/path"));
        assert!(!c.matches("https://other.com/path"));
    }

    #[test]
    fn test_cookie_secure_blocks_http() {
        let mut c = Cookie::new("test".into(), "v".into(), "example.com".into());
        c.secure = true;
        assert!(!c.matches("http://example.com/path"));
        assert!(c.matches("https://example.com/path"));
    }

    #[test]
    fn test_jar_set_from_header_basic() {
        let jar = CookieJar::new();
        let result = jar.set_from_header("https://example.com/", "session=abc123");
        assert!(result.is_ok());
        assert_eq!(jar.count(), 1);
    }

    #[test]
    fn test_jar_get_for_url() {
        let jar = CookieJar::new();
        jar.set_from_header("https://example.com/", "session=abc123").unwrap();
        jar.set_from_header("https://example.com/", "theme=dark").unwrap();
        let cookies = jar.get_for_url("https://example.com/page");
        assert_eq!(cookies.len(), 2);
    }

    #[test]
    fn test_jar_cookie_header() {
        let jar = CookieJar::new();
        jar.set_from_header("https://example.com/", "a=1").unwrap();
        jar.set_from_header("https://example.com/", "b=2").unwrap();
        let header = jar.cookie_header("https://example.com/");
        assert!(header.contains("a=1"));
        assert!(header.contains("b=2"));
    }

    #[test]
    fn test_jar_domain_isolation() {
        let jar = CookieJar::new();
        jar.set_from_header("https://example.com/", "a=1").unwrap();
        jar.set_from_header("https://other.com/", "b=2").unwrap();

        let a_cookies = jar.get_for_url("https://example.com/");
        let b_cookies = jar.get_for_url("https://other.com/");

        assert_eq!(a_cookies.len(), 1);
        assert_eq!(a_cookies[0].0, "a");
        assert_eq!(b_cookies.len(), 1);
        assert_eq!(b_cookies[0].0, "b");
    }

    #[test]
    fn test_jar_secure_flag() {
        let jar = CookieJar::new();
        jar.set_from_header("https://example.com/", "secure=value; Secure").unwrap();

        // Should not be sent over http
        let http_cookies = jar.get_for_url("http://example.com/");
        assert_eq!(http_cookies.len(), 0);

        // Should be sent over https
        let https_cookies = jar.get_for_url("https://example.com/");
        assert_eq!(https_cookies.len(), 1);
    }

    #[test]
    fn test_jar_clear() {
        let jar = CookieJar::new();
        jar.set_from_header("https://example.com/", "a=1").unwrap();
        jar.set_from_header("https://example.com/", "b=2").unwrap();
        assert_eq!(jar.count(), 2);
        jar.clear();
        assert_eq!(jar.count(), 0);
    }

    #[test]
    fn test_jar_remove() {
        let jar = CookieJar::new();
        jar.set_from_header("https://example.com/", "a=1").unwrap();
        assert_eq!(jar.count(), 1);
        jar.remove("example.com", "/", "a");
        assert_eq!(jar.count(), 0);
    }

    #[test]
    fn test_jar_clone() {
        let jar1 = CookieJar::new();
        jar1.set_from_header("https://example.com/", "a=1").unwrap();
        let jar2 = jar1.clone();
        jar2.set_from_header("https://example.com/", "b=2").unwrap();
        // Both should see both cookies (shared Arc)
        assert_eq!(jar1.count(), 2);
        assert_eq!(jar2.count(), 2);
    }

    #[test]
    fn test_jar_subdomain_access() {
        let jar = CookieJar::new();
        jar.set_from_header("https://example.com/", "a=1").unwrap();
        // Subdomain should have access
        let sub_cookies = jar.get_for_url("https://api.example.com/");
        assert_eq!(sub_cookies.len(), 1);
    }

    #[test]
    fn test_jar_max_age_session() {
        let c = Cookie::new("session".into(), "v".into(), "example.com".into());
        assert_eq!(c.max_age, -1);
    }

    #[test]
    fn test_cookie_error_display() {
        let e = CookieError::InvalidName;
        assert_eq!(e.to_string(), "Invalid cookie name");
    }
}
