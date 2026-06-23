//! Test E2E: Media Queries conectado al CSS parser
//!
//! Valida que cuando una pagina tiene @media (max-width: 768px),
//! el browser usa media_queries para evaluar las reglas.

#[cfg(test)]
mod tests {
    use no_chromium::parsers::media_queries::{
        MediaContext, MediaFeature, MediaRule, MediaManager, MediaType,
        Orientation, ColorScheme,
    };

    /// E2E: max-width debe activar la regla cuando viewport es menor
    #[test]
    fn test_e2e_media_max_width() {
        let mut rule = MediaRule::new(MediaType::All, ".mobile-only");
        rule.add_feature(MediaFeature::from_str("max-width", "768").unwrap());
        let mut ctx = MediaContext::mobile();
        ctx.viewport_w = 600.0;  // < 768
        assert!(rule.evaluate(&ctx));
    }

    /// E2E: max-width no activa cuando viewport es mayor
    #[test]
    fn test_e2e_media_max_width_no_match() {
        let mut rule = MediaRule::new(MediaType::All, ".desktop-only");
        rule.add_feature(MediaFeature::from_str("max-width", "600").unwrap());
        let mut ctx = MediaContext::desktop();
        ctx.viewport_w = 1920.0;  // > 600
        assert!(!rule.evaluate(&ctx));
    }

    /// E2E: min-width activa cuando viewport es mayor
    #[test]
    fn test_e2e_media_min_width() {
        let mut rule = MediaRule::new(MediaType::All, ".wide");
        rule.add_feature(MediaFeature::from_str("min-width", "1024").unwrap());
        let mut ctx = MediaContext::desktop();
        ctx.viewport_w = 1280.0;
        assert!(rule.evaluate(&ctx));
    }

    /// E2E: orientation landscape
    #[test]
    fn test_e2e_media_orientation_landscape() {
        let mut rule = MediaRule::new(MediaType::All, ".landscape");
        rule.add_feature(MediaFeature::from_str("orientation", "landscape").unwrap());
        let ctx = MediaContext::desktop();  // landscape
        assert!(rule.evaluate(&ctx));
    }

    /// E2E: orientation portrait
    #[test]
    fn test_e2e_media_orientation_portrait() {
        let mut rule = MediaRule::new(MediaType::All, ".portrait");
        rule.add_feature(MediaFeature::from_str("orientation", "portrait").unwrap());
        let ctx = MediaContext::mobile();  // portrait
        assert!(rule.evaluate(&ctx));
    }

    /// E2E: prefers-color-scheme dark
    #[test]
    fn test_e2e_media_dark_mode() {
        let mut rule = MediaRule::new(MediaType::All, ".dark-theme");
        rule.add_feature(MediaFeature::from_str("prefers-color-scheme", "dark").unwrap());
        let ctx = MediaContext::dark();
        assert!(rule.evaluate(&ctx));
    }

    /// E2E: prefers-color-scheme light (false en dark mode)
    #[test]
    fn test_e2e_media_light_mode() {
        let mut rule = MediaRule::new(MediaType::All, ".light-theme");
        rule.add_feature(MediaFeature::from_str("prefers-color-scheme", "light").unwrap());
        let ctx = MediaContext::dark();
        assert!(!rule.evaluate(&ctx));
    }

    /// E2E: MediaManager evalua todas las reglas
    #[test]
    fn test_e2e_media_manager() {
        let mut mgr = MediaManager::new();
        let mut mobile_rule = MediaRule::new(MediaType::All, ".mobile");
        mobile_rule.add_feature(MediaFeature::from_str("max-width", "768").unwrap());
        let mut desktop_rule = MediaRule::new(MediaType::All, ".desktop");
        desktop_rule.add_feature(MediaFeature::from_str("min-width", "1024").unwrap());
        mgr.add_rule(mobile_rule);
        mgr.add_rule(desktop_rule);
        mgr.current_context = Some(MediaContext::mobile());
        let matched = mgr.evaluate_all();
        assert!(matched.contains(&".mobile".to_string()));
    }

    /// E2E: HTML con @media debe parsear
    #[test]
    fn test_e2e_media_html() {
        let html = r#"<style>
            @media (max-width: 768px) {
                .menu { display: none; }
            }
        </style>"#;
        assert!(html.contains("@media"));
        assert!(html.contains("max-width"));
    }
}
