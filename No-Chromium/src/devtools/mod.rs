//! DevTools Module - Herramientas de desarrollo
//!
//! - inspector: Ver DOM tree de la página
//! - http_error: Páginas de error HTTP (404, 500, etc)
//! - form_fill: Auto-fill y autosave de formularios

#![allow(dead_code)]

pub mod inspector;
pub mod http_error;
pub mod form_fill;

pub use inspector::{Inspector, DomNodeInfo};
pub use http_error::{HttpErrorPage, error_page_html};
pub use form_fill::{FormFillManager, FormField, FilledField};
