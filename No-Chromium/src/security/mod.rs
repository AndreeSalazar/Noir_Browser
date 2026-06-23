//! Security Module - Suite de seguridad web
//!
//! Submódulos:
//! - `cookies/`: Cookie jar con SameSite, Secure, HttpOnly, expiración
//! - `csp/`: Content-Security-Policy parser y enforcement
//! - `origin/`: Same-Origin Policy y Origin validation
//! - `cors/`: Cross-Origin Resource Sharing validation
//! - `url/`: URL parsing seguro (anti-spoofing)
//! - `headers/`: Security headers (HSTS, X-Frame-Options, etc.)
//! - `mixed_content/`: Detector de mixed content (HTTP en HTTPS)
//! - `referrer/`: Referrer-Policy enforcement

#![allow(dead_code)]

pub mod cookies;
pub mod csp;
pub mod origin;
pub mod cors;
pub mod url;
pub mod headers;
pub mod mixed_content;
pub mod referrer;

pub use cookies::{Cookie, CookieJar, SameSite, CookieError};
pub use csp::{CspPolicy, CspDirective, CspViolation};
pub use origin::{Origin, OriginError};
pub use cors::{CorsConfig, CorsError};
pub use url::{SafeUrl, UrlError};
pub use headers::{SecurityHeaders, SecurityLevel};
pub use mixed_content::{MixedContentChecker, MixedContentViolation};
pub use referrer::{ReferrerPolicy, ReferrerInfo};
