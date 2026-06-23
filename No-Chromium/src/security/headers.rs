//! Security Headers - HSTS, X-Frame-Options, CSP, etc.

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SecurityLevel {
    Low,
    Medium,
    High,
    Strict,
}

#[derive(Debug, Clone, Default)]
pub struct SecurityHeaders {
    pub hsts_enabled: bool,
    pub hsts_max_age: u64,
    pub hsts_subdomains: bool,
    pub hsts_preload: bool,
    pub x_frame_options: Option<String>,
    pub x_content_type_options: bool,
    pub x_xss_protection: bool,
    pub referrer_policy: Option<String>,
    pub csp: Option<String>,
    pub permissions_policy: Option<String>,
}

impl SecurityHeaders {
    pub fn from_response_headers(headers: &HashMap<String, String>) -> Self {
        let mut sec = SecurityHeaders::default();

        if let Some(hsts) = get_header(headers, "strict-transport-security") {
            sec.hsts_enabled = true;
            // Parse max-age
            for part in hsts.split(';') {
                let part = part.trim();
                if part.starts_with("max-age=") {
                    if let Ok(n) = part[8..].parse::<u64>() {
                        sec.hsts_max_age = n;
                    }
                } else if part == "includeSubDomains" {
                    sec.hsts_subdomains = true;
                } else if part == "preload" {
                    sec.hsts_preload = true;
                }
            }
        }

        sec.x_frame_options = get_header(headers, "x-frame-options");
        sec.x_content_type_options = get_header(headers, "x-content-type-options")
            .map(|_| true).unwrap_or(false);
        sec.x_xss_protection = get_header(headers, "x-xss-protection").is_some();
        sec.referrer_policy = get_header(headers, "referrer-policy");
        sec.csp = get_header(headers, "content-security-policy");
        sec.permissions_policy = get_header(headers, "permissions-policy");

        sec
    }

    pub fn level(&self) -> SecurityLevel {
        if self.hsts_enabled && self.csp.is_some() && self.x_frame_options.is_some() {
            SecurityLevel::Strict
        } else if self.hsts_enabled || self.csp.is_some() {
            SecurityLevel::High
        } else if self.x_frame_options.is_some() {
            SecurityLevel::Medium
        } else {
            SecurityLevel::Low
        }
    }
}

fn get_header(headers: &HashMap<String, String>, name: &str) -> Option<String> {
    let lower_name = name.to_lowercase();
    headers.iter()
        .find(|(k, _)| k.to_lowercase() == lower_name)
        .map(|(_, v)| v.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_headers_default() {
        let h = SecurityHeaders::default();
        assert!(!h.hsts_enabled);
        assert_eq!(h.level(), SecurityLevel::Low);
    }

    #[test]
    fn test_security_headers_hsts() {
        let mut headers = HashMap::new();
        headers.insert("Strict-Transport-Security".to_string(),
                       "max-age=31536000; includeSubDomains".to_string());
        let h = SecurityHeaders::from_response_headers(&headers);
        assert!(h.hsts_enabled);
        assert_eq!(h.hsts_max_age, 31536000);
        assert!(h.hsts_subdomains);
    }

    #[test]
    fn test_security_headers_x_frame() {
        let mut headers = HashMap::new();
        headers.insert("X-Frame-Options".to_string(), "DENY".to_string());
        let h = SecurityHeaders::from_response_headers(&headers);
        assert_eq!(h.x_frame_options, Some("DENY".to_string()));
    }

    #[test]
    fn test_security_level_low() {
        let h = SecurityHeaders::default();
        assert_eq!(h.level(), SecurityLevel::Low);
    }

    #[test]
    fn test_security_level_high() {
        let h = SecurityHeaders {
            hsts_enabled: true,
            ..Default::default()
        };
        assert_eq!(h.level(), SecurityLevel::High);
    }

    #[test]
    fn test_security_level_strict() {
        let h = SecurityHeaders {
            hsts_enabled: true,
            csp: Some("default-src 'self'".to_string()),
            x_frame_options: Some("DENY".to_string()),
            ..Default::default()
        };
        assert_eq!(h.level(), SecurityLevel::Strict);
    }
}
