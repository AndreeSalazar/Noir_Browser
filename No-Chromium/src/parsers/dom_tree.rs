use crate::parsers::html_elements::HtmlTag;
use html5ever::parse_document;
use html5ever::tendril::TendrilSink;
use markup5ever_rcdom::{Handle, NodeData, RcDom};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum DomNode {
    Element {
        tag: HtmlTag,
        attributes: HashMap<String, String>,
        children: Vec<DomNode>,
    },
    Text(String),
}

pub fn parse_html(html: &str) -> Vec<DomNode> {
    let dom = parse_document(RcDom::default(), Default::default())
        .from_utf8()
        .read_from(&mut html.as_bytes())
        .unwrap();

    let mut children = Vec::new();
    for child in dom.document.children.borrow().iter() {
        if let Some(node) = visit_node(child) {
            children.push(node);
        }
    }
    children
}

fn visit_node(handle: &Handle) -> Option<DomNode> {
    match &handle.data {
        NodeData::Element { name, attrs, .. } => {
            let tag_name = name.local.as_ref();
            let tag = map_tag_name(tag_name);

            // Skip script, noscript, and style elements completely
            if matches!(tag, HtmlTag::Noscript | HtmlTag::Script | HtmlTag::Style) {
                return None;
            }

            let mut attributes = HashMap::new();
            for attr in attrs.borrow().iter() {
                attributes.insert(attr.name.local.to_string(), attr.value.to_string());
            }

            let mut children = Vec::new();
            for child in handle.children.borrow().iter() {
                if let Some(node) = visit_node(child) {
                    children.push(node);
                }
            }

            Some(DomNode::Element {
                tag,
                attributes,
                children,
            })
        }
        NodeData::Text { contents } => {
            let text = contents.borrow().to_string();
            // Optional: trim whitespace
            if text.trim().is_empty() {
                None
            } else {
                Some(DomNode::Text(text))
            }
        }
        NodeData::Document => None,
        NodeData::Doctype { .. } => None,
        NodeData::Comment { .. } => None,
        NodeData::ProcessingInstruction { .. } => None,
    }
}

fn map_tag_name(name: &str) -> HtmlTag {
    match name.to_lowercase().as_str() {
        "address" => HtmlTag::Address,
        "article" => HtmlTag::Article,
        "aside" => HtmlTag::Aside,
        "footer" => HtmlTag::Footer,
        "header" => HtmlTag::Header,
        "h1" => HtmlTag::H1,
        "h2" => HtmlTag::H2,
        "h3" => HtmlTag::H3,
        "h4" => HtmlTag::H4,
        "h5" => HtmlTag::H5,
        "h6" => HtmlTag::H6,
        "main" => HtmlTag::Main,
        "nav" => HtmlTag::Nav,
        "section" => HtmlTag::Section,
        "blockquote" => HtmlTag::Blockquote,
        "dd" => HtmlTag::Dd,
        "div" => HtmlTag::Div,
        "dl" => HtmlTag::Dl,
        "dt" => HtmlTag::Dt,
        "figcaption" => HtmlTag::Figcaption,
        "figure" => HtmlTag::Figure,
        "hr" => HtmlTag::Hr,
        "li" => HtmlTag::Li,
        "ol" => HtmlTag::Ol,
        "p" => HtmlTag::P,
        "pre" => HtmlTag::Pre,
        "ul" => HtmlTag::Ul,
        "a" => HtmlTag::A,
        "abbr" => HtmlTag::Abbr,
        "b" => HtmlTag::B,
        "bdi" => HtmlTag::Bdi,
        "bdo" => HtmlTag::Bdo,
        "br" => HtmlTag::Br,
        "cite" => HtmlTag::Cite,
        "code" => HtmlTag::Code,
        "data" => HtmlTag::Data,
        "dfn" => HtmlTag::Dfn,
        "em" => HtmlTag::Em,
        "i" => HtmlTag::I,
        "kbd" => HtmlTag::Kbd,
        "mark" => HtmlTag::Mark,
        "q" => HtmlTag::Q,
        "rp" => HtmlTag::Rp,
        "rt" => HtmlTag::Rt,
        "ruby" => HtmlTag::Ruby,
        "s" => HtmlTag::S,
        "samp" => HtmlTag::Samp,
        "small" => HtmlTag::Small,
        "span" => HtmlTag::Span,
        "strong" => HtmlTag::Strong,
        "sub" => HtmlTag::Sub,
        "sup" => HtmlTag::Sup,
        "time" => HtmlTag::Time,
        "u" => HtmlTag::U,
        "var" => HtmlTag::Var,
        "wbr" => HtmlTag::Wbr,
        "area" => HtmlTag::Area,
        "audio" => HtmlTag::Audio,
        "img" => HtmlTag::Img,
        "map" => HtmlTag::Map,
        "track" => HtmlTag::Track,
        "video" => HtmlTag::Video,
        "embed" => HtmlTag::Embed,
        "iframe" => HtmlTag::Iframe,
        "object" => HtmlTag::Object,
        "param" => HtmlTag::Param,
        "picture" => HtmlTag::Picture,
        "portal" => HtmlTag::Portal,
        "source" => HtmlTag::Source,
        "svg" => HtmlTag::Svg,
        "math" => HtmlTag::Math,
        "canvas" => HtmlTag::Canvas,
        "noscript" => HtmlTag::Noscript,
        "script" => HtmlTag::Script,
        "style" => HtmlTag::Style,
        "meta" => HtmlTag::Custom("meta".into()),
        "link" => HtmlTag::Custom("link".into()),
        "head" => HtmlTag::Custom("head".into()),
        "title" => HtmlTag::Title,
        "del" => HtmlTag::Del,
        "ins" => HtmlTag::Ins,
        "caption" => HtmlTag::Caption,
        "col" => HtmlTag::Col,
        "colgroup" => HtmlTag::Colgroup,
        "table" => HtmlTag::Table,
        "tbody" => HtmlTag::Tbody,
        "td" => HtmlTag::Td,
        "tfoot" => HtmlTag::Tfoot,
        "th" => HtmlTag::Th,
        "thead" => HtmlTag::Thead,
        "tr" => HtmlTag::Tr,
        "button" => HtmlTag::Button,
        "datalist" => HtmlTag::Datalist,
        "fieldset" => HtmlTag::Fieldset,
        "form" => HtmlTag::Form,
        "input" => HtmlTag::Input,
        "label" => HtmlTag::Label,
        "legend" => HtmlTag::Legend,
        "meter" => HtmlTag::Meter,
        "optgroup" => HtmlTag::Optgroup,
        "option" => HtmlTag::Option,
        "output" => HtmlTag::Output,
        "progress" => HtmlTag::Progress,
        "select" => HtmlTag::Select,
        "textarea" => HtmlTag::Textarea,
        "details" => HtmlTag::Details,
        "dialog" => HtmlTag::Dialog,
        "menu" => HtmlTag::Menu,
        "summary" => HtmlTag::Summary,
        "slot" => HtmlTag::Slot,
        "template" => HtmlTag::Template,
        _ => HtmlTag::Custom(name.to_string()),
    }
}
