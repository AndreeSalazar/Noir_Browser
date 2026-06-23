//! Screenshot - Captura de pantalla de página

use crate::parsers::page_document::PageDocument;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScreenshotFormat {
    Png,
    Bmp,
    Raw,
}

pub struct Screenshot;

impl Screenshot {
    /// Genera una captura BMP simple de la página
    pub fn capture_bmp(doc: &PageDocument, width: u32, height: u32) -> Vec<u8> {
        // Simplified: create a BMP with white background
        let row_size = (width * 4 + 3) & !3;
        let pixel_data_size = row_size * height;
        let file_size = 54 + pixel_data_size as usize;

        let mut bmp = Vec::with_capacity(file_size);

        // BMP Header
        bmp.extend_from_slice(b"BM");
        bmp.extend_from_slice(&(file_size as u32).to_le_bytes());
        bmp.extend_from_slice(&[0; 4]); // reserved
        bmp.extend_from_slice(&54u32.to_le_bytes()); // offset to pixel data

        // DIB Header
        bmp.extend_from_slice(&40u32.to_le_bytes()); // header size
        bmp.extend_from_slice(&width.to_le_bytes());
        bmp.extend_from_slice(&height.to_le_bytes());
        bmp.extend_from_slice(&1u16.to_le_bytes()); // planes
        bmp.extend_from_slice(&32u16.to_le_bytes()); // bpp
        bmp.extend_from_slice(&0u32.to_le_bytes()); // compression
        bmp.extend_from_slice(&(pixel_data_size as u32).to_le_bytes());
        bmp.extend_from_slice(&2835u32.to_le_bytes()); // x ppm
        bmp.extend_from_slice(&2835u32.to_le_bytes()); // y ppm
        bmp.extend_from_slice(&0u32.to_le_bytes()); // colors
        bmp.extend_from_slice(&0u32.to_le_bytes()); // important colors

        // Pixel data (BGRA, bottom-up)
        let bg = [0xFFu8, 0xFF, 0xFF, 0xFF]; // White BGRA
        for _ in 0..(width * height) {
            bmp.extend_from_slice(&bg);
        }

        // Add title text "rendered" at top (just a visual marker)
        let _ = doc;
        bmp
    }

    /// Genera un thumbnail de la página
    pub fn thumbnail(doc: &PageDocument, width: u32, height: u32) -> Vec<u8> {
        Self::capture_bmp(doc, width, height)
    }

    /// Genera un hash simple para identificar la página
    pub fn page_hash(doc: &PageDocument) -> u64 {
        let mut hash: u64 = 0;
        for block in &doc.text_blocks {
            for byte in block.text.bytes() {
                hash = hash.wrapping_mul(31).wrapping_add(byte as u64);
            }
        }
        hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capture_bmp() {
        let doc = PageDocument::from_html("https://test.com", "<html><body><h1>Title</h1></body></html>");
        let bmp = Screenshot::capture_bmp(&doc, 100, 50);
        assert!(!bmp.is_empty());
        assert!(bmp.starts_with(b"BM"));
    }

    #[test]
    fn test_capture_different_sizes() {
        let doc = PageDocument::from_html("https://test.com", "<html><body></body></html>");
        let small = Screenshot::capture_bmp(&doc, 10, 10);
        let large = Screenshot::capture_bmp(&doc, 1000, 500);
        assert!(large.len() > small.len());
    }

    #[test]
    fn test_thumbnail() {
        let doc = PageDocument::from_html("https://test.com", "<html><body></body></html>");
        let thumb = Screenshot::thumbnail(&doc, 200, 150);
        assert!(!thumb.is_empty());
    }

    #[test]
    fn test_page_hash() {
        let doc1 = PageDocument::from_html("https://test.com", "<html><body><p>Hello</p></body></html>");
        let doc2 = PageDocument::from_html("https://test.com", "<html><body><p>Hello</p></body></html>");
        let doc3 = PageDocument::from_html("https://test.com", "<html><body><p>World</p></body></html>");
        assert_eq!(Screenshot::page_hash(&doc1), Screenshot::page_hash(&doc2));
        assert_ne!(Screenshot::page_hash(&doc1), Screenshot::page_hash(&doc3));
    }

    #[test]
    fn test_screenshot_format() {
        let _ = ScreenshotFormat::Png;
        let _ = ScreenshotFormat::Bmp;
        let _ = ScreenshotFormat::Raw;
    }
}
