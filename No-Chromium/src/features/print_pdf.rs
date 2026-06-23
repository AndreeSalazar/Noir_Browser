//! Print to PDF - Guardar página como PDF
//!
//! Genera un PDF simple con el texto de la página.

use crate::parsers::page_document::PageDocument;

pub struct PrintPdf;

impl PrintPdf {
    /// Genera un PDF simple con el contenido de la página
    pub fn generate(doc: &PageDocument) -> Vec<u8> {
        let mut pdf = Vec::new();

        // PDF header
        pdf.extend_from_slice(b"%PDF-1.4\n");
        pdf.extend_from_slice(b"%\xE2\xE3\xCF\xD3\n");

        // Objects
        let mut objects: Vec<Vec<u8>> = Vec::new();

        // Object 1: Catalog
        objects.push(b"<< /Type /Catalog /Pages 2 0 R >>".to_vec());

        // Object 2: Pages
        objects.push(format!("<< /Type /Pages /Kids [3 0 R] /Count {} >>",
            (doc.text_blocks.len() / 30 + 1).max(1)).into_bytes());

        // Object 3+: Page + Content streams
        let mut page_refs = Vec::new();
        for chunk in doc.text_blocks.chunks(30) {
            let page_idx = objects.len() + 1;
            page_refs.push(format!("{} 0 R", page_idx));

            // Content stream
            let mut content = String::new();
            content.push_str("BT\n/F1 12 Tf\n50 750 Td\n");
            for block in chunk {
                let escaped = pdf_escape(&block.text);
                content.push_str(&format!("({}) Tj\n0 -16 Td\n", escaped));
            }
            content.push_str("ET\n");
            let content_bytes = content.into_bytes();

            let content_idx = objects.len() + 1;
            let stream_obj = format!(
                "<< /Length {} >>\nstream\n{}\nendstream",
                content_bytes.len(),
                String::from_utf8_lossy(&content_bytes)
            );
            objects.push(stream_obj.into_bytes());

            // Page object
            let page_idx = page_refs.last().unwrap().split(' ').next().unwrap().parse::<usize>().unwrap();
            let content_ref = format!("{} 0 R", content_idx);
            let page_obj = format!(
                "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents {} >>",
                content_ref
            );
            objects.push(page_obj.into_bytes());

            let _ = page_idx;
        }

        // Update pages object with actual page refs
        let pages_content = format!("<< /Type /Pages /Kids [{}] /Count {} >>",
            page_refs.join(" "), page_refs.len());
        objects[1] = pages_content.into_bytes();

        // Font object
        objects.push(b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_vec());
        let font_idx = objects.len();

        // Write objects
        let mut xref_offsets = Vec::new();
        let mut offset = pdf.len();
        for (i, obj) in objects.iter().enumerate() {
            xref_offsets.push(offset);
            let header = format!("{} 0 obj\n", i + 1);
            pdf.extend_from_slice(header.as_bytes());
            pdf.extend_from_slice(obj);
            pdf.extend_from_slice(b"\nendobj\n");
            offset = pdf.len();
        }

        // Xref
        let xref_offset = pdf.len();
        pdf.extend_from_slice(format!("xref\n0 {}\n", objects.len() + 1).as_bytes());
        pdf.extend_from_slice(b"0000000000 65535 f \n");
        for off in &xref_offsets {
            pdf.extend_from_slice(format!("{:010} 00000 n \n", off).as_bytes());
        }

        // Trailer
        pdf.extend_from_slice(format!("trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF",
            objects.len() + 1, xref_offset).as_bytes());

        let _ = font_idx;
        pdf
    }

    /// Genera texto plano (alternativa simple)
    pub fn to_plain_text(doc: &PageDocument) -> String {
        let mut output = String::new();
        output.push_str(&doc.title);
        output.push_str("\n\n");
        for block in &doc.text_blocks {
            output.push_str(&block.text);
            output.push_str("\n\n");
        }
        output
    }
}

fn pdf_escape(s: &str) -> String {
    s.replace('\\', "\\\\")
     .replace('(', "\\(")
     .replace(')', "\\)")
     .replace('\n', " ")
     .replace('\r', "")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pdf_generation() {
        let doc = PageDocument::from_html("https://test.com", "<html><body><h1>Title</h1><p>Content</p></body></html>");
        let pdf = PrintPdf::generate(&doc);
        assert!(!pdf.is_empty());
        assert!(pdf.starts_with(b"%PDF-1.4"));
        assert!(pdf.ends_with(b"%%EOF"));
    }

    #[test]
    fn test_pdf_with_long_content() {
        let long_text = "word ".repeat(100);
        let html = format!("<html><body><p>{}</p></body></html>", long_text);
        let doc = PageDocument::from_html("https://test.com", &html);
        let pdf = PrintPdf::generate(&doc);
        assert!(pdf.len() > 500);
    }

    #[test]
    fn test_plain_text() {
        let doc = PageDocument::from_html("https://test.com", "<html><body><h1>Title</h1><p>Content</p></body></html>");
        let text = PrintPdf::to_plain_text(&doc);
        assert!(text.contains("Title"));
        assert!(text.contains("Content"));
    }

    #[test]
    fn test_pdf_escape() {
        assert_eq!(pdf_escape("(hello)"), "\\(hello\\)");
        assert_eq!(pdf_escape("back\\slash"), "back\\\\slash");
    }

    #[test]
    fn test_pdf_empty() {
        let doc = PageDocument::from_html("https://test.com", "<html><body></body></html>");
        let pdf = PrintPdf::generate(&doc);
        assert!(!pdf.is_empty());
    }
}
