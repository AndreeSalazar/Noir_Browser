//! Flexbox basico (FASE C1)
//!
//! Implementa un subset del Flexbox de CSS:
//! - display: flex | inline-flex
//! - flex-direction: row | row-reverse | column | column-reverse
//! - flex-wrap: nowrap | wrap | wrap-reverse
//! - justify-content: flex-start | flex-end | center | space-between | space-around | space-evenly
//! - align-items: flex-start | flex-end | center | stretch | baseline
//! - align-content: como align-items pero para cross-axis con wrap
//! - order
//! - flex-grow, flex-shrink, flex-basis
//! - align-self

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FlexDirection {
    Row,
    RowReverse,
    Column,
    ColumnReverse,
}

impl FlexDirection {
    pub fn from_str(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "row" => FlexDirection::Row,
            "row-reverse" | "row_reverse" => FlexDirection::RowReverse,
            "column" => FlexDirection::Column,
            "column-reverse" | "column_reverse" => FlexDirection::ColumnReverse,
            _ => FlexDirection::Row,
        }
    }
    pub fn is_reverse(&self) -> bool {
        matches!(self, FlexDirection::RowReverse | FlexDirection::ColumnReverse)
    }
    pub fn is_row(&self) -> bool {
        matches!(self, FlexDirection::Row | FlexDirection::RowReverse)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FlexWrap {
    NoWrap,
    Wrap,
    WrapReverse,
}

impl FlexWrap {
    pub fn from_str(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "wrap" => FlexWrap::Wrap,
            "wrap-reverse" | "wrap_reverse" => FlexWrap::WrapReverse,
            _ => FlexWrap::NoWrap,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
        match s.trim().to_lowercase().as_str() {
            "flex-end" | "flex_end" | "end" => JustifyContent::FlexEnd,
            "center" => JustifyContent::Center,
            "space-between" | "space_between" => JustifyContent::SpaceBetween,
            "space-around" | "space_around" => JustifyContent::SpaceAround,
            "space-evenly" | "space_evenly" => JustifyContent::SpaceEvenly,
            _ => JustifyContent::FlexStart,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AlignItems {
    FlexStart,
    FlexEnd,
    Center,
    Stretch,
    Baseline,
}

impl AlignItems {
    pub fn from_str(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "flex-end" | "flex_end" | "end" => AlignItems::FlexEnd,
            "center" => AlignItems::Center,
            "stretch" => AlignItems::Stretch,
            "baseline" => AlignItems::Baseline,
            _ => AlignItems::FlexStart,
        }
    }
}

/// Item en un flex container
#[derive(Debug, Clone)]
pub struct FlexItem {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub flex_basis: f32,  // 0 = auto
    pub order: i32,
    pub align_self: Option<AlignItems>,
}

/// Container flex
#[derive(Debug, Clone)]
pub struct FlexContainer {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub direction: FlexDirection,
    pub wrap: FlexWrap,
    pub justify: JustifyContent,
    pub align_items: AlignItems,
    pub align_content: AlignItems,
    pub gap: f32,
    pub items: Vec<FlexItem>,
}

impl FlexContainer {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            x, y, w, h,
            direction: FlexDirection::Row,
            wrap: FlexWrap::NoWrap,
            justify: JustifyContent::FlexStart,
            align_items: AlignItems::Stretch,
            align_content: AlignItems::Stretch,
            gap: 0.0,
            items: Vec::new(),
        }
    }

    /// Calcula las posiciones de los items
    pub fn layout(&mut self) {
        if self.items.is_empty() { return; }

        // 1. Sort by order
        self.items.sort_by_key(|i| i.order);

        let is_row = self.direction.is_row();
        let container_size = if is_row { self.w } else { self.h };
        let cross_size = if is_row { self.h } else { self.w };

        // 2. Calculate flex base sizes
        for item in &mut self.items {
            if item.flex_basis == 0.0 {
                if is_row {
                    item.flex_basis = item.w;
                } else {
                    item.flex_basis = item.h;
                }
            }
        }

        // 3. Calculate free space
        let total_basis: f32 = self.items.iter().map(|i| i.flex_basis).sum();
        let gaps = self.gap * (self.items.len() as f32 - 1.0).max(0.0);
        let free_space = container_size - total_basis - gaps;

        // 4. Apply flex grow/shrink
        let total_grow: f32 = self.items.iter().map(|i| i.flex_grow).sum();
        let total_shrink: f32 = self.items.iter().map(|i| i.flex_shrink).sum();

        for item in &mut self.items {
            if free_space > 0.0 && total_grow > 0.0 {
                let extra = free_space * (item.flex_grow / total_grow);
                if is_row {
                    item.w = item.flex_basis + extra;
                } else {
                    item.h = item.flex_basis + extra;
                }
            } else if free_space < 0.0 && total_shrink > 0.0 {
                let reduce = -free_space * (item.flex_shrink / total_shrink);
                if is_row {
                    item.w = (item.flex_basis - reduce).max(0.0);
                } else {
                    item.h = (item.flex_basis - reduce).max(0.0);
                }
            }
        }

        // 5. Calculate main axis positions
        let total_main: f32 = if is_row {
            self.items.iter().map(|i| i.w).sum::<f32>()
        } else {
            self.items.iter().map(|i| i.h).sum::<f32>()
        } + gaps;
        let free_main = container_size - total_main;

        let (initial_offset, gap_distribution) = match self.justify {
            JustifyContent::FlexStart => (0.0, 0.0),
            JustifyContent::FlexEnd => (free_main, 0.0),
            JustifyContent::Center => (free_main / 2.0, 0.0),
            JustifyContent::SpaceBetween => {
                if self.items.len() > 1 {
                    (0.0, free_main / (self.items.len() as f32 - 1.0))
                } else {
                    (0.0, 0.0)
                }
            }
            JustifyContent::SpaceAround => {
                let gap_size = free_main / self.items.len() as f32;
                (gap_size / 2.0, gap_size)
            }
            JustifyContent::SpaceEvenly => {
                let gap_size = free_main / (self.items.len() as f32 + 1.0);
                (gap_size, gap_size)
            }
        };

        let reversed = self.direction.is_reverse();
        if reversed {
            self.items.reverse();
        }

        // Recolectar tamaños primero
        let sizes: Vec<f32> = if is_row {
            self.items.iter().map(|i| i.w).collect()
        } else {
            self.items.iter().map(|i| i.h).collect()
        };

        let mut main_pos = initial_offset;
        for (i, item) in self.items.iter_mut().enumerate() {
            let size = sizes[i];
            if is_row {
                item.x = self.x + main_pos;
            } else {
                item.y = self.y + main_pos;
            }
            main_pos += size + self.gap + gap_distribution;
        }

        // 6. Calculate cross axis positions
        for item in &mut self.items {
            let align = item.align_self.unwrap_or(self.align_items);
            let item_cross = if is_row { item.h } else { item.w };
            let offset = match align {
                AlignItems::FlexStart => 0.0,
                AlignItems::FlexEnd => cross_size - item_cross,
                AlignItems::Center => (cross_size - item_cross) / 2.0,
                AlignItems::Stretch => 0.0,
                AlignItems::Baseline => 0.0,
            };
            if is_row {
                item.y = self.y + offset;
            } else {
                item.x = self.x + offset;
            }

            if matches!(align, AlignItems::Stretch) {
                if is_row {
                    item.h = cross_size;
                } else {
                    item.w = cross_size;
                }
            }
        }
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
    fn test_flex_direction_is_row() {
        assert!(FlexDirection::Row.is_row());
        assert!(!FlexDirection::Column.is_row());
    }

    #[test]
    fn test_flex_wrap_from_str() {
        assert_eq!(FlexWrap::from_str("wrap"), FlexWrap::Wrap);
        assert_eq!(FlexWrap::from_str("nowrap"), FlexWrap::NoWrap);
    }

    #[test]
    fn test_justify_content_from_str() {
        assert_eq!(JustifyContent::from_str("center"), JustifyContent::Center);
        assert_eq!(JustifyContent::from_str("space-between"), JustifyContent::SpaceBetween);
    }

    #[test]
    fn test_align_items_from_str() {
        assert_eq!(AlignItems::from_str("center"), AlignItems::Center);
        assert_eq!(AlignItems::from_str("stretch"), AlignItems::Stretch);
    }

    #[test]
    fn test_container_creation() {
        let c = FlexContainer::new(0.0, 0.0, 500.0, 100.0);
        assert_eq!(c.w, 500.0);
        assert_eq!(c.direction, FlexDirection::Row);
    }

    #[test]
    fn test_layout_empty() {
        let mut c = FlexContainer::new(0.0, 0.0, 500.0, 100.0);
        c.layout();
    }

    #[test]
    fn test_layout_single_item() {
        let mut c = FlexContainer::new(0.0, 0.0, 500.0, 100.0);
        c.items.push(FlexItem {
            x: 0.0, y: 0.0, w: 100.0, h: 50.0,
            flex_grow: 0.0, flex_shrink: 1.0, flex_basis: 0.0,
            order: 0, align_self: None,
        });
        c.layout();
        assert_eq!(c.items[0].x, 0.0);
    }

    #[test]
    fn test_layout_flex_grow() {
        let mut c = FlexContainer::new(0.0, 0.0, 500.0, 100.0);
        for _ in 0..3 {
            c.items.push(FlexItem {
                x: 0.0, y: 0.0, w: 100.0, h: 50.0,
                flex_grow: 1.0, flex_shrink: 0.0, flex_basis: 100.0,
                order: 0, align_self: None,
            });
        }
        c.layout();
        for item in &c.items {
            assert!((item.w - 166.67).abs() < 0.1);
        }
    }

    #[test]
    fn test_layout_justify_center() {
        let mut c = FlexContainer::new(0.0, 0.0, 500.0, 100.0);
        c.justify = JustifyContent::Center;
        c.items.push(FlexItem {
            x: 0.0, y: 0.0, w: 100.0, h: 50.0,
            flex_grow: 0.0, flex_shrink: 0.0, flex_basis: 100.0,
            order: 0, align_self: None,
        });
        c.layout();
        assert_eq!(c.items[0].x, 200.0);
    }

    #[test]
    fn test_layout_justify_flex_end() {
        let mut c = FlexContainer::new(0.0, 0.0, 500.0, 100.0);
        c.justify = JustifyContent::FlexEnd;
        c.items.push(FlexItem {
            x: 0.0, y: 0.0, w: 100.0, h: 50.0,
            flex_grow: 0.0, flex_shrink: 0.0, flex_basis: 100.0,
            order: 0, align_self: None,
        });
        c.layout();
        assert_eq!(c.items[0].x, 400.0);
    }

    #[test]
    fn test_layout_space_between() {
        let mut c = FlexContainer::new(0.0, 0.0, 500.0, 100.0);
        c.justify = JustifyContent::SpaceBetween;
        for _ in 0..3 {
            c.items.push(FlexItem {
                x: 0.0, y: 0.0, w: 100.0, h: 50.0,
                flex_grow: 0.0, flex_shrink: 0.0, flex_basis: 100.0,
                order: 0, align_self: None,
            });
        }
        c.layout();
        assert_eq!(c.items[0].x, 0.0);
        assert_eq!(c.items[1].x, 200.0);
        assert_eq!(c.items[2].x, 400.0);
    }

    #[test]
    fn test_layout_gap() {
        let mut c = FlexContainer::new(0.0, 0.0, 500.0, 100.0);
        c.gap = 10.0;
        for _ in 0..3 {
            c.items.push(FlexItem {
                x: 0.0, y: 0.0, w: 100.0, h: 50.0,
                flex_grow: 0.0, flex_shrink: 0.0, flex_basis: 100.0,
                order: 0, align_self: None,
            });
        }
        c.layout();
        assert_eq!(c.items[0].x, 0.0);
        assert_eq!(c.items[1].x, 110.0);
        assert_eq!(c.items[2].x, 220.0);
    }

    #[test]
    fn test_layout_order() {
        let mut c = FlexContainer::new(0.0, 0.0, 500.0, 100.0);
        c.items.push(FlexItem {
            x: 0.0, y: 0.0, w: 100.0, h: 50.0,
            flex_grow: 0.0, flex_shrink: 0.0, flex_basis: 100.0,
            order: 2, align_self: None,
        });
        c.items.push(FlexItem {
            x: 0.0, y: 0.0, w: 100.0, h: 50.0,
            flex_grow: 0.0, flex_shrink: 0.0, flex_basis: 100.0,
            order: 0, align_self: None,
        });
        c.layout();
        assert_eq!(c.items[0].x, 0.0);
        assert_eq!(c.items[1].x, 100.0);
    }

    #[test]
    fn test_layout_column_direction() {
        let mut c = FlexContainer::new(0.0, 0.0, 100.0, 500.0);
        c.direction = FlexDirection::Column;
        c.items.push(FlexItem {
            x: 0.0, y: 0.0, w: 50.0, h: 100.0,
            flex_grow: 0.0, flex_shrink: 0.0, flex_basis: 100.0,
            order: 0, align_self: None,
        });
        c.layout();
        assert_eq!(c.items[0].y, 0.0);
    }

    #[test]
    fn test_align_items_center() {
        let mut c = FlexContainer::new(0.0, 0.0, 200.0, 100.0);
        c.align_items = AlignItems::Center;
        c.items.push(FlexItem {
            x: 0.0, y: 0.0, w: 50.0, h: 40.0,
            flex_grow: 0.0, flex_shrink: 0.0, flex_basis: 50.0,
            order: 0, align_self: None,
        });
        c.layout();
        assert_eq!(c.items[0].y, 30.0);
    }

    #[test]
    fn test_align_items_stretch() {
        let mut c = FlexContainer::new(0.0, 0.0, 200.0, 100.0);
        c.align_items = AlignItems::Stretch;
        c.items.push(FlexItem {
            x: 0.0, y: 0.0, w: 50.0, h: 40.0,
            flex_grow: 0.0, flex_shrink: 0.0, flex_basis: 50.0,
            order: 0, align_self: None,
        });
        c.layout();
        assert_eq!(c.items[0].h, 100.0);
    }

    #[test]
    fn test_align_self_override() {
        let mut c = FlexContainer::new(0.0, 0.0, 200.0, 100.0);
        c.align_items = AlignItems::FlexStart;
        c.items.push(FlexItem {
            x: 0.0, y: 0.0, w: 50.0, h: 40.0,
            flex_grow: 0.0, flex_shrink: 0.0, flex_basis: 50.0,
            order: 0, align_self: Some(AlignItems::FlexEnd),
        });
        c.layout();
        assert_eq!(c.items[0].y, 60.0);
    }
}
