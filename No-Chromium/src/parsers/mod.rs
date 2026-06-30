pub mod html_elements;
pub mod dom_tree;
pub mod dom_native;
pub mod css_simple;
pub mod css_engine;
pub mod css_lexer;
pub mod js_lexer;
pub mod page_document;
pub mod layout;
pub mod events;
pub mod resource_loader;
pub mod style_collector;
pub mod webidl_bridge;
pub mod flexbox;
pub mod flexbox_v2;
pub mod css_grid_v2;
pub mod position_v2;
pub mod media_queries;
pub mod style_cache;     // Firefox Stylo-style sharing cache
pub mod rule_tree;       // Firefox CSS rule tree
pub mod invalidation;    // Blink-style invalidation
pub mod css_loader;      // FASE A1: CSS externo real
pub mod youtube_extract; // YouTube no-JS compatibility extraction
