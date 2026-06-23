//! Referrer Policy - Control de Referer header
//!
//! Implementa las directivas de Referrer-Policy.

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ReferrerPolicy {
    NoReferrer,
    NoReferrerWhenDowngrade,
    SameOrigin,
    Origin,
    StrictOrigin,
    OriginWhenCrossOrigin,
    StrictOriginWhenCrossOrigin,
    UnsafeUrl,
}

impl ReferrerPolicy {
    pub fn from_header(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "no-referrer" => ReferrerPolicy::NoReferrer,
            "no-referrer-when-downgrade" => ReferrerPolicy::NoReferrerWhenDowngrade,
            "same-origin" => ReferrerPolicy::SameOrigin,
            "origin" => ReferrerPolicy::Origin,
            "strict-origin" => ReferrerPolicy::StrictOrigin,
            "origin-when-cross-origin" => ReferrerPolicy::OriginWhenCrossOrigin,
            "strict-origin-when-cross-origin" => ReferrerPolicy::StrictOriginWhenCrossOrigin,
            "unsafe-url" => ReferrerPolicy::UnsafeUrl,
            _ => ReferrerPolicy::StrictOriginWhenCrossOrigin, // Default
        }
    }

    /// Determina qué Referer enviar para esta navegación
    pub fn apply(&self, from: &str, to: &str) -> Option<String> {
        let from_https = from.starts_with("https://");
        let to_https = to.starts_with("https://");

        match self {
            ReferrerPolicy::NoReferrer => None,
            ReferrerPolicy::NoReferrerWhenDowngrade => {
                if from_https && !to_https { None } else { Some(from.to_string()) }
            }
            ReferrerPolicy::SameOrigin => {
                let from_origin = origin_of(from);
                let to_origin = origin_of(to);
                if from_origin == to_origin { Some(from.to_string()) } else { None }
            }
            ReferrerPolicy::Origin => Some(from_origin_string(from)),
            ReferrerPolicy::StrictOrigin => {
                if from_https && !to_https { None } else { Some(from_origin_string(from)) }
            }
            ReferrerPolicy::OriginWhenCrossOrigin => {
                let from_origin = origin_of(from);
                let to_origin = origin_of(to);
                if from_origin == to_origin {
                    Some(from.to_string())
                } else {
                    Some(from_origin_string(from))
                }
            }
            ReferrerPolicy::StrictOriginWhenCrossOrigin => {
                if from_https && !to_https { None } else {
                    let from_origin = origin_of(from);
                    let to_origin = origin_of(to);
                    if from_origin == to_origin {
                        Some(from.to_string())
                    } else {
                        Some(from_origin_string(from))
                    }
                }
            }
            ReferrerPolicy::UnsafeUrl => Some(from.to_string()),
        }
    }
}

fn origin_of(url: &str) -> String {
    if let Ok(parsed) = url::Url::parse(url) {
        format!("{}://{}", parsed.scheme(), parsed.host_str().unwrap_or(""))
    } else {
        String::new()
    }
}

fn from_origin_string(url: &str) -> String {
    origin_of(url)
}

#[derive(Debug, Clone)]
pub struct ReferrerInfo {
    pub policy: ReferrerPolicy,
    pub header: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_referrer_policy_from_header() {
        assert_eq!(ReferrerPolicy::from_header("no-referrer"), ReferrerPolicy::NoReferrer);
        assert_eq!(ReferrerPolicy::from_header("origin"), ReferrerPolicy::Origin);
        assert_eq!(ReferrerPolicy::from_header("same-origin"), ReferrerPolicy::SameOrigin);
    }

    #[test]
    fn test_no_referrer() {
        let p = ReferrerPolicy::NoReferrer;
        assert_eq!(p.apply("https://a.com/", "https://b.com/"), None);
    }

    #[test]
    fn test_no_referrer_when_downgrade() {
        let p = ReferrerPolicy::NoReferrerWhenDowngrade;
        assert!(p.apply("https://a.com/", "http://b.com/").is_none());
        assert!(p.apply("https://a.com/", "https://b.com/").is_some());
    }

    #[test]
    fn test_same_origin() {
        let p = ReferrerPolicy::SameOrigin;
        assert!(p.apply("https://a.com/x", "https://a.com/y").is_some());
        assert!(p.apply("https://a.com/", "https://b.com/").is_none());
    }

    #[test]
    fn test_origin_policy() {
        let p = ReferrerPolicy::Origin;
        let r = p.apply("https://a.com/x/y", "https://b.com/");
        assert!(r.is_some());
        assert!(r.unwrap().contains("a.com"));
    }

    #[test]
    fn test_strict_origin() {
        let p = ReferrerPolicy::StrictOrigin;
        assert!(p.apply("https://a.com/", "http://b.com/").is_none());
        assert!(p.apply("https://a.com/", "https://b.com/").is_some());
    }

    #[test]
    fn test_unsafe_url() {
        let p = ReferrerPolicy::UnsafeUrl;
        assert!(p.apply("https://a.com/", "http://b.com/").is_some());
    }
}
