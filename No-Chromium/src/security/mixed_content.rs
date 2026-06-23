//! Mixed Content - Detector de mixed content
//!
//! Detecta cuando un recurso HTTP es cargado en una página HTTPS.

#[derive(Debug, Clone)]
pub struct MixedContentViolation {
    pub page_url: String,
    pub resource_url: String,
    pub severity: MixedContentSeverity,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MixedContentSeverity {
    /// Mixed passive content (images, video)
    Passive,
    /// Mixed active content (scripts, stylesheets)
    Active,
}

pub struct MixedContentChecker;

impl MixedContentChecker {
    /// Verifica si cargar un recurso en una página HTTPS es mixed content
    pub fn check(page_url: &str, resource_url: &str) -> Option<MixedContentViolation> {
        let page_https = page_url.starts_with("https://");
        let resource_http = resource_url.starts_with("http://");

        if page_https && resource_http {
            let is_active = is_active_content(resource_url);
            Some(MixedContentViolation {
                page_url: page_url.to_string(),
                resource_url: resource_url.to_string(),
                severity: if is_active {
                    MixedContentSeverity::Active
                } else {
                    MixedContentSeverity::Passive
                },
            })
        } else {
            None
        }
    }
}

fn is_active_content(url: &str) -> bool {
    let lower = url.to_lowercase();
    lower.ends_with(".js")
        || lower.ends_with(".css")
        || lower.ends_with(".json")
        || lower.ends_with(".xml")
        || lower.contains("/api/")
        || lower.contains("/script")
        || lower.contains("/style")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_https_page_http_resource_is_mixed() {
        let v = MixedContentChecker::check(
            "https://example.com/",
            "http://cdn.example.com/img.png",
        );
        assert!(v.is_some());
    }

    #[test]
    fn test_https_page_https_resource_ok() {
        let v = MixedContentChecker::check(
            "https://example.com/",
            "https://cdn.example.com/img.png",
        );
        assert!(v.is_none());
    }

    #[test]
    fn test_http_page_http_resource_ok() {
        let v = MixedContentChecker::check(
            "http://example.com/",
            "http://other.com/img.png",
        );
        assert!(v.is_none());
    }

    #[test]
    fn test_active_content_detection() {
        assert!(is_active_content("https://example.com/script.js"));
        assert!(is_active_content("https://example.com/style.css"));
        assert!(!is_active_content("https://example.com/img.png"));
    }

    #[test]
    fn test_active_vs_passive_severity() {
        let active = MixedContentChecker::check(
            "https://example.com/",
            "http://evil.com/script.js",
        ).unwrap();
        assert_eq!(active.severity, MixedContentSeverity::Active);

        let passive = MixedContentChecker::check(
            "https://example.com/",
            "http://evil.com/img.png",
        ).unwrap();
        assert_eq!(passive.severity, MixedContentSeverity::Passive);
    }
}
