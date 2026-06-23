//! HTTP Range Requests - Partial content download
//!
//! Permite descargar partes de archivos para streaming.

use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ByteRange {
    pub start: u64,
    pub end: u64,
}

impl ByteRange {
    pub fn new(start: u64, end: u64) -> Self {
        Self { start, end }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        // Format: "start-end" or "start-"
        if let Some(dash) = s.find('-') {
            let start_str = &s[..dash];
            let end_str = &s[dash+1..];
            let start: u64 = start_str.parse().ok()?;
            let end: u64 = if end_str.is_empty() {
                u64::MAX
            } else {
                end_str.parse().ok()?
            };
            return Some(Self { start, end });
        }
        None
    }

    pub fn size(&self) -> u64 {
        if self.end == u64::MAX { return u64::MAX; }
        self.end.saturating_sub(self.start) + 1
    }

    pub fn to_header(&self) -> String {
        if self.end == u64::MAX {
            format!("bytes={}-", self.start)
        } else {
            format!("bytes={}-{}", self.start, self.end)
        }
    }

    pub fn contains(&self, pos: u64) -> bool {
        pos >= self.start && pos <= self.end
    }
}

#[derive(Debug, Clone)]
pub struct RangeRequest {
    pub url: String,
    pub range: ByteRange,
    pub timeout_ms: u64,
    pub retry_count: u32,
    pub headers: Vec<(String, String)>,
}

impl RangeRequest {
    pub fn new(url: &str, range: ByteRange) -> Self {
        Self {
            url: url.to_string(),
            range,
            timeout_ms: 30_000,
            retry_count: 3,
            headers: Vec::new(),
        }
    }

    pub fn with_timeout(mut self, ms: u64) -> Self {
        self.timeout_ms = ms;
        self
    }

    pub fn with_header(mut self, key: &str, value: &str) -> Self {
        self.headers.push((key.to_string(), value.to_string()));
        self
    }
}

pub struct RangeDownloader {
    pub chunk_size: u64,
    pub max_concurrent: u32,
    pub retries: u32,
    pub timeout: Duration,
    pub user_agent: String,
    pub cookies: String,
}

impl RangeDownloader {
    pub fn new() -> Self {
        Self {
            chunk_size: 256 * 1024, // 256 KB
            max_concurrent: 4,
            retries: 3,
            timeout: Duration::from_secs(30),
            user_agent: "NoirBrowser/1.0".to_string(),
            cookies: String::new(),
        }
    }

    pub fn with_chunk_size(mut self, size: u64) -> Self {
        self.chunk_size = size;
        self
    }

    pub fn with_max_concurrent(mut self, n: u32) -> Self {
        self.max_concurrent = n;
        self
    }

    pub fn calculate_chunks(&self, total_size: u64) -> Vec<ByteRange> {
        let mut ranges = Vec::new();
        let mut pos = 0u64;
        while pos < total_size {
            let end = (pos + self.chunk_size - 1).min(total_size - 1);
            ranges.push(ByteRange::new(pos, end));
            pos += self.chunk_size;
        }
        ranges
    }

    pub fn estimate_chunks(&self, total_size: u64) -> u32 {
        ((total_size + self.chunk_size - 1) / self.chunk_size) as u32
    }

    /// Build request for a chunk
    pub fn build_request(&self, url: &str, range: ByteRange) -> RangeRequest {
        let mut req = RangeRequest::new(url, range)
            .with_timeout(self.timeout.as_millis() as u64)
            .with_header("User-Agent", &self.user_agent)
            .with_header("Accept", "*/*");
        if !self.cookies.is_empty() {
            req = req.with_header("Cookie", &self.cookies);
        }
        req
    }

    /// Parse Content-Range header
    pub fn parse_content_range(header: &str) -> Option<(u64, u64, u64)> {
        // Format: "bytes start-end/total"
        let s = header.strip_prefix("bytes ")?;
        let parts: Vec<&str> = s.split('/').collect();
        if parts.len() != 2 { return None; }
        let range_parts: Vec<&str> = parts[0].split('-').collect();
        if range_parts.len() != 2 { return None; }
        let start: u64 = range_parts[0].parse().ok()?;
        let end: u64 = range_parts[1].parse().ok()?;
        let total: u64 = if parts[1] == "*" { 0 } else { parts[1].parse().ok()? };
        Some((start, end, total))
    }

    /// Parse Accept-Ranges header
    pub fn parse_accept_ranges(header: &str) -> bool {
        header.to_lowercase().contains("bytes")
    }
}

impl Default for RangeDownloader {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_byte_range_new() {
        let r = ByteRange::new(0, 100);
        assert_eq!(r.start, 0);
        assert_eq!(r.end, 100);
        assert_eq!(r.size(), 101);
    }

    #[test]
    fn test_byte_range_from_str() {
        let r = ByteRange::from_str("0-1023").unwrap();
        assert_eq!(r.start, 0);
        assert_eq!(r.end, 1023);
    }

    #[test]
    fn test_byte_range_from_str_open_end() {
        let r = ByteRange::from_str("1024-").unwrap();
        assert_eq!(r.start, 1024);
        assert_eq!(r.end, u64::MAX);
    }

    #[test]
    fn test_byte_range_from_str_invalid() {
        assert!(ByteRange::from_str("invalid").is_none());
    }

    #[test]
    fn test_byte_range_to_header() {
        let r = ByteRange::new(0, 99);
        assert_eq!(r.to_header(), "bytes=0-99");
    }

    #[test]
    fn test_byte_range_to_header_open_end() {
        let r = ByteRange::new(1024, u64::MAX);
        assert_eq!(r.to_header(), "bytes=1024-");
    }

    #[test]
    fn test_byte_range_contains() {
        let r = ByteRange::new(10, 20);
        assert!(r.contains(10));
        assert!(r.contains(15));
        assert!(r.contains(20));
        assert!(!r.contains(5));
        assert!(!r.contains(25));
    }

    #[test]
    fn test_range_request_new() {
        let r = RangeRequest::new("https://x.com/file", ByteRange::new(0, 99));
        assert_eq!(r.url, "https://x.com/file");
        assert_eq!(r.timeout_ms, 30_000);
        assert_eq!(r.retry_count, 3);
    }

    #[test]
    fn test_range_request_with_timeout() {
        let r = RangeRequest::new("u", ByteRange::new(0, 99)).with_timeout(5000);
        assert_eq!(r.timeout_ms, 5000);
    }

    #[test]
    fn test_range_request_with_header() {
        let r = RangeRequest::new("u", ByteRange::new(0, 99))
            .with_header("X-Test", "value");
        assert_eq!(r.headers.len(), 1);
    }

    #[test]
    fn test_downloader_new() {
        let d = RangeDownloader::new();
        assert_eq!(d.chunk_size, 256 * 1024);
        assert_eq!(d.max_concurrent, 4);
    }

    #[test]
    fn test_downloader_calculate_chunks() {
        let d = RangeDownloader::new().with_chunk_size(1024);
        let chunks = d.calculate_chunks(4096);
        assert_eq!(chunks.len(), 4);
        assert_eq!(chunks[0], ByteRange::new(0, 1023));
        assert_eq!(chunks[3], ByteRange::new(3072, 4095));
    }

    #[test]
    fn test_downloader_estimate_chunks() {
        let d = RangeDownloader::new().with_chunk_size(1024);
        assert_eq!(d.estimate_chunks(1024), 1);
        assert_eq!(d.estimate_chunks(1025), 2);
        assert_eq!(d.estimate_chunks(2048), 2);
        assert_eq!(d.estimate_chunks(2049), 3);
    }

    #[test]
    fn test_downloader_partial_chunk() {
        let d = RangeDownloader::new().with_chunk_size(1024);
        let chunks = d.calculate_chunks(1500);
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[1], ByteRange::new(1024, 1499));
    }

    #[test]
    fn test_parse_content_range() {
        let r = RangeDownloader::parse_content_range("bytes 0-1023/2048").unwrap();
        assert_eq!(r, (0, 1023, 2048));
    }

    #[test]
    fn test_parse_content_range_unknown_total() {
        let r = RangeDownloader::parse_content_range("bytes 0-1023/*").unwrap();
        assert_eq!(r, (0, 1023, 0));
    }

    #[test]
    fn test_parse_content_range_invalid() {
        assert!(RangeDownloader::parse_content_range("invalid").is_none());
    }

    #[test]
    fn test_parse_accept_ranges() {
        assert!(RangeDownloader::parse_accept_ranges("bytes"));
        assert!(RangeDownloader::parse_accept_ranges("BYTES"));
        assert!(!RangeDownloader::parse_accept_ranges("none"));
    }

    #[test]
    fn test_build_request_basic() {
        let d = RangeDownloader::new();
        let req = d.build_request("https://x.com/file", ByteRange::new(0, 99));
        assert_eq!(req.url, "https://x.com/file");
        assert_eq!(req.range.start, 0);
        // Verifica headers
        let has_ua = req.headers.iter().any(|(k, _)| k == "User-Agent");
        assert!(has_ua);
    }

    #[test]
    fn test_build_request_with_cookies() {
        let mut d = RangeDownloader::new();
        d.cookies = "session=abc".to_string();
        let req = d.build_request("https://x.com", ByteRange::new(0, 99));
        let has_cookie = req.headers.iter().any(|(k, v)| k == "Cookie" && v == "session=abc");
        assert!(has_cookie);
    }

    #[test]
    fn test_chunks_zero_size() {
        let d = RangeDownloader::new();
        let chunks = d.calculate_chunks(0);
        assert!(chunks.is_empty());
    }
}
