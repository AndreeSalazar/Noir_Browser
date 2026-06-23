//! CORS - Cross-Origin Resource Sharing
//!
//! Valida requests cross-origin contra headers CORS.

use std::collections::HashSet;

#[derive(Debug, Clone, Default)]
pub struct CorsConfig {
    pub allowed_origins: HashSet<String>,
    pub allowed_methods: HashSet<String>,
    pub allowed_headers: HashSet<String>,
    pub allow_credentials: bool,
}

impl CorsConfig {
    pub fn new() -> Self {
        Self::default()
    }

    /// Verifica si un origin está permitido
    pub fn is_origin_allowed(&self, origin: &str) -> bool {
        if self.allowed_origins.is_empty() {
            return false;
        }
        self.allowed_origins.contains("*")
            || self.allowed_origins.contains(origin)
    }

    /// Verifica si un método está permitido
    pub fn is_method_allowed(&self, method: &str) -> bool {
        self.allowed_methods.is_empty()
            || self.allowed_methods.contains("*")
            || self.allowed_methods.contains(method)
    }

    /// Genera header Access-Control-Allow-Origin
    pub fn allow_origin_header(&self, origin: &str) -> Option<String> {
        if self.is_origin_allowed(origin) {
            if self.allow_credentials {
                Some(origin.to_string())
            } else if self.allowed_origins.contains("*") {
                Some("*".to_string())
            } else {
                Some(origin.to_string())
            }
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CorsError {
    OriginNotAllowed,
    MethodNotAllowed,
    CredentialsNotAllowed,
}

impl std::fmt::Display for CorsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CorsError::OriginNotAllowed => write!(f, "Origin not allowed"),
            CorsError::MethodNotAllowed => write!(f, "Method not allowed"),
            CorsError::CredentialsNotAllowed => write!(f, "Credentials not allowed"),
        }
    }
}

impl std::error::Error for CorsError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cors_wildcard() {
        let mut config = CorsConfig::new();
        config.allowed_origins.insert("*".to_string());
        assert!(config.is_origin_allowed("https://any.com"));
    }

    #[test]
    fn test_cors_specific_origin() {
        let mut config = CorsConfig::new();
        config.allowed_origins.insert("https://example.com".to_string());
        assert!(config.is_origin_allowed("https://example.com"));
        assert!(!config.is_origin_allowed("https://other.com"));
    }

    #[test]
    fn test_cors_method() {
        let mut config = CorsConfig::new();
        config.allowed_methods.insert("GET".to_string());
        config.allowed_methods.insert("POST".to_string());
        assert!(config.is_method_allowed("GET"));
        assert!(config.is_method_allowed("POST"));
        assert!(!config.is_method_allowed("DELETE"));
    }

    #[test]
    fn test_cors_allow_origin_header() {
        let mut config = CorsConfig::new();
        config.allowed_origins.insert("https://example.com".to_string());
        let h = config.allow_origin_header("https://example.com");
        assert_eq!(h, Some("https://example.com".to_string()));
    }

    #[test]
    fn test_cors_empty() {
        let config = CorsConfig::new();
        assert!(!config.is_origin_allowed("https://example.com"));
    }

    #[test]
    fn test_cors_credentials() {
        let mut config = CorsConfig::new();
        config.allowed_origins.insert("https://example.com".to_string());
        config.allow_credentials = true;
        let h = config.allow_origin_header("https://example.com");
        assert_eq!(h, Some("https://example.com".to_string()));
    }
}
