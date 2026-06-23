//! HTTP Error Pages - Páginas de error HTTP (404, 500, etc)
//!
//! Genera páginas HTML para códigos de error HTTP.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HttpErrorPage {
    pub status: u16,
    pub title: &'static str,
    pub description: &'static str,
    pub suggestion: &'static str,
}

impl HttpErrorPage {
    /// Genera página de error para un código HTTP
    pub fn from_status(status: u16) -> Self {
        match status {
            400 => Self {
                status,
                title: "Bad Request",
                description: "The server cannot process the request due to a client error.",
                suggestion: "Check the URL and try again.",
            },
            401 => Self {
                status,
                title: "Unauthorized",
                description: "You need to authenticate to access this resource.",
                suggestion: "Please log in and try again.",
            },
            403 => Self {
                status,
                title: "Forbidden",
                description: "You don't have permission to access this resource.",
                suggestion: "Check your credentials or contact the administrator.",
            },
            404 => Self {
                status,
                title: "Not Found",
                description: "The page you're looking for doesn't exist or has been moved.",
                suggestion: "Check the URL for typos or go back to the homepage.",
            },
            405 => Self {
                status,
                title: "Method Not Allowed",
                description: "The request method is not supported for this resource.",
                suggestion: "Try a different request method.",
            },
            408 => Self {
                status,
                title: "Request Timeout",
                description: "The server took too long to respond.",
                suggestion: "Check your internet connection and try again.",
            },
            500 => Self {
                status,
                title: "Internal Server Error",
                description: "The server encountered an unexpected error.",
                suggestion: "Try again later. If the problem persists, contact the site owner.",
            },
            502 => Self {
                status,
                title: "Bad Gateway",
                description: "The server received an invalid response from an upstream server.",
                suggestion: "Try again in a few minutes.",
            },
            503 => Self {
                status,
                title: "Service Unavailable",
                description: "The server is temporarily unable to handle the request.",
                suggestion: "Try again later. The site might be under maintenance.",
            },
            504 => Self {
                status,
                title: "Gateway Timeout",
                description: "The upstream server didn't respond in time.",
                suggestion: "Try again later.",
            },
            _ => Self {
                status,
                title: "Error",
                description: "An unexpected error occurred.",
                suggestion: "Try again later.",
            },
        }
    }
}

/// Genera HTML para una página de error
pub fn error_page_html(url: &str, status: u16) -> String {
    let page = HttpErrorPage::from_status(status);
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
  <title>Error {} - Noir Browser</title>
  <style>
    body {{ font-family: sans-serif; background: #1a1a1a; color: #fff; padding: 40px; text-align: center; }}
    h1 {{ color: #ff5555; font-size: 72px; margin: 20px; }}
    h2 {{ color: #ffaa55; }}
    p {{ color: #ccc; max-width: 600px; margin: 20px auto; }}
    .url {{ background: #2a2a2a; padding: 10px; border-radius: 5px; font-family: monospace; }}
    .suggestion {{ background: #2a3a2a; padding: 15px; border-radius: 5px; margin-top: 30px; color: #88ff88; }}
    a {{ color: #5599ff; text-decoration: none; padding: 10px 20px; border: 1px solid #5599ff; border-radius: 5px; display: inline-block; margin-top: 20px; }}
  </style>
</head>
<body>
  <h1>{}</h1>
  <h2>{}</h2>
  <p>{}</p>
  <div class="url">{}</div>
  <div class="suggestion">💡 {}</div>
  <a href="noir://newtab">Go to homepage</a>
</body>
</html>"#,
        page.status, page.status, page.title, page.description, url, page.suggestion
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_404() {
        let page = HttpErrorPage::from_status(404);
        assert_eq!(page.status, 404);
        assert_eq!(page.title, "Not Found");
    }

    #[test]
    fn test_500() {
        let page = HttpErrorPage::from_status(500);
        assert_eq!(page.title, "Internal Server Error");
    }

    #[test]
    fn test_401() {
        let page = HttpErrorPage::from_status(401);
        assert_eq!(page.title, "Unauthorized");
    }

    #[test]
    fn test_503() {
        let page = HttpErrorPage::from_status(503);
        assert_eq!(page.title, "Service Unavailable");
    }

    #[test]
    fn test_unknown_status() {
        let page = HttpErrorPage::from_status(418);
        assert_eq!(page.title, "Error");
    }

    #[test]
    fn test_error_page_html() {
        let html = error_page_html("https://example.com/missing", 404);
        assert!(html.contains("404"));
        assert!(html.contains("Not Found"));
        assert!(html.contains("example.com/missing"));
    }

    #[test]
    fn test_error_page_html_500() {
        let html = error_page_html("https://broken.com/", 500);
        assert!(html.contains("500"));
        assert!(html.contains("Internal Server Error"));
    }
}
