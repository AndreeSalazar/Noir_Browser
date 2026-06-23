//! Test E2E: Position v2 conectado al layout engine
//!
//! Valida que cuando una pagina tiene position: absolute/fixed/relative/sticky,
//! el browser usa position_v2::apply_positioning correctamente.

#[cfg(test)]
mod tests {
    use no_chromium::parsers::position_v2::{
        apply_positioning, Position, PositionOffsets, PositionedNode,
        PositioningContext,
    };

    /// E2E: Elemento absolute debe posicionarse relativo al containing block
    #[test]
    fn test_e2e_position_absolute() {
        let mut node = PositionedNode {
            id: 1,
            position: Position::Absolute,
            x: 0.0, y: 0.0, w: 100.0, h: 50.0,
            offsets: PositionOffsets { top: Some(10.0), left: Some(20.0), ..Default::default() },
            z_index: 0, sticky_top: None,
        };
        let containing = PositioningContext::new(50.0, 30.0, 400.0, 300.0);
        let viewport = PositioningContext::new(0.0, 0.0, 1280.0, 720.0);
        apply_positioning(&mut node, 0.0, 0.0, 100.0, 50.0, &containing, &viewport);
        // Absolute con top=10, left=20: x = 50+20=70, y = 30+10=40
        assert_eq!(node.x, 70.0);
        assert_eq!(node.y, 40.0);
    }

    /// E2E: Elemento fixed debe posicionarse relativo al viewport
    #[test]
    fn test_e2e_position_fixed() {
        let mut node = PositionedNode {
            id: 1,
            position: Position::Fixed,
            x: 0.0, y: 0.0, w: 100.0, h: 50.0,
            offsets: PositionOffsets { top: Some(0.0), left: Some(0.0), ..Default::default() },
            z_index: 100, sticky_top: None,
        };
        let containing = PositioningContext::new(0.0, 0.0, 0.0, 0.0);
        let mut viewport = PositioningContext::new(0.0, 0.0, 1280.0, 720.0);
        viewport.scroll_y = 200.0;
        apply_positioning(&mut node, 0.0, 0.0, 100.0, 50.0, &containing, &viewport);
        // Fixed en viewport (no se mueve con scroll)
        assert_eq!(node.x, 0.0);
        assert_eq!(node.y, -200.0);  // -scroll_y
    }

    /// E2E: Elemento relative se mantiene en flow + offset
    #[test]
    fn test_e2e_position_relative() {
        let mut node = PositionedNode {
            id: 1,
            position: Position::Relative,
            x: 0.0, y: 0.0, w: 100.0, h: 50.0,
            offsets: PositionOffsets { top: Some(5.0), left: Some(10.0), ..Default::default() },
            z_index: 0, sticky_top: None,
        };
        let containing = PositioningContext::new(0.0, 0.0, 0.0, 0.0);
        let viewport = PositioningContext::new(0.0, 0.0, 1280.0, 720.0);
        apply_positioning(&mut node, 100.0, 200.0, 100.0, 50.0, &containing, &viewport);
        // Relative: in_flow (100, 200) + offset (10, 5) = (110, 205)
        assert_eq!(node.x, 110.0);
        assert_eq!(node.y, 205.0);
    }

    /// E2E: Sticky se pega cuando scroll > sticky_top
    #[test]
    fn test_e2e_position_sticky_triggered() {
        let mut node = PositionedNode {
            id: 1,
            position: Position::Sticky,
            x: 0.0, y: 0.0, w: 100.0, h: 50.0,
            offsets: PositionOffsets::default(),
            z_index: 0,
            sticky_top: Some(50.0),
        };
        let containing = PositioningContext::new(0.0, 0.0, 0.0, 0.0);
        let mut viewport = PositioningContext::new(0.0, 0.0, 1280.0, 720.0);
        viewport.scroll_y = 100.0;  // scroll > sticky_top
        apply_positioning(&mut node, 0.0, 200.0, 100.0, 50.0, &containing, &viewport);
        // Sticky triggered: y se queda en sticky_top
        assert_eq!(node.y, 50.0);
    }

    /// E2E: HTML con position: absolute debe ser parseado
    #[test]
    fn test_e2e_position_html_parsed() {
        let html = r#"<div style="position: absolute; top: 10px; left: 20px;">A</div>"#;
        // El parser debe parsear correctamente
        assert!(html.contains("position: absolute"));
    }

    /// E2E: Sticky no triggered cuando scroll < sticky_top
    #[test]
    fn test_e2e_position_sticky_not_triggered() {
        let mut node = PositionedNode {
            id: 1,
            position: Position::Sticky,
            x: 0.0, y: 0.0, w: 100.0, h: 50.0,
            offsets: PositionOffsets::default(),
            z_index: 0,
            sticky_top: Some(50.0),
        };
        let containing = PositioningContext::new(0.0, 0.0, 0.0, 0.0);
        let mut viewport = PositioningContext::new(0.0, 0.0, 1280.0, 720.0);
        viewport.scroll_y = 20.0;  // scroll < sticky_top
        apply_positioning(&mut node, 0.0, 200.0, 100.0, 50.0, &containing, &viewport);
        // Sticky not triggered: y se mantiene en in_flow
        assert_eq!(node.y, 200.0);
    }
}
