//! URL Resolver (RFC 3986 - WHATWG URL Standard subset)
//!
//! Resuelve URLs relativas contra una base URL.
//! Usado para CSS externo, JS externo, imagenes, favicons, etc.
//!
//! Ejemplos:
//! - resolver("https://youtube.com/results", "search") = "https://youtube.com/search"
//! - resolver("https://youtube.com/a/b/c.html", "/d.css") = "https://youtube.com/d.css"
//! - resolver("https://youtube.com/a/b/", "c.css") = "https://youtube.com/a/b/c.css"

/// Componentes de una URL
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedUrl {
    pub scheme: String,
    pub host: String,
    pub port: Option<u16>,
    pub path: String,
    pub query: Option<String>,
    pub fragment: Option<String>,
}

impl ParsedUrl {
    /// Parsea una URL absoluta
    pub fn parse(url: &str) -> Result<Self, &'static str> {
        if url.is_empty() {
            return Err("Empty URL");
        }
        // scheme://host[:port]/path[?query][#fragment]
        let (scheme, rest) = if let Some(idx) = url.find("://") {
            (url[..idx].to_string(), &url[idx + 3..])
        } else if url.starts_with("//") {
            ("https".to_string(), &url[2..])  // protocol-relative
        } else {
            return Err("URL sin scheme");
        };

        // authority: host[:port] + path
        let (authority, after_authority) = if let Some(idx) = rest.find('/') {
            (&rest[..idx], &rest[idx..])
        } else if let Some(idx) = rest.find('?') {
            (&rest[..idx], &rest[idx..])
        } else if let Some(idx) = rest.find('#') {
            (&rest[..idx], &rest[idx..])
        } else {
            (rest, "")
        };

        let (host_port, port) = if let Some(idx) = authority.find(':') {
            let port_str = &authority[idx + 1..];
            let port = port_str.parse().ok();
            (authority[..idx].to_string(), port)
        } else {
            (authority.to_string(), None)
        };

        let host = host_port.to_lowercase();

        // path + query + fragment
        let (path, query, fragment) = parse_path_query_fragment(after_authority);

        Ok(Self {
            scheme: scheme.to_lowercase(),
            host,
            port,
            path,
            query,
            fragment,
        })
    }

    /// Reconstruye como string
    pub fn to_string(&self) -> String {
        let mut s = format!("{}://{}", self.scheme, self.host);
        if let Some(port) = self.port {
            s.push_str(&format!(":{}", port));
        }
        s.push_str(&self.path);
        if let Some(q) = &self.query {
            s.push('?');
            s.push_str(q);
        }
        if let Some(f) = &self.fragment {
            s.push('#');
            s.push_str(f);
        }
        s
    }

    /// Es valida (al menos scheme + host)
    pub fn is_valid(&self) -> bool {
        !self.scheme.is_empty() && !self.host.is_empty()
    }
}

fn parse_path_query_fragment(s: &str) -> (String, Option<String>, Option<String>) {
    let mut path = String::new();
    let mut query = None;
    let mut fragment = None;

    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '?' {
            let mut q = String::new();
            while let Some(&nc) = chars.peek() {
                if nc == '#' { break; }
                q.push(nc);
                chars.next();
            }
            query = Some(q);
        } else if c == '#' {
            let f: String = chars.collect();
            fragment = Some(f);
            break;
        } else {
            path.push(c);
        }
    }

    (path, query, fragment)
}

/// Resuelve una URL relativa contra una base
pub fn resolve(base: &str, relative: &str) -> Result<String, &'static str> {
    if relative.is_empty() {
        return Ok(base.to_string());
    }

    // URL absoluta (con scheme) -> retornar
    if relative.contains("://") {
        return Ok(relative.to_string());
    }

    // Fragment-only -> agregar a base
    if relative.starts_with('#') {
        let base_parsed = ParsedUrl::parse(base)?;
        let mut result = base_parsed.to_string();
        if !result.ends_with(relative) {
            // Reemplazar fragment en base
            if let Some(idx) = result.find('#') {
                result.truncate(idx);
            }
            result.push_str(relative);
        }
        return Ok(result);
    }

    // Query-only -> agregar a base (reemplazando query si existe)
    if relative.starts_with('?') {
        let base_parsed = ParsedUrl::parse(base)?;
        let mut result = base_parsed.to_string();
        if let Some(idx) = result.find('?') {
            result.truncate(idx);
        }
        if let Some(idx) = result.find('#') {
            result.truncate(idx);
        }
        result.push_str(relative);
        return Ok(result);
    }

    // Parsear base
    let base_parsed = ParsedUrl::parse(base)?;

    // Schema-relative (//host/...) - usa scheme del base, host del relative
    if relative.starts_with("//") {
        // Extraer host y resto de "//host[:port]/path..."
        let after_slashes = &relative[2..];
        let (host_part, path_part) = if let Some(idx) = after_slashes.find('/') {
            (&after_slashes[..idx], &after_slashes[idx..])
        } else if let Some(idx) = after_slashes.find('?') {
            (&after_slashes[..idx], &after_slashes[idx..])
        } else {
            (after_slashes, "/")
        };
        let (host, port) = if let Some(idx) = host_part.find(':') {
            let port_str = &host_part[idx + 1..];
            (host_part[..idx].to_string(), port_str.parse::<u16>().ok())
        } else {
            (host_part.to_string(), None)
        };
        return Ok(format!("{}://{}{}{}",
            base_parsed.scheme,
            host,
            port.map(|p| format!(":{}", p)).unwrap_or_default(),
            path_part
        ));
    }

    // Path-absolute (empieza con /)
    if relative.starts_with('/') {
        return Ok(format!("{}://{}{}{}",
            base_parsed.scheme,
            base_parsed.host,
            base_parsed.port.map(|p| format!(":{}", p)).unwrap_or_default(),
            relative
        ));
    }

    // Path-relative: resolver contra el directorio de base.path
    let base_path = base_parsed.path.as_str();
    let dir = if let Some(idx) = base_path.rfind('/') {
        &base_path[..=idx]
    } else {
        "/"
    };

    let resolved_path = resolve_path(dir, relative);

    Ok(format!("{}://{}{}{}",
        base_parsed.scheme,
        base_parsed.host,
        base_parsed.port.map(|p| format!(":{}", p)).unwrap_or_default(),
        resolved_path
    ))
}

/// Resuelve path relativo (resuelve . y .. segun RFC 3986)
fn resolve_path(base: &str, relative: &str) -> String {
    // Combinar base + relative, split por /, resolver . y ..
    let combined = format!("{}{}", base, relative);
    let parts: Vec<&str> = combined.split('/').collect();
    let mut stack: Vec<&str> = Vec::new();

    for part in &parts {
        match *part {
            "" | "." => {
                // Mantener el "" inicial para que empiece con /
                if stack.is_empty() && parts[0].is_empty() {
                    stack.push("");
                }
            }
            ".." => {
                // Pop, pero no pasar del root
                if stack.len() > 1 || (stack.len() == 1 && stack[0].is_empty()) {
                    stack.pop();
                }
            }
            _ => {
                stack.push(part);
            }
        }
    }

    // Reconstruir
    let result = stack.join("/");
    if result.starts_with("//") {
        // Caso: "//" se convierte en "/"
        result[1..].to_string()
    } else {
        result
    }
}

/// Parsea query string a pairs (key=value)
pub fn parse_query(query: &str) -> Vec<(String, String)> {
    query.split('&')
        .filter(|s| !s.is_empty())
        .map(|pair| {
            if let Some(idx) = pair.find('=') {
                (
                    url_decode(&pair[..idx]),
                    url_decode(&pair[idx + 1..]),
                )
            } else {
                (url_decode(pair), String::new())
            }
        })
        .collect()
}

/// URL decode
pub fn url_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'+' {
            out.push(b' ');
            i += 1;
        } else if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(h), Some(l)) = (
                hex_digit(bytes[i + 1]),
                hex_digit(bytes[i + 2]),
            ) {
                out.push((h << 4) | l);
                i += 3;
            } else {
                out.push(bytes[i]);
                i += 1;
            }
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8_lossy(&out).to_string()
}

fn hex_digit(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

/// URL encode
pub fn url_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '~') {
            out.push(c);
        } else {
            let mut buf = [0u8; 4];
            let s = c.encode_utf8(&mut buf);
            for b in s.bytes() {
                out.push_str(&format!("%{:02X}", b));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_url() {
        let u = ParsedUrl::parse("https://example.com/path?query=1#frag").unwrap();
        assert_eq!(u.scheme, "https");
        assert_eq!(u.host, "example.com");
        assert_eq!(u.path, "/path");
        assert_eq!(u.query, Some("query=1".to_string()));
        assert_eq!(u.fragment, Some("frag".to_string()));
    }

    #[test]
    fn test_parse_with_port() {
        let u = ParsedUrl::parse("http://localhost:8080/api").unwrap();
        assert_eq!(u.host, "localhost");
        assert_eq!(u.port, Some(8080));
        assert_eq!(u.path, "/api");
    }

    #[test]
    fn test_parse_invalid() {
        assert!(ParsedUrl::parse("").is_err());
        assert!(ParsedUrl::parse("noscheme").is_err());
    }

    #[test]
    fn test_resolve_absolute() {
        assert_eq!(
            resolve("https://example.com/page", "https://other.com/x").unwrap(),
            "https://other.com/x"
        );
    }

    #[test]
    fn test_resolve_protocol_relative() {
        assert_eq!(
            resolve("https://example.com/page", "//cdn.com/file.js").unwrap(),
            "https://cdn.com/file.js"
        );
    }

    #[test]
    fn test_resolve_path_absolute() {
        assert_eq!(
            resolve("https://example.com/a/b/c.html", "/static/css/main.css").unwrap(),
            "https://example.com/static/css/main.css"
        );
    }

    #[test]
    fn test_resolve_path_relative() {
        assert_eq!(
            resolve("https://example.com/a/b/c.html", "style.css").unwrap(),
            "https://example.com/a/b/style.css"
        );
    }

    #[test]
    fn test_resolve_parent_dir() {
        assert_eq!(
            resolve("https://example.com/a/b/c.html", "../d.css").unwrap(),
            "https://example.com/a/d.css"
        );
    }

    #[test]
    fn test_resolve_dot() {
        assert_eq!(
            resolve("https://example.com/a/b/", "./c.css").unwrap(),
            "https://example.com/a/b/c.css"
        );
    }

    #[test]
    fn test_resolve_query_only() {
        assert_eq!(
            resolve("https://example.com/page?q=1", "?q=2").unwrap(),
            "https://example.com/page?q=2"
        );
    }

    #[test]
    fn test_resolve_fragment_only() {
        assert_eq!(
            resolve("https://example.com/page", "#section").unwrap(),
            "https://example.com/page#section"
        );
    }

    #[test]
    fn test_resolve_complex() {
        assert_eq!(
            resolve("https://youtube.com/results?search_query=yt", "page.html").unwrap(),
            "https://youtube.com/page.html"
        );
    }

    #[test]
    fn test_parse_query() {
        let q = parse_query("a=1&b=2&c=hello+world");
        assert_eq!(q.len(), 3);
        assert_eq!(q[0], ("a".to_string(), "1".to_string()));
        assert_eq!(q[1], ("b".to_string(), "2".to_string()));
        assert_eq!(q[2], ("c".to_string(), "hello world".to_string()));
    }

    #[test]
    fn test_parse_query_encoded() {
        let q = parse_query("q=hello%20world&x=%26");
        assert_eq!(q[0].1, "hello world");
        assert_eq!(q[1].1, "&");
    }

    #[test]
    fn test_url_encode_decode() {
        let s = "Hello World!";
        let encoded = url_encode(s);
        let decoded = url_decode(&encoded);
        assert_eq!(decoded, s);
    }

    #[test]
    fn test_url_to_string() {
        let u = ParsedUrl::parse("https://example.com:8080/path?q=1").unwrap();
        assert_eq!(u.to_string(), "https://example.com:8080/path?q=1");
    }

    #[test]
    fn test_resolve_dotdot_chain() {
        assert_eq!(
            resolve("https://example.com/a/b/c/d.html", "../../e.css").unwrap(),
            "https://example.com/a/e.css"
        );
    }
}
