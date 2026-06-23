//! Image Subresource Loader (FASE A2)
//!
//! Detecta imagenes en HTML, resuelve URLs relativas, y mantiene una cola de carga.
//! Tambien maneja srcset (responsive images) y favicons.

use std::collections::HashMap;

use crate::network::url_resolver::resolve;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImageType {
    Img,        // <img src="...">
    Favicon,    // <link rel="icon" href="...">
    BgImage,    // CSS background-image
    Picture,    // <picture>
    Source,     // <source srcset="...">
    InlineSvg,  // SVG inline
}

impl ImageType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ImageType::Img => "img",
            ImageType::Favicon => "favicon",
            ImageType::BgImage => "background",
            ImageType::Picture => "picture",
            ImageType::Source => "source",
            ImageType::InlineSvg => "inline-svg",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ImageResource {
    pub original_src: String,
    pub resolved_url: String,
    pub image_type: ImageType,
    pub alt: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub loading: LoadingStrategy,  // eager, lazy
    pub srcset: Vec<SrcsetCandidate>,
    pub loaded: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LoadingStrategy {
    Eager,
    Lazy,
}

#[derive(Debug, Clone)]
pub struct SrcsetCandidate {
    pub url: String,
    pub descriptor: String,  // "2x", "480w", etc.
}

#[derive(Debug, Default)]
pub struct ImageLoader {
    pub resources: Vec<ImageResource>,
    pub loaded: HashMap<String, Vec<u8>>,  // url -> bytes
    pub total_loaded_bytes: usize,
    pub failed_count: usize,
}

impl ImageLoader {
    pub fn new() -> Self { Self::default() }

    /// Procesa HTML y extrae imagenes
    pub fn extract_from_html(&mut self, base_url: &str, html: &str) {
        // <img src="..." alt="...">
        let lower = html.to_lowercase();
        let mut pos = 0;
        while let Some(idx) = lower[pos..].find("<img") {
            let abs_start = pos + idx;
            if let Some(end_idx) = lower[abs_start..].find('>') {
                let tag = &html[abs_start..abs_start + end_idx + 1];
                self.process_img_tag(base_url, tag);
                pos = abs_start + end_idx + 1;
            } else {
                break;
            }
        }
        // <link rel="icon" href="...">
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

    fn process_img_tag(&mut self, base_url: &str, tag: &str) {
        if let Some(src) = extract_attr(tag, "src") {
            let resolved = resolve(base_url, &src).unwrap_or(src.clone());
            let alt = extract_attr(tag, "alt").unwrap_or_default();
            let width = extract_attr(tag, "width").and_then(|w| w.parse().ok());
            let height = extract_attr(tag, "height").and_then(|h| h.parse().ok());
            let loading = match extract_attr(tag, "loading").as_deref() {
                Some("lazy") => LoadingStrategy::Lazy,
                _ => LoadingStrategy::Eager,
            };
            let mut srcset = Vec::new();
            if let Some(set) = extract_attr(tag, "srcset") {
                srcset = parse_srcset(&set);
                // Resolver URLs relativas del srcset contra base_url
                for cand in &mut srcset {
                    cand.url = resolve(base_url, &cand.url).unwrap_or_else(|_| cand.url.clone());
                }
            }
            // No duplicar
            if self.resources.iter().any(|r| r.resolved_url == resolved) {
                return;
            }
            self.resources.push(ImageResource {
                original_src: src,
                resolved_url: resolved,
                image_type: ImageType::Img,
                alt,
                width,
                height,
                loading,
                srcset,
                loaded: false,
                error: None,
            });
        }
    }

    fn process_link_tag(&mut self, base_url: &str, tag: &str) {
        let lower = tag.to_lowercase();
        let is_icon = lower.contains("rel=\"icon\"")
            || lower.contains("rel=\"shortcut icon\"")
            || lower.contains("rel=icon");
        if !is_icon { return; }
        if let Some(href) = extract_attr(tag, "href") {
            let resolved = resolve(base_url, &href).unwrap_or(href.clone());
            if self.resources.iter().any(|r| r.resolved_url == resolved) {
                return;
            }
            self.resources.push(ImageResource {
                original_src: href,
                resolved_url: resolved,
                image_type: ImageType::Favicon,
                alt: "Favicon".to_string(),
                width: None,
                height: None,
                loading: LoadingStrategy::Eager,
                srcset: vec![],
                loaded: false,
                error: None,
            });
        }
    }

    /// Registra una imagen cargada
    pub fn set_loaded(&mut self, url: &str, bytes: Vec<u8>) {
        self.total_loaded_bytes += bytes.len();
        self.loaded.insert(url.to_string(), bytes);
        for r in &mut self.resources {
            if r.resolved_url == url {
                r.loaded = true;
            }
        }
    }

    pub fn mark_failed(&mut self, url: &str, error: &str) {
        self.failed_count += 1;
        for r in &mut self.resources {
            if r.resolved_url == url {
                r.error = Some(error.to_string());
            }
        }
    }

    /// Devuelve la URL de la mejor srcset candidate segun viewport width
    pub fn best_srcset(&self, original_url: &str, viewport_w: u32) -> Option<String> {
        let resource = self.resources.iter().find(|r| r.resolved_url == original_url)?;
        if resource.srcset.is_empty() {
            return Some(original_url.to_string());
        }
        // Buscar mejor candidato: el mas cercano a viewport_w pero sin excederlo
        // (descargar mas de lo que se ve es desperdicio)
        let mut best: Option<&SrcsetCandidate> = None;
        let mut best_w: u32 = 0;
        for cand in &resource.srcset {
            if cand.descriptor.ends_with('w') {
                if let Ok(w) = cand.descriptor[..cand.descriptor.len() - 1].parse::<u32>() {
                    if w <= viewport_w && w > best_w {
                        best = Some(cand);
                        best_w = w;
                    }
                }
            } else if cand.descriptor.ends_with('x') {
                if best.is_none() {
                    best = Some(cand);
                }
            }
        }
        // Si ninguno entra en viewport, tomar el mas pequeno
        if best.is_none() {
            let mut min_w: u32 = u32::MAX;
            for cand in &resource.srcset {
                if cand.descriptor.ends_with('w') {
                    if let Ok(w) = cand.descriptor[..cand.descriptor.len() - 1].parse::<u32>() {
                        if w < min_w {
                            min_w = w;
                            best = Some(cand);
                        }
                    }
                }
            }
        }
        best.map(|c| c.url.clone())
    }

    pub fn stats(&self) -> (usize, usize, usize, usize) {
        let loaded = self.resources.iter().filter(|r| r.loaded).count();
        (self.resources.len(), loaded, self.failed_count, self.total_loaded_bytes)
    }

    pub fn clear(&mut self) {
        self.resources.clear();
        self.loaded.clear();
        self.total_loaded_bytes = 0;
        self.failed_count = 0;
    }
}

/// Parsea srcset attribute
fn parse_srcset(s: &str) -> Vec<SrcsetCandidate> {
    s.split(',')
        .filter_map(|entry| {
            let entry = entry.trim();
            if entry.is_empty() { return None; }
            let parts: Vec<&str> = entry.split_whitespace().collect();
            if parts.is_empty() { return None; }
            Some(SrcsetCandidate {
                url: parts[0].to_string(),
                descriptor: parts.get(1).unwrap_or(&"").to_string(),
            })
        })
        .collect()
}

fn extract_attr(tag: &str, attr: &str) -> Option<String> {
    let lower = tag.to_lowercase();
    let pattern = format!("{}=\"", attr);
    if let Some(idx) = lower.find(&pattern) {
        let start = idx + pattern.len();
        if let Some(end_idx) = tag[start..].find('"') {
            return Some(tag[start..start + end_idx].to_string());
        }
    }
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
    fn test_extract_img() {
        let mut loader = ImageLoader::new();
        let html = r#"<img src="logo.png" alt="Logo">"#;
        loader.extract_from_html("https://example.com/", html);
        assert_eq!(loader.resources.len(), 1);
        assert_eq!(loader.resources[0].resolved_url, "https://example.com/logo.png");
        assert_eq!(loader.resources[0].alt, "Logo");
    }

    #[test]
    fn test_extract_with_size() {
        let mut loader = ImageLoader::new();
        let html = r#"<img src="x.jpg" width="200" height="100">"#;
        loader.extract_from_html("https://x.com/", html);
        assert_eq!(loader.resources[0].width, Some(200));
        assert_eq!(loader.resources[0].height, Some(100));
    }

    #[test]
    fn test_lazy_loading() {
        let mut loader = ImageLoader::new();
        let html = r#"<img src="a.jpg" loading="lazy">"#;
        loader.extract_from_html("https://x.com/", html);
        assert_eq!(loader.resources[0].loading, LoadingStrategy::Lazy);
    }

    #[test]
    fn test_favicon() {
        let mut loader = ImageLoader::new();
        let html = r#"<link rel="icon" href="favicon.ico">"#;
        loader.extract_from_html("https://x.com/", html);
        assert_eq!(loader.resources.len(), 1);
        assert_eq!(loader.resources[0].image_type, ImageType::Favicon);
    }

    #[test]
    fn test_set_loaded() {
        let mut loader = ImageLoader::new();
        let html = r#"<img src="a.jpg">"#;
        loader.extract_from_html("https://x.com/", html);
        loader.set_loaded("https://x.com/a.jpg", vec![0xFF; 100]);
        assert!(loader.resources[0].loaded);
        assert_eq!(loader.total_loaded_bytes, 100);
    }

    #[test]
    fn test_mark_failed() {
        let mut loader = ImageLoader::new();
        let html = r#"<img src="a.jpg">"#;
        loader.extract_from_html("https://x.com/", html);
        loader.mark_failed("https://x.com/a.jpg", "404");
        assert_eq!(loader.resources[0].error, Some("404".to_string()));
    }

    #[test]
    fn test_no_duplicates() {
        let mut loader = ImageLoader::new();
        let html = r#"<img src="a.jpg"><img src="a.jpg">"#;
        loader.extract_from_html("https://x.com/", html);
        assert_eq!(loader.resources.len(), 1);
    }

    #[test]
    fn test_srcset_parse() {
        let s = "small.jpg 480w, medium.jpg 800w, large.jpg 1200w";
        let parsed = parse_srcset(s);
        assert_eq!(parsed.len(), 3);
        assert_eq!(parsed[0].descriptor, "480w");
    }

    #[test]
    fn test_best_srcset() {
        let mut loader = ImageLoader::new();
        let html = r#"<img src="default.jpg" srcset="small.jpg 480w, medium.jpg 800w, large.jpg 1200w">"#;
        loader.extract_from_html("https://x.com/", html);
        let best = loader.best_srcset("https://x.com/default.jpg", 1000);
        assert_eq!(best, Some("https://x.com/medium.jpg".to_string()));
    }

    #[test]
    fn test_clear() {
        let mut loader = ImageLoader::new();
        loader.extract_from_html("https://x.com/", r#"<img src="a.jpg">"#);
        loader.clear();
        assert_eq!(loader.resources.len(), 0);
    }

    #[test]
    fn test_stats() {
        let mut loader = ImageLoader::new();
        let html = r#"<img src="a.jpg"><img src="b.jpg">"#;
        loader.extract_from_html("https://x.com/", html);
        loader.set_loaded("https://x.com/a.jpg", vec![1, 2, 3]);
        let (total, loaded, failed, size) = loader.stats();
        assert_eq!(total, 2);
        assert_eq!(loaded, 1);
        assert_eq!(failed, 0);
        assert_eq!(size, 3);
    }
}
