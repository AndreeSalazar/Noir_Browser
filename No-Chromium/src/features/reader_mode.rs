//! Reader Mode - Vista limpia sin distracciones
//!
//! Detecta el contenido principal y lo muestra sin ads/nav/footer.

use crate::parsers::page_document::PageDocument;

pub struct ReaderMode;

#[derive(Debug, Clone)]
pub struct ReaderContent {
    pub title: String,
    pub paragraphs: Vec<String>,
    pub word_count: usize,
    pub estimated_reading_time_minutes: u32,
}

impl ReaderMode {
    /// Extrae el contenido principal de una página
    pub fn extract(doc: &PageDocument) -> ReaderContent {
        let title = doc.title.clone();
        let paragraphs: Vec<String> = doc.text_blocks.iter()
            .filter(|b| {
                // Solo bloques grandes, no links
                b.text.len() > 30 && b.link.is_none()
            })
            .map(|b| b.text.clone())
            .collect();

        let word_count: usize = paragraphs.iter()
            .map(|p| p.split_whitespace().count())
            .sum();

        let estimated_reading_time_minutes = (word_count / 200) as u32;

        ReaderContent {
            title,
            paragraphs,
            word_count,
            estimated_reading_time_minutes,
        }
    }

    /// Verifica si una página es apta para reader mode
    pub fn is_suitable(doc: &PageDocument) -> bool {
        let total_words: usize = doc.text_blocks.iter()
            .map(|b| b.text.split_whitespace().count())
            .sum();
        total_words >= 100
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reader_mode_extraction() {
        let html = r#"
            <html>
                <head><title>Article Title</title></head>
                <body>
                    <p>This is a long paragraph that should be included in reader mode because it has more than thirty characters of text content.</p>
                    <p>This is another long paragraph with substantial content for the reader mode extraction algorithm to detect properly.</p>
                    <a href="/foo">Link</a>
                </body>
            </html>
        "#;
        let doc = PageDocument::from_html("https://test.com", html);
        let reader = ReaderMode::extract(&doc);
        assert_eq!(reader.title, "Article Title");
        assert!(reader.paragraphs.len() >= 2);
    }

    #[test]
    fn test_reader_word_count() {
        let html = r#"
            <html><body>
                <h1>Title</h1>
                <p>One two three four five six seven eight nine ten eleven twelve thirteen fourteen fifteen.</p>
            </body></html>
        "#;
        let doc = PageDocument::from_html("https://test.com", html);
        let reader = ReaderMode::extract(&doc);
        assert!(reader.word_count >= 15);
    }

    #[test]
    fn test_reader_reading_time() {
        let html = r#"
            <html><body>
                <p>{}</p>
            </body></html>
        "#;
        let long_para = "word ".repeat(500);
        let html = html.replace("{}", &long_para);
        let doc = PageDocument::from_html("https://test.com", &html);
        let reader = ReaderMode::extract(&doc);
        assert!(reader.estimated_reading_time_minutes >= 2);
    }

    #[test]
    fn test_is_suitable_short() {
        let html = r#"<html><body><h1>Short</h1></body></html>"#;
        let doc = PageDocument::from_html("https://test.com", html);
        assert!(!ReaderMode::is_suitable(&doc));
    }

    #[test]
    fn test_is_suitable_long() {
        let long_text = "word ".repeat(150);
        let html = format!("<html><body><p>{}</p></body></html>", long_text);
        let doc = PageDocument::from_html("https://test.com", &html);
        assert!(ReaderMode::is_suitable(&doc));
    }

    #[test]
    fn test_reader_excludes_links() {
        let html = r#"
            <html><body>
                <p>This is a long paragraph that should be included in reader mode because it has more than thirty characters of text content.</p>
                <a href="/foo">This is a link with text but should be excluded from reader mode entirely</a>
            </body></html>
        "#;
        let doc = PageDocument::from_html("https://test.com", html);
        let reader = ReaderMode::extract(&doc);
        // Link should be excluded
        let has_link = reader.paragraphs.iter().any(|p| p.contains("link with text"));
        assert!(!has_link);
    }
}
