//! CSS Grid Layout - Display grid con rows/columns auto/fixed
//!
//! Procesa `display: grid` con `grid-template-columns/rows`

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GridTrackSize {
    Auto,           // Se ajusta al contenido
    Fixed(f32),     // px
    Fraction(u32),  // fr units
    Percent(f32),   // % del container
}

impl GridTrackSize {
    pub fn from_str(s: &str) -> Self {
        let s = s.trim();
        if s == "auto" { return Self::Auto; }
        if let Some(fr) = s.strip_suffix("fr") {
            if let Ok(n) = fr.trim().parse::<u32>() {
                return Self::Fraction(n);
            }
        }
        if let Some(pct) = s.strip_suffix('%') {
            if let Ok(n) = pct.trim().parse::<f32>() {
                return Self::Percent(n);
            }
        }
        // Strip "px" suffix
        let num_str = s.strip_suffix("px").unwrap_or(s);
        if let Ok(px) = num_str.trim().parse::<f32>() {
            return Self::Fixed(px);
        }
        Self::Auto
    }

    pub fn is_flexible(&self) -> bool {
        matches!(self, Self::Fraction(_) | Self::Percent(_) | Self::Auto)
    }

    pub fn resolve(&self, available: f32) -> f32 {
        match self {
            Self::Auto => 0.0,
            Self::Fixed(v) => *v,
            Self::Fraction(_) => 0.0,
            Self::Percent(p) => available * p / 100.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GridTemplate {
    pub columns: Vec<GridTrackSize>,
    pub rows: Vec<GridTrackSize>,
    pub column_gap: f32,
    pub row_gap: f32,
}

impl GridTemplate {
    pub fn new() -> Self {
        Self {
            columns: Vec::new(),
            rows: Vec::new(),
            column_gap: 0.0,
            row_gap: 0.0,
        }
    }

    pub fn from_columns(s: &str) -> Self {
        let mut t = Self::new();
        for track in s.split_whitespace() {
            t.columns.push(GridTrackSize::from_str(track));
        }
        t
    }

    pub fn from_template(template_columns: &str, template_rows: &str) -> Self {
        let mut t = Self::new();
        for track in template_columns.split_whitespace() {
            t.columns.push(GridTrackSize::from_str(track));
        }
        for track in template_rows.split_whitespace() {
            t.rows.push(GridTrackSize::from_str(track));
        }
        t
    }
}

impl Default for GridTemplate {
    fn default() -> Self { Self::new() }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GridAutoFlow {
    Row,
    Column,
    RowDense,
    ColumnDense,
}

impl GridAutoFlow {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "column" => Self::Column,
            "row dense" => Self::RowDense,
            "column dense" => Self::ColumnDense,
            _ => Self::Row,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GridPlacement {
    pub column: u32,
    pub row: u32,
    pub column_span: u32,
    pub row_span: u32,
}

impl GridPlacement {
    pub fn auto() -> Self {
        Self {
            column: 0,
            row: 0,
            column_span: 1,
            row_span: 1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GridItem {
    pub id: u32,
    pub content: String,
    pub placement: GridPlacement,
    pub width: f32,
    pub height: f32,
}

impl GridItem {
    pub fn new(id: u32, content: &str) -> Self {
        Self {
            id,
            content: content.to_string(),
            placement: GridPlacement::auto(),
            width: 0.0,
            height: 0.0,
        }
    }

    pub fn with_placement(mut self, placement: GridPlacement) -> Self {
        self.placement = placement;
        self
    }

    pub fn with_size(mut self, w: f32, h: f32) -> Self {
        self.width = w;
        self.height = h;
        self
    }
}

pub struct CssGrid {
    pub template: GridTemplate,
    pub auto_flow: GridAutoFlow,
    pub items: Vec<GridItem>,
    pub container_width: f32,
    pub container_height: f32,
}

impl CssGrid {
    pub fn new(container_width: f32) -> Self {
        Self {
            template: GridTemplate::new(),
            auto_flow: GridAutoFlow::Row,
            items: Vec::new(),
            container_width,
            container_height: 0.0,
        }
    }

    pub fn with_template(mut self, template: GridTemplate) -> Self {
        self.template = template;
        self
    }

    pub fn with_auto_flow(mut self, flow: GridAutoFlow) -> Self {
        self.auto_flow = flow;
        self
    }

    pub fn add(&mut self, item: GridItem) {
        self.items.push(item);
    }

    /// Resuelve el ancho de cada columna
    pub fn resolve_columns(&self) -> Vec<f32> {
        let n_cols = self.template.columns.len().max(1);
        let total_gaps = self.template.column_gap * (n_cols as f32 - 1.0).max(0.0);
        let available = (self.container_width - total_gaps).max(0.0);
        let mut widths = vec![0.0; n_cols];
        let mut fixed_total = 0.0;
        let mut fr_total: u32 = 0;
        for (i, t) in self.template.columns.iter().enumerate() {
            match t {
                GridTrackSize::Fixed(v) => {
                    widths[i] = *v;
                    fixed_total += v;
                }
                GridTrackSize::Percent(p) => {
                    widths[i] = available * p / 100.0;
                    fixed_total += widths[i];
                }
                GridTrackSize::Fraction(fr) => fr_total += fr,
                _ => {}
            }
        }
        let remaining = (available - fixed_total).max(0.0);
        if fr_total > 0 {
            for (i, t) in self.template.columns.iter().enumerate() {
                if let GridTrackSize::Fraction(fr) = t {
                    widths[i] = remaining * (*fr as f32) / (fr_total as f32);
                }
            }
        }
        widths
    }

    /// Calcula layout final con posiciones
    pub fn compute_layout(&self) -> Vec<(f32, f32, f32, f32, &GridItem)> {
        let col_widths = self.resolve_columns();
        let n_cols = col_widths.len();
        let mut out = Vec::new();
        let mut row_idx = 0u32;
        let mut col_idx = 0u32;
        let mut row_heights: HashMap<u32, f32> = HashMap::new();
        for item in &self.items {
            let placement = if item.placement.column == 0 && item.placement.row == 0 {
                GridPlacement {
                    column: col_idx,
                    row: row_idx,
                    column_span: 1,
                    row_span: 1,
                }
            } else {
                item.placement
            };
            let x: f32 = (0..placement.column)
                .map(|c| col_widths.get(c as usize).copied().unwrap_or(0.0) + self.template.column_gap)
                .sum();
            let y: f32 = (0..placement.row)
                .filter_map(|r| row_heights.get(&r))
                .sum::<f32>()
                + self.template.row_gap * placement.row as f32;
            let w = (0..placement.column_span)
                .map(|c| col_widths.get((placement.column + c) as usize).copied().unwrap_or(0.0))
                .sum::<f32>()
                + self.template.column_gap * (placement.column_span as f32 - 1.0).max(0.0);
            let h = item.height;
            row_heights.insert(placement.row, h);
            out.push((x, y, w, h, item));
            col_idx += 1;
            if col_idx as usize >= n_cols {
                col_idx = 0;
                row_idx += 1;
            }
        }
        out
    }

    pub fn total_height(&self) -> f32 {
        if self.items.is_empty() { return 0.0; }
        self.compute_layout()
            .iter()
            .map(|(_, y, _, h, _)| y + h)
            .fold(0.0f32, f32::max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_track_size_from_str_auto() {
        assert_eq!(GridTrackSize::from_str("auto"), GridTrackSize::Auto);
    }

    #[test]
    fn test_track_size_from_str_fixed() {
        assert_eq!(GridTrackSize::from_str("100px"), GridTrackSize::Fixed(100.0));
    }

    #[test]
    fn test_track_size_from_str_fraction() {
        assert_eq!(GridTrackSize::from_str("1fr"), GridTrackSize::Fraction(1));
        assert_eq!(GridTrackSize::from_str("2fr"), GridTrackSize::Fraction(2));
    }

    #[test]
    fn test_track_size_from_str_percent() {
        assert_eq!(GridTrackSize::from_str("50%"), GridTrackSize::Percent(50.0));
    }

    #[test]
    fn test_track_size_is_flexible() {
        assert!(GridTrackSize::Auto.is_flexible());
        assert!(GridTrackSize::Fraction(1).is_flexible());
        assert!(!GridTrackSize::Fixed(100.0).is_flexible());
    }

    #[test]
    fn test_track_size_resolve() {
        assert_eq!(GridTrackSize::Fixed(100.0).resolve(1000.0), 100.0);
        assert_eq!(GridTrackSize::Percent(50.0).resolve(1000.0), 500.0);
    }

    #[test]
    fn test_template_new() {
        let t = GridTemplate::new();
        assert_eq!(t.columns.len(), 0);
    }

    #[test]
    fn test_template_from_columns() {
        let t = GridTemplate::from_columns("1fr 1fr 200px");
        assert_eq!(t.columns.len(), 3);
        assert_eq!(t.columns[2], GridTrackSize::Fixed(200.0));
    }

    #[test]
    fn test_template_from_template() {
        let t = GridTemplate::from_template("1fr 1fr", "auto 100px");
        assert_eq!(t.columns.len(), 2);
        assert_eq!(t.rows.len(), 2);
    }

    #[test]
    fn test_auto_flow_from_str() {
        assert_eq!(GridAutoFlow::from_str("row"), GridAutoFlow::Row);
        assert_eq!(GridAutoFlow::from_str("column"), GridAutoFlow::Column);
        assert_eq!(GridAutoFlow::from_str("row dense"), GridAutoFlow::RowDense);
    }

    #[test]
    fn test_placement_auto() {
        let p = GridPlacement::auto();
        assert_eq!(p.column, 0);
        assert_eq!(p.row_span, 1);
    }

    #[test]
    fn test_item_new() {
        let item = GridItem::new(1, "test");
        assert_eq!(item.id, 1);
        assert_eq!(item.content, "test");
    }

    #[test]
    fn test_item_with_placement() {
        let item = GridItem::new(1, "t")
            .with_placement(GridPlacement { column: 2, row: 1, column_span: 1, row_span: 1 });
        assert_eq!(item.placement.column, 2);
    }

    #[test]
    fn test_item_with_size() {
        let item = GridItem::new(1, "t").with_size(100.0, 50.0);
        assert_eq!(item.width, 100.0);
    }

    #[test]
    fn test_grid_new() {
        let g = CssGrid::new(800.0);
        assert_eq!(g.container_width, 800.0);
        assert!(g.items.is_empty());
    }

    #[test]
    fn test_grid_with_template() {
        let t = GridTemplate::from_columns("1fr 1fr");
        let g = CssGrid::new(800.0).with_template(t);
        assert_eq!(g.template.columns.len(), 2);
    }

    #[test]
    fn test_grid_with_auto_flow() {
        let g = CssGrid::new(800.0).with_auto_flow(GridAutoFlow::Column);
        assert_eq!(g.auto_flow, GridAutoFlow::Column);
    }

    #[test]
    fn test_grid_add() {
        let mut g = CssGrid::new(800.0);
        g.add(GridItem::new(1, "a"));
        g.add(GridItem::new(2, "b"));
        assert_eq!(g.items.len(), 2);
    }

    #[test]
    fn test_grid_resolve_columns_fr() {
        let t = GridTemplate::from_columns("1fr 1fr");
        let g = CssGrid::new(800.0).with_template(t);
        let widths = g.resolve_columns();
        // Cada col = (800-0)/2 = 400
        assert_eq!(widths[0], 400.0);
        assert_eq!(widths[1], 400.0);
    }

    #[test]
    fn test_grid_resolve_columns_mixed() {
        let t = GridTemplate::from_columns("1fr 200px 1fr");
        let g = CssGrid::new(1000.0).with_template(t);
        let widths = g.resolve_columns();
        // Fixed 200, remaining 800 split = 400 cada una
        assert_eq!(widths[0], 400.0);
        assert_eq!(widths[1], 200.0);
        assert_eq!(widths[2], 400.0);
    }

    #[test]
    fn test_grid_resolve_columns_with_gap() {
        let mut t = GridTemplate::from_columns("1fr 1fr 1fr");
        t.column_gap = 10.0;
        let g = CssGrid::new(630.0).with_template(t);
        let widths = g.resolve_columns();
        // (630 - 20) / 3 = 203.33
        assert!((widths[0] - 203.33).abs() < 0.1);
    }

    #[test]
    fn test_grid_compute_layout() {
        let t = GridTemplate::from_columns("1fr 1fr");
        let mut g = CssGrid::new(800.0).with_template(t);
        g.add(GridItem::new(1, "a").with_size(0.0, 50.0));
        g.add(GridItem::new(2, "b").with_size(0.0, 50.0));
        let layout = g.compute_layout();
        assert_eq!(layout.len(), 2);
        // Primer item en (0, 0), segundo en (400, 0)
        assert_eq!(layout[0].0, 0.0);
        assert_eq!(layout[1].0, 400.0);
    }

    #[test]
    fn test_grid_compute_layout_wraps() {
        let t = GridTemplate::from_columns("1fr 1fr");
        let mut g = CssGrid::new(800.0).with_template(t);
        g.add(GridItem::new(1, "a").with_size(0.0, 50.0));
        g.add(GridItem::new(2, "b").with_size(0.0, 50.0));
        g.add(GridItem::new(3, "c").with_size(0.0, 50.0));
        let layout = g.compute_layout();
        // Tercer item debe estar en row 1
        assert_eq!(layout[2].1, 50.0);
    }

    #[test]
    fn test_grid_total_height() {
        let t = GridTemplate::from_columns("1fr 1fr");
        let mut g = CssGrid::new(800.0).with_template(t);
        g.add(GridItem::new(1, "a").with_size(0.0, 50.0));
        g.add(GridItem::new(2, "b").with_size(0.0, 50.0));
        g.add(GridItem::new(3, "c").with_size(0.0, 50.0));
        let h = g.total_height();
        assert!(h > 0.0);
    }
}
