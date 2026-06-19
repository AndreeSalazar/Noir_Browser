//! Tests para el layout mejorado con videos grandes e imágenes

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
        // Should be close to content_w (1280 - 80 = 1200)
        assert!(v.w > 1000.0, "Video width should be large: {}", v.w);
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
        // Should be centered in content area
        let content_center = 40.0 + (1200.0 / 2.0);
        let image_center = i.x + (i.w / 2.0);
        assert!((content_center - image_center).abs() < 5.0);
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
        // Should be roughly centered
        let content_center_x = 40.0 + 600.0;
        let video_center_x = v.x + (v.w / 2.0);
        assert!((content_center_x - video_center_x).abs() < 100.0);
    } else {
        panic!("Expected video");
    }
}

#[test]
fn test_video_dimensions_proportional() {
    let html = r#"
        <html><body>
            <iframe src="https://www.youtube.com/embed/abc" width="500" height="500"></iframe>
        </body></html>
    "#;
    let doc = PageDocument::from_html("https://test.com", html);
    let items = layout_page(&doc, 800.0);
    if let LayoutItem::Video(v) = &items[0] {
        // 1:1 aspect should be preserved
        let aspect = v.h / v.w;
        assert!((aspect - 1.0).abs() < 0.1);
    } else {
        panic!("Expected video");
    }
}

#[test]
fn test_video_in_youtube_typical() {
    let html = r#"
        <html><body>
            <ytd-video-renderer>
                <div id="thumbnail">
                    <img src="thumb1.jpg">
                </div>
                <div id="title-wrapper">
                    <h3>Video Title 1</h3>
                </div>
            </ytd-video-renderer>
            <ytd-video-renderer>
                <div id="thumbnail">
                    <img src="thumb2.jpg">
                </div>
                <div id="title-wrapper">
                    <h3>Video Title 2</h3>
                </div>
            </ytd-video-renderer>
        </body></html>
    "#;
    let doc = PageDocument::from_html("https://youtube.com", html);
    let items = layout_page(&doc, 1280.0);
    // Should have 2 images and 2 headings
    let images = items.iter().filter(|i| matches!(i, LayoutItem::Image(_))).count();
    let headings = items.iter().filter(|i| matches!(i, LayoutItem::Text(_))).count();
    assert_eq!(images, 2);
    assert!(headings >= 2);
}

#[test]
fn test_layout_with_narrow_viewport() {
    let html = r#"
        <html><body>
            <iframe src="https://www.youtube.com/embed/abc" width="800" height="450"></iframe>
        </body></html>
    "#;
    let doc = PageDocument::from_html("https://test.com", html);
    let items = layout_page(&doc, 600.0);
    if let LayoutItem::Video(v) = &items[0] {
        // Should not exceed content width
        assert!(v.w <= 520.0, "Video too wide: {}", v.w);
    }
}

#[test]
fn test_layout_with_wide_viewport() {
    let html = r#"
        <html><body>
            <iframe src="https://www.youtube.com/embed/abc" width="800" height="450"></iframe>
        </body></html>
    "#;
    let doc = PageDocument::from_html("https://test.com", html);
    let items = layout_page(&doc, 1920.0);
    if let LayoutItem::Video(v) = &items[0] {
        // Should be large
        assert!(v.w > 1500.0, "Video should be wide: {}", v.w);
    }
}

#[test]
fn test_multiple_videos() {
    let html = r#"
        <html><body>
            <iframe src="https://www.youtube.com/embed/a"></iframe>
            <iframe src="https://www.youtube.com/embed/b"></iframe>
            <iframe src="https://www.youtube.com/embed/c"></iframe>
        </body></html>
    "#;
    let doc = PageDocument::from_html("https://test.com", html);
    let items = layout_page(&doc, 1280.0);
    let videos = items.iter().filter(|i| matches!(i, LayoutItem::Video(_))).count();
    assert_eq!(videos, 3);
}

#[test]
fn test_mixed_content() {
    let html = r#"
        <html><body>
            <h1>Title</h1>
            <iframe src="https://www.youtube.com/embed/abc"></iframe>
            <p>Description</p>
            <img src="thumb.jpg">
            <iframe src="https://www.youtube.com/embed/xyz"></iframe>
        </body></html>
    "#;
    let doc = PageDocument::from_html("https://test.com", html);
    let items = layout_page(&doc, 1280.0);
    let text_count = items.iter().filter(|i| matches!(i, LayoutItem::Text(_))).count();
    let img_count = items.iter().filter(|i| matches!(i, LayoutItem::Image(_))).count();
    let vid_count = items.iter().filter(|i| matches!(i, LayoutItem::Video(_))).count();
    assert!(text_count >= 2, "Text blocks: {}", text_count);
    assert_eq!(img_count, 1);
    assert_eq!(vid_count, 2);
}

#[test]
fn test_video_default_size() {
    let html = r#"
        <html><body>
            <iframe src="https://www.youtube.com/embed/abc"></iframe>
        </body></html>
    "#;
    let doc = PageDocument::from_html("https://test.com", html);
    let items = layout_page(&doc, 1280.0);
    if let LayoutItem::Video(v) = &items[0] {
        // Default should be 800x450
        assert!(v.w >= 800.0);
    }
}

#[test]
fn test_image_default_size() {
    let html = r#"
        <html><body>
            <img src="test.jpg">
        </body></html>
    "#;
    let doc = PageDocument::from_html("https://test.com", html);
    let items = layout_page(&doc, 1280.0);
    if let LayoutItem::Image(i) = &items[0] {
        // Default should be reasonable
        assert!(i.w > 100.0);
        assert!(i.h > 50.0);
    }
}

#[test]
fn test_layout_bounds_check() {
    let html = "<html><body></body></html>";
    let doc = PageDocument::from_html("https://test.com", html);
    let items = layout_page(&doc, 1280.0);
    assert!(items.is_empty());
}

#[test]
fn test_layout_with_only_text() {
    let html = r#"
        <html><body>
            <h1>Title</h1>
            <p>Paragraph 1</p>
            <p>Paragraph 2</p>
        </body></html>
    "#;
    let doc = PageDocument::from_html("https://test.com", html);
    let items = layout_page(&doc, 1280.0);
    let text_count = items.iter().filter(|i| matches!(i, LayoutItem::Text(_))).count();
    assert_eq!(text_count, 3);
}
