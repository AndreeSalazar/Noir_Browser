//! Tests para hit_test_link, recalculate_layout y eventos

use no_chromium::app::AppConfig;
use no_chromium::app::context::AppContext;
use no_chromium::parsers::layout::{hit_test_link, LayoutItem, LayoutBlock};
use no_chromium::parsers::page_document::PageDocument;

fn make_link(x: f32, y: f32, w: f32, h: f32, href: &str) -> LayoutItem {
    LayoutItem::Text(LayoutBlock {
        x, y, w, h,
        text: "Test Link".to_string(),
        font_size: 14.0,
        bold: false,
        color: [1.0, 1.0, 1.0, 1.0],
        bg_color: None,
        href: Some(href.to_string()),
        is_link: true,
        padding_top: 0.0,
        padding_bottom: 0.0,
        padding_left: 0.0,
        margin_top: 0.0,
        margin_bottom: 0.0,
    })
}

#[test]
fn test_hit_test_link_basic() {
    let items = vec![make_link(100.0, 200.0, 80.0, 20.0, "https://example.com")];
    let result = hit_test_link(&items, 120.0, 210.0, 0.0);
    assert_eq!(result, Some("https://example.com".to_string()));
}

#[test]
fn test_hit_test_link_outside() {
    let items = vec![make_link(100.0, 200.0, 80.0, 20.0, "https://example.com")];
    let result = hit_test_link(&items, 50.0, 50.0, 0.0);
    assert_eq!(result, None);
}

#[test]
fn test_hit_test_link_with_scroll() {
    let items = vec![make_link(100.0, 200.0, 80.0, 20.0, "https://example.com")];
    let result = hit_test_link(&items, 120.0, 10.0, 200.0);
    assert_eq!(result, Some("https://example.com".to_string()));
}

#[test]
fn test_hit_test_link_not_a_link() {
    let item = LayoutItem::Text(LayoutBlock {
        x: 100.0, y: 200.0, w: 80.0, h: 20.0,
        text: "Text".to_string(),
        font_size: 14.0,
        bold: false,
        color: [1.0, 1.0, 1.0, 1.0],
        bg_color: None,
        href: None,
        is_link: false,
        padding_top: 0.0,
        padding_bottom: 0.0,
        padding_left: 0.0,
        margin_top: 0.0,
        margin_bottom: 0.0,
    });
    let items = vec![item];
    let result = hit_test_link(&items, 120.0, 210.0, 0.0);
    assert_eq!(result, None);
}

#[test]
fn test_hit_test_multiple_links() {
    let items = vec![
        make_link(100.0, 100.0, 80.0, 20.0, "https://a.com"),
        make_link(100.0, 200.0, 80.0, 20.0, "https://b.com"),
    ];
    let result_a = hit_test_link(&items, 120.0, 110.0, 0.0);
    let result_b = hit_test_link(&items, 120.0, 210.0, 0.0);
    assert_eq!(result_a, Some("https://a.com".to_string()));
    assert_eq!(result_b, Some("https://b.com".to_string()));
}

#[test]
fn test_recalculate_layout() {
    let mut ctx = AppContext::new(AppConfig::default());
    ctx.width = 1920;
    ctx.height = 1080;

    let html = r#"
        <html><body>
            <h1>Test</h1>
            <p>Some content here</p>
        </body></html>
    "#;
    let page = PageDocument::from_html("https://test.com", html);
    ctx.tabs[ctx.active_tab].page = Some(page);
    ctx.tabs[ctx.active_tab].url = "https://test.com".to_string();

    ctx.recalculate_layout();

    let blocks = &ctx.tabs[ctx.active_tab].layout_blocks;
    assert!(!blocks.is_empty());
}

#[test]
fn test_recalculate_layout_with_no_page() {
    let mut ctx = AppContext::new(AppConfig::default());
    ctx.recalculate_layout();
    assert!(ctx.tabs[ctx.active_tab].layout_blocks.is_empty());
}

#[test]
fn test_recalculate_layout_different_widths() {
    let mut ctx = AppContext::new(AppConfig::default());

    let html = "<html><body><p>Hello world this is a test of layout width</p></body></html>";
    let page = PageDocument::from_html("https://test.com", html);

    ctx.width = 800;
    ctx.tabs[ctx.active_tab].page = Some(page.clone());
    ctx.recalculate_layout();
    let blocks_800 = ctx.tabs[ctx.active_tab].layout_blocks.clone();
    let count_800 = blocks_800.len();

    ctx.width = 1920;
    ctx.tabs[ctx.active_tab].page = Some(page.clone());
    ctx.recalculate_layout();
    let blocks_1920 = ctx.tabs[ctx.active_tab].layout_blocks.clone();
    let count_1920 = blocks_1920.len();

    assert_eq!(count_800, count_1920);
}

#[test]
fn test_hit_test_link_edge_cases() {
    let items = vec![make_link(0.0, 0.0, 100.0, 20.0, "https://test.com")];

    // Exactly on top-left corner
    assert_eq!(hit_test_link(&items, 0.0, 0.0, 0.0), Some("https://test.com".to_string()));

    // Exactly on bottom-right corner
    assert_eq!(hit_test_link(&items, 100.0, 20.0, 0.0), Some("https://test.com".to_string()));

    // Just outside bottom
    assert_eq!(hit_test_link(&items, 50.0, 25.0, 0.0), None);
}
