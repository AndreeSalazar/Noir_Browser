//! Video URL Extractor - Extrae URLs de m3u8/mpd desde páginas HTML
//!
//! Detecta URLs de streaming en HTML, JSON embebido, y metadata de páginas

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VideoSourceType {
    Mp4,
    Webm,
    Hls,
    Dash,
    Unknown,
}

impl VideoSourceType {
    pub fn from_url(url: &str) -> Self {
        let lower = url.to_lowercase();
        if lower.contains(".m3u8") { return Self::Hls; }
        if lower.contains(".mpd") { return Self::Dash; }
        if lower.contains(".mp4") { return Self::Mp4; }
        if lower.contains(".webm") { return Self::Webm; }
        Self::Unknown
    }

    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "mp4" | "m4v" => Self::Mp4,
            "webm" => Self::Webm,
            "m3u8" => Self::Hls,
            "mpd" => Self::Dash,
            _ => Self::Unknown,
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            Self::Mp4 => "mp4",
            Self::Webm => "webm",
            Self::Hls => "hls",
            Self::Dash => "dash",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExtractedVideoSource {
    pub url: String,
    pub source_type: VideoSourceType,
    pub quality: String,
    pub mime_type: String,
    pub bandwidth: u32,
    pub width: u32,
    pub height: u32,
    pub codec: String,
}

impl ExtractedVideoSource {
    pub fn new(url: &str) -> Self {
        let source_type = VideoSourceType::from_url(url);
        Self {
            url: url.to_string(),
            source_type,
            quality: String::new(),
            mime_type: String::new(),
            bandwidth: 0,
            width: 0,
            height: 0,
            codec: String::new(),
        }
    }

    pub fn with_quality(mut self, q: &str) -> Self {
        self.quality = q.to_string();
        self
    }

    pub fn with_bandwidth(mut self, b: u32) -> Self {
        self.bandwidth = b;
        self
    }
}

#[derive(Debug, Clone)]
pub struct ExtractionResult {
    pub sources: Vec<ExtractedVideoSource>,
    pub title: String,
    pub duration_secs: u32,
    pub thumbnail: String,
    pub site_name: String,
    pub raw_url_count: usize,
}

pub struct VideoUrlExtractor {
    pub max_results: usize,
    pub include_unknown: bool,
}

impl VideoUrlExtractor {
    pub fn new() -> Self {
        Self {
            max_results: 50,
            include_unknown: false,
        }
    }

    pub fn with_max_results(mut self, n: usize) -> Self {
        self.max_results = n;
        self
    }

    /// Extrae URLs de video de un HTML
    pub fn extract_from_html(&self, html: &str, base_url: &str) -> ExtractionResult {
        let mut result = ExtractionResult {
            sources: Vec::new(),
            title: String::new(),
            duration_secs: 0,
            thumbnail: String::new(),
            site_name: String::new(),
            raw_url_count: 0,
        };
        if let Some(t) = Self::extract_meta_content(html, "og:title") {
            result.title = t;
        } else if let Some(t) = Self::extract_title(html) {
            result.title = t;
        }
        if let Some(t) = Self::extract_meta_content(html, "og:image") {
            result.thumbnail = t;
        }
        if let Some(s) = Self::extract_meta_content(html, "og:site_name") {
            result.site_name = s;
        }
        if let Some(d) = Self::extract_meta_content(html, "video:duration") {
            result.duration_secs = d.parse().unwrap_or(0);
        }
        self.find_urls_in_html(html, base_url, &mut result);
        self.find_video_tags(html, base_url, &mut result);
        self.find_json_video_urls(html, &mut result);
        result.raw_url_count = result.sources.len();
        result
    }

    fn find_urls_in_html(&self, html: &str, base_url: &str, result: &mut ExtractionResult) {
        let urls = Self::extract_urls_from_text(html);
        for url in urls {
            if result.sources.len() >= self.max_results { break; }
            let resolved = Self::resolve_url(&url, base_url);
            let source_type = VideoSourceType::from_url(&resolved);
            if source_type == VideoSourceType::Unknown && !self.include_unknown {
                continue;
            }
            if !result.sources.iter().any(|s| s.url == resolved) {
                if result.sources.len() >= self.max_results { break; }
                result.sources.push(ExtractedVideoSource::new(&resolved));
            }
        }
    }

    fn find_video_tags(&self, html: &str, base_url: &str, result: &mut ExtractionResult) {
        let patterns = [
            "<video",
            "<source",
        ];
        for pat in &patterns {
            let mut pos = 0;
            while let Some(start) = html[pos..].find(pat) {
                if result.sources.len() >= self.max_results { break; }
                let abs_start = pos + start;
                if let Some(end_offset) = html[abs_start..].find('>') {
                    let tag = &html[abs_start..abs_start + end_offset];
                    if let Some(src) = Self::extract_attr_from_tag(tag, "src") {
                        let resolved = Self::resolve_url(&src, base_url);
                        if !result.sources.iter().any(|s| s.url == resolved) {
                            if result.sources.len() >= self.max_results { break; }
                            let mut source = ExtractedVideoSource::new(&resolved);
                            if let Some(t) = Self::extract_attr_from_tag(tag, "type") {
                                source.mime_type = t;
                            }
                            result.sources.push(source);
                        }
                    }
                }
                pos = abs_start + 1;
            }
            if result.sources.len() >= self.max_results { break; }
        }
    }

    fn find_json_video_urls(&self, html: &str, result: &mut ExtractionResult) {
        if let Some(start) = html.find("ytInitialPlayerResponse") {
            if result.sources.len() >= self.max_results { return; }
            if let Some(json_start) = html[start..].find('{') {
                let abs_start = start + json_start;
                if let Some(end) = Self::extract_balanced_json(&html[abs_start..]) {
                    let json = &html[abs_start..abs_start + end];
                    if let Some(streaming_data) = Self::find_key(json, "streamingData") {
                        if result.sources.len() < self.max_results {
                            if let Some(formats) = Self::find_key(streaming_data, "formats") {
                                Self::parse_youtube_formats(formats, result, false);
                            }
                        }
                        if result.sources.len() < self.max_results {
                            if let Some(adaptive) = Self::find_key(streaming_data, "adaptiveFormats") {
                                Self::parse_youtube_formats(adaptive, result, true);
                            }
                        }
                    }
                    if let Some(details) = Self::find_key(json, "videoDetails") {
                        if let Some(title) = Self::find_string_value(details, "title") {
                            if result.title.is_empty() {
                                result.title = title;
                            }
                        }
                        if let Some(length) = Self::find_string_value(details, "lengthSeconds") {
                            result.duration_secs = length.parse().unwrap_or(0);
                        }
                        if let Some(thumb) = Self::find_string_value(details, "thumbnail") {
                            if result.thumbnail.is_empty() {
                                result.thumbnail = thumb;
                            }
                        }
                    }
                }
            }
        }
        if let Some(start) = html.find("vimeo.config") {
            if let Some(json_start) = html[start..].find('{') {
                let abs_start = start + json_start;
                if let Some(end) = Self::extract_balanced_json(&html[abs_start..]) {
                    let json = &html[abs_start..abs_start + end];
                    if let Some(video) = Self::find_key(json, "video") {
                        if let Some(urls) = Self::find_key(video, "progressive") {
                            Self::parse_progressive_array(urls, result);
                        }
                    }
                }
            }
        }
    }

    fn parse_youtube_formats(json: &str, result: &mut ExtractionResult, adaptive: bool) {
        let mut pos = 0;
        while let Some(start) = json[pos..].find('{') {
            let abs_start = pos + start;
            if let Some(end) = Self::extract_balanced_json(&json[abs_start..]) {
                let entry = &json[abs_start..abs_start + end];
                let mut source = ExtractedVideoSource::new("");
                if let Some(url) = Self::find_string_value(entry, "url") {
                    source.url = url;
                } else if let Some(sig) = Self::find_string_value(entry, "signatureCipher") {
                    source.url = format!("signature:{}", sig);
                }
                source.source_type = if adaptive {
                    VideoSourceType::Dash
                } else {
                    VideoSourceType::Mp4
                };
                if let Some(q) = Self::find_string_value(entry, "qualityLabel") {
                    source.quality = q;
                } else if let Some(q) = Self::find_string_value(entry, "quality") {
                    source.quality = q;
                }
                if let Some(mime) = Self::find_string_value(entry, "mimeType") {
                    source.mime_type = mime;
                }
                if let Some(w) = Self::find_string_value(entry, "width") {
                    source.width = w.parse().unwrap_or(0);
                }
                if let Some(h) = Self::find_string_value(entry, "height") {
                    source.height = h.parse().unwrap_or(0);
                }
                if let Some(bw) = Self::find_string_value(entry, "bitrate") {
                    source.bandwidth = bw.parse().unwrap_or(0);
                }
                if let Some(c) = Self::find_string_value(entry, "codec") {
                    source.codec = c;
                }
                if !source.url.is_empty() && !result.sources.iter().any(|s| s.url == source.url) {
                    result.sources.push(source);
                }
                pos = abs_start + end;
            } else {
                pos = abs_start + 1;
            }
        }
    }

    fn parse_progressive_array(json: &str, result: &mut ExtractionResult) {
        let mut pos = 0;
        while let Some(start) = json[pos..].find('{') {
            let abs_start = pos + start;
            if let Some(end) = Self::extract_balanced_json(&json[abs_start..]) {
                let entry = &json[abs_start..abs_start + end];
                if let Some(url) = Self::find_string_value(entry, "url") {
                    if !result.sources.iter().any(|s| s.url == url) {
                        result.sources.push(ExtractedVideoSource::new(&url));
                    }
                }
                pos = abs_start + end;
            } else {
                pos = abs_start + 1;
            }
        }
    }

    fn extract_urls_from_text(text: &str) -> Vec<String> {
        let mut urls = Vec::new();
        let mut pos = 0;
        while pos < text.len() {
            let c = text.as_bytes()[pos];
            if c == b'"' || c == b'\'' {
                let quote = c;
                if let Some(end_offset) = text[pos+1..].find(quote as char) {
                    let candidate = &text[pos+1..pos+1+end_offset];
                    if Self::looks_like_video_url(candidate) {
                        urls.push(candidate.to_string());
                    }
                    pos = pos + 1 + end_offset + 1;
                    continue;
                }
            }
            pos += 1;
        }
        urls
    }

    fn looks_like_video_url(s: &str) -> bool {
        let lower = s.to_lowercase();
        if s.len() < 10 { return false; }
        if !s.starts_with("http://") && !s.starts_with("https://") && !s.starts_with("//") {
            return false;
        }
        lower.contains(".m3u8") || lower.contains(".mpd") ||
        lower.contains(".mp4") || lower.contains(".webm") ||
        lower.contains("/videoplayback") || lower.contains("/manifest") ||
        lower.contains("googlevideo.com") || lower.contains("/video/")
    }

    fn resolve_url(url: &str, base: &str) -> String {
        if url.starts_with("http://") || url.starts_with("https://") {
            return url.to_string();
        }
        if url.starts_with("//") {
            if let Some(scheme_end) = base.find("://") {
                return format!("{}:{}", &base[..scheme_end], url);
            }
            return format!("https:{}", url);
        }
        if url.starts_with('/') {
            if let Some(scheme_end) = base.find("://") {
                if let Some(host_end) = base[scheme_end+3..].find('/') {
                    return format!("{}{}", &base[..scheme_end+3+host_end], url);
                }
            }
            return url.to_string();
        }
        if let Some(last_slash) = base.rfind('/') {
            return format!("{}/{}", &base[..last_slash], url);
        }
        url.to_string()
    }

    fn extract_meta_content(html: &str, property: &str) -> Option<String> {
        let patterns = [
            format!("property=\"{}\"", property),
            format!("property='{}'", property),
            format!("name=\"{}\"", property),
            format!("name='{}'", property),
        ];
        for pat in &patterns {
            if let Some(idx) = html.find(pat.as_str()) {
                let after = &html[idx + pat.len()..];
                if let Some(content_start) = after.find("content=\"") {
                    let val_in_after = content_start + "content=\"".len();
                    if let Some(end) = after[val_in_after..].find('"') {
                        return Some(after[val_in_after..val_in_after+end].to_string());
                    }
                }
                if let Some(content_start) = after.find("content='") {
                    let val_in_after = content_start + "content='".len();
                    if let Some(end) = after[val_in_after..].find('\'') {
                        return Some(after[val_in_after..val_in_after+end].to_string());
                    }
                }
            }
        }
        None
    }

    fn extract_title(html: &str) -> Option<String> {
        if let Some(start) = html.find("<title>") {
            if let Some(end) = html[start+7..].find("</title>") {
                return Some(html[start+7..start+7+end].to_string());
            }
        }
        None
    }

    fn extract_attr_from_tag(tag: &str, attr: &str) -> Option<String> {
        let pat1 = format!("{}=\"", attr);
        if let Some(start) = tag.find(pat1.as_str()) {
            let val_start = start + pat1.len();
            if let Some(end) = tag[val_start..].find('"') {
                return Some(tag[val_start..val_start+end].to_string());
            }
        }
        None
    }

    fn extract_balanced_json(s: &str) -> Option<usize> {
        if s.is_empty() { return None; }
        let first = s.as_bytes()[0];
        if first != b'{' && first != b'[' { return None; }
        let open = first;
        let close = if first == b'{' { b'}' } else { b']' };
        let mut depth = 0i32;
        let mut in_string = false;
        let mut escape = false;
        for (i, &b) in s.as_bytes().iter().enumerate() {
            if escape { escape = false; continue; }
            if b == b'\\' && in_string { escape = true; continue; }
            if b == b'"' { in_string = !in_string; continue; }
            if in_string { continue; }
            if b == open { depth += 1; }
            else if b == close {
                depth -= 1;
                if depth == 0 { return Some(i + 1); }
            }
        }
        None
    }

    fn find_key<'a>(json: &'a str, key: &str) -> Option<&'a str> {
        let pat = format!("\"{}\"", key);
        if let Some(idx) = json.find(pat.as_str()) {
            let after = &json[idx + pat.len()..];
            let after = after.trim_start().strip_prefix(':')?;
            let after = after.trim_start();
            if after.starts_with('{') || after.starts_with('[') {
                return Self::extract_balanced_json(after).map(|end| &after[..end]);
            }
        }
        None
    }

    fn find_string_value(json: &str, key: &str) -> Option<String> {
        let pat = format!("\"{}\"", key);
        if let Some(idx) = json.find(pat.as_str()) {
            let after = &json[idx + pat.len()..];
            let after = after.trim_start().strip_prefix(':')?;
            let after = after.trim_start();
            if after.starts_with('"') {
                if let Some(end) = Self::find_string_end(&after[1..]) {
                    return Some(after[1..1+end].to_string());
                }
            }
        }
        None
    }

    fn find_string_end(s: &str) -> Option<usize> {
        let mut escape = false;
        for (i, c) in s.char_indices() {
            if escape { escape = false; continue; }
            if c == '\\' { escape = true; continue; }
            if c == '"' { return Some(i); }
        }
        None
    }

    /// Encuentra la mejor source (mayor bandwidth o resolution)
    pub fn best_source<'a>(sources: &'a [ExtractedVideoSource]) -> Option<&'a ExtractedVideoSource> {
        let mut best: Option<&ExtractedVideoSource> = None;
        for s in sources {
            if s.url.is_empty() || s.url.starts_with("signature:") { continue; }
            match best {
                None => best = Some(s),
                Some(b) => {
                    if s.height > b.height || (s.height == b.height && s.bandwidth > b.bandwidth) {
                        best = Some(s);
                    }
                }
            }
        }
        best
    }

    /// Encuentra source por calidad preferida
    pub fn source_for_quality<'a>(sources: &'a [ExtractedVideoSource], quality: &str) -> Option<&'a ExtractedVideoSource> {
        for s in sources {
            if s.url.is_empty() || s.url.starts_with("signature:") { continue; }
            if s.quality.contains(quality) {
                return Some(s);
            }
        }
        None
    }
}

impl Default for VideoUrlExtractor {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_type_from_url() {
        assert_eq!(VideoSourceType::from_url("https://x.com/v.m3u8"), VideoSourceType::Hls);
        assert_eq!(VideoSourceType::from_url("https://x.com/v.mpd"), VideoSourceType::Dash);
        assert_eq!(VideoSourceType::from_url("https://x.com/v.mp4"), VideoSourceType::Mp4);
        assert_eq!(VideoSourceType::from_url("https://x.com/v.webm"), VideoSourceType::Webm);
        assert_eq!(VideoSourceType::from_url("https://x.com/"), VideoSourceType::Unknown);
    }

    #[test]
    fn test_source_type_to_str() {
        assert_eq!(VideoSourceType::Hls.to_str(), "hls");
    }

    #[test]
    fn test_video_source_new() {
        let s = ExtractedVideoSource::new("https://x.com/v.mp4");
        assert_eq!(s.source_type, VideoSourceType::Mp4);
    }

    #[test]
    fn test_video_source_builder() {
        let s = ExtractedVideoSource::new("https://x.com/v.mp4")
            .with_quality("720p")
            .with_bandwidth(2_500_000);
        assert_eq!(s.quality, "720p");
        assert_eq!(s.bandwidth, 2_500_000);
    }

    #[test]
    fn test_extractor_new() {
        let e = VideoUrlExtractor::new();
        assert_eq!(e.max_results, 50);
    }

    #[test]
    fn test_resolve_url_absolute() {
        assert_eq!(VideoUrlExtractor::resolve_url("https://x.com/v.mp4", "https://a.com/"), "https://x.com/v.mp4");
    }

    #[test]
    fn test_resolve_url_protocol_relative() {
        assert_eq!(VideoUrlExtractor::resolve_url("//cdn.com/v.mp4", "https://a.com/"), "https://cdn.com/v.mp4");
    }

    #[test]
    fn test_resolve_url_absolute_path() {
        assert_eq!(VideoUrlExtractor::resolve_url("/v.mp4", "https://a.com/foo/bar"), "https://a.com/v.mp4");
    }

    #[test]
    fn test_resolve_url_relative() {
        assert_eq!(VideoUrlExtractor::resolve_url("v.mp4", "https://a.com/foo/"), "https://a.com/foo/v.mp4");
    }

    #[test]
    fn test_extract_meta_content() {
        let html = r#"<meta property="og:title" content="My Video">"#;
        assert_eq!(VideoUrlExtractor::extract_meta_content(html, "og:title"), Some("My Video".to_string()));
    }

    #[test]
    fn test_extract_meta_content_single_quote() {
        let html = r#"<meta property='og:title' content='My Video'>"#;
        assert_eq!(VideoUrlExtractor::extract_meta_content(html, "og:title"), Some("My Video".to_string()));
    }

    #[test]
    fn test_extract_title() {
        let html = "<html><head><title>Test</title></head></html>";
        assert_eq!(VideoUrlExtractor::extract_title(html), Some("Test".to_string()));
    }

    #[test]
    fn test_looks_like_video_url() {
        assert!(VideoUrlExtractor::looks_like_video_url("https://x.com/v.m3u8"));
        assert!(VideoUrlExtractor::looks_like_video_url("https://x.com/v.mpd"));
        assert!(VideoUrlExtractor::looks_like_video_url("https://googlevideo.com/videoplayback"));
        assert!(!VideoUrlExtractor::looks_like_video_url("https://x.com/image.png"));
        assert!(!VideoUrlExtractor::looks_like_video_url("javascript:void"));
    }

    #[test]
    fn test_extract_balanced_json() {
        let s = r#"{"a": 1, "b": {"c": 2}}"#;
        assert_eq!(VideoUrlExtractor::extract_balanced_json(s), Some(s.len()));
    }

    #[test]
    fn test_extract_balanced_json_array() {
        let s = r#"[1, 2, [3, 4]]"#;
        assert_eq!(VideoUrlExtractor::extract_balanced_json(s), Some(s.len()));
    }

    #[test]
    fn test_extract_balanced_json_with_strings() {
        let s = r#"{"a": "hello {world}", "b": 1}"#;
        let end = VideoUrlExtractor::extract_balanced_json(s).unwrap();
        assert_eq!(end, s.len());
    }

    #[test]
    fn test_find_string_value() {
        let json = r#"{"title": "Hello", "count": 5}"#;
        assert_eq!(VideoUrlExtractor::find_string_value(json, "title"), Some("Hello".to_string()));
        assert_eq!(VideoUrlExtractor::find_string_value(json, "missing"), None);
    }

    #[test]
    fn test_find_string_value_with_escape() {
        // Test que el parser maneja correctamente los escapes
        let json = r#"{"text": "He said \"hi\""}"#;
        let result = VideoUrlExtractor::find_string_value(json, "text");
        // El parser no desescape, solo encuentra el final del string
        assert!(result.is_some());
        assert!(result.unwrap().contains("He said"));
    }

    #[test]
    fn test_extract_from_html_basic() {
        let html = r#"<html><head>
<meta property="og:title" content="Test Video">
<meta property="og:image" content="https://x.com/thumb.jpg">
</head><body>
<video src="https://cdn.com/movie.mp4" type="video/mp4"></video>
</body></html>"#;
        let e = VideoUrlExtractor::new();
        let result = e.extract_from_html(html, "https://x.com/");
        assert_eq!(result.title, "Test Video");
        assert_eq!(result.thumbnail, "https://x.com/thumb.jpg");
        assert!(result.sources.iter().any(|s| s.url == "https://cdn.com/movie.mp4"));
    }

    #[test]
    fn test_extract_youtube_player_response() {
        let html = r#"<script>var ytInitialPlayerResponse = {"streamingData": {"formats": [{"url": "https://googlevideo.com/v1", "qualityLabel": "720p", "width": 1280, "height": 720, "mimeType": "video/mp4"}], "adaptiveFormats": [{"url": "https://googlevideo.com/v2", "qualityLabel": "720p", "width": 1280, "height": 720, "mimeType": "video/webm"}]}};</script>"#;
        let e = VideoUrlExtractor::new();
        let result = e.extract_from_html(html, "https://youtube.com/");
        assert!(result.sources.iter().any(|s| s.url.contains("googlevideo.com/v1")));
        assert!(result.sources.iter().any(|s| s.url.contains("googlevideo.com/v2")));
    }

    #[test]
    fn test_extract_youtube_with_title() {
        let html = r#"var ytInitialPlayerResponse = {"videoDetails": {"title": "My YouTube Video", "lengthSeconds": "300", "thumbnail": "https://i.ytimg.com/vi/abc/0.jpg"}}"#;
        let e = VideoUrlExtractor::new();
        let result = e.extract_from_html(html, "https://youtube.com/");
        assert_eq!(result.title, "My YouTube Video");
        assert_eq!(result.duration_secs, 300);
        assert_eq!(result.thumbnail, "https://i.ytimg.com/vi/abc/0.jpg");
    }

    #[test]
    fn test_extract_video_source_tag() {
        let html = r#"<video><source src="https://cdn.com/v.mp4" type="video/mp4"></video>"#;
        let e = VideoUrlExtractor::new();
        let result = e.extract_from_html(html, "https://x.com/");
        assert!(result.sources.iter().any(|s| s.url == "https://cdn.com/v.mp4"));
    }

    #[test]
    fn test_extract_max_results() {
        let mut html = String::new();
        for i in 0..100 {
            html.push_str(&format!(r#"<source src="https://cdn.com/v{}.mp4" type="video/mp4">"#, i));
        }
        let e = VideoUrlExtractor::new().with_max_results(10);
        let result = e.extract_from_html(&html, "https://x.com/");
        assert_eq!(result.sources.len(), 10);
    }

    #[test]
    fn test_best_source() {
        let sources = vec![
            ExtractedVideoSource::new("https://x.com/360.mp4").with_quality("360p").with_bandwidth(500_000),
            ExtractedVideoSource::new("https://x.com/1080.mp4").with_quality("1080p").with_bandwidth(5_000_000),
            ExtractedVideoSource::new("https://x.com/720.mp4").with_quality("720p").with_bandwidth(2_500_000),
        ];
        let best = VideoUrlExtractor::best_source(&sources).unwrap();
        assert!(best.url.contains("1080"));
    }

    #[test]
    fn test_source_for_quality() {
        let sources = vec![
            ExtractedVideoSource::new("https://x.com/360.mp4").with_quality("360p"),
            ExtractedVideoSource::new("https://x.com/720.mp4").with_quality("720p"),
        ];
        let q = VideoUrlExtractor::source_for_quality(&sources, "720").unwrap();
        assert!(q.url.contains("720"));
    }

    #[test]
    fn test_signature_cipher_skipped() {
        let sources = vec![
            ExtractedVideoSource::new("signature:sig123"),
        ];
        let best = VideoUrlExtractor::best_source(&sources);
        assert!(best.is_none());
    }

    #[test]
    fn test_extract_urls_in_text() {
        let text = r#"src="https://cdn.com/v.mp4" data="https://other.com/v.webm""#;
        let urls = VideoUrlExtractor::extract_urls_from_text(text);
        assert_eq!(urls.len(), 2);
    }

    #[test]
    fn test_extract_no_urls() {
        let html = "<html><body>Hello world</body></html>";
        let e = VideoUrlExtractor::new();
        let result = e.extract_from_html(html, "https://x.com/");
        assert_eq!(result.sources.len(), 0);
    }
}
