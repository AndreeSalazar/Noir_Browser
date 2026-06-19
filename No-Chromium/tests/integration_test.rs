//! Tests de integración para el navegador completo
//!
//! Verifica el flujo end-to-end de las funcionalidades principales.

use no_chromium::app::navigation::resolve_url;
use no_chromium::app::state::TabState;
use no_chromium::app::AppConfig;
use no_chromium::js_engine_v3::{eval_script, Interpreter};
use no_chromium::parsers::page_document::PageDocument;

#[test]
fn test_resolve_url_https() {
    let result = resolve_url("https://example.com");
    assert_eq!(result, "https://example.com");
}

#[test]
fn test_resolve_url_http() {
    let result = resolve_url("http://example.com");
    assert_eq!(result, "http://example.com");
}

#[test]
fn test_resolve_url_domain_without_protocol() {
    let result = resolve_url("example.com");
    assert_eq!(result, "https://example.com");
}

#[test]
fn test_resolve_url_google_search() {
    let result = resolve_url("rust programming");
    assert!(result.contains("duckduckgo.com"));
    assert!(result.contains("rust"));
}

#[test]
fn test_resolve_url_youtube_prefix() {
    let result = resolve_url("yt rust tutorial");
    assert!(result.contains("youtube.com"));
    assert!(result.contains("rust+tutorial"));
}

#[test]
fn test_resolve_url_github_prefix() {
    let result = resolve_url("gh webgpu");
    assert!(result.contains("github.com"));
    assert!(result.contains("webgpu"));
}

#[test]
fn test_resolve_url_wikipedia_prefix() {
    let result = resolve_url("wiki rust language");
    assert!(result.contains("wikipedia.org"));
    assert!(result.contains("rust"));
}

#[test]
fn test_resolve_url_trims_whitespace() {
    let result = resolve_url("  example.com  ");
    assert_eq!(result, "https://example.com");
}

#[test]
fn test_resolve_url_all_prefixes() {
    let prefixes = vec![
        ("yt", "youtube.com"),
        ("gg", "google.com"),
        ("gh", "github.com"),
        ("ddg", "duckduckgo.com"),
        ("wiki", "wikipedia.org"),
        ("reddit", "reddit.com"),
        ("mdn", "developer.mozilla.org"),
        ("crates", "crates.io"),
        ("docs", "docs.rs"),
        ("npm", "npmjs.com"),
    ];

    for (prefix, expected_domain) in prefixes {
        let result = resolve_url(&format!("{} test query", prefix));
        assert!(
            result.contains(expected_domain),
            "Prefix '{}' should resolve to '{}', got '{}'",
            prefix,
            expected_domain,
            result
        );
    }
}

#[test]
fn test_resolve_url_empty_string_defaults_to_search() {
    let result = resolve_url("");
    assert!(result.contains("duckduckgo.com"));
}

#[test]
fn test_tab_state_default() {
    let tab = TabState::default();
    assert_eq!(tab.title, "New Tab");
    assert_eq!(tab.url, "");
    assert!(tab.page.is_none());
    assert!(tab.layout_blocks.is_empty());
    assert_eq!(tab.scroll_y, 0.0);
    assert_eq!(tab.content_height, 0.0);
    assert_eq!(tab.tab_id, 0);
}

#[test]
fn test_tab_state_initializes_js_engine() {
    let tab = TabState::default();
    let _ = tab.js_engine;
}

#[test]
fn test_appconfig_default_values() {
    let config = AppConfig::default();
    assert_eq!(config.max_tabs, 20);
    assert_eq!(config.cache_size_mb, 512);
    assert!(!config.enable_tor_mode);
    assert!(!config.debug_webgpu);
    assert!(!config.enable_msdf_fonts);
}

#[test]
fn test_appconfig_clone() {
    let config = AppConfig::default();
    let cloned = config.clone();
    assert_eq!(cloned.max_tabs, config.max_tabs);
    assert_eq!(cloned.cache_size_mb, config.cache_size_mb);
}

#[test]
fn test_appconfig_debug_impl() {
    let config = AppConfig::default();
    let debug_str = format!("{:?}", config);
    assert!(debug_str.contains("AppConfig"));
    assert!(debug_str.contains("max_tabs"));
}

#[test]
fn test_page_document_new() {
    let doc = PageDocument::new("https://example.com");
    assert_eq!(doc.url, "https://example.com");
}

#[test]
fn test_page_document_from_simple_html() {
    let html = r#"
        <html>
        <head><title>Test Page</title></head>
        <body>
            <h1>Hello World</h1>
            <p>This is a test page.</p>
        </body>
        </html>
    "#;
    let doc = PageDocument::from_html("https://test.com", html);
    assert_eq!(doc.url, "https://test.com");
}

#[test]
fn test_page_document_from_html_with_links() {
    let html = r#"
        <html><body>
            <a href="/page1">Link 1</a>
            <a href="https://external.com">External</a>
        </body></html>
    "#;
    let doc = PageDocument::from_html("https://test.com", html);
    assert_eq!(doc.url, "https://test.com");
}

#[test]
fn test_page_document_from_empty_html() {
    let html = "";
    let doc = PageDocument::from_html("https://empty.com", html);
    assert_eq!(doc.url, "https://empty.com");
}

#[test]
fn test_page_document_resolve_relative_href() {
    let doc = PageDocument::new("https://example.com/page/");
    let resolved = doc.resolve_href_simple("/other.html");
    assert!(resolved.contains("example.com"));
    assert!(resolved.contains("other.html"));
}

#[test]
fn test_page_document_resolve_absolute_href() {
    let doc = PageDocument::new("https://example.com/page/");
    let resolved = doc.resolve_href_simple("https://other.com/test");
    assert_eq!(resolved, "https://other.com/test");
}

#[test]
fn test_page_document_complex_html() {
    let html = r#"
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <title>Complex Test</title>
            <style>body { font-family: sans-serif; }</style>
        </head>
        <body>
            <header><h1>Welcome</h1></header>
            <nav>
                <ul>
                    <li><a href="/home">Home</a></li>
                    <li><a href="/about">About</a></li>
                </ul>
            </nav>
            <main>
                <article>
                    <h2>Article Title</h2>
                    <p>Paragraph 1</p>
                    <p>Paragraph 2</p>
                </article>
            </main>
            <footer>&copy; 2026</footer>
        </body>
        </html>
    "#;
    let doc = PageDocument::from_html("https://complex.com", html);
    assert_eq!(doc.url, "https://complex.com");
}

#[test]
fn test_page_document_html_with_scripts() {
    let html = r#"
        <html>
        <head><title>JS Test</title></head>
        <body>
            <h1>Before</h1>
            <script>var x = 42;</script>
            <h2>After</h2>
        </body>
        </html>
    "#;
    let doc = PageDocument::from_html("https://js.com", html);
    assert_eq!(doc.url, "https://js.com");
}

#[test]
fn test_interpreter_new() {
    let _interp = Interpreter::new();
}

#[test]
fn test_eval_script_simple_var() {
    let mut interp = Interpreter::new();
    let result = eval_script(&mut interp, 0, "var x = 5;");
    assert!(result.is_ok());
}

#[test]
fn test_eval_script_arithmetic() {
    let mut interp = Interpreter::new();
    let result = eval_script(&mut interp, 0, "1 + 2 * 3");
    assert!(result.is_ok());
}

#[test]
fn test_eval_script_string_literal() {
    let mut interp = Interpreter::new();
    let result = eval_script(&mut interp, 0, r#""hello world""#);
    assert!(result.is_ok());
}

#[test]
fn test_eval_script_function_declaration() {
    let mut interp = Interpreter::new();
    let result = eval_script(&mut interp, 0, "function add(a, b) { return a + b; }");
    assert!(result.is_ok());
}

#[test]
fn test_eval_script_if_statement() {
    let mut interp = Interpreter::new();
    let result = eval_script(&mut interp, 0, "if (true) { var x = 1; }");
    assert!(result.is_ok());
}

#[test]
fn test_eval_script_while_loop() {
    let mut interp = Interpreter::new();
    let result = eval_script(&mut interp, 0, "var i = 0; while (i < 10) { i = i + 1; }");
    assert!(result.is_ok());
}

#[test]
fn test_eval_script_multiple_statements() {
    let mut interp = Interpreter::new();
    let result = eval_script(
        &mut interp,
        0,
        "var a = 1; var b = 2; var c = a + b;"
    );
    assert!(result.is_ok());
}

#[test]
fn test_eval_script_nested_blocks() {
    let mut interp = Interpreter::new();
    let result = eval_script(&mut interp, 0, "if (true) { var x = 1; if (false) { var y = 2; } }");
    assert!(result.is_ok());
}

#[test]
fn test_eval_script_invalid_syntax_returns_err() {
    let mut interp = Interpreter::new();
    let result = eval_script(&mut interp, 0, "var = ;");
    assert!(result.is_err());
}

#[test]
fn test_create_browser_returns_ok() {
    let result = no_chromium::create_browser(AppConfig::default());
    assert!(result.is_ok());
}

#[test]
fn test_browser_instance_has_config() {
    let browser = no_chromium::create_browser(AppConfig::default()).unwrap();
    let config = browser.config();
    assert_eq!(config.max_tabs, 20);
}

#[test]
fn test_end_to_end_workflow() {
    let mut interp = Interpreter::new();
    let _html = PageDocument::from_html(
        "https://test.com",
        r#"<html><body><h1>Test</h1></body></html>"#,
    );
    let js_result = eval_script(&mut interp, 0, "var greeting = 'Hello'; var count = 42;");
    assert!(js_result.is_ok());
    let url = resolve_url("test.com");
    assert_eq!(url, "https://test.com");
}
