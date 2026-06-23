//! CSS Grid basico (FASE C2)
//!
//! Implementa un subset del CSS Grid:
//! - display: grid | inline-grid
//! - grid-template-columns/rows: fr, px, %, auto
//! - grid-gap (row-gap, column-gap, gap)
//! - grid-column/row: span N
//!
//! Implementacion simplificada: el grid auto-posiciona items en celdas.

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum TrackSize {
    Auto,
    Px(f32),
    Percent(f32),
    Fr(f32),
}

impl TrackSize {
    pub fn from_str(s: &str) -> Self {
        let s = s.trim();
        if s == "auto" || s.is_empty() {
            TrackSize::Auto
        } else if let Some(num) = s.strip_suffix("fr") {
            TrackSize::Fr(num.trim().parse().unwrap_or(1.0))
        } else if let Some(num) = s.strip_suffix('%') {
            TrackSize::Percent(num.trim().parse().unwrap_or(0.0))
        } else if let Some(num) = s.strip_suffix("px") {
            TrackSize::Px(num.trim().parse().unwrap_or(0.0))
        } else {
            // Asumir px
            TrackSize::Px(s.parse().unwrap_or(0.0))
        }
    }

    /// Resuelve el tamano en pixels dado un container size
    pub fn resolve(&self, container_size: f32, fr_remaining: f32, total_fr: f32) -> f32 {
        match self {
            TrackSize::Auto => 0.0,  // auto sera determinado despues
            TrackSize::Px(v) => *v,
            TrackSize::Percent(p) => container_size * p / 100.0,
            TrackSize::Fr(fr) => {
                if total_fr > 0.0 {
                    fr_remaining * fr / total_fr
                } else {
                    0.0
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GridAutoFlow {
    Row,
    Column,
    RowDense,
    ColumnDense,
}

impl GridAutoFlow {
    pub fn from_str(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "column" => GridAutoFlow::Column,
            "row dense" | "row-dense" => GridAutoFlow::RowDense,
            "column dense" | "column-dense" => GridAutoFlow::ColumnDense,
            _ => GridAutoFlow::Row,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GridItem {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub column: u32,
    pub row: u32,
    pub column_span: u32,
    pub row_span: u32,
}

#[derive(Debug, Clone)]
pub struct GridContainer {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub columns: Vec<TrackSize>,
    pub rows: Vec<TrackSize>,
    pub column_gap: f32,
    pub row_gap: f32,
    pub auto_flow: GridAutoFlow,
    pub items: Vec<GridItem>,
}

impl GridContainer {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            x, y, w, h,
            columns: vec![TrackSize::Fr(1.0)],
            rows: vec![TrackSize::Auto],
            column_gap: 0.0,
            row_gap: 0.0,
            auto_flow: GridAutoFlow::Row,
            items: Vec::new(),
        }
    }

    /// Parsea "1fr 200px 1fr 1fr" en vec de TrackSize
    pub fn set_columns_str(&mut self, s: &str) {
        self.columns = s.split_whitespace()
            .map(TrackSize::from_str)
            .collect();
    }

    pub fn set_rows_str(&mut self, s: &str) {
        self.rows = s.split_whitespace()
            .map(TrackSize::from_str)
            .collect();
    }

    /// Calcula el layout del grid
    pub fn layout(&mut self) {
        if self.items.is_empty() { return; }

        // 1. Resolver column widths
        let col_widths = self.resolve_track_sizes(&self.columns, self.w, self.column_gap);
        // 2. Resolver row heights (initial - mas sera calculado)
        let _row_heights = self.resolve_track_sizes(&self.rows, self.h, self.row_gap);

        // 3. Auto-flow placement
        self.auto_place_items(&col_widths);
    }

    fn resolve_track_sizes(&self, tracks: &[TrackSize], container: f32, gap: f32) -> Vec<f32> {
        let n = tracks.len().max(1);
        let total_gaps = gap * (n as f32 - 1.0).max(0.0);

        // Paso 1: sumar px, percent, auto
        let mut fixed_total = 0.0;
        let mut fr_indices = Vec::new();
        let mut total_fr = 0.0;
        for (i, t) in tracks.iter().enumerate() {
            match t {
                TrackSize::Px(v) => fixed_total += v,
                TrackSize::Percent(p) => fixed_total += container * p / 100.0,
                TrackSize::Auto => {}  // se calcula despues
                TrackSize::Fr(fr) => {
                    fr_indices.push(i);
                    total_fr += fr;
                }
            }
        }
        let remaining = (container - fixed_total - total_gaps).max(0.0);
        let fr_unit = if total_fr > 0.0 { remaining / total_fr } else { 0.0 };

        // Paso 2: asignar
        let mut out = vec![0.0; n];
        for (i, t) in tracks.iter().enumerate() {
            out[i] = t.resolve(container, remaining, total_fr);
        }
        out
    }

    fn auto_place_items(&mut self, col_widths: &[f32]) {
        let n_cols = col_widths.len();
        let mut row_y = 0.0;
        let mut current_col: u32 = 0;
        let mut current_row: u32 = 0;
        let mut row_heights: HashMap<u32, f32> = HashMap::new();

        let container_x = self.x;
        let container_y = self.y;
        let col_gap = self.column_gap;
        let row_gap = self.row_gap;

        for item in &mut self.items {
            let col_span = if item.column_span == 0 { 1 } else { item.column_span };

            // Si no cabe en la fila actual, mover a la siguiente
            if current_col + col_span > n_cols as u32 {
                current_col = 0;
                current_row += 1;
            }
            item.column = current_col;
            item.row = current_row;

            // Calcular x
            let mut x = container_x;
            for c in 0..current_col {
                x += col_widths[c as usize] + col_gap;
            }
            let span_width: f32 = (current_col..current_col + col_span)
                .map(|c| col_widths[c as usize])
                .sum::<f32>() + col_gap * (col_span as f32 - 1.0);
            item.x = x;
            item.w = span_width;

            // Calcular y
            item.y = container_y + row_y;

            // Actualizar row_heights
            let h = item.h;
            let entry = row_heights.entry(current_row).or_insert(0.0);
            if h > *entry {
                *entry = h;
            }

            // Calcular nueva row_y
            row_y = 0.0;
            for r in 0..=current_row {
                if let Some(h) = row_heights.get(&r) {
                    row_y += h + row_gap;
                }
            }

            current_col += col_span;
            if current_col >= n_cols as u32 {
                current_col = 0;
                current_row += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_track_size_from_str() {
        assert_eq!(TrackSize::from_str("100px"), TrackSize::Px(100.0));
        assert_eq!(TrackSize::from_str("50%"), TrackSize::Percent(50.0));
        assert_eq!(TrackSize::from_str("1fr"), TrackSize::Fr(1.0));
        assert_eq!(TrackSize::from_str("auto"), TrackSize::Auto);
    }

    #[test]
    fn test_track_size_resolve_px() {
        let t = TrackSize::Px(100.0);
        assert_eq!(t.resolve(500.0, 0.0, 0.0), 100.0);
    }

    #[test]
    fn test_track_size_resolve_percent() {
        let t = TrackSize::Percent(50.0);
        assert_eq!(t.resolve(500.0, 0.0, 0.0), 250.0);
    }

    #[test]
    fn test_track_size_resolve_fr() {
        let t = TrackSize::Fr(1.0);
        assert_eq!(t.resolve(500.0, 300.0, 3.0), 100.0);
    }

    #[test]
    fn test_auto_flow_from_str() {
        assert_eq!(GridAutoFlow::from_str("row"), GridAutoFlow::Row);
        assert_eq!(GridAutoFlow::from_str("column"), GridAutoFlow::Column);
        assert_eq!(GridAutoFlow::from_str("row dense"), GridAutoFlow::RowDense);
    }

    #[test]
    fn test_grid_creation() {
        let g = GridContainer::new(0.0, 0.0, 500.0, 300.0);
        assert_eq!(g.w, 500.0);
    }

    #[test]
    fn test_set_columns() {
        let mut g = GridContainer::new(0.0, 0.0, 500.0, 300.0);
        g.set_columns_str("1fr 200px 1fr");
        assert_eq!(g.columns.len(), 3);
    }

    #[test]
    fn test_layout_three_columns() {
        let mut g = GridContainer::new(0.0, 0.0, 600.0, 300.0);
        g.set_columns_str("1fr 1fr 1fr");
        g.column_gap = 0.0;
        for i in 0..3 {
            g.items.push(GridItem {
                x: 0.0, y: 0.0, w: 0.0, h: 50.0,
                column: 0, row: 0,
                column_span: 1, row_span: 1,
            });
            let _ = i;
        }
        g.layout();
        // Cada col = 200px
        assert_eq!(g.items[0].x, 0.0);
        assert_eq!(g.items[1].x, 200.0);
        assert_eq!(g.items[2].x, 400.0);
    }

    #[test]
    fn test_layout_with_gap() {
        let mut g = GridContainer::new(0.0, 0.0, 500.0, 300.0);
        g.set_columns_str("100px 100px");
        g.column_gap = 10.0;
        for _ in 0..2 {
            g.items.push(GridItem {
                x: 0.0, y: 0.0, w: 0.0, h: 50.0,
                column: 0, row: 0,
                column_span: 1, row_span: 1,
            });
        }
        g.layout();
        assert_eq!(g.items[0].x, 0.0);
        assert_eq!(g.items[1].x, 110.0);  // 100 + gap
    }

    #[test]
    fn test_layout_wrap_to_next_row() {
        let mut g = GridContainer::new(0.0, 0.0, 300.0, 300.0);
        g.set_columns_str("100px 100px");
        for _ in 0..3 {
            g.items.push(GridItem {
                x: 0.0, y: 0.0, w: 0.0, h: 50.0,
                column: 0, row: 0,
                column_span: 1, row_span: 1,
            });
        }
        g.layout();
        // Tercer item va a la siguiente fila
        assert_eq!(g.items[2].row, 1);
    }

    #[test]
    fn test_layout_column_span() {
        let mut g = GridContainer::new(0.0, 0.0, 500.0, 300.0);
        g.set_columns_str("100px 100px 100px");
        g.items.push(GridItem {
            x: 0.0, y: 0.0, w: 0.0, h: 50.0,
            column: 0, row: 0,
            column_span: 2, row_span: 1,
        });
        g.layout();
        // item toma 2 columnas: ancho = 100 + 100 = 200
        assert_eq!(g.items[0].w, 200.0);
    }

    #[test]
    fn test_layout_rows_calculated() {
        let mut g = GridContainer::new(0.0, 0.0, 300.0, 300.0);
        g.set_columns_str("100px");
        g.items.push(GridItem {
            x: 0.0, y: 0.0, w: 0.0, h: 30.0,
            column: 0, row: 0,
            column_span: 1, row_span: 1,
        });
        g.items.push(GridItem {
            x: 0.0, y: 0.0, w: 0.0, h: 50.0,
            column: 0, row: 0,
            column_span: 1, row_span: 1,
        });
        g.layout();
        // El segundo item deberia estar en la siguiente fila porque solo hay 1 col
        assert_eq!(g.items[1].row, 1);
    }
}
