//! CSS Subresource Loader
//!
//! Detecta links a CSS externo en HTML, descarga y los aplica.
//! Es parte de FASE A1: hacer el navegador "Noir Lite" funcional.
//!
//! Soporta:
//! - <link rel="stylesheet" href="...">
//! - URLs relativas (via url_resolver)
//! - Cascada (CSS del documento + CSS externos)
//! - Cache en memoria

use std::collections::HashMap;
use std::sync::Arc;

use crate::network::url_resolver::resolve;

/// Un CSS cargado (link tag en HTML o style inline)
#[derive(Debug, Clone)]
pub struct CssLink {
    pub href: String,
    pub resolved_url: String,
    pub content: String,
    pub media_query: Option<String>,
    pub loaded: bool,
    pub error: Option<String>,
}

/// Loader de CSS externos
#[derive(Debug, Default)]
pub struct CssLoader {
    pub links: Vec<CssLink>,
    pub cache: HashMap<String, String>,  // url -> content
    pub total_size: usize,
    pub failed_count: usize,
    pub loaded_count: usize,
}

impl CssLoader {
    pub fn new() -> Self {
        Self::default()
    }

    /// Procesa HTML y extrae links a CSS externo
    /// (simulado - en la realidad parseamos el HTML)
    pub fn extract_css_links_from_html(&mut self, base_url: &str, html: &str) {
        // Buscar <link rel="stylesheet" href="..."> en el HTML
        let lower = html.to_lowercase();
        let mut pos = 0;
        while let Some(idx) = lower[pos..].find("<link") {
            let abs_start = pos + idx;
            if let Some(end_idx) = lower[abs_start..].find('>') {
                let tag = &html[abs_start..abs_start + end_idx + 1];
                self.process_link_tag(base_url, tag);
                pos = abs_start + end_idx + 1;
            } else {
                break;
            }
        }
    }

    fn process_link_tag(&mut self, base_url: &str, tag: &str) {
        let lower = tag.to_lowercase();
        // Solo procesar si es stylesheet
        if !lower.contains("rel=\"stylesheet\"") && !lower.contains("rel=stylesheet") {
            return;
        }

        // Extraer href
        if let Some(href) = extract_attr(tag, "href") {
            let resolved = match resolve(base_url, &href) {
                Ok(u) => u,
                Err(_) => href.clone(),
            };
            let media = extract_attr(tag, "media");

            // No duplicar
            if self.links.iter().any(|l| l.resolved_url == resolved) {
                return;
            }

            self.links.push(CssLink {
                href,
                resolved_url: resolved,
                content: String::new(),
                media_query: media,
                loaded: false,
                error: None,
            });
        }
    }

    /// Registra contenido CSS descargado (de la red)
    pub fn set_css_content(&mut self, url: &str, content: String) {
        self.cache.insert(url.to_string(), content.clone());
        for link in &mut self.links {
            if link.resolved_url == url {
                link.content = content.clone();
                link.loaded = true;
                self.loaded_count += 1;
                self.total_size += content.len();
                return;
            }
        }
    }

    /// Marca un link como failed
    pub fn mark_failed(&mut self, url: &str, error: &str) {
        for link in &mut self.links {
            if link.resolved_url == url {
                link.error = Some(error.to_string());
                self.failed_count += 1;
                return;
            }
        }
    }

    /// Obtiene todos los CSS cargados exitosamente (sin media queries restrictivas)
    pub fn get_applicable_css(&self) -> String {
        let mut out = String::new();
        for link in &self.links {
            if link.loaded && link.error.is_none() {
                if let Some(media) = &link.media_query {
                    // Solo aplicar si media query es "all" o "screen"
                    let ml = media.to_lowercase();
                    if !ml.contains("print") {
                        out.push_str(&format!("/* {} */\n{}\n\n", link.resolved_url, link.content));
                    }
                } else {
                    out.push_str(&format!("/* {} */\n{}\n\n", link.resolved_url, link.content));
                }
            }
        }
        out
    }

    /// Estadisticas
    pub fn stats(&self) -> (usize, usize, usize, usize) {
        (self.links.len(), self.loaded_count, self.failed_count, self.total_size)
    }

    /// Limpia todo
    pub fn clear(&mut self) {
        self.links.clear();
        self.cache.clear();
        self.total_size = 0;
        self.failed_count = 0;
        self.loaded_count = 0;
    }
}

/// Extrae un atributo de un tag HTML
fn extract_attr(tag: &str, attr: &str) -> Option<String> {
    let lower = tag.to_lowercase();
    let pattern = format!("{}=\"", attr);
    if let Some(idx) = lower.find(&pattern) {
        let start = idx + pattern.len();
        if let Some(end_idx) = tag[start..].find('"') {
            return Some(tag[start..start + end_idx].to_string());
        }
    }
    // Tambien con comillas simples
    let pattern = format!("{}='", attr);
    if let Some(idx) = lower.find(&pattern) {
        let start = idx + pattern.len();
        if let Some(end_idx) = tag[start..].find('\'') {
            return Some(tag[start..start + end_idx].to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_attr() {
        assert_eq!(
            extract_attr("<link rel=\"stylesheet\" href=\"main.css\">", "href"),
            Some("main.css".to_string())
        );
        assert_eq!(
            extract_attr("<link rel='stylesheet' href='main.css'>", "href"),
            Some("main.css".to_string())
        );
    }

    #[test]
    fn test_no_extract() {
        assert_eq!(extract_attr("<a href='x'>", "rel"), None);
    }

    #[test]
    fn test_extract_from_html() {
        let mut loader = CssLoader::new();
        let html = r#"
            <html>
            <head>
                <link rel="stylesheet" href="main.css">
                <link rel="stylesheet" href="https://cdn.com/style.css">
                <link rel="icon" href="favicon.ico">  // No es stylesheet
            </head>
            </html>
        "#;
        loader.extract_css_links_from_html("https://example.com/page.html", html);
        assert_eq!(loader.links.len(), 2);
        assert_eq!(loader.links[0].resolved_url, "https://example.com/main.css");
        assert_eq!(loader.links[1].resolved_url, "https://cdn.com/style.css");
    }

    #[test]
    fn test_set_css_content() {
        let mut loader = CssLoader::new();
        let html = r#"<link rel="stylesheet" href="a.css">"#;
        loader.extract_css_links_from_html("https://x.com/", html);
        loader.set_css_content("https://x.com/a.css", "body { color: red; }".to_string());
        let css = loader.get_applicable_css();
        assert!(css.contains("body { color: red; }"));
    }

    #[test]
    fn test_mark_failed() {
        let mut loader = CssLoader::new();
        let html = r#"<link rel="stylesheet" href="a.css">"#;
        loader.extract_css_links_from_html("https://x.com/", html);
        loader.mark_failed("https://x.com/a.css", "404");
        let css = loader.get_applicable_css();
        assert!(!css.contains("404"));
    }

    #[test]
    fn test_no_duplicates() {
        let mut loader = CssLoader::new();
        let html = r#"<link rel="stylesheet" href="a.css"><link rel="stylesheet" href="a.css">"#;
        loader.extract_css_links_from_html("https://x.com/", html);
        assert_eq!(loader.links.len(), 1);
    }

    #[test]
    fn test_media_query_print_excluded() {
        let mut loader = CssLoader::new();
        let html = r#"<link rel="stylesheet" href="print.css" media="print">"#;
        loader.extract_css_links_from_html("https://x.com/", html);
        loader.set_css_content("https://x.com/print.css", "p { color: black; }".to_string());
        let css = loader.get_applicable_css();
        assert!(!css.contains("p { color: black; }"));
    }

    #[test]
    fn test_clear() {
        let mut loader = CssLoader::new();
        let html = r#"<link rel="stylesheet" href="a.css">"#;
        loader.extract_css_links_from_html("https://x.com/", html);
        loader.clear();
        assert_eq!(loader.links.len(), 0);
    }

    #[test]
    fn test_stats() {
        let mut loader = CssLoader::new();
        let html = r#"<link rel="stylesheet" href="a.css"><link rel="stylesheet" href="b.css">"#;
        loader.extract_css_links_from_html("https://x.com/", html);
        loader.set_css_content("https://x.com/a.css", "body{}".to_string());
        let (total, loaded, failed, size) = loader.stats();
        assert_eq!(total, 2);
        assert_eq!(loaded, 1);
        assert_eq!(failed, 0);
        assert!(size > 0);
    }
}
