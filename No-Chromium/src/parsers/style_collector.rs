use crate::parsers::dom_tree::DomNode;
use crate::parsers::html_elements::HtmlTag;
use url::Url;

#[derive(Debug, Clone)]
pub struct StylesheetLink {
    pub url: String,
}

pub fn collect_stylesheets(nodes: &[DomNode], base_url: Option<&Url>) -> Vec<StylesheetLink> {
    let mut links = Vec::new();
    collect_stylesheets_inner(nodes, base_url, &mut links);
    links
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
