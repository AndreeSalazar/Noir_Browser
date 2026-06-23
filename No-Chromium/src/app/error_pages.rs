//! Error Pages (FASE A4)
//!
//! Paginas de error bonitas estilo Chrome/Brave.
//! Incluyen: DNS error, connection failed, timeout, 404, etc.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorKind {
    DnsFailure,
    ConnectionRefused,
    ConnectionReset,
    Timeout,
    TlsError,
    HttpNotFound,
    HttpServerError,
    InvalidUrl,
    UnsupportedProtocol,
    TooManyRedirects,
    Offline,
    Unknown,
}

impl ErrorKind {
    pub fn title(&self) -> &'static str {
        match self {
            ErrorKind::DnsFailure => "This site can't be reached",
            ErrorKind::ConnectionRefused => "This site can't be reached",
            ErrorKind::ConnectionReset => "Connection was reset",
            ErrorKind::Timeout => "This site took too long to respond",
            ErrorKind::TlsError => "Your connection is not private",
            ErrorKind::HttpNotFound => "Page not found",
            ErrorKind::HttpServerError => "The site is currently unable to handle this request",
            ErrorKind::InvalidUrl => "The URL is invalid",
            ErrorKind::UnsupportedProtocol => "Unsupported protocol",
            ErrorKind::TooManyRedirects => "Too many redirects",
            ErrorKind::Offline => "You are offline",
            ErrorKind::Unknown => "Something went wrong",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            ErrorKind::DnsFailure => "x",
            ErrorKind::ConnectionRefused => "x",
            ErrorKind::ConnectionReset => "x",
            ErrorKind::Timeout => "?",
            ErrorKind::TlsError => "!",
            ErrorKind::HttpNotFound => "404",
            ErrorKind::HttpServerError => "500",
            ErrorKind::InvalidUrl => "?",
            ErrorKind::UnsupportedProtocol => "?",
            ErrorKind::TooManyRedirects => "->",
            ErrorKind::Offline => "/",
            ErrorKind::Unknown => "?",
        }
    }

    pub fn suggestion(&self) -> &'static str {
        match self {
            ErrorKind::DnsFailure => "Check your internet connection, firewall, and DNS settings.",
            ErrorKind::ConnectionRefused => "The server may be down or refusing connections.",
            ErrorKind::ConnectionReset => "The connection was reset. Try again.",
            ErrorKind::Timeout => "The server is taking too long. Check your connection or try again later.",
            ErrorKind::TlsError => "The certificate may be invalid or expired.",
            ErrorKind::HttpNotFound => "The page may have been moved or deleted.",
            ErrorKind::HttpServerError => "Try again later or contact the site administrator.",
            ErrorKind::InvalidUrl => "Check the URL for typos.",
            ErrorKind::UnsupportedProtocol => "This browser does not support this protocol.",
            ErrorKind::TooManyRedirects => "The site has a redirect loop.",
            ErrorKind::Offline => "Connect to the internet and try again.",
            ErrorKind::Unknown => "Try reloading the page.",
        }
    }

    pub fn color(&self) -> u32 {
        match self {
            ErrorKind::TlsError => 0xFFE57373,       // red
            ErrorKind::HttpNotFound => 0xFFFFB74D,    // orange
            ErrorKind::HttpServerError => 0xFFE57373, // red
            ErrorKind::Offline => 0xFF757575,         // gray
            _ => 0xFF8CB4FF,                          // blue
        }
    }
}

#[derive(Debug, Clone)]
pub struct ErrorPage {
    pub kind: ErrorKind,
    pub url: String,
    pub detail: String,
}

impl ErrorPage {
    pub fn new(kind: ErrorKind, url: &str) -> Self {
        Self {
            kind,
            url: url.to_string(),
            detail: String::new(),
        }
    }

    pub fn with_detail(mut self, detail: &str) -> Self {
        self.detail = detail.to_string();
        self
    }

    /// Render a string version of the error page (for clipboard, accessibility)
    pub fn to_text(&self) -> String {
        format!(
            "{}\n\n{}\n\nURL: {}\n\n{}{}",
            self.kind.title(),
            self.kind.suggestion(),
            self.url,
            if self.detail.is_empty() { String::new() } else { format!("Detail: {}\n", self.detail) },
            "Press F5 or click 'Reload' to try again."
        )
    }
}

impl fmt::Display for ErrorPage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} ({})", self.kind.title(), self.url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_kind_titles() {
        assert_eq!(ErrorKind::DnsFailure.title(), "This site can't be reached");
        assert_eq!(ErrorKind::HttpNotFound.title(), "Page not found");
        assert_eq!(ErrorKind::Timeout.title(), "This site took too long to respond");
    }

    #[test]
    fn test_error_kind_icons() {
        assert_eq!(ErrorKind::HttpNotFound.icon(), "404");
        assert_eq!(ErrorKind::HttpServerError.icon(), "500");
    }

    #[test]
    fn test_error_kind_suggestions() {
        assert!(!ErrorKind::DnsFailure.suggestion().is_empty());
        assert!(!ErrorKind::TlsError.suggestion().is_empty());
    }

    #[test]
    fn test_error_page_creation() {
        let p = ErrorPage::new(ErrorKind::DnsFailure, "https://example.com");
        assert_eq!(p.kind, ErrorKind::DnsFailure);
        assert_eq!(p.url, "https://example.com");
    }

    #[test]
    fn test_error_page_with_detail() {
        let p = ErrorPage::new(ErrorKind::Timeout, "https://x.com")
            .with_detail("Connection timed out after 30s");
        assert!(p.detail.contains("timed out"));
    }

    #[test]
    fn test_error_page_to_text() {
        let p = ErrorPage::new(ErrorKind::HttpNotFound, "https://x.com/missing");
        let s = p.to_text();
        assert!(s.contains("Page not found"));
        assert!(s.contains("https://x.com/missing"));
    }

    #[test]
    fn test_error_page_display() {
        let p = ErrorPage::new(ErrorKind::Offline, "https://x.com");
        assert_eq!(format!("{}", p), "You are offline (https://x.com)");
    }

    #[test]
    fn test_error_colors() {
        assert_ne!(ErrorKind::TlsError.color(), ErrorKind::Offline.color());
    }
}
