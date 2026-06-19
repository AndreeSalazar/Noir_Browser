use crate::parsers::dom_tree::DomNode;
use crate::parsers::html_elements::HtmlTag;
use crate::parsers::page_document::{PageDocument, TextBlock, ImageBlock};
use std::collections::HashMap;

pub fn sync_dom_to_js_engine(nodes: &[DomNode]) {
    let mut elements = HashMap::new();
    collect_elements(nodes, &mut elements, None);

    if let Some(global) = crate::js_engine::dom_bridge::get_elements_static() {
        let mut map = global.lock().unwrap();
        for (id, elem) in elements {
            map.insert(id, elem);
        }
    }
}

fn collect_elements(
    nodes: &[DomNode],
    out: &mut HashMap<String, crate::js_engine::dom_bridge::DomElement>,
    parent_id: Option<String>,
) {
    for node in nodes {
        match node {
            DomNode::Element {
                tag,
                attributes,
                children,
            } => {
                let tag_name = format!("{:?}", tag).to_lowercase();
                let id = attributes
                    .get("id")
                    .cloned()
                    .unwrap_or_else(|| format!("elem_{}", rand::random::<u32>()));
                let text_content = collect_text_content(children);

                let mut attrs = attributes.clone();
                if let Some(parent) = &parent_id {
                    attrs.insert("__parent".to_string(), parent.clone());
                }

                let elem = crate::js_engine::dom_bridge::DomElement {
                    id: id.clone(),
                    tag_name: tag_name.clone(),
                    text_content: text_content.clone(),
                    inner_html: text_content.clone(),
                    attributes: attrs,
                    children_ids: Vec::new(),
                    parent_id: parent_id.clone(),
                };

                out.insert(id.clone(), elem);
                collect_elements(children, out, Some(id));
            }
            DomNode::Text(_) => {}
        }
    }
}

fn collect_text_content(nodes: &[DomNode]) -> String {
    let mut parts = Vec::new();
    for node in nodes {
        match node {
            DomNode::Text(text) => {
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    parts.push(trimmed.to_string());
                }
            }
            DomNode::Element { children, .. } => {
                let t = collect_text_content(children);
                if !t.is_empty() {
                    parts.push(t);
                }
            }
        }
    }
    parts.join(" ")
}

pub fn extract_inline_scripts(nodes: &[DomNode]) -> Vec<String> {
    let mut scripts = Vec::new();
    extract_scripts_recursive(nodes, &mut scripts);
    scripts
}

fn extract_scripts_recursive(nodes: &[DomNode], scripts: &mut Vec<String>) {
    for node in nodes {
        match node {
            DomNode::Element {
                tag, children, ..
            } => {
                if matches!(tag, HtmlTag::Script) {
                    let code = collect_text_content(children);
                    if !code.is_empty() {
                        scripts.push(code);
                    }
                }
                extract_scripts_recursive(children, scripts);
            }
            _ => {}
        }
    }
}

pub fn rebuild_page_from_dom(doc: &mut PageDocument) {
    use crate::js_engine::dom_bridge::DomBridge;

    let elements = DomBridge::get_all_elements();
    if elements.is_empty() {
        return;
    }

    let elem_map: HashMap<String, _> = elements.iter().map(|e| (e.id.clone(), e.clone())).collect();

    let mut body_id = None;
    for elem in &elements {
        if elem.tag_name == "body" || elem.id == "body" {
            body_id = Some(elem.id.clone());
            break;
        }
    }

    if let Some(bid) = body_id {
        render_element_children(&bid, &elem_map, doc, 0, &mut Vec::new(), None);
    } else {
        for elem in &elements {
            if elem.parent_id.is_none() && elem.tag_name != "document" {
                render_element(elem, &elem_map, doc, 0, None);
            }
        }
    }
}

fn render_element_children(
    parent_id: &str,
    elem_map: &HashMap<String, crate::js_engine::dom_bridge::DomElement>,
    doc: &mut PageDocument,
    indent: u32,
    _ancestors: &mut Vec<String>,
    current_href: Option<String>,
) {
    if let Some(parent) = elem_map.get(parent_id) {
        let children = parent.children_ids.clone();
        for child_id in &children {
            if let Some(child) = elem_map.get(child_id) {
                render_element(child, elem_map, doc, indent, current_href.clone());
            }
        }
    }
}

fn render_element(
    elem: &crate::js_engine::dom_bridge::DomElement,
    elem_map: &HashMap<String, crate::js_engine::dom_bridge::DomElement>,
    doc: &mut PageDocument,
    indent: u32,
    current_href: Option<String>,
) {
    let tag = elem.tag_name.as_str();
    let text = elem.text_content.trim();
    let is_empty = text.is_empty() && elem.children_ids.is_empty();

    if is_empty {
        return;
    }

    match tag {
        "h1" => {
            if !text.is_empty() {
                doc.text_blocks.push(TextBlock {
                    text: text.to_string(),
                    tag: "h1".into(),
                    font_size: 28.0,
                    bold: true,
                    link: current_href.clone(),
                    indent_level: indent,
                    attributes: elem.attributes.clone(),
                });
            }
        }
        "h2" => {
            if !text.is_empty() {
                doc.text_blocks.push(TextBlock {
                    text: text.to_string(),
                    tag: "h2".into(),
                    font_size: 22.0,
                    bold: true,
                    link: current_href.clone(),
                    indent_level: indent,
                    attributes: elem.attributes.clone(),
                });
            }
        }
        "h3" => {
            if !text.is_empty() {
                doc.text_blocks.push(TextBlock {
                    text: text.to_string(),
                    tag: "h3".into(),
                    font_size: 18.0,
                    bold: true,
                    link: current_href.clone(),
                    indent_level: indent,
                    attributes: elem.attributes.clone(),
                });
            }
        }
        "h4" | "h5" | "h6" => {
            if !text.is_empty() {
                doc.text_blocks.push(TextBlock {
                    text: text.to_string(),
                    tag: "h4".into(),
                    font_size: 16.0,
                    bold: true,
                    link: current_href.clone(),
                    indent_level: indent,
                    attributes: elem.attributes.clone(),
                });
            }
        }
        "p" => {
            if !text.is_empty() {
                doc.text_blocks.push(TextBlock {
                    text: text.to_string(),
                    tag: "p".into(),
                    font_size: 14.0,
                    bold: false,
                    link: current_href.clone(),
                    indent_level: indent,
                    attributes: elem.attributes.clone(),
                });
            }
        }
        "a" => {
            let href = elem.attributes.get("href").cloned().unwrap_or_default();
            let resolved = if !href.is_empty() {
                Some(doc.resolve_href_simple(&href))
            } else {
                current_href.clone()
            };
            if !text.is_empty() {
                doc.text_blocks.push(TextBlock {
                    text: text.to_string(),
                    tag: "a".into(),
                    font_size: 14.0,
                    bold: false,
                    link: resolved.clone(),
                    indent_level: indent,
                    attributes: elem.attributes.clone(),
                });
            }
            for child_id in &elem.children_ids {
                if let Some(child) = elem_map.get(child_id) {
                    render_element(child, elem_map, doc, indent, resolved.clone());
                }
            }
            return;
        }
        "b" | "strong" => {
            if !text.is_empty() {
                doc.text_blocks.push(TextBlock {
                    text: text.to_string(),
                    tag: "b".into(),
                    font_size: 14.0,
                    bold: true,
                    link: current_href.clone(),
                    indent_level: indent,
                    attributes: elem.attributes.clone(),
                });
            }
        }
        "li" => {
            if !text.is_empty() {
                doc.text_blocks.push(TextBlock {
                    text: format!("  * {}", text),
                    tag: "li".into(),
                    font_size: 14.0,
                    bold: false,
                    link: current_href.clone(),
                    indent_level: indent + 1,
                    attributes: elem.attributes.clone(),
                });
            }
        }
        "pre" | "code" => {
            if !text.is_empty() {
                doc.text_blocks.push(TextBlock {
                    text: text.to_string(),
                    tag: "code".into(),
                    font_size: 12.0,
                    bold: false,
                    link: current_href.clone(),
                    indent_level: indent,
                    attributes: elem.attributes.clone(),
                });
            }
        }
        "blockquote" => {
            if !text.is_empty() {
                doc.text_blocks.push(TextBlock {
                    text: format!("> {}", text),
                    tag: "blockquote".into(),
                    font_size: 14.0,
                    bold: false,
                    link: current_href.clone(),
                    indent_level: indent + 1,
                    attributes: elem.attributes.clone(),
                });
            }
        }
        "hr" => {
            doc.text_blocks.push(TextBlock {
                text: "────────────────────────────────".into(),
                tag: "hr".into(),
                font_size: 14.0,
                bold: false,
                link: None,
                indent_level: indent,
                attributes: elem.attributes.clone(),
            });
        }
        "img" => {
            if let Some(src) = elem.attributes.get("src") {
                let resolved = doc.resolve_href_simple(src);
                let alt = elem.attributes.get("alt").cloned().unwrap_or_default();
                doc.image_blocks.push(ImageBlock {
                    src: resolved,
                    alt,
                    width: None,
                    height: None,
                    lazy: false,
                });
            }
        }
        "style" | "script" | "noscript" | "meta" | "link" | "head" | "title" => {}
        _ => {
            for child_id in &elem.children_ids {
                if let Some(child) = elem_map.get(child_id) {
                    render_element(child, elem_map, doc, indent, current_href.clone());
                }
            }
            return;
        }
    }

    for child_id in &elem.children_ids {
        if let Some(child) = elem_map.get(child_id) {
            render_element(child, elem_map, doc, indent + 1, current_href.clone());
        }
    }
}
