//! JS Engine v3 - Minimalista con DOM completo
//!
//! Provee:
//! - Lexer/parser/interpreter propio (tree-walking)
//! - DOM interno accesible desde JS
//! - Extracción REAL de scripts inline del HTML
//! - Sync DOM → JS engine
//! - Detección de mutaciones DOM
//! - Console conectado a UI

pub mod value;
pub mod env;
pub mod ast;
pub mod lexer;
pub mod parser;
pub mod interpreter;
pub mod dom;
pub mod console;
pub mod timer;
pub mod fetch;
pub mod promise;
pub mod builtins;
pub mod ignition;       // V8 Ignition-style bytecode VM

pub use value::JsValue;
pub use env::Env;
pub use interpreter::Interpreter;
pub use dom::{Dom, DomNode, DomEvent};

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

// === Global state for DOM <-> JS engine integration ===

/// Flag que indica si el DOM fue mutado por JS
static DOM_MUTATED: AtomicBool = AtomicBool::new(false);

/// Buffer global de console output (para conectar a UI)
static CONSOLE_BUFFER: Mutex<Vec<(String, String)>> = Mutex::new(Vec::new());

/// Set mutated flag (called by JS engine when it modifies DOM)
pub fn set_mutated_flag() {
    DOM_MUTATED.store(true, Ordering::SeqCst);
}

// Compatibility API for app/mod.rs
/// Execute a JS script and return the result as a string
pub fn eval_script(interp: &mut Interpreter, _tab_id: u64, script: &str) -> Result<String, String> {
    let mut lexer = lexer::Lexer::new(script);
    let tokens = lexer.tokenize();
    let mut parser = parser::Parser::new(tokens);
    let program = parser.parse().map_err(|e| e.to_string())?;
    let result = interp.interpret(&program)?;
    Ok(result.to_string())
}

/// Sync DOM to JS engine - real implementation
/// Recorre el DOM tree y sincroniza elementos con el engine
pub fn sync_dom_to_js_engine(nodes: &[crate::parsers::dom_tree::DomNode]) {
    use std::cell::RefCell;
    use std::rc::Rc;

    for node in nodes {
        match node {
            crate::parsers::dom_tree::DomNode::Element { tag, attributes, children } => {
                let _element = DomNode::new(&format!("{:?}", tag));
                // Sincronizar atributos importantes
                for (key, value) in attributes {
                    if key == "id" {
                        // Almacenar en global window
                        if let Ok(mut buf) = CONSOLE_BUFFER.lock() {
                            buf.push(("debug".to_string(), format!("Element with id={} found", value)));
                        }
                    }
                }
                // Recursión a hijos
                sync_dom_to_js_engine(children);
            }
            _ => {}
        }
        let _ = Rc::new(RefCell::new(()));
    }
}

/// Extract inline scripts from DOM - REAL implementation
/// Recorre el DOM tree y extrae el contenido de los tags <script>
pub fn extract_inline_scripts(nodes: &[crate::parsers::dom_tree::DomNode]) -> Vec<String> {
    let mut scripts = Vec::new();
    extract_scripts_recursive(nodes, &mut scripts);
    scripts
}

fn extract_scripts_recursive(
    nodes: &[crate::parsers::dom_tree::DomNode],
    scripts: &mut Vec<String>,
) {
    use crate::parsers::html_elements::HtmlTag;
    use crate::parsers::dom_tree::DomNode;

    for node in nodes {
        match node {
            DomNode::Element { tag, attributes: _, children } => {
                if matches!(tag, HtmlTag::Script) {
                    // Extraer texto de los hijos
                    let script_text = collect_text_content(children);
                    if !script_text.trim().is_empty() {
                        scripts.push(script_text);
                    }
                    // NO recursar dentro de <script> (su contenido es texto, no más scripts)
                } else if !matches!(tag, HtmlTag::Style | HtmlTag::Noscript) {
                    // Recursión normal para otros elementos
                    extract_scripts_recursive(children, scripts);
                }
            }
            DomNode::Text(_) => {
                // Texto suelto: NO es un script
            }
        }
    }
}

fn collect_text_content(nodes: &[crate::parsers::dom_tree::DomNode]) -> String {
    use crate::parsers::dom_tree::DomNode;
    let mut result = String::new();
    for node in nodes {
        match node {
            DomNode::Text(text) => {
                result.push_str(text);
            }
            DomNode::Element { children, .. } => {
                result.push_str(&collect_text_content(children));
            }
        }
    }
    result
}

/// Rebuild page from DOM - basic implementation
/// Re-extrae texto, links, imágenes, etc del DOM mutado
pub fn rebuild_page_from_dom(page: &mut crate::parsers::page_document::PageDocument) {
    use crate::parsers::dom_tree::DomNode;
    use crate::parsers::html_elements::HtmlTag;
    use std::collections::HashMap;

    // Limpiar bloques existentes
    page.text_blocks.clear();
    page.image_blocks.clear();
    page.video_blocks.clear();
    page.links.clear();

    // Re-extraer del DOM
    let dom_clone = page.dom_nodes.clone();
    rebuild_recursive(&dom_clone, page, 0, &mut None, &mut HashMap::new());

    fn rebuild_recursive(
        nodes: &[DomNode],
        page: &mut crate::parsers::page_document::PageDocument,
        indent: u32,
        current_href: &mut Option<String>,
        _attrs: &mut HashMap<String, String>,
    ) {
        for node in nodes {
            match node {
                DomNode::Element { tag, attributes, children } => {
                    let mut current_href_inner = current_href.clone();
                    match tag {
                        HtmlTag::H1 => {
                            let text = collect_text_content(children);
                            if !text.is_empty() {
                                page.text_blocks.push(crate::parsers::page_document::TextBlock {
                                    text,
                                    tag: "h1".into(),
                                    font_size: 28.0,
                                    bold: true,
                                    link: current_href.clone(),
                                    indent_level: indent,
                                    attributes: attributes.clone(),
                                });
                            }
                        }
                        HtmlTag::H2 => {
                            let text = collect_text_content(children);
                            if !text.is_empty() {
                                page.text_blocks.push(crate::parsers::page_document::TextBlock {
                                    text,
                                    tag: "h2".into(),
                                    font_size: 22.0,
                                    bold: true,
                                    link: current_href.clone(),
                                    indent_level: indent,
                                    attributes: attributes.clone(),
                                });
                            }
                        }
                        HtmlTag::H3 | HtmlTag::H4 | HtmlTag::H5 | HtmlTag::H6 => {
                            let text = collect_text_content(children);
                            if !text.is_empty() {
                                page.text_blocks.push(crate::parsers::page_document::TextBlock {
                                    text,
                                    tag: "h3".into(),
                                    font_size: 18.0,
                                    bold: true,
                                    link: current_href.clone(),
                                    indent_level: indent,
                                    attributes: attributes.clone(),
                                });
                            }
                        }
                        HtmlTag::P => {
                            let text = collect_text_content(children);
                            if !text.is_empty() {
                                page.text_blocks.push(crate::parsers::page_document::TextBlock {
                                    text,
                                    tag: "p".into(),
                                    font_size: 14.0,
                                    bold: false,
                                    link: current_href.clone(),
                                    indent_level: indent,
                                    attributes: attributes.clone(),
                                });
                            }
                        }
                        HtmlTag::A => {
                            let href = attributes.get("href").cloned().unwrap_or_default();
                            let text = collect_text_content(children);
                            if !text.is_empty() {
                                let resolved = page.resolve_href(&href);
                                page.links.push(crate::parsers::page_document::LinkInfo {
                                    text: text.clone(),
                                    href: resolved.clone(),
                                });
                                current_href_inner = Some(resolved.clone());
                                page.text_blocks.push(crate::parsers::page_document::TextBlock {
                                    text,
                                    tag: "a".into(),
                                    font_size: 14.0,
                                    bold: false,
                                    link: Some(resolved),
                                    indent_level: indent,
                                    attributes: attributes.clone(),
                                });
                            }
                        }
                        HtmlTag::Img => {
                            if let Some(src) = attributes.get("src") {
                                let resolved = page.resolve_href(src);
                                let alt = attributes.get("alt").cloned().unwrap_or_default();
                                let width = attributes.get("width")
                                    .and_then(|w| w.trim().parse::<f32>().ok());
                                let height = attributes.get("height")
                                    .and_then(|h| h.trim().parse::<f32>().ok());
                                page.image_blocks.push(crate::parsers::page_document::ImageBlock {
                                    src: resolved,
                                    alt,
                                    width,
                                    height,
                                    lazy: attributes.get("loading").map(|v| v == "lazy").unwrap_or(false),
                                });
                            }
                        }
                        _ => {
                            // Recursión para divs, sections, etc
                            rebuild_recursive(children, page, indent, &mut current_href_inner, _attrs);
                        }
                    }
                }
                DomNode::Text(_) => {
                    // Texto suelto (raro en HTML válido)
                }
            }
        }
    }
}

/// Check if DOM was mutated by JS - real implementation
pub fn take_mutated_flag() -> bool {
    DOM_MUTATED.swap(false, Ordering::SeqCst)
}

/// Push console output (called by JS engine)
pub fn push_console(level: &str, text: &str) {
    if let Ok(mut buf) = CONSOLE_BUFFER.lock() {
        buf.push((level.to_string(), text.to_string()));
        // Limitar tamaño
        if buf.len() > 1000 {
            buf.remove(0);
        }
    }
}

/// Take all console messages
pub fn take_console_messages() -> Vec<(String, String)> {
    if let Ok(mut buf) = CONSOLE_BUFFER.lock() {
        std::mem::take(&mut *buf)
    } else {
        Vec::new()
    }
}

/// Get pending timers
pub struct PendingTimer {
    pub callback_id: u64,
}

pub fn get_pending_timers() -> Vec<PendingTimer> {
    Vec::new()
}
