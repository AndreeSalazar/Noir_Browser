//! Find in Page - Buscar texto en página
//!
//! Búsqueda con highlighting y navegación entre matches.

use crate::parsers::page_document::PageDocument;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FindOptions {
    pub case_sensitive: bool,
    pub whole_word: bool,
    pub backwards: bool,
}

impl Default for FindOptions {
    fn default() -> Self {
        Self {
            case_sensitive: false,
            whole_word: false,
            backwards: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FindMatch {
    pub text: String,
    pub block_index: usize,
    pub start: usize,
    pub end: usize,
    pub context_before: String,
    pub context_after: String,
}

pub struct FindInPage;

impl FindInPage {
    /// Busca todas las ocurrencias en la página
    pub fn find_all(doc: &PageDocument, query: &str, options: FindOptions) -> Vec<FindMatch> {
        if query.is_empty() {
            return Vec::new();
        }

        let mut results = Vec::new();
        for (block_index, block) in doc.text_blocks.iter().enumerate() {
            Self::find_in_text(&block.text, block_index, query, &options, &mut results);
        }
        results
    }

    /// Cuenta matches
    pub fn count(doc: &PageDocument, query: &str, options: FindOptions) -> usize {
        Self::find_all(doc, query, options).len()
    }

    /// Encuentra el siguiente match después de posición
    pub fn find_next(
        doc: &PageDocument,
        query: &str,
        options: FindOptions,
        after_index: usize,
        after_pos: usize,
    ) -> Option<FindMatch> {
        let results = Self::find_all(doc, query, options);
        for result in &results {
            if result.block_index > after_index
                || (result.block_index == after_index && result.start > after_pos) {
                return Some(result.clone());
            }
        }
        results.into_iter().next()
    }

    fn find_in_text(
        text: &str,
        block_index: usize,
        query: &str,
        options: &FindOptions,
        results: &mut Vec<FindMatch>,
    ) {
        let haystack = if options.case_sensitive {
            text.to_string()
        } else {
            text.to_lowercase()
        };
        let needle = if options.case_sensitive {
            query.to_string()
        } else {
            query.to_lowercase()
        };

        let mut start = 0;
        while let Some(pos) = haystack[start..].find(&needle) {
            let absolute_pos = start + pos;

            // Check whole-word: both sides must be non-word (or boundary)
            if options.whole_word {
                let before_ok = absolute_pos == 0
                    || !is_word_char(haystack.as_bytes()[absolute_pos - 1]);
                let after_pos = absolute_pos + needle.len();
                let after_ok = after_pos >= haystack.len()
                    || !is_word_char(haystack.as_bytes()[after_pos]);
                if !(before_ok && after_ok) {
                    start = absolute_pos + 1;
                    continue;
                }
            }

            // Get context
            let context_start = absolute_pos.saturating_sub(30);
            let context_end = (absolute_pos + needle.len() + 30).min(text.len());
            let context_before = text[context_start..absolute_pos].to_string();
            let context_after = text[absolute_pos + needle.len()..context_end].to_string();

            results.push(FindMatch {
                text: query.to_string(),
                block_index,
                start: absolute_pos,
                end: absolute_pos + needle.len(),
                context_before,
                context_after,
            });

            start = absolute_pos + needle.len();
        }
    }
}

fn is_word_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_empty_query() {
        let doc = PageDocument::from_html("https://test.com", "<html><body><p>Hello</p></body></html>");
        let results = FindInPage::find_all(&doc, "", FindOptions::default());
        assert!(results.is_empty());
    }

    #[test]
    fn test_find_simple() {
        let html = r#"<html><body><p>Hello world hello</p></body></html>"#;
        let doc = PageDocument::from_html("https://test.com", html);
        let results = FindInPage::find_all(&doc, "hello", FindOptions::default());
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_find_case_sensitive() {
        let html = r#"<html><body><p>Hello HELLO hello</p></body></html>"#;
        let doc = PageDocument::from_html("https://test.com", html);

        let insensitive = FindInPage::find_all(&doc, "hello", FindOptions::default());
        assert_eq!(insensitive.len(), 3);

        let sensitive = FindInPage::find_all(&doc, "hello", FindOptions { case_sensitive: true, whole_word: false, backwards: false });
        assert_eq!(sensitive.len(), 1);
    }

    #[test]
    fn test_find_whole_word() {
        // Only "hello" at the beginning is a whole word (others are concatenated)
        let html = r#"<html><body><p>hello hellohello helloworld</p></body></html>"#;
        let doc = PageDocument::from_html("https://test.com", html);
        let whole = FindInPage::find_all(&doc, "hello", FindOptions { whole_word: true, case_sensitive: false, backwards: false });
        assert_eq!(whole.len(), 1); // only first "hello" is whole word
    }

    #[test]
    fn test_find_in_multiple_blocks() {
        let html = r#"
            <html><body>
                <h1>First</h1>
                <p>Block one with target</p>
                <h2>Second</h2>
                <p>Block two with target</p>
            </body></html>
        "#;
        let doc = PageDocument::from_html("https://test.com", html);
        let results = FindInPage::find_all(&doc, "target", FindOptions::default());
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_find_count() {
        let html = r#"<html><body><p>foo bar foo baz foo</p></body></html>"#;
        let doc = PageDocument::from_html("https://test.com", html);
        let count = FindInPage::count(&doc, "foo", FindOptions::default());
        assert_eq!(count, 3);
    }

    #[test]
    fn test_find_next() {
        let html = r#"<html><body><p>foo foo foo</p></body></html>"#;
        let doc = PageDocument::from_html("https://test.com", html);
        let results = FindInPage::find_all(&doc, "foo", FindOptions::default());
        assert!(results.len() >= 3);

        // Find next after first match (position 0, pos 0)
        let next = FindInPage::find_next(&doc, "foo", FindOptions::default(), 0, 0);
        assert!(next.is_some());
    }

    #[test]
    fn test_find_context() {
        let html = r#"<html><body><p>This is some text with a needle in the middle of it</p></body></html>"#;
        let doc = PageDocument::from_html("https://test.com", html);
        let results = FindInPage::find_all(&doc, "needle", FindOptions::default());
        assert_eq!(results.len(), 1);
        let m = &results[0];
        assert!(m.context_before.contains("text with a "));
        assert!(m.context_after.contains(" in the middle"));
    }
}
