//! DOM module - Live mutable DOM (FASE B1)

pub mod live_dom;

pub use live_dom::{
    DomNode, NodeType,
    query_selector, query_selector_all,
    get_element_by_id,
    get_elements_by_tag_name,
    get_elements_by_class_name,
};
