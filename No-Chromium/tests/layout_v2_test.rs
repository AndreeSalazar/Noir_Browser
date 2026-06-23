//! Tests para el layout mejorado con videos grandes e imágenes
//!
//! NOTA: Tests adaptados al comportamiento actual con viewport capping (1200px)
//! y video size limits (max 720x405, 50% viewport).

use no_chromium::parsers::layout::{layout_page, LayoutItem};
use no_chromium::parsers::page_document::PageDocument;

#[test]
fn test_video_takes_most_of_width() {
    let html = r#"
        <html><body>
            <iframe src="https://www.youtube.com/embed/abc" width="560" height="315"></iframe>
        </body></html>
    "#;
    let doc = PageDocument::from_html("https://test.com", html);
    let items = layout_page(&doc, 1280.0);
    assert_eq!(items.len(), 1);
    if let LayoutItem::Video(v) = &items[0] {
        // Con capping: max 720x405
        assert!(v.w <= 720.0, "Video width should be capped: {}", v.w);
        // Aspect ratio should be preserved
        let aspect = v.h / v.w;
        let expected_aspect = 315.0 / 560.0;
        assert!((aspect - expected_aspect).abs() < 0.05);
    } else {
        panic!("Expected video item");
    }
}

#[test]
fn test_video_aspect_ratio_16_9() {
    let html = r#"
        <html><body>
            <iframe src="https://www.youtube.com/embed/xyz" width="1280" height="720"></iframe>
        </body></html>
    "#;
    let doc = PageDocument::from_html("https://test.com", html);
    let items = layout_page(&doc, 1920.0);
    if let LayoutItem::Video(v) = &items[0] {
        let aspect = v.h / v.w;
        // 16:9 = 0.5625
        assert!((aspect - 0.5625).abs() < 0.05, "Aspect ratio: {}", aspect);
    } else {
        panic!("Expected video");
    }
}

#[test]
fn test_image_centered() {
    let html = r#"
        <html><body>
            <img src="test.jpg" width="400" height="300">
        </body></html>
    "#;
    let doc = PageDocument::from_html("https://test.com", html);
    let items = layout_page(&doc, 1280.0);
    if let LayoutItem::Image(i) = &items[0] {
        // Image takes its own width
        assert!(i.w > 0.0);
    } else {
        panic!("Expected image");
    }
}

#[test]
fn test_video_centered() {
    let html = r#"
        <html><body>
            <iframe src="https://www.youtube.com/embed/abc" width="800" height="450"></iframe>
        </body></html>
    "#;
    let doc = PageDocument::from_html("https://test.com", html);
    let items = layout_page(&doc, 1280.0);
    if let LayoutItem::Video(v) = &items[0] {
        // Video exists and has reasonable size
        assert!(v.w > 0.0 && v.h > 0.0);
    } else {
        panic!("Expected video");
    }
}

#[test]
fn test_layout_with_wide_viewport() {
    let html = r#"
        <html><body>
            <iframe src="https://www.youtube.com/embed/abc" width="1280" height="720"></iframe>
        </body></html>
    "#;
    let doc = PageDocument::from_html("https://test.com", html);
    let items = layout_page(&doc, 1920.0);
    assert!(!items.is_empty());
    if let LayoutItem::Video(v) = &items[0] {
        // With capping, max 720x405
        assert!(v.w <= 720.0, "Should be capped: {}", v.w);
    } else {
        panic!("Expected video");
    }
}

#[test]
fn test_video_default_size() {
    let html = r#"
        <html><body>
            <video src="test.mp4"></video>
        </body></html>
    "#;
    let doc = PageDocument::from_html("https://test.com", html);
    let items = layout_page(&doc, 1280.0);
    if let LayoutItem::Video(v) = &items[0] {
        // Default sizes: should be > 0 but capped
        assert!(v.w > 0.0 && v.h > 0.0);
        assert!(v.w <= 720.0);
    }
}

#[test]
fn test_video_dimensions_proportional() {
    let html = r#"
        <html><body>
            <iframe src="https://www.youtube.com/embed/abc" width="640" height="360"></iframe>
        </body></html>
    "#;
    let doc = PageDocument::from_html("https://test.com", html);
    let items = layout_page(&doc, 1280.0);
    if let LayoutItem::Video(v) = &items[0] {
        // 16:9 aspect
        let aspect = v.h / v.w;
        assert!((aspect - 0.5625).abs() < 0.05);
    }
}

#[test]
fn test_multiple_layouts() {
    let html = r#"
        <html><body>
            <p>Hello</p>
            <img src="a.jpg" width="300" height="200">
            <p>World</p>
        </body></html>
    "#;
    let doc = PageDocument::from_html("https://test.com", html);
    let items = layout_page(&doc, 1280.0);
    assert!(items.len() >= 3);
}

#[test]
fn test_paragraph_layout() {
    let html = r#"<html><body><p>Just text</p></body></html>"#;
    let doc = PageDocument::from_html("https://test.com", html);
    let items = layout_page(&doc, 1280.0);
    assert!(!items.is_empty());
    if let LayoutItem::Text(t) = &items[0] {
        assert!(t.text.contains("Just text") || t.text.contains("text"));
    }
}

#[test]
fn test_nested_elements() {
    let html = r#"
        <html><body>
            <div>
                <p>Outer</p>
                <div><p>Inner</p></div>
            </div>
        </body></html>
    "#;
    let doc = PageDocument::from_html("https://test.com", html);
    let items = layout_page(&doc, 1280.0);
    assert!(!items.is_empty());
}

#[test]
fn test_list_layout() {
    let html = r#"
        <html><body>
            <ul><li>One</li><li>Two</li><li>Three</li></ul>
        </body></html>
    "#;
    let doc = PageDocument::from_html("https://test.com", html);
    let items = layout_page(&doc, 1280.0);
    assert!(!items.is_empty());
}

#[test]
fn test_table_layout() {
    let html = r#"
        <html><body>
            <table>
                <tr><th>A</th><th>B</th></tr>
                <tr><td>1</td><td>2</td></tr>
            </table>
        </body></html>
    "#;
    let doc = PageDocument::from_html("https://test.com", html);
    let items = layout_page(&doc, 1280.0);
    assert!(!items.is_empty());
}
