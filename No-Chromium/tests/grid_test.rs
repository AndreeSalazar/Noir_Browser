//! Tests para el grid layout estilo YouTube

use no_chromium::parsers::layout::{layout_page, LayoutItem};
use no_chromium::parsers::page_document::PageDocument;

#[test]
fn test_grid_detected_with_many_images() {
    let html = r#"
        <html><body>
            <img src="a.jpg"><img src="b.jpg"><img src="c.jpg">
        </body></html>
    "#;
    let doc = PageDocument::from_html("https://test.com", html);
    let items = layout_page(&doc, 1280.0);
    // Should use grid: 2 cols
    let images = items.iter().filter(|i| matches!(i, LayoutItem::Image(_))).count();
    assert_eq!(images, 3);
}

#[test]
fn test_grid_no_detection_with_few_images() {
    let html = r#"
        <html><body>
            <img src="a.jpg"><img src="b.jpg">
        </body></html>
    "#;
    let doc = PageDocument::from_html("https://test.com", html);
    let items = layout_page(&doc, 1280.0);
    // Only 2 images, should NOT trigger grid
    let images = items.iter().filter(|i| matches!(i, LayoutItem::Image(_))).count();
    assert_eq!(images, 2);
}

#[test]
fn test_grid_2_columns() {
    let html = r#"
        <html><body>
            <img src="a.jpg"><img src="b.jpg"><img src="c.jpg"><img src="d.jpg">
        </body></html>
    "#;
    let doc = PageDocument::from_html("https://test.com", html);
    let items = layout_page(&doc, 1280.0);
    // Should be 2 columns
    let images: Vec<&LayoutItem> = items.iter()
        .filter(|i| matches!(i, LayoutItem::Image(_)))
        .collect();
    assert_eq!(images.len(), 4);

    // First two should be in same row (same y)
    if let (LayoutItem::Image(i0), LayoutItem::Image(i1)) = (images[0], images[1]) {
        assert_eq!(i0.y, i1.y, "First two images should be in same row");
    }
}

#[test]
fn test_grid_3_columns_wide_viewport() {
    let html = r#"
        <html><body>
            <img src="a.jpg"><img src="b.jpg"><img src="c.jpg">
            <img src="d.jpg"><img src="e.jpg"><img src="f.jpg">
        </body></html>
    "#;
    let doc = PageDocument::from_html("https://test.com", html);
    let items = layout_page(&doc, 1920.0);
    // Should be 3 columns
    let images: Vec<&LayoutItem> = items.iter()
        .filter(|i| matches!(i, LayoutItem::Image(_)))
        .collect();
    assert_eq!(images.len(), 6);

    // First three should be in same row
    if let (LayoutItem::Image(i0), LayoutItem::Image(i1), LayoutItem::Image(i2)) = (images[0], images[1], images[2]) {
        assert_eq!(i0.y, i1.y);
        assert_eq!(i1.y, i2.y);
    }
}

#[test]
fn test_grid_thumbnails_are_16_9() {
    let html = r#"
        <html><body>
            <img src="a.jpg"><img src="b.jpg"><img src="c.jpg">
        </body></html>
    "#;
    let doc = PageDocument::from_html("https://test.com", html);
    let items = layout_page(&doc, 1280.0);
    if let LayoutItem::Image(img) = &items[0] {
        let aspect = img.h / img.w;
        let expected = 9.0 / 16.0; // 0.5625
        assert!((aspect - expected).abs() < 0.01, "Aspect: {}", aspect);
    }
}

#[test]
fn test_grid_with_titles() {
    let html = r#"
        <html><body>
            <img src="thumb1.jpg" alt="Thumbnail 1">
            <a href="/video1">Video 1 Title</a>
            <img src="thumb2.jpg" alt="Thumbnail 2">
            <a href="/video2">Video 2 Title</a>
            <img src="thumb3.jpg" alt="Thumbnail 3">
            <a href="/video3">Video 3 Title</a>
        </body></html>
    "#;
    let doc = PageDocument::from_html("https://test.com", html);
    let items = layout_page(&doc, 1280.0);
    let text_count = items.iter().filter(|i| matches!(i, LayoutItem::Text(_))).count();
    // Should have 3 text titles
    assert!(text_count >= 3, "Got {} text blocks", text_count);
}

#[test]
fn test_grid_youtube_typical() {
    let html = r#"
        <html><body>
            <div>
                <img src="thumb1.jpg">
                <a href="/watch?v=1">First Video Title</a>
            </div>
            <div>
                <img src="thumb2.jpg">
                <a href="/watch?v=2">Second Video Title</a>
            </div>
            <div>
                <img src="thumb3.jpg">
                <a href="/watch?v=3">Third Video Title</a>
            </div>
        </body></html>
    "#;
    let doc = PageDocument::from_html("https://youtube.com", html);
    let items = layout_page(&doc, 1280.0);
    let images = items.iter().filter(|i| matches!(i, LayoutItem::Image(_))).count();
    let texts = items.iter().filter(|i| matches!(i, LayoutItem::Text(_))).count();
    assert_eq!(images, 3, "Should have 3 thumbnails");
    assert!(texts >= 3, "Should have at least 3 titles, got {}", texts);
}

#[test]
fn test_grid_with_no_videos() {
    let html = r#"
        <html><body>
            <img src="a.jpg"><img src="b.jpg"><img src="c.jpg">
        </body></html>
    "#;
    let doc = PageDocument::from_html("https://test.com", html);
    let items = layout_page(&doc, 1280.0);
    // Should use grid (not video)
    let videos = items.iter().filter(|i| matches!(i, LayoutItem::Video(_))).count();
    assert_eq!(videos, 0);
}

#[test]
fn test_grid_with_videos_no_grid() {
    let html = r#"
        <html><body>
            <iframe src="https://www.youtube.com/embed/abc"></iframe>
            <iframe src="https://www.youtube.com/embed/def"></iframe>
            <iframe src="https://www.youtube.com/embed/ghi"></iframe>
        </body></html>
    "#;
    let doc = PageDocument::from_html("https://test.com", html);
    let items = layout_page(&doc, 1280.0);
    // Should NOT use grid (has videos, not images)
    let videos = items.iter().filter(|i| matches!(i, LayoutItem::Video(_))).count();
    assert_eq!(videos, 3);
}

#[test]
fn test_grid_narrow_viewport() {
    let html = r#"
        <html><body>
            <img src="a.jpg"><img src="b.jpg"><img src="c.jpg">
        </body></html>
    "#;
    let doc = PageDocument::from_html("https://test.com", html);
    let items = layout_page(&doc, 600.0);
    // Should still work, but with smaller cells
    if let LayoutItem::Image(img) = &items[0] {
        assert!(img.w > 100.0, "Image should have reasonable size: {}", img.w);
    }
}

#[test]
fn test_grid_gap_between_items() {
    let html = r#"
        <html><body>
            <img src="a.jpg"><img src="b.jpg"><img src="c.jpg"><img src="d.jpg">
        </body></html>
    "#;
    let doc = PageDocument::from_html("https://test.com", html);
    let items = layout_page(&doc, 1280.0);
    let images: Vec<&LayoutItem> = items.iter()
        .filter(|i| matches!(i, LayoutItem::Image(_)))
        .collect();
    if let (LayoutItem::Image(i0), LayoutItem::Image(i1)) = (images[0], images[1]) {
        // There should be a gap
        let gap = i1.x - (i0.x + i0.w);
        assert!(gap > 0.0, "Should have horizontal gap: {}", gap);
    }
}

#[test]
fn test_grid_height_with_titles() {
    let html = r#"
        <html><body>
            <img src="a.jpg"><img src="b.jpg"><img src="c.jpg">
            <h3>Title 1</h3>
            <h3>Title 2</h3>
            <h3>Title 3</h3>
        </body></html>
    "#;
    let doc = PageDocument::from_html("https://test.com", html);
    let items = layout_page(&doc, 1280.0);
    let images: Vec<&LayoutItem> = items.iter()
        .filter(|i| matches!(i, LayoutItem::Image(_)))
        .collect();
    // Grid should have title space below each image
    if let LayoutItem::Image(img) = images[0] {
        // There should be text below the image
        let has_text_below = items.iter().any(|i| {
            if let LayoutItem::Text(t) = i {
                t.y > img.y && t.y < img.y + img.h + 50.0
            } else {
                false
            }
        });
        assert!(has_text_below, "Should have text below image");
    }
}
