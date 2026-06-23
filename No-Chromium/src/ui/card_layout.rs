//! Card Layout - Card-style rendering para videos, productos, etc.
//!
//! Cada card tiene: thumbnail, title, metadata, avatar, acciones

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CardLayout {
    Vertical,    // Thumbnail arriba, info abajo (YouTube)
    Horizontal,  // Thumbnail izquierda, info derecha
    Compact,     // Solo thumbnail + title pequeño
    Detailed,    // Con descripción y metadata completa
}

impl CardLayout {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "horizontal" | "h" => Self::Horizontal,
            "compact" => Self::Compact,
            "detailed" | "detail" => Self::Detailed,
            _ => Self::Vertical,
        }
    }

    pub fn title_size(&self) -> f32 {
        match self {
            Self::Vertical => 14.0,
            Self::Horizontal => 14.0,
            Self::Compact => 12.0,
            Self::Detailed => 16.0,
        }
    }

    pub fn meta_size(&self) -> f32 {
        match self {
            Self::Vertical => 12.0,
            Self::Horizontal => 12.0,
            Self::Compact => 10.0,
            Self::Detailed => 13.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CardMetadata {
    pub author: String,
    pub views: u64,
    pub age: String,
    pub duration: String,
    pub verified: bool,
}

impl CardMetadata {
    pub fn new(author: &str) -> Self {
        Self {
            author: author.to_string(),
            views: 0,
            age: String::new(),
            duration: String::new(),
            verified: false,
        }
    }

    pub fn with_views(mut self, v: u64) -> Self {
        self.views = v;
        self
    }

    pub fn with_age(mut self, age: &str) -> Self {
        self.age = age.to_string();
        self
    }

    pub fn with_duration(mut self, d: &str) -> Self {
        self.duration = d.to_string();
        self
    }

    pub fn with_verified(mut self, v: bool) -> Self {
        self.verified = v;
        self
    }

    pub fn format_views(&self) -> String {
        if self.views >= 1_000_000 {
            format!("{:.1}M vistas", self.views as f32 / 1_000_000.0)
        } else if self.views >= 1_000 {
            format!("{:.1}K vistas", self.views as f32 / 1_000.0)
        } else if self.views > 0 {
            format!("{} vistas", self.views)
        } else {
            String::new()
        }
    }
}

#[derive(Debug, Clone)]
pub struct Card {
    pub id: u32,
    pub title: String,
    pub thumbnail_src: Option<String>,
    pub layout: CardLayout,
    pub metadata: CardMetadata,
    pub url: String,
}

impl Card {
    pub fn new(id: u32, title: &str, url: &str) -> Self {
        Self {
            id,
            title: title.to_string(),
            thumbnail_src: None,
            layout: CardLayout::Vertical,
            metadata: CardMetadata::new(""),
            url: url.to_string(),
        }
    }

    pub fn with_thumbnail(mut self, src: &str) -> Self {
        self.thumbnail_src = Some(src.to_string());
        self
    }

    pub fn with_layout(mut self, layout: CardLayout) -> Self {
        self.layout = layout;
        self
    }

    pub fn with_metadata(mut self, metadata: CardMetadata) -> Self {
        self.metadata = metadata;
        self
    }
}

pub struct CardGrid {
    pub cards: Vec<Card>,
    pub columns: u32,
    pub gap: f32,
    pub max_width: f32,
    pub show_metadata: bool,
    pub show_duration: bool,
}

impl CardGrid {
    pub fn new(columns: u32) -> Self {
        Self {
            cards: Vec::new(),
            columns,
            gap: 16.0,
            max_width: 1800.0,
            show_metadata: true,
            show_duration: true,
        }
    }

    pub fn add(&mut self, card: Card) {
        self.cards.push(card);
    }

    pub fn count(&self) -> usize {
        self.cards.len()
    }

    pub fn calculate_cell_width(&self, container_w: f32) -> f32 {
        let total_gaps = self.gap * (self.columns as f32 - 1.0);
        (container_w - total_gaps) / self.columns as f32
    }

    pub fn calculate_thumbnail_height(&self, cell_w: f32) -> f32 {
        cell_w * 9.0 / 16.0 // 16:9 aspect
    }

    pub fn calculate_card_height(&self, cell_w: f32) -> f32 {
        let thumb_h = self.calculate_thumbnail_height(cell_w);
        let meta_h = if self.show_metadata { 60.0 } else { 24.0 };
        thumb_h + meta_h + 8.0
    }

    pub fn rows(&self) -> u32 {
        if self.columns == 0 { return 0; }
        ((self.cards.len() as u32 + self.columns - 1) / self.columns) as u32
    }

    pub fn total_height(&self, container_w: f32) -> f32 {
        if self.cards.is_empty() { return 0.0; }
        let cell_w = self.calculate_cell_width(container_w);
        let cell_h = self.calculate_card_height(cell_w);
        let row_gap = self.gap;
        let rows = self.rows() as f32;
        rows * cell_h + (rows - 1.0) * row_gap
    }

    /// Posición (x, y) de una card por índice
    pub fn position_for(&self, index: usize, container_w: f32) -> (f32, f32) {
        let col = (index as u32) % self.columns;
        let row = (index as u32) / self.columns;
        let cell_w = self.calculate_cell_width(container_w);
        let cell_h = self.calculate_card_height(cell_w);
        let x = col as f32 * (cell_w + self.gap);
        let y = row as f32 * (cell_h + self.gap);
        (x, y)
    }
}

pub fn format_views_short(n: u64) -> String {
    if n >= 1_000_000_000 {
        format!("{:.1}B", n as f64 / 1_000_000_000.0)
    } else if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

pub fn format_age(iso: &str) -> String {
    if let Some(date_part) = iso.split('T').next() {
        let parts: Vec<&str> = date_part.split('-').collect();
        if parts.len() == 3 {
            if let (Ok(year), Ok(month), Ok(day)) = (
                parts[0].parse::<i32>(),
                parts[1].parse::<u32>(),
                parts[2].parse::<u32>()
            ) {
                let now_year = 2026;
                let now_month = 6;
                let now_day = 23;
                let years_ago = now_year - year;
                if years_ago > 0 {
                    return format!("hace {} año{}", years_ago, if years_ago > 1 { "s" } else { "" });
                }
                let months_ago = now_month as i32 - month as i32;
                if months_ago > 0 {
                    return format!("hace {} mes{}", months_ago, if months_ago > 1 { "es" } else { "" });
                }
                let days_ago = if now_day >= day { now_day - day } else { day - now_day };
                return format!("hace {} día{}", days_ago, if days_ago > 1 { "s" } else { "" });
            }
        }
    }
    iso.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_from_str() {
        assert_eq!(CardLayout::from_str("vertical"), CardLayout::Vertical);
        assert_eq!(CardLayout::from_str("horizontal"), CardLayout::Horizontal);
        assert_eq!(CardLayout::from_str("compact"), CardLayout::Compact);
        assert_eq!(CardLayout::from_str("detailed"), CardLayout::Detailed);
    }

    #[test]
    fn test_layout_title_size() {
        assert_eq!(CardLayout::Vertical.title_size(), 14.0);
        assert_eq!(CardLayout::Detailed.title_size(), 16.0);
    }

    #[test]
    fn test_metadata_new() {
        let m = CardMetadata::new("Test");
        assert_eq!(m.author, "Test");
    }

    #[test]
    fn test_metadata_with_views() {
        let m = CardMetadata::new("X").with_views(1500);
        assert_eq!(m.views, 1500);
    }

    #[test]
    fn test_format_views() {
        assert_eq!(CardMetadata::new("X").with_views(500).format_views(), "500 vistas");
        assert_eq!(CardMetadata::new("X").with_views(1500).format_views(), "1.5K vistas");
        assert_eq!(CardMetadata::new("X").with_views(2_500_000).format_views(), "2.5M vistas");
    }

    #[test]
    fn test_format_views_zero() {
        let m = CardMetadata::new("X");
        assert_eq!(m.format_views(), "");
    }

    #[test]
    fn test_metadata_verified() {
        let m = CardMetadata::new("X").with_verified(true);
        assert!(m.verified);
    }

    #[test]
    fn test_card_new() {
        let c = Card::new(1, "Title", "https://x.com");
        assert_eq!(c.id, 1);
        assert_eq!(c.title, "Title");
    }

    #[test]
    fn test_card_with_thumbnail() {
        let c = Card::new(1, "Title", "u").with_thumbnail("thumb.jpg");
        assert_eq!(c.thumbnail_src, Some("thumb.jpg".to_string()));
    }

    #[test]
    fn test_card_with_layout() {
        let c = Card::new(1, "T", "u").with_layout(CardLayout::Horizontal);
        assert_eq!(c.layout, CardLayout::Horizontal);
    }

    #[test]
    fn test_card_with_metadata() {
        let c = Card::new(1, "T", "u").with_metadata(CardMetadata::new("Author"));
        assert_eq!(c.metadata.author, "Author");
    }

    #[test]
    fn test_grid_new() {
        let g = CardGrid::new(3);
        assert_eq!(g.columns, 3);
        assert_eq!(g.cards.len(), 0);
    }

    #[test]
    fn test_grid_add() {
        let mut g = CardGrid::new(3);
        g.add(Card::new(1, "T", "u"));
        g.add(Card::new(2, "T2", "u2"));
        assert_eq!(g.count(), 2);
    }

    #[test]
    fn test_grid_cell_width() {
        let g = CardGrid::new(3);
        let w = g.calculate_cell_width(800.0);
        // (800 - 16*2) / 3 = 256
        assert_eq!(w, 256.0);
    }

    #[test]
    fn test_grid_thumbnail_height() {
        let g = CardGrid::new(3);
        let h = g.calculate_thumbnail_height(320.0);
        // 320 * 9 / 16 = 180
        assert_eq!(h, 180.0);
    }

    #[test]
    fn test_grid_card_height() {
        let g = CardGrid::new(3);
        let h = g.calculate_card_height(320.0);
        // 180 + 60 + 8 = 248
        assert_eq!(h, 248.0);
    }

    #[test]
    fn test_grid_rows() {
        let mut g = CardGrid::new(3);
        assert_eq!(g.rows(), 0);
        for i in 0..7 {
            g.add(Card::new(i, "T", "u"));
        }
        // 7 cards / 3 cols = 3 rows (ceil)
        assert_eq!(g.rows(), 3);
    }

    #[test]
    fn test_grid_total_height() {
        let mut g = CardGrid::new(3);
        for i in 0..6 {
            g.add(Card::new(i, "T", "u"));
        }
        let h = g.total_height(800.0);
        // 2 rows * cell_h + 1 gap
        assert!(h > 0.0);
    }

    #[test]
    fn test_grid_position_for() {
        let mut g = CardGrid::new(3);
        for i in 0..6 {
            g.add(Card::new(i, "T", "u"));
        }
        let (x, y) = g.position_for(0, 800.0);
        assert_eq!(x, 0.0);
        assert_eq!(y, 0.0);
        let (_, y) = g.position_for(3, 800.0);
        assert!(y > 0.0);
    }

    #[test]
    fn test_format_views_short() {
        assert_eq!(format_views_short(500), "500");
        assert_eq!(format_views_short(1500), "1.5K");
        assert_eq!(format_views_short(2_500_000), "2.5M");
        assert_eq!(format_views_short(1_500_000_000), "1.5B");
    }

    #[test]
    fn test_format_age() {
        assert_eq!(format_age("2026-06-15"), "hace 8 días");
        assert_eq!(format_age("2026-05-01"), "hace 1 mes");
        assert_eq!(format_age("2025-01-01"), "hace 1 año");
        assert_eq!(format_age("2024-12-01"), "hace 2 años");
    }

    #[test]
    fn test_format_age_invalid() {
        assert_eq!(format_age("not a date"), "not a date");
    }

    #[test]
    fn test_metadata_with_duration() {
        let m = CardMetadata::new("X").with_duration("2:30");
        assert_eq!(m.duration, "2:30");
    }

    #[test]
    fn test_card_layout_meta_size() {
        assert_eq!(CardLayout::Compact.meta_size(), 10.0);
    }
}
