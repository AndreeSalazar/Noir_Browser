//! Tests para los placeholders JS engine que arreglamos

use no_chromium::js_engine_v3::{
    extract_inline_scripts, take_mutated_flag, set_mutated_flag,
    push_console, take_console_messages,
};
use no_chromium::parsers::dom_tree::parse_html;
use no_chromium::js_engine_v3::rebuild_page_from_dom;
use no_chromium::parsers::page_document::PageDocument;

#[test]
fn test_extract_inline_scripts_basic() {
    let html = r#"<html><body>
        <h1>Title</h1>
        <script>var x = 5;</script>
        <p>Text</p>
    </body></html>"#;
    let nodes = parse_html(html);
    let scripts = extract_inline_scripts(&nodes);
    assert_eq!(scripts.len(), 1);
    assert!(scripts[0].contains("var x = 5"));
}

#[test]
fn test_extract_multiple_scripts() {
    let html = r#"<html><body>
        <script>var a = 1;</script>
        <p>Middle</p>
        <script>var b = 2;</script>
        <script>var c = 3;</script>
    </body></html>"#;
    let nodes = parse_html(html);
    let scripts = extract_inline_scripts(&nodes);
    assert_eq!(scripts.len(), 3);
    assert!(scripts[0].contains("var a = 1"));
    assert!(scripts[1].contains("var b = 2"));
    assert!(scripts[2].contains("var c = 3"));
}

#[test]
fn test_extract_no_scripts() {
    let html = r#"<html><body><h1>No scripts here</h1></body></html>"#;
    let nodes = parse_html(html);
    let scripts = extract_inline_scripts(&nodes);
    assert!(scripts.is_empty());
}

#[test]
fn test_extract_script_with_multiline_code() {
    let html = r#"<html><body>
        <script>
            function foo() {
                return 42;
            }
        </script>
    </body></html>"#;
    let nodes = parse_html(html);
    let scripts = extract_inline_scripts(&nodes);
    assert_eq!(scripts.len(), 1);
    assert!(scripts[0].contains("function foo"));
    assert!(scripts[0].contains("return 42"));
}

#[test]
fn test_take_mutated_flag_initial_false() {
    // No mutations yet
    let _ = take_mutated_flag(); // Reset
    let mutated = take_mutated_flag();
    assert!(!mutated);
}

#[test]
fn test_set_and_take_mutated_flag() {
    set_mutated_flag();
    let mutated = take_mutated_flag();
    assert!(mutated);
    // Second take should be false
    let mutated2 = take_mutated_flag();
    assert!(!mutated2);
}

#[test]
fn test_console_push_and_take() {
    push_console("log", "Hello world");
    push_console("error", "Something failed");
    let messages = take_console_messages();
    assert!(messages.len() >= 2);
    let has_log = messages.iter().any(|(level, text)| level == "log" && text == "Hello world");
    let has_error = messages.iter().any(|(level, text)| level == "error" && text == "Something failed");
    assert!(has_log);
    assert!(has_error);
}

#[test]
fn test_console_take_empties_buffer() {
    // Pre-clear buffer by taking everything
    let _ = take_console_messages();
    let unique = "TakeEmptiesTest_a8b9c0";
    push_console("log", unique);
    let first = take_console_messages();
    let found = first.iter().any(|(_, text)| text.contains(unique));
    assert!(found, "First take should find our message: got {:?}", first);
    // Second take: may have other tests' messages, but our unique one should be gone
    let second = take_console_messages();
    let found_again = second.iter().any(|(_, text)| text.contains(unique));
    assert!(!found_again, "Our unique message should be consumed: got {:?}", second);
}

#[test]
fn test_rebuild_page_from_dom_basic() {
    let html = r#"<html><body>
        <h1>Rebuild Test</h1>
        <p>Some text here</p>
    </body></html>"#;
    let mut page = PageDocument::from_html("https://test.com", html);
    let initial_blocks = page.text_blocks.len();
    assert!(initial_blocks >= 2);

    // Rebuild from DOM
    rebuild_page_from_dom(&mut page);
    // Should still have blocks
    assert!(!page.text_blocks.is_empty());
}

#[test]
fn test_extract_script_in_nested_div() {
    let html = r#"<html><body>
        <div>
            <div>
                <span>
                    <script>var nested = true;</script>
                </span>
            </div>
        </div>
    </body></html>"#;
    let nodes = parse_html(html);
    let scripts = extract_inline_scripts(&nodes);
    assert!(!scripts.is_empty());
    assert!(scripts[0].contains("var nested"));
}

#[test]
fn test_console_max_messages() {
    use no_chromium::js_engine_v3::push_console;
    for i in 0..1500 {
        push_console("log", &format!("Message {}", i));
    }
    let messages = take_console_messages();
    // Should be capped at 1000
    assert!(messages.len() <= 1000);
}

#[test]
fn test_rebuild_preserves_url() {
    let html = r#"<html><body><h1>Title</h1></body></html>"#;
    let mut page = PageDocument::from_html("https://example.com", html);
    rebuild_page_from_dom(&mut page);
    assert_eq!(page.url, "https://example.com");
}
