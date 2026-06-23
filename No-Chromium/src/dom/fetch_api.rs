//! URLSearchParams (FASE B4)
//!
//! API de manipulacion de query strings estilo web.
//! `const params = new URLSearchParams("?foo=bar&baz=qux")`
//! `params.get("foo")` -> "bar"
//! `params.append("key", "value")`
//! `params.toString()` -> "foo=bar&baz=qux&key=value"

use std::collections::HashMap;
use crate::network::url_resolver::url_decode;

#[derive(Debug, Clone, Default)]
pub struct UrlSearchParams {
    pairs: Vec<(String, String)>,
}

impl UrlSearchParams {
    pub fn new() -> Self {
        Self::default()
    }

    /// Parsear desde string (con o sin '?')
    pub fn from_string(s: &str) -> Self {
        let s = s.trim_start_matches('?');
        let mut params = Self::default();
        if s.is_empty() {
            return params;
        }
        for pair in s.split('&') {
            if pair.is_empty() { continue; }
            if let Some(idx) = pair.find('=') {
                let k = url_decode(&pair[..idx]);
                let v = url_decode(&pair[idx + 1..]);
                params.pairs.push((k, v));
            } else {
                params.pairs.push((url_decode(pair), String::new()));
            }
        }
        params
    }

    /// append(key, value) - agregar nuevo par
    pub fn append(&mut self, key: &str, value: &str) {
        self.pairs.push((key.to_string(), value.to_string()));
    }

    /// get(key) - obtener el primer valor
    pub fn get(&self, key: &str) -> Option<String> {
        self.pairs.iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.clone())
    }

    /// getAll(key) - todos los valores
    pub fn get_all(&self, key: &str) -> Vec<String> {
        self.pairs.iter()
            .filter(|(k, _)| k == key)
            .map(|(_, v)| v.clone())
            .collect()
    }

    /// has(key) - existe el key?
    pub fn has(&self, key: &str) -> bool {
        self.pairs.iter().any(|(k, _)| k == key)
    }

    /// set(key, value) - reemplazar todos los valores para key
    pub fn set(&mut self, key: &str, value: &str) {
        self.pairs.retain(|(k, _)| k != key);
        self.pairs.push((key.to_string(), value.to_string()));
    }

    /// delete(key) - eliminar todos los pares con key
    pub fn delete(&mut self, key: &str) {
        self.pairs.retain(|(k, _)| k != key);
    }

    /// toString() - serializar a query string
    pub fn to_string(&self) -> String {
        let mut out = String::new();
        for (k, v) in &self.pairs {
            if !out.is_empty() {
                out.push('&');
            }
            out.push_str(&url_encode_simple(k));
            out.push('=');
            out.push_str(&url_encode_simple(v));
        }
        out
    }

    /// entries - iterator de (key, value)
    pub fn entries(&self) -> Vec<(String, String)> {
        self.pairs.clone()
    }

    /// keys
    pub fn keys(&self) -> Vec<String> {
        self.pairs.iter().map(|(k, _)| k.clone()).collect()
    }

    /// values
    pub fn values(&self) -> Vec<String> {
        self.pairs.iter().map(|(_, v)| v.clone()).collect()
    }

    /// count of pairs
    pub fn size(&self) -> usize {
        self.pairs.len()
    }

    /// sort keys alphabetically
    pub fn sort(&mut self) {
        self.pairs.sort_by(|a, b| a.0.cmp(&b.0));
    }
}

fn url_encode_simple(s: &str) -> String {
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

/// fetch API (FASE B4 - basico)
/// Solo los tipos, la implementacion real va al HTTP layer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FetchMethod {
    Get,
    Post,
    Put,
    Delete,
    Head,
    Options,
    Patch,
}

impl FetchMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            FetchMethod::Get => "GET",
            FetchMethod::Post => "POST",
            FetchMethod::Put => "PUT",
            FetchMethod::Delete => "DELETE",
            FetchMethod::Head => "HEAD",
            FetchMethod::Options => "OPTIONS",
            FetchMethod::Patch => "PATCH",
        }
    }
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "GET" => Some(FetchMethod::Get),
            "POST" => Some(FetchMethod::Post),
            "PUT" => Some(FetchMethod::Put),
            "DELETE" => Some(FetchMethod::Delete),
            "HEAD" => Some(FetchMethod::Head),
            "OPTIONS" => Some(FetchMethod::Options),
            "PATCH" => Some(FetchMethod::Patch),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FetchOptions {
    pub method: FetchMethod,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
    pub mode: FetchMode,
    pub credentials: FetchCredentials,
}

impl Default for FetchOptions {
    fn default() -> Self {
        Self {
            method: FetchMethod::Get,
            headers: HashMap::new(),
            body: None,
            mode: FetchMode::Cors,
            credentials: FetchCredentials::SameOrigin,
        }
    }
}

impl FetchOptions {
    pub fn new() -> Self { Self::default() }
    pub fn method(mut self, m: FetchMethod) -> Self { self.method = m; self }
    pub fn body(mut self, b: &str) -> Self { self.body = Some(b.to_string()); self }
    pub fn header(mut self, k: &str, v: &str) -> Self {
        self.headers.insert(k.to_string(), v.to_string());
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FetchMode {
    Cors,
    NoCors,
    SameOrigin,
    Navigate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FetchCredentials {
    Omit,
    SameOrigin,
    Include,
}

#[derive(Debug, Clone)]
pub struct FetchResponse {
    pub status: u16,
    pub status_text: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub ok: bool,
}

impl FetchResponse {
    pub fn json(&self) -> Result<HashMap<String, String>, String> {
        // Simplified: parse JSON-like
        let s = String::from_utf8_lossy(&self.body);
        let mut map = HashMap::new();
        let s = s.trim();
        if s.starts_with('{') && s.ends_with('}') {
            let inner = &s[1..s.len() - 1];
            for pair in inner.split(',') {
                if let Some(idx) = pair.find(':') {
                    let k = pair[..idx].trim().trim_matches('"').to_string();
                    let v = pair[idx + 1..].trim().trim_matches('"').to_string();
                    map.insert(k, v);
                }
            }
        }
        Ok(map)
    }

    pub fn text(&self) -> String {
        String::from_utf8_lossy(&self.body).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_from_string() {
        let p = UrlSearchParams::from_string("foo=bar&baz=qux");
        assert_eq!(p.get("foo"), Some("bar".to_string()));
        assert_eq!(p.get("baz"), Some("qux".to_string()));
    }

    #[test]
    fn test_parse_with_question_mark() {
        let p = UrlSearchParams::from_string("?key=value");
        assert_eq!(p.get("key"), Some("value".to_string()));
    }

    #[test]
    fn test_parse_empty() {
        let p = UrlSearchParams::from_string("");
        assert_eq!(p.size(), 0);
    }

    #[test]
    fn test_get_missing_key() {
        let p = UrlSearchParams::from_string("foo=bar");
        assert_eq!(p.get("missing"), None);
    }

    #[test]
    fn test_get_all() {
        let p = UrlSearchParams::from_string("k=1&k=2&k=3");
        let vals = p.get_all("k");
        assert_eq!(vals.len(), 3);
    }

    #[test]
    fn test_append() {
        let mut p = UrlSearchParams::new();
        p.append("a", "1");
        p.append("b", "2");
        assert_eq!(p.size(), 2);
    }

    #[test]
    fn test_set() {
        let mut p = UrlSearchParams::from_string("a=1&b=2&a=3");
        p.set("a", "new");
        assert_eq!(p.get("a"), Some("new".to_string()));
        assert_eq!(p.get_all("a").len(), 1);
    }

    #[test]
    fn test_delete() {
        let mut p = UrlSearchParams::from_string("a=1&b=2");
        p.delete("a");
        assert!(!p.has("a"));
        assert!(p.has("b"));
    }

    #[test]
    fn test_has() {
        let p = UrlSearchParams::from_string("a=1");
        assert!(p.has("a"));
        assert!(!p.has("b"));
    }

    #[test]
    fn test_to_string() {
        let mut p = UrlSearchParams::new();
        p.append("a", "1");
        p.append("b", "2");
        assert_eq!(p.to_string(), "a=1&b=2");
    }

    #[test]
    fn test_keys_values() {
        let p = UrlSearchParams::from_string("a=1&b=2&c=3");
        let keys = p.keys();
        let values = p.values();
        assert_eq!(keys.len(), 3);
        assert_eq!(values.len(), 3);
    }

    #[test]
    fn test_sort() {
        let mut p = UrlSearchParams::from_string("c=1&a=2&b=3");
        p.sort();
        assert_eq!(p.keys(), vec!["a", "b", "c"]);
    }

    #[test]
    fn test_fetch_method_str() {
        assert_eq!(FetchMethod::Get.as_str(), "GET");
        assert_eq!(FetchMethod::from_str("get"), Some(FetchMethod::Get));
        assert_eq!(FetchMethod::from_str("INVALID"), None);
    }

    #[test]
    fn test_fetch_options_builder() {
        let opts = FetchOptions::new()
            .method(FetchMethod::Post)
            .body("data")
            .header("X-Custom", "value");
        assert_eq!(opts.method, FetchMethod::Post);
        assert_eq!(opts.body, Some("data".to_string()));
        assert_eq!(opts.headers.get("X-Custom"), Some(&"value".to_string()));
    }

    #[test]
    fn test_fetch_response_text() {
        let resp = FetchResponse {
            status: 200,
            status_text: "OK".to_string(),
            headers: HashMap::new(),
            body: b"Hello, World!".to_vec(),
            ok: true,
        };
        assert!(resp.ok);
        assert_eq!(resp.text(), "Hello, World!");
    }

    #[test]
    fn test_fetch_response_json() {
        let resp = FetchResponse {
            status: 200,
            status_text: "OK".to_string(),
            headers: HashMap::new(),
            body: br#"{"name":"John","age":30}"#.to_vec(),
            ok: true,
        };
        let json = resp.json().unwrap();
        assert_eq!(json.get("name"), Some(&"John".to_string()));
        assert_eq!(json.get("age"), Some(&"30".to_string()));
    }

    #[test]
    fn test_url_encoded() {
        let p = UrlSearchParams::from_string("key=hello%20world");
        assert_eq!(p.get("key"), Some("hello world".to_string()));
    }
}
