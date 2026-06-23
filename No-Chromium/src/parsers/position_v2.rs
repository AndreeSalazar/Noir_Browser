//! CSS Position (FASE C3)
//!
//! Implementa el sistema de positioning de CSS:
//! - static: por defecto, en el flow normal
//! - relative: offset respecto a su posicion normal
//! - absolute: posicionado respecto al ancestor positioned mas cercano
//! - fixed: posicionado respecto al viewport
//! - sticky: como relative pero se "pega" al hacer scroll
//!
//! Con z-index para stacking contexts.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Position {
    Static,
    Relative,
    Absolute,
    Fixed,
    Sticky,
}

impl Position {
    pub fn from_str(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "relative" => Position::Relative,
            "absolute" => Position::Absolute,
            "fixed" => Position::Fixed,
            "sticky" => Position::Sticky,
            _ => Position::Static,
        }
    }
    pub fn is_positioned(&self) -> bool {
        !matches!(self, Position::Static)
    }
    pub fn is_out_of_flow(&self) -> bool {
        matches!(self, Position::Absolute | Position::Fixed)
    }
}

/// Offsets de un elemento positioned
#[derive(Debug, Clone, Copy, Default)]
pub struct PositionOffsets {
    pub top: Option<f32>,
    pub right: Option<f32>,
    pub bottom: Option<f32>,
    pub left: Option<f32>,
}

impl PositionOffsets {
    pub fn from_css(top: Option<&str>, right: Option<&str>, bottom: Option<&str>, left: Option<&str>) -> Self {
        Self {
            top: top.and_then(parse_px),
            right: right.and_then(parse_px),
            bottom: bottom.and_then(parse_px),
            left: left.and_then(parse_px),
        }
    }
}

fn parse_px(s: &str) -> Option<f32> {
    s.trim().strip_suffix("px").and_then(|n| n.trim().parse().ok())
        .or_else(|| s.trim().parse().ok())
}

/// Un nodo con position
#[derive(Debug, Clone)]
pub struct PositionedNode {
    pub id: u64,
    pub position: Position,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub offsets: PositionOffsets,
    pub z_index: i32,
    /// Para sticky: la posicion Y donde se "pega"
    pub sticky_top: Option<f32>,
}

/// Contenedor (relative o viewport) que sirve como referencia
#[derive(Debug, Clone)]
pub struct PositioningContext {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub scroll_x: f32,
    pub scroll_y: f32,
}

impl PositioningContext {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h, scroll_x: 0.0, scroll_y: 0.0 }
    }
}

/// Aplica positioning a un nodo
pub fn apply_positioning(
    node: &mut PositionedNode,
    in_flow_x: f32,
    in_flow_y: f32,
    in_flow_w: f32,
    in_flow_h: f32,
    containing: &PositioningContext,
    viewport: &PositioningContext,
) {
    match node.position {
        Position::Static => {
            // Mantener su posicion natural en el flow
            node.x = in_flow_x;
            node.y = in_flow_y;
        }
        Position::Relative => {
            node.x = in_flow_x;
            node.y = in_flow_y;
            if let Some(t) = node.offsets.top {
                node.y += t;
            } else if let Some(b) = node.offsets.bottom {
                node.y = in_flow_y - b;
            }
            if let Some(l) = node.offsets.left {
                node.x += l;
            } else if let Some(r) = node.offsets.right {
                node.x = in_flow_x - r;
            }
        }
        Position::Absolute => {
            if let Some(t) = node.offsets.top {
                node.y = containing.y + t;
            } else if let Some(b) = node.offsets.bottom {
                node.y = containing.y + containing.h - node.h - b;
            } else {
                node.y = in_flow_y;
            }
            if let Some(l) = node.offsets.left {
                node.x = containing.x + l;
            } else if let Some(r) = node.offsets.right {
                node.x = containing.x + containing.w - node.w - r;
            } else {
                node.x = in_flow_x;
            }
        }
        Position::Fixed => {
            if let Some(t) = node.offsets.top {
                node.y = viewport.y + t - viewport.scroll_y;
            } else if let Some(b) = node.offsets.bottom {
                node.y = viewport.y + viewport.h - node.h - b;
            }
            if let Some(l) = node.offsets.left {
                node.x = viewport.x + l;
            } else if let Some(r) = node.offsets.right {
                node.x = viewport.x + viewport.w - node.w - r;
            }
        }
        Position::Sticky => {
            if let Some(stick_y) = node.sticky_top {
                if viewport.scroll_y > stick_y {
                    node.y = viewport.y + stick_y;
                } else {
                    node.y = in_flow_y;
                }
            } else {
                node.y = in_flow_y;
            }
            node.x = in_flow_x;
        }
    }
}

/// Stacking context - para z-index
#[derive(Debug, Clone)]
pub struct StackingContext {
    pub nodes: Vec<PositionedNode>,
    pub z_auto: i32,
}

impl StackingContext {
    pub fn new() -> Self {
        Self { nodes: Vec::new(), z_auto: 0 }
    }
    pub fn add(&mut self, node: PositionedNode) {
        self.nodes.push(node);
        self.z_auto += 1;
    }
    /// Ordena por z-index (menor primero)
    pub fn sorted(&self) -> Vec<&PositionedNode> {
        let mut sorted: Vec<&PositionedNode> = self.nodes.iter().collect();
        sorted.sort_by_key(|n| n.z_index);
        sorted
    }
    /// z-index auto incrementa
    pub fn next_auto_z(&mut self) -> i32 {
        let z = self.z_auto;
        self.z_auto += 1;
        z
    }
}

impl Default for StackingContext {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_from_str() {
        assert_eq!(Position::from_str("static"), Position::Static);
        assert_eq!(Position::from_str("relative"), Position::Relative);
        assert_eq!(Position::from_str("absolute"), Position::Absolute);
        assert_eq!(Position::from_str("fixed"), Position::Fixed);
        assert_eq!(Position::from_str("sticky"), Position::Sticky);
    }

    #[test]
    fn test_position_is_positioned() {
        assert!(!Position::Static.is_positioned());
        assert!(Position::Relative.is_positioned());
        assert!(Position::Absolute.is_positioned());
        assert!(Position::Fixed.is_positioned());
    }

    #[test]
    fn test_position_is_out_of_flow() {
        assert!(!Position::Static.is_out_of_flow());
        assert!(!Position::Relative.is_out_of_flow());
        assert!(Position::Absolute.is_out_of_flow());
        assert!(Position::Fixed.is_out_of_flow());
    }

    #[test]
    fn test_offsets_from_css() {
        let o = PositionOffsets::from_css(Some("10px"), None, Some("20px"), None);
        assert_eq!(o.top, Some(10.0));
        assert_eq!(o.bottom, Some(20.0));
        assert_eq!(o.left, None);
    }

    #[test]
    fn test_static_positioning() {
        let mut n = PositionedNode {
            id: 1, position: Position::Static,
            x: 0.0, y: 0.0, w: 100.0, h: 50.0,
            offsets: PositionOffsets::default(),
            z_index: 0, sticky_top: None,
        };
        let viewport = PositioningContext::new(0.0, 0.0, 1000.0, 800.0);
        let containing = PositioningContext::new(0.0, 0.0, 500.0, 500.0);
        apply_positioning(&mut n, 10.0, 20.0, 100.0, 50.0, &containing, &viewport);
        assert_eq!(n.x, 10.0);
        assert_eq!(n.y, 20.0);
    }

    #[test]
    fn test_relative_offset_top_left() {
        let mut n = PositionedNode {
            id: 1, position: Position::Relative,
            x: 0.0, y: 0.0, w: 100.0, h: 50.0,
            offsets: PositionOffsets { top: Some(10.0), left: Some(20.0), ..Default::default() },
            z_index: 0, sticky_top: None,
        };
        let viewport = PositioningContext::new(0.0, 0.0, 1000.0, 800.0);
        let containing = PositioningContext::new(0.0, 0.0, 500.0, 500.0);
        apply_positioning(&mut n, 50.0, 50.0, 100.0, 50.0, &containing, &viewport);
        assert_eq!(n.x, 70.0);
        assert_eq!(n.y, 60.0);
    }

    #[test]
    fn test_absolute_positioning() {
        let mut n = PositionedNode {
            id: 1, position: Position::Absolute,
            x: 0.0, y: 0.0, w: 100.0, h: 50.0,
            offsets: PositionOffsets { top: Some(10.0), left: Some(20.0), ..Default::default() },
            z_index: 0, sticky_top: None,
        };
        let viewport = PositioningContext::new(0.0, 0.0, 1000.0, 800.0);
        let containing = PositioningContext::new(100.0, 50.0, 500.0, 500.0);
        apply_positioning(&mut n, 0.0, 0.0, 100.0, 50.0, &containing, &viewport);
        assert_eq!(n.x, 120.0);
        assert_eq!(n.y, 60.0);
    }

    #[test]
    fn test_absolute_bottom_right() {
        let mut n = PositionedNode {
            id: 1, position: Position::Absolute,
            x: 0.0, y: 0.0, w: 100.0, h: 50.0,
            offsets: PositionOffsets { bottom: Some(20.0), right: Some(30.0), ..Default::default() },
            z_index: 0, sticky_top: None,
        };
        let viewport = PositioningContext::new(0.0, 0.0, 1000.0, 800.0);
        let containing = PositioningContext::new(0.0, 0.0, 500.0, 500.0);
        apply_positioning(&mut n, 0.0, 0.0, 100.0, 50.0, &containing, &viewport);
        assert_eq!(n.x, 370.0);
        assert_eq!(n.y, 430.0);
    }

    #[test]
    fn test_fixed_positioning() {
        let mut n = PositionedNode {
            id: 1, position: Position::Fixed,
            x: 0.0, y: 0.0, w: 100.0, h: 50.0,
            offsets: PositionOffsets { top: Some(10.0), left: Some(20.0), ..Default::default() },
            z_index: 0, sticky_top: None,
        };
        let mut viewport = PositioningContext::new(0.0, 0.0, 1000.0, 800.0);
        viewport.scroll_y = 200.0;
        let containing = PositioningContext::new(100.0, 50.0, 500.0, 500.0);
        apply_positioning(&mut n, 0.0, 0.0, 100.0, 50.0, &containing, &viewport);
        assert_eq!(n.x, 20.0);
        assert_eq!(n.y, -190.0);
    }

    #[test]
    fn test_sticky_positioning() {
        let mut n = PositionedNode {
            id: 1, position: Position::Sticky,
            x: 0.0, y: 0.0, w: 100.0, h: 50.0,
            offsets: PositionOffsets::default(),
            z_index: 0, sticky_top: Some(50.0),
        };
        let mut viewport = PositioningContext::new(0.0, 0.0, 1000.0, 800.0);
        viewport.scroll_y = 100.0;
        let containing = PositioningContext::new(0.0, 0.0, 500.0, 500.0);
        apply_positioning(&mut n, 100.0, 200.0, 100.0, 50.0, &containing, &viewport);
        assert_eq!(n.y, 50.0);
    }

    #[test]
    fn test_sticky_no_trigger() {
        let mut n = PositionedNode {
            id: 1, position: Position::Sticky,
            x: 0.0, y: 0.0, w: 100.0, h: 50.0,
            offsets: PositionOffsets::default(),
            z_index: 0, sticky_top: Some(50.0),
        };
        let mut viewport = PositioningContext::new(0.0, 0.0, 1000.0, 800.0);
        viewport.scroll_y = 20.0;
        let containing = PositioningContext::new(0.0, 0.0, 500.0, 500.0);
        apply_positioning(&mut n, 100.0, 200.0, 100.0, 50.0, &containing, &viewport);
        assert_eq!(n.y, 200.0);
    }

    #[test]
    fn test_stacking_context() {
        let mut ctx = StackingContext::new();
        ctx.add(PositionedNode { id: 1, position: Position::Absolute, x: 0.0, y: 0.0, w: 0.0, h: 0.0, offsets: PositionOffsets::default(), z_index: 5, sticky_top: None });
        ctx.add(PositionedNode { id: 2, position: Position::Absolute, x: 0.0, y: 0.0, w: 0.0, h: 0.0, offsets: PositionOffsets::default(), z_index: 2, sticky_top: None });
        ctx.add(PositionedNode { id: 3, position: Position::Absolute, x: 0.0, y: 0.0, w: 0.0, h: 0.0, offsets: PositionOffsets::default(), z_index: 10, sticky_top: None });
        let sorted = ctx.sorted();
        assert_eq!(sorted[0].id, 2);
        assert_eq!(sorted[1].id, 1);
        assert_eq!(sorted[2].id, 3);
    }

    #[test]
    fn test_stacking_auto_z() {
        let mut ctx = StackingContext::new();
        assert_eq!(ctx.next_auto_z(), 0);
        assert_eq!(ctx.next_auto_z(), 1);
        assert_eq!(ctx.next_auto_z(), 2);
    }
}
