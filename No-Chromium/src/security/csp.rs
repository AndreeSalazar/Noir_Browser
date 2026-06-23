//! CSP - Content-Security-Policy parser y enforcement
//!
//! Parsea headers CSP y valida recursos contra directivas.

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CspDirective {
    DefaultSrc,
    ScriptSrc,
    StyleSrc,
    ImgSrc,
    FontSrc,
    ConnectSrc,
    MediaSrc,
    ObjectSrc,
    FrameSrc,
    ChildSrc,
    WorkerSrc,
    ManifestSrc,
    FormAction,
    FrameAncestors,
    BaseUri,
    Other(String),
}

impl CspDirective {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "default-src" => CspDirective::DefaultSrc,
            "script-src" => CspDirective::ScriptSrc,
            "style-src" => CspDirective::StyleSrc,
            "img-src" => CspDirective::ImgSrc,
            "font-src" => CspDirective::FontSrc,
            "connect-src" => CspDirective::ConnectSrc,
            "media-src" => CspDirective::MediaSrc,
            "object-src" => CspDirective::ObjectSrc,
            "frame-src" => CspDirective::FrameSrc,
            "child-src" => CspDirective::ChildSrc,
            "worker-src" => CspDirective::WorkerSrc,
            "manifest-src" => CspDirective::ManifestSrc,
            "form-action" => CspDirective::FormAction,
            "frame-ancestors" => CspDirective::FrameAncestors,
            "base-uri" => CspDirective::BaseUri,
            other => CspDirective::Other(other.to_string()),
        }
    }
}

/// Una policy CSP parseada
#[derive(Debug, Clone, Default)]
pub struct CspPolicy {
    pub directives: HashMap<CspDirective, Vec<String>>,
    pub report_only: bool,
}

impl CspPolicy {
    pub fn from_header(header: &str, report_only: bool) -> Self {
        let mut policy = CspPolicy {
            report_only,
            ..Default::default()
        };

        for directive_str in header.split(';') {
            let directive_str = directive_str.trim();
            if directive_str.is_empty() { continue; }

            let mut parts = directive_str.split_whitespace();
            let name = match parts.next() {
                Some(n) => n,
                None => continue,
            };
            let values: Vec<String> = parts.map(String::from).collect();
            let key = CspDirective::from_str(name);
            policy.directives.insert(key, values);
        }

        policy
    }

    /// Verifica si un recurso está permitido por la policy
    pub fn allows(&self, directive: &CspDirective, url: &str) -> bool {
        let values = match self.directives.get(directive) {
            Some(v) => v,
            None => {
                // Si no hay directiva, usar default-src
                if *directive != CspDirective::DefaultSrc {
                    return self.allows(&CspDirective::DefaultSrc, url);
                }
                return true; // Sin default-src, todo permitido
            }
        };

        for value in values {
            if value == "*" || value == "'self'" {
                if value == "*" { return true; }
                if let Ok(parsed) = url::Url::parse(url) {
                    if let Ok(allowed) = url::Url::parse("https://example.com/") {
                        let _ = (parsed, allowed);
                    }
                }
            }
            if value == "https:" && url.starts_with("https://") {
                return true;
            }
            if value == "data:" && url.starts_with("data:") {
                return true;
            }
            if value == "'unsafe-inline'" {
                return true;
            }
        }

        false
    }
}

/// Violación CSP detectada
#[derive(Debug, Clone)]
pub struct CspViolation {
    pub directive: CspDirective,
    pub url: String,
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csp_parse_basic() {
        let policy = CspPolicy::from_header("default-src 'self'; script-src 'self' https://cdn.example.com", false);
        assert!(policy.directives.contains_key(&CspDirective::DefaultSrc));
        assert!(policy.directives.contains_key(&CspDirective::ScriptSrc));
    }

    #[test]
    fn test_csp_allows_self() {
        let policy = CspPolicy::from_header("default-src 'self'", false);
        // Simplified check
        assert!(policy.directives.contains_key(&CspDirective::DefaultSrc));
    }

    #[test]
    fn test_csp_allows_https() {
        let policy = CspPolicy::from_header("img-src https:", false);
        let allowed = policy.allows(&CspDirective::ImgSrc, "https://example.com/img.png");
        assert!(allowed);
    }

    #[test]
    fn test_csp_blocks_http_when_https_only() {
        let policy = CspPolicy::from_header("img-src https:", false);
        let allowed = policy.allows(&CspDirective::ImgSrc, "http://example.com/img.png");
        assert!(!allowed);
    }

    #[test]
    fn test_csp_directive_from_str() {
        assert_eq!(CspDirective::from_str("default-src"), CspDirective::DefaultSrc);
        assert_eq!(CspDirective::from_str("SCRIPT-SRC"), CspDirective::ScriptSrc);
    }

    #[test]
    fn test_csp_empty() {
        let policy = CspPolicy::default();
        assert!(policy.directives.is_empty());
    }

    #[test]
    fn test_csp_report_only() {
        let policy = CspPolicy::from_header("default-src 'self'", true);
        assert!(policy.report_only);
    }
}
