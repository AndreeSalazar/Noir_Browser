use crate::parsers::dom_tree::DomNode;
use crate::parsers::html_elements::HtmlTag;
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
                let _class = attributes.get("class").cloned().unwrap_or_default();
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
                };

                out.insert(id.clone(), elem);

                // Also index by tag name for querySelector("tag")
                if !matches!(tag, HtmlTag::Div | HtmlTag::Span | HtmlTag::P | HtmlTag::A) {
                    let tag_elem = crate::js_engine::dom_bridge::DomElement {
                        id: id.clone(),
                        tag_name: tag_name.clone(),
                        text_content: text_content.clone(),
                        inner_html: text_content,
                        attributes: out.get(&id).map(|e| e.attributes.clone()).unwrap_or_default(),
                    };
                    out.insert(format!("__tag_{}", tag_name), tag_elem);
                }

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
