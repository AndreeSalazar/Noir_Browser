use crate::parsers::dom_tree::DomNode;
use crate::parsers::html_elements::HtmlTag;
use std::collections::HashMap;
use url::Url;

const MEDIA_EXTENSIONS: &[&str] = &[
    ".mp3", ".m4a", ".aac", ".opus", ".ogg", ".oga", ".wav", ".flac", ".mp4", ".m4v", ".webm",
    ".mov", ".m3u8", ".mpd",
];

#[derive(Clone, Debug, Default)]
pub struct MediaReport {
    pub items: Vec<MediaItem>,
    pub streaming_app_hint: Option<String>,
}

#[derive(Clone, Debug)]
pub struct MediaItem {
    pub kind: MediaKind,
    pub url: Option<String>,
    pub mime_type: Option<String>,
    pub label: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MediaKind {
    Audio,
    Video,
    Source,
    Track,
    Embed,
    StreamingApp,
}

impl MediaReport {
    pub fn is_empty(&self) -> bool {
        self.items.is_empty() && self.streaming_app_hint.is_none()
    }

    pub fn summary(&self) -> Option<String> {
        if self.is_empty() {
            return None;
        }

        let audio = self
            .items
            .iter()
            .filter(|item| item.kind == MediaKind::Audio)
            .count();
        let video = self
            .items
            .iter()
            .filter(|item| item.kind == MediaKind::Video)
            .count();
        let source = self
            .items
            .iter()
            .filter(|item| matches!(item.kind, MediaKind::Source | MediaKind::Embed))
            .count();

        let mut parts = Vec::new();
        if audio > 0 {
            parts.push(format!("{audio} audio"));
        }
        if video > 0 {
            parts.push(format!("{video} video"));
        }
        if source > 0 {
            parts.push(format!("{source} fuente"));
        }

        let resolved_urls = self.items.iter().filter(|item| item.url.is_some()).count();
        let typed_sources = self
            .items
            .iter()
            .filter(|item| item.mime_type.is_some())
            .count();
        let labeled_sources = self
            .items
            .iter()
            .filter(|item| item.label.is_some())
            .count();
        if resolved_urls > 0 {
            parts.push(format!("{resolved_urls} URL"));
        }
        if typed_sources > 0 {
            parts.push(format!("{typed_sources} MIME"));
        }
        if labeled_sources > 0 {
            parts.push(format!("{labeled_sources} etiqueta"));
        }

        if let Some(hint) = &self.streaming_app_hint {
            parts.push(hint.clone());
        }

        Some(format!("Media detectada: {}", parts.join(" / ")))
    }
}

pub fn discover_media(dom: &[DomNode], page_url: &str) -> MediaReport {
    let base_url = Url::parse(page_url).ok();
    let mut report = MediaReport::default();

    if let Some(host) = base_url.as_ref().and_then(Url::host_str) {
        if host.contains("youtube.com") || host.contains("youtu.be") {
            report.streaming_app_hint = Some("YouTube/MSE".to_string());
            report.items.push(MediaItem {
                kind: MediaKind::StreamingApp,
                url: Some(page_url.to_string()),
                mime_type: None,
                label: Some("Streaming app".to_string()),
            });
        }
    }

    collect_media(dom, base_url.as_ref(), &mut report);
    report
}

fn collect_media(nodes: &[DomNode], base_url: Option<&Url>, report: &mut MediaReport) {
    for node in nodes {
        let DomNode::Element {
            tag,
            attributes,
            children,
        } = node
        else {
            continue;
        };

        match tag {
            HtmlTag::Audio => push_media_item(report, MediaKind::Audio, attributes, base_url),
            HtmlTag::Video => push_media_item(report, MediaKind::Video, attributes, base_url),
            HtmlTag::Source => push_media_item(report, MediaKind::Source, attributes, base_url),
            HtmlTag::Track => push_media_item(report, MediaKind::Track, attributes, base_url),
            HtmlTag::Embed | HtmlTag::Iframe | HtmlTag::Object => {
                push_media_item(report, MediaKind::Embed, attributes, base_url)
            }
            HtmlTag::A => {
                if let Some(href) = attributes.get("href") {
                    if looks_like_media_url(href) {
                        report.items.push(MediaItem {
                            kind: infer_media_kind(href),
                            url: resolve_url(base_url, href),
                            mime_type: None,
                            label: attributes.get("title").cloned(),
                        });
                    }
                }
            }
            _ => {}
        }

        collect_media(children, base_url, report);
    }
}

fn push_media_item(
    report: &mut MediaReport,
    kind: MediaKind,
    attributes: &HashMap<String, String>,
    base_url: Option<&Url>,
) {
    let src = attributes
        .get("src")
        .or_else(|| attributes.get("data-src"))
        .or_else(|| attributes.get("href"));
    let mime_type = attributes.get("type").cloned();

    if src.is_none() && mime_type.is_none() && !matches!(kind, MediaKind::Audio | MediaKind::Video)
    {
        return;
    }

    report.items.push(MediaItem {
        kind,
        url: src.and_then(|url| resolve_url(base_url, url)),
        mime_type,
        label: attributes
            .get("aria-label")
            .or_else(|| attributes.get("title"))
            .cloned(),
    });
}

fn resolve_url(base_url: Option<&Url>, url: &str) -> Option<String> {
    if url.starts_with("data:") || url.starts_with("blob:") {
        return Some(url.to_string());
    }

    match base_url {
        Some(base) => base.join(url).ok().map(|joined| joined.to_string()),
        None => Some(url.to_string()),
    }
}

fn looks_like_media_url(url: &str) -> bool {
    let lower = url.to_ascii_lowercase();
    MEDIA_EXTENSIONS.iter().any(|ext| lower.contains(ext))
}

fn infer_media_kind(url: &str) -> MediaKind {
    let lower = url.to_ascii_lowercase();
    if [
        ".mp3", ".m4a", ".aac", ".opus", ".ogg", ".oga", ".wav", ".flac",
    ]
    .iter()
    .any(|ext| lower.contains(ext))
    {
        MediaKind::Audio
    } else {
        MediaKind::Video
    }
}
