//! Tests para el bug crítico: JS inline no debe renderizarse como texto

use no_chromium::parsers::page_document::PageDocument;

#[test]
fn test_script_content_not_rendered() {
    let html = r#"
        <html><body>
            <h1>Title</h1>
            <script>var x = "secret_code_xyz"; console.log(x);</script>
            <p>Visible text</p>
        </body></html>
    "#;
    let doc = PageDocument::from_html("https://test.com", html);
    let has_script_text = doc.text_blocks.iter()
        .any(|b| b.text.contains("secret_code_xyz") || b.text.contains("var x") || b.text.contains("console.log"));
    assert!(!has_script_text, "Script content should NOT appear in text_blocks");
}

#[test]
fn test_style_content_not_rendered() {
    let html = r#"
        <html><body>
            <h1>Title</h1>
            <style>body { color: red; } .hidden_secret { display: none; }</style>
            <p>Visible text</p>
        </body></html>
    "#;
    let doc = PageDocument::from_html("https://test.com", html);
    let has_css = doc.text_blocks.iter()
        .any(|b| b.text.contains("color: red") || b.text.contains(".hidden_secret"));
    assert!(!has_css, "Style content should NOT appear in text_blocks");
}

#[test]
fn test_noscript_content_not_rendered() {
    let html = r#"
        <html><body>
            <noscript>Please enable JavaScript</noscript>
            <p>Real content</p>
        </body></html>
    "#;
    let doc = PageDocument::from_html("https://test.com", html);
    let has_noscript = doc.text_blocks.iter()
        .any(|b| b.text.contains("Please enable JavaScript"));
    assert!(!has_noscript, "Noscript content should NOT appear in text_blocks");
}

#[test]
fn test_normal_text_still_rendered() {
    let html = r#"
        <html><body>
            <h1>Title</h1>
            <p>Visible text</p>
            <div>Div content</div>
        </body></html>
    "#;
    let doc = PageDocument::from_html("https://test.com", html);
    let has_title = doc.text_blocks.iter().any(|b| b.text == "Title");
    let has_p = doc.text_blocks.iter().any(|b| b.text == "Visible text");
    let has_div = doc.text_blocks.iter().any(|b| b.text == "Div content");
    assert!(has_title);
    assert!(has_p);
    assert!(has_div);
}

#[test]
fn test_youtube_typical_html() {
    let html = r#"
        <html><body>
            <div class="search_query">[]</div>
            <a href="https://www.youtube.com/about/">Acerca de</a>
            <a href="https://www.youtube.com/about/press/">Prensa</a>
            <script>if (window.ytcfg) { ytcfg.set(...) }</script>
            <div class="ytcd-ghostbox"></div>
        </body></html>
    "#;
    let doc = PageDocument::from_html("https://youtube.com", html);
    // Should have the links and div text
    let has_search = doc.text_blocks.iter().any(|b| b.text.contains("[]"));
    let has_about = doc.links.iter().any(|l| l.text == "Acerca de");
    let has_press = doc.links.iter().any(|l| l.text == "Prensa");
    // Should NOT have the script content
    let has_script = doc.text_blocks.iter().any(|b| b.text.contains("ytcfg"));
    assert!(has_search);
    assert!(has_about);
    assert!(has_press);
    assert!(!has_script);
}

#[test]
fn test_inline_json_in_script_not_rendered() {
    let html = r#"
        <html><body>
            <script type="application/ld+json">
            {"@context":"https://schema.org","@type":"WebPage"}
            </script>
            <h1>Title</h1>
        </body></html>
    "#;
    let doc = PageDocument::from_html("https://test.com", html);
    let has_json = doc.text_blocks.iter().any(|b| b.text.contains("schema.org"));
    assert!(!has_json, "JSON-LD should NOT appear in text_blocks");
}
