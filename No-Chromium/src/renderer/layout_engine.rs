// Layout Engine stubs for Fase 0
// Stub implemented to resolve import errors

/// Layout box representing a rendered element
#[derive(Clone, Debug)]
pub struct RenderBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub element_id: Option<String>,
    pub is_link: bool,
    pub is_input: bool,
}

impl RenderBox {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
            element_id: None,
            is_link: false,
            is_input: false,
        }
    }
}

/// Fragment of layout for incremental rendering
#[derive(Clone, Debug)]
pub enum LayoutFragment {
    Text(TextFragment),
    Image(ImageFragment),
    Box(RenderBox),
}

#[derive(Clone, Debug)]
pub struct TextFragment {
    pub text: String,
    pub x: f32,
    pub y: f32,
    pub form_action: Option<String>,
    pub input_name: String,
    pub input_value: String,
    pub is_input: bool,
}

#[derive(Clone, Debug)]
pub struct ImageFragment {
    pub src: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}
