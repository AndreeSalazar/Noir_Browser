use crate::parsers::dom_tree::DomNode;
use crate::parsers::html_elements::HtmlTag;
use url::Url;

#[derive(Debug, Clone)]
pub struct StylesheetLink {
    pub url: String,
}

#[derive(Debug, Clone, Default)]
pub struct StylesheetBundle {
    pub blocks: Vec<String>,
    pub external_count: usize,
    pub loaded_external: usize,
    pub inline_count: usize,
}

pub fn collect_stylesheets(nodes: &[DomNode], base_url: Option<&Url>) -> Vec<StylesheetLink> {
    let mut links = Vec::new();
    collect_stylesheets_inner(nodes, base_url, &mut links);
    links
}

pub fn collect_inline_styles(nodes: &[DomNode]) -> Vec<String> {
    let mut styles = Vec::new();
    collect_inline_styles_inner(nodes, &mut styles);
    styles
}

fn collect_inline_styles_inner(nodes: &[DomNode], styles: &mut Vec<String>) {
    for node in nodes {
        let DomNode::Element { tag, children, .. } = node else {
            continue;
        };

        if matches!(tag, HtmlTag::Custom(name) if name == "style") {
            let css = children
                .iter()
                .filter_map(|child| match child {
                    DomNode::Text(text) => Some(text.as_str()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join("\n");
            if !css.trim().is_empty() {
                styles.push(css);
            }
        }

        collect_inline_styles_inner(children, styles);
    }
}

fn collect_stylesheets_inner(
    nodes: &[DomNode],
    base_url: Option<&Url>,
    links: &mut Vec<StylesheetLink>,
) {
    for node in nodes {
        let DomNode::Element {
            tag,
            attributes,
            children,
        } = node
        else {
            continue;
        };

        let is_stylesheet = matches!(tag, HtmlTag::Custom(name) if name == "link")
            && attributes
                .get("rel")
                .is_some_and(|rel| rel.to_ascii_lowercase().contains("stylesheet"));

        if is_stylesheet {
            if let Some(url) = attributes
                .get("href")
                .and_then(|href| resolve_url(base_url, href))
            {
                links.push(StylesheetLink { url });
            }
        }

        collect_stylesheets_inner(children, base_url, links);
    }
}

fn resolve_url(base_url: Option<&Url>, href: &str) -> Option<String> {
    if href.starts_with('#') || href.starts_with("javascript:") || href.starts_with("data:") {
        return None;
    }

    match base_url {
        Some(base) => base.join(href).ok().map(|url| url.to_string()),
        None => Some(href.to_string()),
    }
}
