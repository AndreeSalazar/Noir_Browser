//! DOM module - Live mutable DOM (FASE B1)

pub mod live_dom;
pub mod events;
pub mod timers;
pub mod fetch_api;

pub use live_dom::{
    DomNode, NodeType,
    query_selector, query_selector_all,
    get_element_by_id,
    get_elements_by_tag_name,
    get_elements_by_class_name,
};
pub use events::{
    EventType, EventPhase, Event, EventListener, EventTarget, EventLog,
    CallbackId, should_prevent_default,
};
pub use timers::{TimerManager, TimerId, TimerKind, Timer};
pub use fetch_api::{
    UrlSearchParams, FetchMethod, FetchMode, FetchCredentials,
    FetchOptions, FetchResponse,
};
