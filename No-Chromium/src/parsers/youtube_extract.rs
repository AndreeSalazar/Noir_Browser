//! YouTube compatibility extraction.
//!
//! YouTube is a JavaScript-heavy app. Noir does not need a full Chromium-style
//! runtime to show useful search/watch metadata: the HTML usually embeds a
//! `ytInitialData` JSON blob. This module extracts a small, stable subset from
//! that blob and turns it into normal `PageDocument` blocks.

use std::collections::HashSet;

use super::page_document::{ImageBlock, LinkInfo, PageDocument, TextBlock, VideoBlock};

#[derive(Debug, Clone, PartialEq, Eq)]
struct YouTubeVideoCard {
    video_id: String,
    title: String,
    thumbnail_url: Option<String>,
    channel: Option<String>,
    metadata: Option<String>,
}

/// Adds lightweight YouTube cards extracted from raw HTML.
pub fn enhance_page_document(doc: &mut PageDocument, html: &str) {
    if !is_youtube_url(&doc.url) {
        return;
    }

    let videos = extract_video_cards(html, 24);
    if videos.is_empty() {
        if !has_existing_youtube_video_content(doc) {
            replace_with_youtube_lite_message(doc);
        }
        return;
    }

    // YouTube's normal HTML contains a lot of footer/legal/navigation text that
    // is useful for a full DOM engine, but noisy for Noir's lightweight mode.
    // For now, prefer a clean extracted-results view.
    doc.text_blocks.clear();
    doc.image_blocks.clear();
    doc.video_blocks.clear();
    doc.links.clear();

    doc.text_blocks.push(TextBlock {
        text: "YouTube results extracted by Noir".to_string(),
        tag: "h2".to_string(),
        font_size: 22.0,
        bold: true,
        link: None,
        indent_level: 0,
        attributes: Default::default(),
    });

    for video in videos {
        let watch_url = format!("https://www.youtube.com/watch?v={}", video.video_id);

        if let Some(src) = video.thumbnail_url.clone() {
            doc.image_blocks.push(ImageBlock {
                src,
                alt: video.title.clone(),
                width: Some(320.0),
                height: Some(180.0),
                lazy: false,
            });
        }

        doc.links.push(LinkInfo {
            text: video.title.clone(),
            href: watch_url.clone(),
        });

        doc.text_blocks.push(TextBlock {
            text: video.title.clone(),
            tag: "a".to_string(),
            font_size: 15.0,
            bold: true,
            link: Some(watch_url.clone()),
            indent_level: 0,
            attributes: Default::default(),
        });

        if let Some(channel) = video.channel {
            doc.text_blocks.push(TextBlock {
                text: channel,
                tag: "p".to_string(),
                font_size: 12.0,
                bold: false,
                link: None,
                indent_level: 0,
                attributes: Default::default(),
            });
        }

        if let Some(metadata) = video.metadata {
            doc.text_blocks.push(TextBlock {
                text: metadata,
                tag: "p".to_string(),
                font_size: 12.0,
                bold: false,
                link: None,
                indent_level: 0,
                attributes: Default::default(),
            });
        }

        // Add a video block too. The current renderer draws a playable-looking
        // placeholder; later this can be connected to a real media pipeline.
        doc.video_blocks.push(VideoBlock {
            src: watch_url,
            poster: video.thumbnail_url,
            controls: true,
            autoplay: false,
            loop_video: false,
            muted: false,
            width: Some(640.0),
            height: Some(360.0),
            title: Some(video.title),
        });
    }
}

fn replace_with_youtube_lite_message(doc: &mut PageDocument) {
    doc.text_blocks.clear();
    doc.image_blocks.clear();
    doc.video_blocks.clear();
    doc.links.clear();

    let has_query = doc.url.contains("search_query=")
        && !doc.url.ends_with("search_query=");
    let message = if has_query {
        "No video metadata found in YouTube HTML. YouTube may have served a JS-only or consent page."
    } else {
        "Type `yt <search>` in the address bar to use Noir YouTube Lite."
    };

    doc.title = "YouTube Lite".to_string();
    doc.text_blocks.push(TextBlock {
        text: "Noir YouTube Lite".to_string(),
        tag: "h1".to_string(),
        font_size: 28.0,
        bold: true,
        link: None,
        indent_level: 0,
        attributes: Default::default(),
    });
    doc.text_blocks.push(TextBlock {
        text: message.to_string(),
        tag: "p".to_string(),
        font_size: 14.0,
        bold: false,
        link: None,
        indent_level: 0,
        attributes: Default::default(),
    });
    doc.text_blocks.push(TextBlock {
        text: "Goal: extract titles, thumbnails and watch links without running the full YouTube app.".to_string(),
        tag: "p".to_string(),
        font_size: 13.0,
        bold: false,
        link: None,
        indent_level: 0,
        attributes: Default::default(),
    });
}

fn has_existing_youtube_video_content(doc: &PageDocument) -> bool {
    doc.links.iter().any(|link| link.href.contains("/watch") || link.href.contains("youtu.be"))
        || doc.image_blocks.iter().any(|image| image.src.contains("ytimg.com"))
        || doc.video_blocks.iter().any(|video| video.src.contains("/watch") || video.src.contains("embed"))
}

fn is_youtube_url(url: &str) -> bool {
    url.contains("youtube.com") || url.contains("youtu.be")
}

fn extract_video_cards(html: &str, limit: usize) -> Vec<YouTubeVideoCard> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    let mut search_from = 0;

    while out.len() < limit {
        let Some(relative) = html[search_from..].find("\"videoId\":\"") else {
            break;
        };
        let pos = search_from + relative;
        let id_start = pos + "\"videoId\":\"".len();
        let Some(id_end_rel) = html[id_start..].find('"') else {
            break;
        };
        let video_id = html[id_start..id_start + id_end_rel].to_string();
        search_from = id_start + id_end_rel;

        if video_id.is_empty() || !seen.insert(video_id.clone()) {
            continue;
        }

        let segment_end = html.len().min(pos + 12_000);
        let segment = &html[pos..segment_end];
        let title = extract_text_after(segment, "\"title\":{\"runs\":[{\"text\":\"")
            .or_else(|| extract_text_after(segment, "\"title\":{\"simpleText\":\""))
            .unwrap_or_else(|| format!("YouTube video {}", video_id));

        let channel = extract_text_after(segment, "\"ownerText\":{\"runs\":[{\"text\":\"")
            .or_else(|| extract_text_after(segment, "\"shortBylineText\":{\"runs\":[{\"text\":\""));
        let metadata = extract_text_after(segment, "\"publishedTimeText\":{\"simpleText\":\"")
            .or_else(|| extract_text_after(segment, "\"lengthText\":{\"simpleText\":\""));

        out.push(YouTubeVideoCard {
            video_id,
            title,
            thumbnail_url: extract_thumbnail_url(segment),
            channel,
            metadata,
        });
    }

    out
}

fn extract_thumbnail_url(segment: &str) -> Option<String> {
    let mut search_from = 0;
    while let Some(relative) = segment[search_from..].find("\"url\":\"") {
        let start = search_from + relative + "\"url\":\"".len();
        let Some(end_rel) = find_json_string_end(&segment[start..]) else {
            return None;
        };
        let decoded = decode_jsonish_string(&segment[start..start + end_rel]);
        if decoded.contains("ytimg.com") {
            return Some(decoded);
        }
        search_from = start + end_rel;
    }
    None
}

fn extract_text_after(segment: &str, marker: &str) -> Option<String> {
    let start = segment.find(marker)? + marker.len();
    let end = find_json_string_end(&segment[start..])?;
    let decoded = decode_jsonish_string(&segment[start..start + end]);
    let trimmed = decoded.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn find_json_string_end(s: &str) -> Option<usize> {
    let mut escaped = false;
    for (idx, ch) in s.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        match ch {
            '\\' => escaped = true,
            '"' => return Some(idx),
            _ => {}
        }
    }
    None
}

fn decode_jsonish_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            out.push(ch);
            continue;
        }

        match chars.next() {
            Some('"') => out.push('"'),
            Some('\\') => out.push('\\'),
            Some('/') => out.push('/'),
            Some('n') => out.push('\n'),
            Some('r') => out.push('\r'),
            Some('t') => out.push('\t'),
            Some('u') => {
                let hex = chars.by_ref().take(4).collect::<String>();
                if let Ok(code) = u32::from_str_radix(&hex, 16) {
                    if let Some(decoded) = char::from_u32(code) {
                        out.push(decoded);
                    }
                }
            }
            Some(other) => out.push(other),
            None => break,
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_basic_video_renderer() {
        let html = r#"
        <script>
        var ytInitialData = {"contents":{"videoRenderer":{
          "videoId":"abc123",
          "thumbnail":{"thumbnails":[{"url":"https://i.ytimg.com/vi/abc123/hqdefault.jpg"}]},
          "title":{"runs":[{"text":"Noir Browser demo"}]},
          "ownerText":{"runs":[{"text":"Noir Channel"}]},
          "publishedTimeText":{"simpleText":"1 day ago"}
        }}};
        </script>
        "#;

        let cards = extract_video_cards(html, 10);
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].video_id, "abc123");
        assert_eq!(cards[0].title, "Noir Browser demo");
        assert_eq!(cards[0].channel.as_deref(), Some("Noir Channel"));
        assert!(cards[0].thumbnail_url.as_ref().unwrap().contains("ytimg.com"));
    }

    #[test]
    fn enhances_youtube_page_document() {
        let mut doc = PageDocument::new("https://www.youtube.com/results?search_query=noir");
        enhance_page_document(&mut doc, r#"{"videoId":"abc","title":{"simpleText":"Title"},"thumbnail":{"thumbnails":[{"url":"https://i.ytimg.com/vi/abc/default.jpg"}]}}"#);
        assert_eq!(doc.links.len(), 1);
        assert_eq!(doc.image_blocks.len(), 1);
        assert_eq!(doc.video_blocks.len(), 1);
        assert!(doc.text_blocks.iter().any(|block| block.text == "Title"));
    }

    #[test]
    fn shows_lite_message_when_no_metadata_exists() {
        let mut doc = PageDocument::new("https://www.youtube.com/");
        doc.text_blocks.push(TextBlock {
            text: "Acerca de".to_string(),
            tag: "a".to_string(),
            font_size: 14.0,
            bold: false,
            link: None,
            indent_level: 0,
            attributes: Default::default(),
        });
        enhance_page_document(&mut doc, "<html><body>YouTube footer</body></html>");
        assert_eq!(doc.title, "YouTube Lite");
        assert!(doc.text_blocks.iter().any(|block| block.text == "Noir YouTube Lite"));
        assert!(!doc.text_blocks.iter().any(|block| block.text == "Acerca de"));
    }
}
