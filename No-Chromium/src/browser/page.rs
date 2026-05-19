#[path = "page_modular/mod.rs"]
mod page_modular;

pub use page_modular::{load_page_document, render_page, PageDocument, RenderBox};
