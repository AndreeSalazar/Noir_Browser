//! Test E2E: Pagina web real con flexbox + position + media queries
//!
//! Simula una pagina web moderna y verifica que los componentes
//! trabajan juntos correctamente.

#[cfg(test)]
mod tests {
    use no_chromium::parsers::flexbox_v2::{FlexContainer, FlexDirection, FlexItem};
    use no_chromium::parsers::media_queries::{MediaContext, MediaFeature, MediaRule, MediaType};
    use no_chromium::parsers::position_v2::{
        apply_positioning, Position, PositionOffsets, PositionedNode, PositioningContext,
    };
    use no_chromium::parsers::page_document::PageDocument;

    /// E2E: Twitter-like header (flexbox row + position sticky)
    #[test]
    fn test_e2e_twitter_like_header() {
        // Top bar flexbox row
        let mut header = FlexContainer::new(0.0, 0.0, 1280.0, 60.0);
        header.direction = FlexDirection::Row;
        header.gap = 20.0;
        for i in 0..3 {
            header.items.push(no_chromium::layout::FlexItem {
                x: 0.0, y: 0.0,
                w: 0.0, h: 60.0,
                flex_grow: if i == 1 { 1.0 } else { 0.0 },
                flex_shrink: 0.0,
                flex_basis: 100.0,
                order: 0,
                align_self: None,
            });
        }
        header.layout();
        // Logo a la izquierda, contenido al medio (flex-grow), menu a la derecha
        assert_eq!(header.items[0].x, 0.0);
        // El item 1 (flex) toma el espacio restante
        assert!(header.items[1].w > 100.0);
        // Menu al final
        assert!(header.items[2].x > header.items[1].x + header.items[1].w);
    }

    /// E2E: YouTube-like video card (flexbox column + thumbnail)
    #[test]
    fn test_e2e_youtube_like_card() {
        // Card con flex-direction: column
        let mut card = FlexContainer::new(0.0, 0.0, 300.0, 250.0);
        card.direction = FlexDirection::Column;
        card.gap = 8.0;
        for h in [180, 40, 20] {
            card.items.push(no_chromium::layout::FlexItem {
                x: 0.0, y: 0.0,
                w: 300.0, h: h as f32,
                flex_grow: 0.0, flex_shrink: 0.0,
                flex_basis: h as f32,
                order: 0,
                align_self: None,
            });
        }
        card.layout();
        // Thumbnail arriba
        assert_eq!(card.items[0].y, 0.0);
        assert_eq!(card.items[0].h, 180.0);
        // Title en medio
        assert_eq!(card.items[1].y, 188.0);  // 180 + 8 gap
        // Metadata abajo
        assert_eq!(card.items[2].y, 236.0);  // 188 + 40 + 8
    }

    /// E2E: Mobile vs desktop layout
    #[test]
    fn test_e2e_responsive_layout() {
        // Mobile: < 768px - menu colapsado
        let mut mobile_ctx = MediaContext::mobile();
        mobile_ctx.viewport_w = 375.0;
        let mut mobile_rule = MediaRule::new(MediaType::All, ".menu-collapse");
        mobile_rule.add_feature(MediaFeature::from_str("max-width", "768").unwrap());
        assert!(mobile_rule.evaluate(&mobile_ctx));

        // Desktop: >= 1024px - menu expandido
        let mut desktop_ctx = MediaContext::desktop();
        desktop_ctx.viewport_w = 1920.0;
        let mut desktop_rule = MediaRule::new(MediaType::All, ".menu-expand");
        desktop_rule.add_feature(MediaFeature::from_str("min-width", "1024").unwrap());
        assert!(desktop_rule.evaluate(&desktop_ctx));
    }

    /// E2E: Sticky sidebar que se queda arriba
    #[test]
    fn test_e2e_sticky_sidebar() {
        let mut sidebar = PositionedNode {
            id: 1,
            position: Position::Sticky,
            x: 0.0, y: 0.0, w: 250.0, h: 500.0,
            offsets: PositionOffsets::default(),
            z_index: 10,
            sticky_top: Some(60.0),
        };
        let containing = PositioningContext::new(0.0, 0.0, 0.0, 0.0);
        let mut viewport = PositioningContext::new(0.0, 0.0, 1280.0, 720.0);
        viewport.scroll_y = 100.0;  // scroll > sticky_top (60)
        apply_positioning(&mut sidebar, 0.0, 200.0, 250.0, 500.0, &containing, &viewport);
        // Sticky triggered: y queda en sticky_top
        assert_eq!(sidebar.y, 60.0);
    }

    /// E2E: HTML real con flexbox + position combinados
    #[test]
    fn test_e2e_real_layout_combination() {
        let html = r#"
        <!DOCTYPE html>
        <html>
        <head>
            <style>
                .header {
                    display: flex;
                    position: sticky;
                    top: 0;
                    background: white;
                    z-index: 100;
                }
                .logo { width: 200px; }
                .content { flex: 1; }
                .menu { width: 200px; }
                @media (max-width: 768px) {
                    .menu { display: none; }
                }
            </style>
        </head>
        <body>
            <div class="header">
                <div class="logo">Logo</div>
                <div class="content">Content</div>
                <div class="menu">Menu</div>
            </div>
        </body>
        </html>
        "#;
        let page = PageDocument::from_html("https://example.com", html);
        // El parser debe parsear correctamente
        assert!(!page.text_blocks.is_empty() || !page.dom_nodes.is_empty(),
            "Page should be parsed");
    }

    /// E2E: Dark mode toggle (media query + theme)
    #[test]
    fn test_e2e_dark_mode_toggle() {
        let html = r#"
        <html>
        <head>
            <style>
                body { background: white; color: black; }
                @media (prefers-color-scheme: dark) {
                    body { background: #121212; color: #e0e0e0; }
                }
            </style>
        </head>
        <body>Content</body>
        </html>
        "#;
        // Parse the page
        let page = PageDocument::from_html("https://x.com", html);
        // Eval media query
        let mut dark_rule = MediaRule::new(MediaType::All, "");
        dark_rule.add_feature(MediaFeature::from_str("prefers-color-scheme", "dark").unwrap());
        assert!(dark_rule.evaluate(&MediaContext::dark()));
        // La pagina debe parsear correctamente
        assert!(!page.style_blocks.is_empty() || !page.dom_nodes.is_empty());
    }
}
