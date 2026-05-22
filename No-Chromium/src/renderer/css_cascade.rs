// CSS Cascade stubs for Fase 0
// Stub implemented to resolve import errors

/// Computed style after CSS cascade resolution
#[derive(Clone, Debug, Default)]
pub struct ComputedStyle {
    pub background_color: Option<String>,
    pub color: Option<String>,
    pub width: Option<String>,
    pub height: Option<String>,
    pub font_size: Option<f32>,
    pub margin: Option<f32>,
    pub padding: Option<f32>,
}

impl ComputedStyle {
    pub fn merge(&mut self, other: &ComputedStyle) {
        if other.background_color.is_some() {
            self.background_color.clone_from(&other.background_color);
        }
        if other.color.is_some() {
            self.color.clone_from(&other.color);
        }
        if other.width.is_some() {
            self.width.clone_from(&other.width);
        }
        if other.height.is_some() {
            self.height.clone_from(&other.height);
        }
        if other.font_size.is_some() {
            self.font_size = other.font_size;
        }
        if other.margin.is_some() {
            self.margin = other.margin;
        }
        if other.padding.is_some() {
            self.padding = other.padding;
        }
    }
}
