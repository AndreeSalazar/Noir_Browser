//! CSS Media Queries (FASE C4)
//!
//! Media queries permiten CSS condicional basado en:
//! - viewport width/height
//! - device width/height
//! - orientation
//! - resolution
//! - prefers-color-scheme (dark/light)
//! - prefers-reduced-motion
//!
//! Ejemplo: @media (max-width: 768px) { ... } @media (prefers-color-scheme: dark) { ... }

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MediaType {
    All,
    Screen,
    Print,
    Speech,
}

impl MediaType {
    pub fn from_str(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "screen" => MediaType::Screen,
            "print" => MediaType::Print,
            "speech" => MediaType::Speech,
            _ => MediaType::All,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Orientation {
    Portrait,
    Landscape,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColorScheme {
    Light,
    Dark,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MediaFeature {
    Width(f32),                // viewport width en px
    Height(f32),
    MinWidth(f32),
    MaxWidth(f32),
    MinHeight(f32),
    MaxHeight(f32),
    DeviceWidth(f32),
    DeviceHeight(f32),
    Orientation(Orientation),
    Resolution(f32),            // dpi
    MinResolution(f32),
    MaxResolution(f32),
    ColorScheme(ColorScheme),
    PrefersReducedMotion(bool),
    AspectRatio(f32, f32),      // width / height
    MinAspectRatio(f32, f32),
    MaxAspectRatio(f32, f32),
}

impl MediaFeature {
    pub fn from_str(name: &str, value: &str) -> Option<Self> {
        let value = value.trim();
        let name = name.trim().to_lowercase();
        // Parse value con posibles sufijos (px, dpi, dpcm)
        let parse_px = |v: &str| -> Option<f32> {
            v.strip_suffix("px")
                .or_else(|| v.strip_suffix("dpi"))
                .or_else(|| v.strip_suffix("dpcm"))
                .unwrap_or(v)
                .trim()
                .parse()
                .ok()
        };
        match name.as_str() {
            "width" => parse_px(value).map(MediaFeature::Width),
            "height" => parse_px(value).map(MediaFeature::Height),
            "min-width" => parse_px(value).map(MediaFeature::MinWidth),
            "max-width" => parse_px(value).map(MediaFeature::MaxWidth),
            "min-height" => parse_px(value).map(MediaFeature::MinHeight),
            "max-height" => parse_px(value).map(MediaFeature::MaxHeight),
            "device-width" => parse_px(value).map(MediaFeature::DeviceWidth),
            "device-height" => parse_px(value).map(MediaFeature::DeviceHeight),
            "orientation" => match value {
                "portrait" => Some(MediaFeature::Orientation(Orientation::Portrait)),
                "landscape" => Some(MediaFeature::Orientation(Orientation::Landscape)),
                _ => None,
            },
            "resolution" => parse_px(value).map(MediaFeature::Resolution),
            "min-resolution" => parse_px(value).map(MediaFeature::MinResolution),
            "max-resolution" => parse_px(value).map(MediaFeature::MaxResolution),
            "prefers-color-scheme" => match value {
                "light" => Some(MediaFeature::ColorScheme(ColorScheme::Light)),
                "dark" => Some(MediaFeature::ColorScheme(ColorScheme::Dark)),
                _ => None,
            },
            "prefers-reduced-motion" => match value {
                "reduce" => Some(MediaFeature::PrefersReducedMotion(true)),
                "no-preference" => Some(MediaFeature::PrefersReducedMotion(false)),
                _ => None,
            },
            "aspect-ratio" => parse_aspect_ratio(value).map(|(w, h)| MediaFeature::AspectRatio(w, h)),
            "min-aspect-ratio" => parse_aspect_ratio(value).map(|(w, h)| MediaFeature::MinAspectRatio(w, h)),
            "max-aspect-ratio" => parse_aspect_ratio(value).map(|(w, h)| MediaFeature::MaxAspectRatio(w, h)),
            _ => None,
        }
    }

    pub fn matches(&self, context: &MediaContext) -> bool {
        match self {
            MediaFeature::Width(w) => (context.viewport_w - w).abs() < 0.5,
            MediaFeature::Height(h) => (context.viewport_h - h).abs() < 0.5,
            MediaFeature::MinWidth(w) => context.viewport_w >= *w,
            MediaFeature::MaxWidth(w) => context.viewport_w <= *w,
            MediaFeature::MinHeight(h) => context.viewport_h >= *h,
            MediaFeature::MaxHeight(h) => context.viewport_h <= *h,
            MediaFeature::DeviceWidth(w) => (context.device_w - w).abs() < 0.5,
            MediaFeature::DeviceHeight(h) => (context.device_h - h).abs() < 0.5,
            MediaFeature::Orientation(o) => context.orientation == *o,
            MediaFeature::Resolution(r) => (context.dpi - r).abs() < 0.5,
            MediaFeature::MinResolution(r) => context.dpi >= *r,
            MediaFeature::MaxResolution(r) => context.dpi <= *r,
            MediaFeature::ColorScheme(c) => context.color_scheme == *c,
            MediaFeature::PrefersReducedMotion(b) => context.reduced_motion == *b,
            MediaFeature::AspectRatio(w, h) => {
                let ar = context.viewport_w / context.viewport_h;
                let expected = w / h;
                (ar - expected).abs() < 0.01
            }
            MediaFeature::MinAspectRatio(w, h) => {
                context.viewport_w / context.viewport_h >= w / h
            }
            MediaFeature::MaxAspectRatio(w, h) => {
                context.viewport_w / context.viewport_h <= w / h
            }
        }
    }
}

fn parse_aspect_ratio(s: &str) -> Option<(f32, f32)> {
    let parts: Vec<&str> = s.split('/').collect();
    if parts.len() == 2 {
        let w: f32 = parts[0].trim().parse().ok()?;
        let h: f32 = parts[1].trim().parse().ok()?;
        Some((w, h))
    } else if let Ok(v) = s.parse() {
        Some((v, 1.0))
    } else {
        None
    }
}

/// Contexto del media query
#[derive(Debug, Clone)]
pub struct MediaContext {
    pub viewport_w: f32,
    pub viewport_h: f32,
    pub device_w: f32,
    pub device_h: f32,
    pub dpi: f32,
    pub orientation: Orientation,
    pub color_scheme: ColorScheme,
    pub reduced_motion: bool,
}

impl MediaContext {
    pub fn desktop() -> Self {
        Self {
            viewport_w: 1920.0,
            viewport_h: 1080.0,
            device_w: 1920.0,
            device_h: 1080.0,
            dpi: 96.0,
            orientation: Orientation::Landscape,
            color_scheme: ColorScheme::Light,
            reduced_motion: false,
        }
    }

    pub fn mobile() -> Self {
        Self {
            viewport_w: 375.0,
            viewport_h: 667.0,
            device_w: 375.0,
            device_h: 667.0,
            dpi: 326.0,
            orientation: Orientation::Portrait,
            color_scheme: ColorScheme::Light,
            reduced_motion: false,
        }
    }

    pub fn dark() -> Self {
        let mut c = Self::desktop();
        c.color_scheme = ColorScheme::Dark;
        c
    }
}

/// Una regla @media con condiciones
#[derive(Debug, Clone)]
pub struct MediaRule {
    pub media_type: MediaType,
    pub features: Vec<MediaFeature>,
    pub selector: String,
    pub declarations: HashMap<String, String>,
    pub matched: bool,  // cached result
}

impl MediaRule {
    pub fn new(media_type: MediaType, selector: &str) -> Self {
        Self {
            media_type,
            features: Vec::new(),
            selector: selector.to_string(),
            declarations: HashMap::new(),
            matched: false,
        }
    }

    pub fn add_feature(&mut self, feature: MediaFeature) {
        self.features.push(feature);
    }

    pub fn add_declaration(&mut self, key: &str, value: &str) {
        self.declarations.insert(key.to_string(), value.to_string());
    }

    /// Evaluar si la regla matches el contexto
    pub fn evaluate(&mut self, context: &MediaContext) -> bool {
        if self.media_type != MediaType::All && self.media_type != context_to_media_type(context) {
            self.matched = false;
            return false;
        }
        let result = self.features.iter().all(|f| f.matches(context));
        self.matched = result;
        result
    }
}

fn context_to_media_type(_c: &MediaContext) -> MediaType {
    // Por ahora siempre Screen
    MediaType::Screen
}

/// Manager de media rules
#[derive(Debug, Default)]
pub struct MediaManager {
    pub rules: Vec<MediaRule>,
    pub current_context: Option<MediaContext>,
}

impl MediaManager {
    pub fn new() -> Self { Self::default() }
    pub fn add_rule(&mut self, rule: MediaRule) {
        self.rules.push(rule);
    }
    /// Re-evalua todas las reglas con el contexto actual
    pub fn evaluate_all(&mut self) -> Vec<String> {
        let ctx = match &self.current_context {
            Some(c) => c.clone(),
            None => return Vec::new(),
        };
        let mut matched_selectors = Vec::new();
        for rule in &mut self.rules {
            if rule.evaluate(&ctx) {
                matched_selectors.push(rule.selector.clone());
            }
        }
        matched_selectors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_media_type_from_str() {
        assert_eq!(MediaType::from_str("all"), MediaType::All);
        assert_eq!(MediaType::from_str("screen"), MediaType::Screen);
        assert_eq!(MediaType::from_str("print"), MediaType::Print);
    }

    #[test]
    fn test_feature_max_width() {
        let f = MediaFeature::from_str("max-width", "768").unwrap();
        let mut ctx = MediaContext::mobile();
        ctx.viewport_w = 600.0;
        assert!(f.matches(&ctx));
        ctx.viewport_w = 1024.0;
        assert!(!f.matches(&ctx));
    }

    #[test]
    fn test_feature_min_width() {
        let f = MediaFeature::from_str("min-width", "1024").unwrap();
        let mut ctx = MediaContext::desktop();
        ctx.viewport_w = 1920.0;
        assert!(f.matches(&ctx));
        ctx.viewport_w = 800.0;
        assert!(!f.matches(&ctx));
    }

    #[test]
    fn test_feature_orientation() {
        let f = MediaFeature::from_str("orientation", "landscape").unwrap();
        let mut ctx = MediaContext::desktop();
        assert!(f.matches(&ctx));
        ctx.orientation = Orientation::Portrait;
        assert!(!f.matches(&ctx));
    }

    #[test]
    fn test_feature_color_scheme() {
        let f = MediaFeature::from_str("prefers-color-scheme", "dark").unwrap();
        let mut ctx = MediaContext::dark();
        assert!(f.matches(&ctx));
        ctx.color_scheme = ColorScheme::Light;
        assert!(!f.matches(&ctx));
    }

    #[test]
    fn test_feature_resolution() {
        let f = MediaFeature::from_str("min-resolution", "300dpi").unwrap();
        let mut ctx = MediaContext::mobile();
        ctx.dpi = 326.0;
        assert!(f.matches(&ctx));
    }

    #[test]
    fn test_feature_aspect_ratio() {
        let f = MediaFeature::from_str("aspect-ratio", "16/9").unwrap();
        let mut ctx = MediaContext::desktop();
        ctx.viewport_w = 1920.0;
        ctx.viewport_h = 1080.0;
        assert!(f.matches(&ctx));
    }

    #[test]
    fn test_feature_reduced_motion() {
        let f = MediaFeature::from_str("prefers-reduced-motion", "reduce").unwrap();
        let mut ctx = MediaContext::desktop();
        ctx.reduced_motion = true;
        assert!(f.matches(&ctx));
        ctx.reduced_motion = false;
        assert!(!f.matches(&ctx));
    }

    #[test]
    fn test_media_rule_match() {
        let mut rule = MediaRule::new(MediaType::All, ".mobile-only");
        rule.add_feature(MediaFeature::from_str("max-width", "768px").unwrap());
        let mut ctx = MediaContext::mobile();
        ctx.viewport_w = 600.0;
        assert!(rule.evaluate(&ctx));
    }

    #[test]
    fn test_media_rule_no_match() {
        let mut rule = MediaRule::new(MediaType::All, ".desktop-only");
        rule.add_feature(MediaFeature::from_str("min-width", "1024px").unwrap());
        let mut ctx = MediaContext::mobile();
        ctx.viewport_w = 600.0;
        assert!(!rule.evaluate(&ctx));
    }

    #[test]
    fn test_media_manager() {
        let mut mgr = MediaManager::new();
        let mut rule1 = MediaRule::new(MediaType::All, ".mobile");
        rule1.add_feature(MediaFeature::from_str("max-width", "768px").unwrap());
        let mut rule2 = MediaRule::new(MediaType::All, ".desktop");
        rule2.add_feature(MediaFeature::from_str("min-width", "1024px").unwrap());
        mgr.add_rule(rule1);
        mgr.add_rule(rule2);
        mgr.current_context = Some(MediaContext::mobile());
        let matched = mgr.evaluate_all();
        // mobile: 600px, matches .mobile
        assert!(matched.contains(&".mobile".to_string()));
    }

    #[test]
    fn test_desktop_context() {
        let ctx = MediaContext::desktop();
        assert_eq!(ctx.viewport_w, 1920.0);
        assert_eq!(ctx.orientation, Orientation::Landscape);
    }

    #[test]
    fn test_mobile_context() {
        let ctx = MediaContext::mobile();
        assert_eq!(ctx.viewport_w, 375.0);
        assert_eq!(ctx.orientation, Orientation::Portrait);
    }

    #[test]
    fn test_dark_context() {
        let ctx = MediaContext::dark();
        assert_eq!(ctx.color_scheme, ColorScheme::Dark);
    }
}
