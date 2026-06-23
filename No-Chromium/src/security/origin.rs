//! Origin - Same-Origin Policy
//!
//! Implementa la política de mismo origen (SOP) y comparación de orígenes.

use std::fmt;

/// Origin (scheme + host + port)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Origin {
    pub scheme: String,
    pub host: String,
    pub port: u16,
}

impl Origin {
    pub fn from_url(url: &str) -> Result<Self, OriginError> {
        let parsed = url::Url::parse(url).map_err(|_| OriginError::InvalidUrl)?;
        let host = parsed.host_str().ok_or(OriginError::NoHost)?.to_string();
        let port = parsed.port_or_known_default().unwrap_or(0);
        Ok(Origin {
            scheme: parsed.scheme().to_string(),
            host,
            port,
        })
    }

    /// Compara si dos orígenes son iguales (mismo scheme, host, port)
    pub fn same_as(&self, other: &Origin) -> bool {
        self.scheme == other.scheme
            && self.host.eq_ignore_ascii_case(&other.host)
            && self.port == other.port
    }
}

impl fmt::Display for Origin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.port == 0 || self.port == default_port(&self.scheme) {
            write!(f, "{}://{}", self.scheme, self.host)
        } else {
            write!(f, "{}://{}:{}", self.scheme, self.host, self.port)
        }
    }
}

fn default_port(scheme: &str) -> u16 {
    match scheme {
        "http" => 80,
        "https" => 443,
        "ftp" => 21,
        _ => 0,
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum OriginError {
    InvalidUrl,
    NoHost,
}

impl fmt::Display for OriginError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OriginError::InvalidUrl => write!(f, "Invalid URL"),
            OriginError::NoHost => write!(f, "URL has no host"),
        }
    }
}

impl std::error::Error for OriginError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_origin_from_url_https() {
        let o = Origin::from_url("https://example.com/path").unwrap();
        assert_eq!(o.scheme, "https");
        assert_eq!(o.host, "example.com");
        assert_eq!(o.port, 443);
    }

    #[test]
    fn test_origin_from_url_http() {
        let o = Origin::from_url("http://example.com/").unwrap();
        assert_eq!(o.scheme, "http");
        assert_eq!(o.port, 80);
    }

    #[test]
    fn test_origin_from_url_custom_port() {
        let o = Origin::from_url("http://example.com:8080/").unwrap();
        assert_eq!(o.port, 8080);
    }

    #[test]
    fn test_origin_same_as_true() {
        let a = Origin::from_url("https://example.com/a").unwrap();
        let b = Origin::from_url("https://example.com/b").unwrap();
        assert!(a.same_as(&b));
    }

    #[test]
    fn test_origin_same_as_false_scheme() {
        let a = Origin::from_url("https://example.com/").unwrap();
        let b = Origin::from_url("http://example.com/").unwrap();
        assert!(!a.same_as(&b));
    }

    #[test]
    fn test_origin_same_as_false_host() {
        let a = Origin::from_url("https://example.com/").unwrap();
        let b = Origin::from_url("https://other.com/").unwrap();
        assert!(!a.same_as(&b));
    }

    #[test]
    fn test_origin_same_as_case_insensitive_host() {
        let a = Origin::from_url("https://Example.com/").unwrap();
        let b = Origin::from_url("https://example.com/").unwrap();
        assert!(a.same_as(&b));
    }

    #[test]
    fn test_origin_display() {
        let o = Origin::from_url("https://example.com/").unwrap();
        assert_eq!(o.to_string(), "https://example.com");
    }

    #[test]
    fn test_origin_invalid_url() {
        assert!(Origin::from_url("not a url").is_err());
    }
}
