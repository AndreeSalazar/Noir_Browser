//! URL - URL parsing seguro (anti-spoofing)
//!
//! Wrapper alrededor de `url::Url` con validaciones de seguridad.

use std::fmt;

#[derive(Debug, Clone)]
pub struct SafeUrl {
    url: url::Url,
    raw: String,
}

impl SafeUrl {
    pub fn parse(s: &str) -> Result<Self, UrlError> {
        // Detectar caracteres de control / unicode sospechoso
        for c in s.chars() {
            if c.is_control() {
                return Err(UrlError::ControlChar);
            }
        }

        let url = url::Url::parse(s).map_err(|_| UrlError::InvalidUrl)?;

        // No permitir javascript: o data: en links de navegación
        match url.scheme() {
            "javascript" | "data" | "vbscript" | "file" => {
                return Err(UrlError::DangerousScheme);
            }
            _ => {}
        }

        Ok(SafeUrl { url, raw: s.to_string() })
    }

    pub fn scheme(&self) -> &str { self.url.scheme() }
    pub fn host_str(&self) -> Option<&str> { self.url.host_str() }
    pub fn path(&self) -> &str { self.url.path() }
    pub fn as_str(&self) -> &str { &self.raw }
}

impl fmt::Display for SafeUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.url)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum UrlError {
    InvalidUrl,
    ControlChar,
    DangerousScheme,
}

impl fmt::Display for UrlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UrlError::InvalidUrl => write!(f, "Invalid URL"),
            UrlError::ControlChar => write!(f, "URL contains control characters"),
            UrlError::DangerousScheme => write!(f, "URL uses dangerous scheme"),
        }
    }
}

impl std::error::Error for UrlError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_url_parse_https() {
        let u = SafeUrl::parse("https://example.com/path").unwrap();
        assert_eq!(u.scheme(), "https");
        assert_eq!(u.host_str(), Some("example.com"));
    }

    #[test]
    fn test_safe_url_blocks_javascript() {
        assert!(SafeUrl::parse("javascript:alert(1)").is_err());
    }

    #[test]
    fn test_safe_url_blocks_data() {
        assert!(SafeUrl::parse("data:text/html,<script>").is_err());
    }

    #[test]
    fn test_safe_url_blocks_vbscript() {
        assert!(SafeUrl::parse("vbscript:msgbox(1)").is_err());
    }

    #[test]
    fn test_safe_url_blocks_control_chars() {
        assert!(SafeUrl::parse("https://example.com/\x00").is_err());
    }

    #[test]
    fn test_safe_url_blocks_file() {
        assert!(SafeUrl::parse("file:///etc/passwd").is_err());
    }

    #[test]
    fn test_safe_url_allows_http_https() {
        assert!(SafeUrl::parse("http://example.com").is_ok());
        assert!(SafeUrl::parse("https://example.com").is_ok());
    }
}
