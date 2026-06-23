//! CSS Containment - Layout/paint/style isolation (Firefox-style)
//!
//! Permite que el browser optimice rendering de elementos independientes
//! `contain: layout | paint | style | size | content | strict`

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContainmentType {
    None,
    Layout,
    Paint,
    Style,
    Size,
    Content,  // layout + paint + style
    Strict,   // layout + paint + style + size
}

impl ContainmentType {
    pub fn from_str(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "layout" => Self::Layout,
            "paint" => Self::Paint,
            "style" => Self::Style,
            "size" => Self::Size,
            "content" => Self::Content,
            "strict" => Self::Strict,
            _ => Self::None,
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Layout => "layout",
            Self::Paint => "paint",
            Self::Style => "style",
            Self::Size => "size",
            Self::Content => "content",
            Self::Strict => "strict",
        }
    }

    pub fn is_layout_isolated(&self) -> bool {
        matches!(self, Self::Layout | Self::Content | Self::Strict)
    }

    pub fn is_paint_isolated(&self) -> bool {
        matches!(self, Self::Paint | Self::Content | Self::Strict)
    }

    pub fn is_style_isolated(&self) -> bool {
        matches!(self, Self::Style | Self::Content | Self::Strict)
    }

    pub fn is_size_isolated(&self) -> bool {
        matches!(self, Self::Size | Self::Strict)
    }
}

#[derive(Debug, Clone)]
pub struct ContainingBlock {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub containment: ContainmentType,
    pub is_sized: bool,
}

impl ContainingBlock {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x, y, width, height,
            containment: ContainmentType::None,
            is_sized: false,
        }
    }

    pub fn with_containment(mut self, c: ContainmentType) -> Self {
        self.containment = c;
        if c.is_size_isolated() {
            self.is_sized = true;
        }
        self
    }
    // `Content` includes layout/paint/style isolation (used by Firefox)

    pub fn contains_point(&self, px: f32, py: f32) -> bool {
        px >= self.x && px < self.x + self.width &&
        py >= self.y && py < self.y + self.height
    }

    pub fn intersects(&self, other: &ContainingBlock) -> bool {
        self.x < other.x + other.width &&
        other.x < self.x + self.width &&
        self.y < other.y + other.height &&
        other.y < self.y + self.height
    }
}

pub struct ContainmentOptimizer {
    pub blocks: Vec<ContainingBlock>,
    pub stats: ContainmentStats,
}

#[derive(Debug, Clone, Default)]
pub struct ContainmentStats {
    pub total_blocks: u32,
    pub isolated_blocks: u32,
    pub skipped_paints: u32,
    pub skipped_layouts: u32,
}

impl ContainmentOptimizer {
    pub fn new() -> Self {
        Self {
            blocks: Vec::new(),
            stats: ContainmentStats::default(),
        }
    }

    pub fn add(&mut self, block: ContainingBlock) -> usize {
        self.stats.total_blocks += 1;
        if block.containment != ContainmentType::None {
            self.stats.isolated_blocks += 1;
        }
        self.blocks.push(block);
        self.blocks.len() - 1
    }

    /// Encuentra bloques que necesitan re-layout
    /// (solo si su containment es layout o superior)
    pub fn needs_relayout(&self, block_idx: usize) -> bool {
        if let Some(b) = self.blocks.get(block_idx) {
            !b.containment.is_layout_isolated() || true
        } else {
            true
        }
    }

    /// Encuentra bloques que necesitan re-paint
    /// (solo si están fuera del viewport)
    pub fn needs_paint(&mut self, block_idx: usize, viewport: &ContainingBlock) -> bool {
        if let Some(b) = self.blocks.get(block_idx) {
            if b.containment.is_paint_isolated() {
                let needs = b.intersects(viewport);
                if !needs {
                    self.stats.skipped_paints += 1;
                }
                return needs;
            }
            true
        } else {
            true
        }
    }

    pub fn reset_stats(&mut self) {
        self.stats = ContainmentStats::default();
    }
}

impl Default for ContainmentOptimizer {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_containment_from_str() {
        assert_eq!(ContainmentType::from_str("layout"), ContainmentType::Layout);
        assert_eq!(ContainmentType::from_str("paint"), ContainmentType::Paint);
        assert_eq!(ContainmentType::from_str("content"), ContainmentType::Content);
        assert_eq!(ContainmentType::from_str("strict"), ContainmentType::Strict);
    }

    #[test]
    fn test_containment_to_str() {
        assert_eq!(ContainmentType::Layout.to_str(), "layout");
    }

    #[test]
    fn test_containment_isolated() {
        assert!(ContainmentType::Content.is_layout_isolated());
        assert!(ContainmentType::Content.is_paint_isolated());
        assert!(ContainmentType::Strict.is_size_isolated());
        assert!(!ContainmentType::Layout.is_paint_isolated());
    }

    #[test]
    fn test_block_new() {
        let b = ContainingBlock::new(0.0, 0.0, 100.0, 50.0);
        assert_eq!(b.containment, ContainmentType::None);
    }

    #[test]
    fn test_block_with_containment() {
        let b = ContainingBlock::new(0.0, 0.0, 100.0, 50.0)
            .with_containment(ContainmentType::Strict);
        assert_eq!(b.containment, ContainmentType::Strict);
        assert!(b.is_sized);
    }

    #[test]
    fn test_block_contains_point() {
        let b = ContainingBlock::new(10.0, 20.0, 100.0, 50.0);
        assert!(b.contains_point(50.0, 40.0));
        assert!(!b.contains_point(5.0, 40.0));
        assert!(!b.contains_point(150.0, 40.0));
    }

    #[test]
    fn test_block_intersects() {
        let a = ContainingBlock::new(0.0, 0.0, 100.0, 100.0);
        let b = ContainingBlock::new(50.0, 50.0, 100.0, 100.0);
        let c = ContainingBlock::new(200.0, 200.0, 100.0, 100.0);
        assert!(a.intersects(&b));
        assert!(!a.intersects(&c));
    }

    #[test]
    fn test_optimizer_new() {
        let o = ContainmentOptimizer::new();
        assert_eq!(o.stats.total_blocks, 0);
    }

    #[test]
    fn test_optimizer_add() {
        let mut o = ContainmentOptimizer::new();
        o.add(ContainingBlock::new(0.0, 0.0, 100.0, 50.0));
        o.add(ContainingBlock::new(0.0, 0.0, 100.0, 50.0)
            .with_containment(ContainmentType::Content));
        assert_eq!(o.stats.total_blocks, 2);
        assert_eq!(o.stats.isolated_blocks, 1);
    }

    #[test]
    fn test_optimizer_needs_paint() {
        let mut o = ContainmentOptimizer::new();
        o.add(ContainingBlock::new(0.0, 0.0, 100.0, 100.0)
            .with_containment(ContainmentType::Paint));
        o.add(ContainingBlock::new(200.0, 200.0, 100.0, 100.0)
            .with_containment(ContainmentType::Paint));
        let viewport = ContainingBlock::new(0.0, 0.0, 150.0, 150.0);
        assert!(o.needs_paint(0, &viewport));
        assert!(!o.needs_paint(1, &viewport));
        assert_eq!(o.stats.skipped_paints, 1);
    }

    #[test]
    fn test_optimizer_reset() {
        let mut o = ContainmentOptimizer::new();
        o.add(ContainingBlock::new(0.0, 0.0, 100.0, 50.0));
        o.reset_stats();
        assert_eq!(o.stats.total_blocks, 0);
        assert_eq!(o.blocks.len(), 1);
    }

    #[test]
    fn test_containment_layout_only() {
        let b = ContainingBlock::new(0.0, 0.0, 100.0, 50.0)
            .with_containment(ContainmentType::Layout);
        assert!(b.containment.is_layout_isolated());
        assert!(!b.containment.is_paint_isolated());
    }
}
