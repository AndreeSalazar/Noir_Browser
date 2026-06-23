//! Flexbox Layout Engine - Layout CSS-like para Noir Browser
//!
//! Implementa flexbox y grid simplificado para mejorar el rendering.

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FlexDirection {
    Row,
    Column,
    RowReverse,
    ColumnReverse,
}

impl FlexDirection {
    pub fn from_str(s: &str) -> Self {
        match s.trim() {
            "row" => Self::Row,
            "row-reverse" => Self::RowReverse,
            "column" => Self::Column,
            "column-reverse" => Self::ColumnReverse,
            _ => Self::Row,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JustifyContent {
    FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

impl JustifyContent {
    pub fn from_str(s: &str) -> Self {
        match s.trim() {
            "flex-end" => Self::FlexEnd,
            "center" => Self::Center,
            "space-between" => Self::SpaceBetween,
            "space-around" => Self::SpaceAround,
            "space-evenly" => Self::SpaceEvenly,
            _ => Self::FlexStart,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AlignItems {
    FlexStart,
    FlexEnd,
    Center,
    Stretch,
    Baseline,
}

impl AlignItems {
    pub fn from_str(s: &str) -> Self {
        match s.trim() {
            "flex-end" => Self::FlexEnd,
            "center" => Self::Center,
            "stretch" => Self::Stretch,
            "baseline" => Self::Baseline,
            _ => Self::FlexStart,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FlexWrap {
    NoWrap,
    Wrap,
    WrapReverse,
}

impl FlexWrap {
    pub fn from_str(s: &str) -> Self {
        match s.trim() {
            "wrap" => Self::Wrap,
            "wrap-reverse" => Self::WrapReverse,
            _ => Self::NoWrap,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FlexContainer {
    pub direction: FlexDirection,
    pub justify: JustifyContent,
    pub align: AlignItems,
    pub wrap: FlexWrap,
    pub gap: i32,
    pub width: i32,
    pub height: i32,
}

impl FlexContainer {
    pub fn new(width: i32, height: i32) -> Self {
        Self {
            direction: FlexDirection::Row,
            justify: JustifyContent::FlexStart,
            align: AlignItems::Stretch,
            wrap: FlexWrap::NoWrap,
            gap: 0,
            width,
            height,
        }
    }

    pub fn row() -> Self {
        Self::new(800, 600)
    }

    /// Calcula posición X de un item según justify-content
    pub fn justify_x(&self, item_index: usize, item_w: i32, total_items: usize) -> i32 {
        let total_items_w = self.total_items_width(total_items, item_w);
        let free_space = self.width - total_items_w;
        match self.justify {
            JustifyContent::FlexStart => {
                // Acumula gaps: item 0 = 0, item 1 = w + gap, item 2 = 2*w + 2*gap
                let offset = self.gap * item_index as i32;
                item_index as i32 * item_w + offset
            }
            JustifyContent::FlexEnd => {
                let offset = self.gap * item_index as i32;
                free_space + item_index as i32 * item_w + offset
            }
            JustifyContent::Center => {
                let offset = self.gap * item_index as i32;
                free_space / 2 + item_index as i32 * item_w + offset
            }
            JustifyContent::SpaceBetween => {
                if total_items <= 1 { 0 } else {
                    let space = free_space / (total_items as i32 - 1);
                    let offset = self.gap * item_index as i32;
                    space * item_index as i32 + item_index as i32 * item_w + offset
                }
            }
            JustifyContent::SpaceAround => {
                let space = free_space / total_items as i32;
                let offset = self.gap * item_index as i32;
                (space * item_index as i32 + space / 2) + item_index as i32 * item_w + offset
            }
            JustifyContent::SpaceEvenly => {
                let space = free_space / (total_items as i32 + 1);
                let offset = self.gap * item_index as i32;
                space * (item_index as i32 + 1) + item_index as i32 * item_w + offset
            }
        }
    }

    /// Posición Y según align-items
    pub fn align_y(&self, item_h: i32) -> i32 {
        match self.align {
            AlignItems::FlexStart => 0,
            AlignItems::FlexEnd => self.height - item_h,
            AlignItems::Center => (self.height - item_h) / 2,
            AlignItems::Stretch => 0,
            AlignItems::Baseline => 0,
        }
    }

    fn total_items_width(&self, total_items: usize, item_w: i32) -> i32 {
        let items_w = item_w * total_items as i32;
        let gaps = self.gap * (total_items.saturating_sub(1)) as i32;
        items_w + gaps
    }
}

/// Grid container simplificado
#[derive(Debug, Clone)]
pub struct GridContainer {
    pub cols: u32,
    pub rows: u32,
    pub col_gap: i32,
    pub row_gap: i32,
    pub cell_w: i32,
    pub cell_h: i32,
    pub width: i32,
}

impl GridContainer {
    pub fn new(cols: u32, width: i32) -> Self {
        let col_gap = 16;
        let cell_w = (width - (col_gap * (cols as i32 - 1))) / cols as i32;
        Self {
            cols,
            rows: 0,
            col_gap,
            row_gap: 16,
            cell_w,
            cell_h: cell_w, // 1:1 por defecto
            width,
        }
    }

    /// Calcula (x, y) para un item en grid index
    pub fn position_for(&self, index: usize) -> (i32, i32) {
        let col = (index as u32) % self.cols;
        let row = (index as u32) / self.cols;
        let x = col as i32 * (self.cell_w + self.col_gap);
        let y = row as i32 * (self.cell_h + self.row_gap);
        (x, y)
    }

    /// Auto-fit: cols según items
    pub fn auto_fit(item_count: usize, width: i32) -> Self {
        let target_cell_w = 250;
        let cols = std::cmp::max(1, std::cmp::min(6, (width / target_cell_w) as u32));
        let mut grid = Self::new(cols, width);
        if item_count > 0 {
            grid.rows = ((item_count as u32 + cols - 1) / cols).max(1);
        }
        grid
    }

    /// Grid 16:9 (videos)
    pub fn video_grid(item_count: usize, width: i32) -> Self {
        let mut grid = Self::auto_fit(item_count, width);
        grid.cell_h = grid.cell_w * 9 / 16;
        grid
    }

    /// Calcula altura total
    pub fn total_height(&self) -> i32 {
        (self.cell_h + self.row_gap) * self.rows as i32 - self.row_gap
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flex_direction_from_str() {
        assert_eq!(FlexDirection::from_str("row"), FlexDirection::Row);
        assert_eq!(FlexDirection::from_str("column"), FlexDirection::Column);
        assert_eq!(FlexDirection::from_str("row-reverse"), FlexDirection::RowReverse);
    }

    #[test]
    fn test_justify_from_str() {
        assert_eq!(JustifyContent::from_str("center"), JustifyContent::Center);
        assert_eq!(JustifyContent::from_str("space-between"), JustifyContent::SpaceBetween);
    }

    #[test]
    fn test_align_from_str() {
        assert_eq!(AlignItems::from_str("center"), AlignItems::Center);
    }

    #[test]
    fn test_wrap_from_str() {
        assert_eq!(FlexWrap::from_str("wrap"), FlexWrap::Wrap);
    }

    #[test]
    fn test_flex_container_new() {
        let c = FlexContainer::new(800, 600);
        assert_eq!(c.width, 800);
        assert_eq!(c.direction, FlexDirection::Row);
    }

    #[test]
    fn test_flex_justify_flex_start() {
        let c = FlexContainer::new(800, 600);
        assert_eq!(c.justify_x(0, 100, 3), 0);
        assert_eq!(c.justify_x(1, 100, 3), 100);
        assert_eq!(c.justify_x(2, 100, 3), 200);
    }

    #[test]
    fn test_flex_justify_center() {
        let c = FlexContainer { justify: JustifyContent::Center, gap: 0, ..FlexContainer::new(800, 600) };
        // 3 items * 100 = 300, free = 500, center = 250
        assert_eq!(c.justify_x(0, 100, 3), 250);
        assert_eq!(c.justify_x(1, 100, 3), 350);
    }

    #[test]
    fn test_flex_justify_space_between() {
        let c = FlexContainer { justify: JustifyContent::SpaceBetween, gap: 0, ..FlexContainer::new(800, 600) };
        // 4 items * 100 = 400, free = 400, space = 400/3
        assert_eq!(c.justify_x(0, 100, 4), 0);
        assert!(c.justify_x(1, 100, 4) > 0);
    }

    #[test]
    fn test_flex_align_center() {
        let c = FlexContainer { align: AlignItems::Center, ..FlexContainer::new(800, 600) };
        // 600 - 100 = 500, /2 = 250
        assert_eq!(c.align_y(100), 250);
    }

    #[test]
    fn test_flex_align_stretch() {
        let c = FlexContainer { align: AlignItems::Stretch, ..FlexContainer::new(800, 600) };
        assert_eq!(c.align_y(100), 0);
    }

    #[test]
    fn test_grid_new() {
        let g = GridContainer::new(3, 800);
        assert_eq!(g.cols, 3);
        assert!(g.cell_w > 0);
    }

    #[test]
    fn test_grid_position_for() {
        let g = GridContainer::new(3, 800);
        let (x, y) = g.position_for(0);
        assert_eq!(x, 0);
        assert_eq!(y, 0);
        let (_, y) = g.position_for(3);
        assert!(y > 0);
    }

    #[test]
    fn test_grid_auto_fit() {
        let g = GridContainer::auto_fit(10, 1200);
        assert!(g.cols >= 3);
        assert!(g.cols <= 6);
    }

    #[test]
    fn test_grid_video_16_9() {
        let g = GridContainer::video_grid(8, 1200);
        let expected_h = g.cell_w * 9 / 16;
        assert_eq!(g.cell_h, expected_h);
    }

    #[test]
    fn test_grid_total_height() {
        let mut g = GridContainer::new(3, 800);
        g.rows = 2;
        g.cell_h = 100;
        g.row_gap = 10;
        // (100 + 10) * 2 - 10 = 210
        assert_eq!(g.total_height(), 210);
    }

    #[test]
    fn test_flex_with_gap() {
        let c = FlexContainer { gap: 20, ..FlexContainer::new(800, 600) };
        // 3 items * 100 + 2 gaps * 20 = 340, free = 460
        let c2 = FlexContainer { gap: 20, justify: JustifyContent::FlexStart, ..FlexContainer::new(800, 600) };
        // First item at 0
        assert_eq!(c2.justify_x(0, 100, 3), 0);
    }
}
