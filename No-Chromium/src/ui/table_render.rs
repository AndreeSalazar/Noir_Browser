//! Table Rendering - Tablas HTML con headers, rows, columns
//!
//! Soporta: thead, tbody, tfoot, th, td, caption, col, colgroup

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CellAlign {
    Left,
    Center,
    Right,
}

impl CellAlign {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "center" => Self::Center,
            "right" => Self::Right,
            _ => Self::Left,
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Center => "center",
            Self::Right => "right",
        }
    }
}

#[derive(Debug, Clone)]
pub struct TableCell {
    pub content: String,
    pub is_header: bool,
    pub align: CellAlign,
    pub colspan: u32,
    pub rowspan: u32,
    pub width: Option<u32>,
}

impl TableCell {
    pub fn new(content: &str) -> Self {
        Self {
            content: content.to_string(),
            is_header: false,
            align: CellAlign::Left,
            colspan: 1,
            rowspan: 1,
            width: None,
        }
    }

    pub fn header(content: &str) -> Self {
        let mut cell = Self::new(content);
        cell.is_header = true;
        cell
    }

    pub fn with_align(mut self, align: CellAlign) -> Self {
        self.align = align;
        self
    }

    pub fn with_colspan(mut self, span: u32) -> Self {
        self.colspan = span.max(1);
        self
    }

    pub fn with_rowspan(mut self, span: u32) -> Self {
        self.rowspan = span.max(1);
        self
    }

    pub fn with_width(mut self, w: u32) -> Self {
        self.width = Some(w);
        self
    }
}

#[derive(Debug, Clone)]
pub struct TableRow {
    pub cells: Vec<TableCell>,
}

impl TableRow {
    pub fn new() -> Self {
        Self { cells: Vec::new() }
    }

    pub fn add(&mut self, cell: TableCell) {
        self.cells.push(cell);
    }

    pub fn cell_count(&self) -> usize {
        self.cells.iter().map(|c| c.colspan as usize).sum()
    }
}

impl Default for TableRow {
    fn default() -> Self { Self::new() }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TableSection {
    Head,
    Body,
    Foot,
}

#[derive(Debug, Clone)]
pub struct TableSectionData {
    pub section_type: TableSection,
    pub rows: Vec<TableRow>,
}

impl TableSectionData {
    pub fn new(section_type: TableSection) -> Self {
        Self { section_type, rows: Vec::new() }
    }

    pub fn add_row(&mut self, row: TableRow) {
        self.rows.push(row);
    }

    pub fn row_count(&self) -> usize {
        self.rows.len()
    }
}

#[derive(Debug, Clone)]
pub struct Table {
    pub sections: Vec<TableSectionData>,
    pub caption: Option<String>,
    pub border: bool,
    pub striped: bool,
    pub compact: bool,
}

impl Table {
    pub fn new() -> Self {
        Self {
            sections: Vec::new(),
            caption: None,
            border: true,
            striped: true,
            compact: false,
        }
    }

    pub fn with_caption(mut self, caption: &str) -> Self {
        self.caption = Some(caption.to_string());
        self
    }

    pub fn with_border(mut self, border: bool) -> Self {
        self.border = border;
        self
    }

    pub fn with_striped(mut self, striped: bool) -> Self {
        self.striped = striped;
        self
    }

    pub fn with_compact(mut self, compact: bool) -> Self {
        self.compact = compact;
        self
    }

    pub fn add_section(&mut self, section: TableSectionData) {
        self.sections.push(section);
    }

    pub fn total_rows(&self) -> usize {
        self.sections.iter().map(|s| s.row_count()).sum()
    }

    pub fn max_columns(&self) -> usize {
        self.sections.iter()
            .flat_map(|s| s.rows.iter())
            .map(|r| r.cell_count())
            .max()
            .unwrap_or(0)
    }

    pub fn head_section(&self) -> Option<&TableSectionData> {
        self.sections.iter().find(|s| s.section_type == TableSection::Head)
    }

    pub fn body_sections(&self) -> Vec<&TableSectionData> {
        self.sections.iter().filter(|s| s.section_type == TableSection::Body).collect()
    }

    pub fn foot_section(&self) -> Option<&TableSectionData> {
        self.sections.iter().find(|s| s.section_type == TableSection::Foot)
    }
}

impl Default for Table {
    fn default() -> Self { Self::new() }
}

pub struct TableRenderer {
    pub char_w: f32,
    pub char_h: f32,
    pub border_color: u32,
    pub header_bg: u32,
    pub stripe_bg: u32,
    pub cell_padding: u32,
    pub max_col_width: u32,
}

impl TableRenderer {
    pub fn new() -> Self {
        Self {
            char_w: 7.0,
            char_h: 12.0,
            border_color: 0xFF333340,
            header_bg: 0xFF2A2A35,
            stripe_bg: 0xFF1F1F28,
            cell_padding: 6,
            max_col_width: 200,
        }
    }

    pub fn with_colors(mut self, border: u32, header: u32, stripe: u32) -> Self {
        self.border_color = border;
        self.header_bg = header;
        self.stripe_bg = stripe;
        self
    }

    /// Calcula dimensiones totales de la tabla
    pub fn calculate_size(&self, table: &Table) -> (f32, f32) {
        let cols = table.max_columns() as f32;
        let col_w = (cols * self.char_w + self.cell_padding as f32 * 2.0).min(self.max_col_width as f32);
        let total_w = col_w * cols;
        let line_h = if table.compact { self.char_h * 1.0 } else { self.char_h * 1.5 };
        let header_lines = if table.head_section().is_some() { 1.5 } else { 0.0 };
        let total_h = line_h * (table.total_rows() as f32 + header_lines);
        (total_w, total_h)
    }

    /// Render como ASCII text
    pub fn to_ascii(&self, table: &Table) -> String {
        let mut out = String::new();
        if let Some(caption) = &table.caption {
            out.push_str(&format!("[{}]\n", caption));
        }
        for section in &table.sections {
            if !section.rows.is_empty() {
                for row in &section.rows {
                    out.push_str("| ");
                    for cell in &row.cells {
                        let span = cell.colspan as usize;
                        let pad = span * 12;
                        let cell_text = format!("{:<width$}", cell.content, width = pad);
                        out.push_str(&cell_text);
                        out.push_str(" | ");
                    }
                    out.push('\n');
                }
                if table.border {
                    out.push_str(&"-".repeat(40));
                    out.push('\n');
                }
            }
        }
        out
    }

    /// Genera un layout de celdas con posiciones
    pub fn layout_cells<'a>(&'a self, table: &'a Table) -> Vec<(f32, f32, f32, f32, &'a TableCell)> {
        let mut out = Vec::new();
        let col_w = self.max_col_width as f32 / table.max_columns().max(1) as f32;
        let line_h = if table.compact { self.char_h } else { self.char_h * 1.5 };
        let mut y = 0.0;
        for section in &table.sections {
            for row in &section.rows {
                let mut x = 0.0;
                for cell in &row.cells {
                    let w = col_w * cell.colspan as f32;
                    out.push((x, y, w, line_h, cell));
                    x += w;
                }
                y += line_h;
            }
        }
        out
    }
}

impl Default for TableRenderer {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_align_from_str() {
        assert_eq!(CellAlign::from_str("left"), CellAlign::Left);
        assert_eq!(CellAlign::from_str("center"), CellAlign::Center);
        assert_eq!(CellAlign::from_str("right"), CellAlign::Right);
    }

    #[test]
    fn test_align_to_str() {
        assert_eq!(CellAlign::Left.to_str(), "left");
    }

    #[test]
    fn test_cell_new() {
        let c = TableCell::new("test");
        assert_eq!(c.content, "test");
        assert!(!c.is_header);
    }

    #[test]
    fn test_cell_header() {
        let c = TableCell::header("head");
        assert!(c.is_header);
    }

    #[test]
    fn test_cell_with_align() {
        let c = TableCell::new("t").with_align(CellAlign::Center);
        assert_eq!(c.align, CellAlign::Center);
    }

    #[test]
    fn test_cell_with_colspan() {
        let c = TableCell::new("t").with_colspan(3);
        assert_eq!(c.colspan, 3);
    }

    #[test]
    fn test_cell_with_rowspan() {
        let c = TableCell::new("t").with_rowspan(2);
        assert_eq!(c.rowspan, 2);
    }

    #[test]
    fn test_cell_with_width() {
        let c = TableCell::new("t").with_width(100);
        assert_eq!(c.width, Some(100));
    }

    #[test]
    fn test_cell_colspan_min() {
        let c = TableCell::new("t").with_colspan(0);
        assert_eq!(c.colspan, 1);
    }

    #[test]
    fn test_row_new() {
        let r = TableRow::new();
        assert!(r.cells.is_empty());
    }

    #[test]
    fn test_row_add() {
        let mut r = TableRow::new();
        r.add(TableCell::new("a"));
        r.add(TableCell::new("b"));
        assert_eq!(r.cells.len(), 2);
    }

    #[test]
    fn test_row_cell_count() {
        let mut r = TableRow::new();
        r.add(TableCell::new("a"));
        r.add(TableCell::new("b").with_colspan(2));
        assert_eq!(r.cell_count(), 3);
    }

    #[test]
    fn test_section_new() {
        let s = TableSectionData::new(TableSection::Head);
        assert_eq!(s.section_type, TableSection::Head);
    }

    #[test]
    fn test_section_add_row() {
        let mut s = TableSectionData::new(TableSection::Body);
        s.add_row(TableRow::new());
        assert_eq!(s.row_count(), 1);
    }

    #[test]
    fn test_table_new() {
        let t = Table::new();
        assert!(t.border);
        assert!(t.striped);
    }

    #[test]
    fn test_table_with_caption() {
        let t = Table::new().with_caption("Title");
        assert_eq!(t.caption, Some("Title".to_string()));
    }

    #[test]
    fn test_table_with_options() {
        let t = Table::new().with_border(false).with_striped(false).with_compact(true);
        assert!(!t.border);
        assert!(!t.striped);
        assert!(t.compact);
    }

    #[test]
    fn test_table_add_section() {
        let mut t = Table::new();
        t.add_section(TableSectionData::new(TableSection::Head));
        t.add_section(TableSectionData::new(TableSection::Body));
        assert_eq!(t.sections.len(), 2);
    }

    #[test]
    fn test_table_total_rows() {
        let mut t = Table::new();
        let mut h = TableSectionData::new(TableSection::Head);
        h.add_row(TableRow::new());
        let mut b = TableSectionData::new(TableSection::Body);
        b.add_row(TableRow::new());
        b.add_row(TableRow::new());
        t.add_section(h);
        t.add_section(b);
        assert_eq!(t.total_rows(), 3);
    }

    #[test]
    fn test_table_max_columns() {
        let mut t = Table::new();
        let mut row = TableRow::new();
        row.add(TableCell::new("a"));
        row.add(TableCell::new("b").with_colspan(3));
        let mut s = TableSectionData::new(TableSection::Body);
        s.add_row(row);
        t.add_section(s);
        assert_eq!(t.max_columns(), 4);
    }

    #[test]
    fn test_table_head_section() {
        let mut t = Table::new();
        t.add_section(TableSectionData::new(TableSection::Head));
        t.add_section(TableSectionData::new(TableSection::Body));
        assert!(t.head_section().is_some());
        assert!(t.foot_section().is_none());
    }

    #[test]
    fn test_table_body_sections() {
        let mut t = Table::new();
        t.add_section(TableSectionData::new(TableSection::Body));
        t.add_section(TableSectionData::new(TableSection::Body));
        assert_eq!(t.body_sections().len(), 2);
    }

    #[test]
    fn test_renderer_new() {
        let r = TableRenderer::new();
        assert!(r.char_w > 0.0);
    }

    #[test]
    fn test_renderer_with_colors() {
        let r = TableRenderer::new().with_colors(0xFF0000, 0xFF00FF, 0xFFFF00);
        assert_eq!(r.border_color, 0xFF0000);
    }

    #[test]
    fn test_renderer_calculate_size() {
        let r = TableRenderer::new();
        let mut t = Table::new();
        let mut s = TableSectionData::new(TableSection::Body);
        s.add_row(TableRow::new());
        t.add_section(s);
        let (w, h) = r.calculate_size(&t);
        assert!(w >= 0.0);
        assert!(h >= 0.0);
    }

    #[test]
    fn test_renderer_to_ascii() {
        let r = TableRenderer::new();
        let mut t = Table::new();
        let mut s = TableSectionData::new(TableSection::Body);
        let mut row = TableRow::new();
        row.add(TableCell::new("a"));
        row.add(TableCell::new("b"));
        s.add_row(row);
        t.add_section(s);
        let ascii = r.to_ascii(&t);
        assert!(ascii.contains("a"));
        assert!(ascii.contains("b"));
    }

    #[test]
    fn test_renderer_to_ascii_with_caption() {
        let r = TableRenderer::new();
        let t = Table::new().with_caption("My Table");
        let ascii = r.to_ascii(&t);
        assert!(ascii.contains("My Table"));
    }

    #[test]
    fn test_renderer_layout_cells() {
        let r = TableRenderer::new();
        let mut t = Table::new();
        let mut s = TableSectionData::new(TableSection::Body);
        let mut row = TableRow::new();
        row.add(TableCell::new("a"));
        row.add(TableCell::new("b"));
        s.add_row(row);
        t.add_section(s);
        let cells = r.layout_cells(&t);
        assert_eq!(cells.len(), 2);
    }
}
