//! Style Sharing Cache (Firefox Stylo-inspired)
//!
//! Basado en el algoritmo de Firefox Quantum CSS (Stylo):
//! - Si dos nodos tienen el mismo computed style, ambos apuntan al mismo struct
//! - Ahorra memoria: no duplicamos styles para siblings similares
//! - Ahorra tiempo: skip selector matching en restyles
//!
//! Los checks de Stylo:
//! 1. ¿Mismo id, classes, pseudo-selectors?
//! 2. ¿Mismos inline styles?
//! 3. ¿Mismo parent style?
//! 4. ¿Mismas "dependency bits" (selectores especiales como :first-child)?

use std::collections::HashMap;
use std::sync::Arc;

/// Hash de las propiedades de un DOM node
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct StyleCacheKey {
    pub tag: String,
    pub id: Option<String>,
    pub classes: Vec<String>,
    pub inline_style: Option<String>,
    pub parent_style_id: u64,
    pub pseudo_state: u8,  // bits: hover, focus, first-child, last-child, etc
}

impl StyleCacheKey {
    pub fn new(
        tag: &str,
        id: Option<&str>,
        classes: &[String],
        inline_style: Option<&str>,
        parent_style_id: u64,
        pseudo_state: u8,
    ) -> Self {
        Self {
            tag: tag.to_string(),
            id: id.map(String::from),
            classes: classes.to_vec(),
            inline_style: inline_style.map(String::from),
            parent_style_id,
            pseudo_state,
        }
    }
}

/// Computed style - compartido por multiples nodos
#[derive(Debug, Clone)]
pub struct SharedStyle {
    pub id: u64,
    pub font_size: f32,
    pub color: [u8; 4],
    pub bg_color: Option<[u8; 4]>,
    pub display: String,
    pub margin: (f32, f32, f32, f32),
    pub padding: (f32, f32, f32, f32),
    pub border: (u32, u32, u32, u32),
    pub bold: bool,
    pub ref_count: u32,
}

/// Cache de styles compartidos (Firefox Stylo-style)
pub struct StyleSharingCache {
    /// Map: StyleCacheKey -> SharedStyle ID
    cache: HashMap<StyleCacheKey, u64>,
    /// Map: Style ID -> SharedStyle
    styles: HashMap<u64, SharedStyle>,
    /// Contador de hits/misses para estadisticas
    hits: u64,
    misses: u64,
    next_id: u64,
}

impl StyleSharingCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            styles: HashMap::new(),
            hits: 0,
            misses: 0,
            next_id: 1,
        }
    }

    /// Buscar style compartido o crear uno nuevo
    pub fn get_or_create(
        &mut self,
        key: StyleCacheKey,
        compute: impl FnOnce() -> SharedStyle,
    ) -> Arc<SharedStyle> {
        if let Some(&id) = self.cache.get(&key) {
            self.hits += 1;
            if let Some(s) = self.styles.get_mut(&id) {
                s.ref_count += 1;
                return Arc::new(s.clone());
            }
        }
        self.misses += 1;
        let mut style = compute();
        style.id = self.next_id;
        self.next_id += 1;
        style.ref_count = 1;
        let id = style.id;
        self.styles.insert(id, style.clone());
        self.cache.insert(key, id);
        Arc::new(style)
    }

    /// Estadisticas del cache
    pub fn stats(&self) -> (u64, u64, f32) {
        let total = self.hits + self.misses;
        let hit_rate = if total > 0 { self.hits as f32 / total as f32 } else { 0.0 };
        (self.hits, self.misses, hit_rate)
    }

    /// Limpiar cache
    pub fn clear(&mut self) {
        self.cache.clear();
        self.styles.clear();
        self.hits = 0;
        self.misses = 0;
    }

    /// Tamano del cache
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
}

impl Default for StyleSharingCache {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_creation() {
        let cache = StyleSharingCache::new();
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_cache_hit_miss() {
        let mut cache = StyleSharingCache::new();
        let key1 = StyleCacheKey::new("div", None, &[], None, 0, 0);

        let s1 = cache.get_or_create(key1.clone(), || SharedStyle {
            id: 0, font_size: 14.0, color: [0, 0, 0, 255], bg_color: None,
            display: "block".into(), margin: (0.0, 0.0, 0.0, 0.0),
            padding: (0.0, 0.0, 0.0, 0.0), border: (0, 0, 0, 0),
            bold: false, ref_count: 0,
        });
        assert_eq!(cache.len(), 1);

        // Misma key -> hit
        let s2 = cache.get_or_create(key1.clone(), || SharedStyle {
            id: 0, font_size: 14.0, color: [0, 0, 0, 255], bg_color: None,
            display: "block".into(), margin: (0.0, 0.0, 0.0, 0.0),
            padding: (0.0, 0.0, 0.0, 0.0), border: (0, 0, 0, 0),
            bold: false, ref_count: 0,
        });
        assert_eq!(s1.id, s2.id);
        let (hits, misses, _) = cache.stats();
        assert_eq!(hits, 1);
        assert_eq!(misses, 1);
    }

    #[test]
    fn test_different_keys_different_styles() {
        let mut cache = StyleSharingCache::new();
        let key1 = StyleCacheKey::new("div", None, &[], None, 0, 0);
        let key2 = StyleCacheKey::new("span", None, &[], None, 0, 0);

        let _ = cache.get_or_create(key1, || SharedStyle {
            id: 0, font_size: 14.0, color: [0, 0, 0, 255], bg_color: None,
            display: "block".into(), margin: (0.0, 0.0, 0.0, 0.0),
            padding: (0.0, 0.0, 0.0, 0.0), border: (0, 0, 0, 0),
            bold: false, ref_count: 0,
        });
        let _ = cache.get_or_create(key2, || SharedStyle {
            id: 0, font_size: 14.0, color: [0, 0, 0, 255], bg_color: None,
            display: "inline".into(), margin: (0.0, 0.0, 0.0, 0.0),
            padding: (0.0, 0.0, 0.0, 0.0), border: (0, 0, 0, 0),
            bold: false, ref_count: 0,
        });
        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn test_pseudo_state_creates_different() {
        let mut cache = StyleSharingCache::new();
        let key_normal = StyleCacheKey::new("p", None, &[], None, 0, 0);
        let key_hover = StyleCacheKey::new("p", None, &[], None, 0, 1);

        let s1 = cache.get_or_create(key_normal, || SharedStyle {
            id: 0, font_size: 14.0, color: [200, 200, 200, 255], bg_color: None,
            display: "block".into(), margin: (0.0, 0.0, 0.0, 0.0),
            padding: (0.0, 0.0, 0.0, 0.0), border: (0, 0, 0, 0),
            bold: false, ref_count: 0,
        });
        let s2 = cache.get_or_create(key_hover, || SharedStyle {
            id: 0, font_size: 14.0, color: [255, 255, 255, 255], bg_color: None,
            display: "block".into(), margin: (0.0, 0.0, 0.0, 0.0),
            padding: (0.0, 0.0, 0.0, 0.0), border: (0, 0, 0, 0),
            bold: false, ref_count: 0,
        });
        assert_ne!(s1.id, s2.id);
    }

    #[test]
    fn test_clear() {
        let mut cache = StyleSharingCache::new();
        let key = StyleCacheKey::new("div", None, &[], None, 0, 0);
        let _ = cache.get_or_create(key, || SharedStyle {
            id: 0, font_size: 14.0, color: [0, 0, 0, 255], bg_color: None,
            display: "block".into(), margin: (0.0, 0.0, 0.0, 0.0),
            padding: (0.0, 0.0, 0.0, 0.0), border: (0, 0, 0, 0),
            bold: false, ref_count: 0,
        });
        assert!(!cache.is_empty());
        cache.clear();
        assert!(cache.is_empty());
    }

    #[test]
    fn test_hit_rate() {
        let mut cache = StyleSharingCache::new();
        let key = StyleCacheKey::new("p", None, &[], None, 0, 0);
        for _ in 0..10 {
            let _ = cache.get_or_create(key.clone(), || SharedStyle {
                id: 0, font_size: 14.0, color: [0, 0, 0, 255], bg_color: None,
                display: "block".into(), margin: (0.0, 0.0, 0.0, 0.0),
                padding: (0.0, 0.0, 0.0, 0.0), border: (0, 0, 0, 0),
                bold: false, ref_count: 0,
            });
        }
        let (hits, misses, rate) = cache.stats();
        assert_eq!(hits, 9);
        assert_eq!(misses, 1);
        assert!((rate - 0.9).abs() < 0.01);
    }
}
