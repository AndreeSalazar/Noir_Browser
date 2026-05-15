use crate::parsers::dom_tree::DomNode;
use crate::parsers::html_elements::HtmlTag;
use url::Url;

#[derive(Debug, Clone)]
pub enum ScriptSource {
    Inline(String),
    External(String),
}

pub fn collect_scripts(nodes: &[DomNode], base_url: Option<&Url>) -> Vec<ScriptSource> {
    let mut scripts = Vec::new();
    collect_scripts_inner(nodes, base_url, &mut scripts);
    scripts
}

fn collect_scripts_inner(
    nodes: &[DomNode],
    base_url: Option<&Url>,
    scripts: &mut Vec<ScriptSource>,
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

        if matches!(tag, HtmlTag::Script) {
            if !is_javascript_script(attributes.get("type").map(String::as_str)) {
                continue;
            }

            if let Some(src) = attributes
                .get("src")
                .and_then(|src| resolve_script_url(base_url, src))
            {
                scripts.push(ScriptSource::External(src));
            }

            let inline = script_text(children);
            if !inline.trim().is_empty() {
                scripts.push(ScriptSource::Inline(inline));
            }

            continue;
        }

        collect_scripts_inner(children, base_url, scripts);
    }
}

fn is_javascript_script(script_type: Option<&str>) -> bool {
    let Some(script_type) = script_type else {
        return true;
    };

    matches!(
        script_type.trim().to_ascii_lowercase().as_str(),
        "" | "module"
            | "text/javascript"
            | "application/javascript"
            | "application/ecmascript"
            | "text/ecmascript"
    )
}

fn resolve_script_url(base_url: Option<&Url>, src: &str) -> Option<String> {
    if src.trim().is_empty() {
        return None;
    }

    match base_url {
        Some(base) => base.join(src).ok().map(|url| url.to_string()),
        None => Some(src.to_string()),
    }
}

fn script_text(nodes: &[DomNode]) -> String {
    let mut out = String::new();
    for node in nodes {
        match node {
            DomNode::Text(text) => {
                out.push_str(text);
                out.push('\n');
            }
            DomNode::Element { children, .. } => out.push_str(&script_text(children)),
        }
    }
    out
}
