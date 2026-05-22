// HTML Parser stubs for Fase 0
// Stub implemented to resolve import errors

use crate::renderer::css_cascade::ComputedStyle;
use crate::renderer::layout_engine::LayoutFragment;

/// Documento HTML parseado listo para layout
#[derive(Clone, Debug)]
pub struct PageDocument {
    pub url: String,
    pub title: String,
    pub fragments: Vec<LayoutFragment>,
    pub computed_style: ComputedStyle,
    pub inputs: Vec<InputField>,
}

#[derive(Clone, Debug)]
pub struct InputField {
    pub name: String,
    pub value: String,
    pub input_type: InputType,
}

#[derive(Clone, Debug, PartialEq)]
pub enum InputType {
    Text,
    Password,
    Email,
    Submit,
    Checkbox,
}

impl PageDocument {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            title: String::new(),
            fragments: Vec::new(),
            computed_style: ComputedStyle::default(),
            inputs: Vec::new(),
        }
    }

    pub fn computed_style(&self) -> ComputedStyle {
        self.computed_style.clone()
    }

    pub fn media_summary(&self) -> Option<String> {
        let count = self.fragments.len();
        if count > 0 {
            Some(format!("{} fragments loaded", count))
        } else {
            None
        }
    }

    pub fn get_input_value(&self, idx: usize) -> Option<String> {
        self.inputs.get(idx).map(|i| i.value.clone())
    }

    pub fn set_input_value(&mut self, idx: usize, value: String) {
        if let Some(input) = self.inputs.get_mut(idx) {
            input.value = value;
        }
    }
}

/// Parser HTML zero-copy (stub para Fase 0)
pub struct HtmlParser {
    buffer: Vec<u8>,
}

impl HtmlParser {
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    pub fn parse(&mut self, html: &[u8]) -> Result<PageDocument, String> {
        // TODO: Implementar parser real en Fase 1
        let mut doc = PageDocument::new("about:blank");
        doc.title = "Noir Browser".to_string();
        Ok(doc)
    }
}
