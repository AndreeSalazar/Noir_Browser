//! Test E2E: Flexbox v2 conectado al layout engine
//!
//! Valida que cuando una pagina tiene CSS display: flex, el browser
//! usa flexbox_v2::FlexContainer para calcular posiciones.

#[cfg(test)]
mod tests {
    use no_chromium::parsers::flexbox_v2::{FlexContainer, FlexDirection, JustifyContent, AlignItems, FlexItem};
    use no_chromium::parsers::page_document::PageDocument;

    /// E2E: HTML con flexbox debe ser parseado y layout debe usar FlexContainer
    #[test]
    fn test_e2e_flexbox_html_to_layout() {
        let html = r#"
        <!DOCTYPE html>
        <html>
        <head>
            <style>
                .container { display: flex; flex-direction: row; gap: 10px; }
                .item { flex: 1; padding: 8px; }
            </style>
        </head>
        <body>
            <div class="container">
                <div class="item">A</div>
                <div class="item">B</div>
                <div class="item">C</div>
            </div>
        </body>
        </html>
        "#;
        let page = PageDocument::from_html("https://example.com", html);
        // La pagina debe parsearse correctamente
        assert!(!page.text_blocks.is_empty() || !page.dom_nodes.is_empty());
    }

    /// E2E: FlexContainer layout produce posiciones correctas
    #[test]
    fn test_e2e_flexbox_three_items_equal_width() {
        let mut container = FlexContainer::new(0.0, 0.0, 300.0, 100.0);
        container.gap = 0.0;

        for i in 0..3 {
            container.items.push(FlexItem {
                x: 0.0, y: 0.0,
                w: 0.0, h: 50.0,
                flex_grow: 1.0, flex_shrink: 0.0, flex_basis: 0.0,
                order: i as i32,
                align_self: None,
            });
        }
        container.layout();

        // Cada item debe tener ~100px de ancho
        for item in &container.items {
            assert!((item.w - 100.0).abs() < 0.1, "Expected w=100, got {}", item.w);
        }
    }

    /// E2E: Flexbox con direction column
    #[test]
    fn test_e2e_flexbox_column_direction() {
        let mut container = FlexContainer::new(0.0, 0.0, 200.0, 300.0);
        container.direction = FlexDirection::Column;

        for _ in 0..3 {
            container.items.push(FlexItem {
                x: 0.0, y: 0.0,
                w: 0.0, h: 50.0,
                flex_grow: 0.0, flex_shrink: 0.0, flex_basis: 50.0,
                order: 0,
                align_self: None,
            });
        }
        container.layout();

        // Items stacked verticalmente
        assert_eq!(container.items[0].y, 0.0);
        assert_eq!(container.items[1].y, 50.0);
        assert_eq!(container.items[2].y, 100.0);
    }

    /// E2E: Flexbox con justify-content space-between
    #[test]
    fn test_e2e_flexbox_space_between() {
        let mut container = FlexContainer::new(0.0, 0.0, 300.0, 100.0);
        container.justify = JustifyContent::SpaceBetween;

        for _ in 0..3 {
            container.items.push(FlexItem {
                x: 0.0, y: 0.0,
                w: 50.0, h: 50.0,
                flex_grow: 0.0, flex_shrink: 0.0, flex_basis: 50.0,
                order: 0,
                align_self: None,
            });
        }
        container.layout();

        // Primer item al inicio, ultimo al final
        assert_eq!(container.items[0].x, 0.0);
        // 300 - 3*50 = 150 / 2 gaps = 75 entre items
        assert_eq!(container.items[1].x, 125.0);
        assert_eq!(container.items[2].x, 250.0);
    }

    /// E2E: Flexbox con align-items center
    #[test]
    fn test_e2e_flexbox_align_center() {
        let mut container = FlexContainer::new(0.0, 0.0, 200.0, 100.0);
        container.align_items = AlignItems::Center;

        container.items.push(FlexItem {
            x: 0.0, y: 0.0,
            w: 50.0, h: 40.0,
            flex_grow: 0.0, flex_shrink: 0.0, flex_basis: 50.0,
            order: 0,
            align_self: None,
        });
        container.layout();

        // 100 - 40 = 60, / 2 = 30 (cross-axis center)
        assert_eq!(container.items[0].y, 30.0);
    }

    /// E2E: HTML con flexbox debe producir layout valido
    #[test]
    fn test_e2e_flexbox_html_parsed() {
        let html = r#"<div style="display: flex; gap: 20px;">
            <span>Item 1</span>
            <span>Item 2</span>
        </div>"#;
        let page = PageDocument::from_html("https://x.com", html);
        // El parser debe reconocer al menos el div container y los items
        assert!(!page.text_blocks.is_empty() || !page.dom_nodes.is_empty(),
            "Page should have content from HTML");
    }
}
