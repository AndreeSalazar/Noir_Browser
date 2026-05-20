use super::{normalize_text, FragmentLayout, LayoutFragment, TextFragment};
use crate::parsers::dom_tree::DomNode;
use crate::parsers::html_elements::HtmlTag;
use serde_json::Value;
use url::Url;

#[derive(Default)]
struct PageMetadata {
    title: Option<String>,
    description: Option<String>,
    site_name: Option<String>,
    canonical_url: Option<String>,
}

#[derive(Clone, Debug)]
struct VideoCard {
    title: String,
    url: String,
    subtitle: Option<String>,
    duration: Option<String>,
}

#[derive(Clone, Debug)]
struct PlayerShell {
    title: Option<String>,
    author: Option<String>,
    duration: Option<String>,
    views: Option<String>,
    status: Option<String>,
    direct_streams: Vec<StreamLink>,
    protected_formats: usize,
}

#[derive(Clone, Debug)]
struct StreamLink {
    label: String,
    url: String,
}

pub(super) fn append_app_shell_fallback(
    dom: &[DomNode],
    raw_html: &str,
    page_url: &str,
    fragments: &mut Vec<LayoutFragment>,
    text_color: [f32; 4],
) {
    let visible_fragments = fragments
        .iter()
        .filter(|fragment| {
            if let LayoutFragment::Text(t) = fragment {
                t.px_size >= 15.0 && t.text.len() > 3
            } else {
                false
            }
        })
        .count();
    if visible_fragments >= 3 {
        return;
    }

    let metadata = collect_page_metadata(dom);
    let mut added = 0;
    let is_youtube_watch = is_youtube_watch_shell(page_url, raw_html);

    if let Some(title) = metadata.title.as_deref().filter(|title| !title.is_empty()) {
        push_fallback_fragment(fragments, title, 30.0, true, 38.0, 8.0, text_color, true);
        added += 1;
    }

    if let Some(description) = metadata
        .description
        .as_deref()
        .filter(|description| !description.is_empty())
    {
        push_fallback_fragment(
            fragments,
            description,
            16.0,
            false,
            23.0,
            10.0,
            text_color,
            true,
        );
        added += 1;
    }

    if let Some(player) = extract_embedded_player_shell(raw_html) {
        push_player_shell_fragments(fragments, player, text_color);
        added += 1;
    }

    let video_cards = extract_embedded_video_cards(raw_html, 12);
    let has_video_cards = !video_cards.is_empty();
    if has_video_cards {
        push_fallback_fragment(
            fragments,
            "Videos detectados",
            20.0,
            true,
            28.0,
            8.0,
            text_color,
            true,
        );
        for video in video_cards {
            push_video_card_fragment(fragments, video);
        }
        added += 1;
    }

    if !has_video_cards && is_youtube_home_shell(page_url, raw_html) {
        push_fallback_fragment(
            fragments,
            "YouTube no envio videos en el HTML inicial de la portada. Usa una busqueda para cargar tarjetas ligeras:",
            15.0,
            false,
            22.0,
            6.0,
            text_color,
            true,
        );
        for (label, url) in [
            (
                "Buscar videos de musica",
                "https://www.youtube.com/results?search_query=musica",
            ),
            (
                "Buscar videos de programacion",
                "https://www.youtube.com/results?search_query=programacion",
            ),
            (
                "Buscar Rust programming",
                "https://www.youtube.com/results?search_query=rust+programming",
            ),
        ] {
            push_link_fragment(fragments, label, url);
        }
        added += 1;
    }

    let app_texts = extract_embedded_app_text(raw_html, 10, &metadata);
    if !app_texts.is_empty() && !is_youtube_home_shell(page_url, raw_html) && !is_youtube_watch {
        let source = metadata.site_name.as_deref().unwrap_or("aplicacion web");
        push_fallback_fragment(
            fragments,
            &format!("Vista ligera de {source}"),
            18.0,
            true,
            26.0,
            6.0,
            text_color,
            true,
        );
        for text in app_texts {
            push_fallback_fragment(fragments, &text, 15.0, false, 22.0, 4.0, text_color, true);
        }
        added += 1;
    }

    if added == 0 {
        push_fallback_fragment(
            fragments,
            "Aplicacion web detectada: el HTML inicial no trae contenido visible suficiente.",
            16.0,
            false,
            23.0,
            8.0,
            text_color,
            true,
        );
    }

    if let Some(canonical_url) = metadata.canonical_url {
        push_fallback_fragment(
            fragments,
            &canonical_url,
            13.0,
            false,
            19.0,
            4.0,
            [0.478, 0.635, 0.968, 1.0],
            true,
        );
    }
}

fn push_player_shell_fragments(
    fragments: &mut Vec<LayoutFragment>,
    player: PlayerShell,
    text_color: [f32; 4],
) {
    push_fallback_fragment(
        fragments,
        "Reproductor ligero",
        20.0,
        true,
        28.0,
        8.0,
        text_color,
        true,
    );

    if let Some(title) = player.title {
        push_fallback_fragment(fragments, &title, 17.0, true, 25.0, 5.0, text_color, true);
    }

    let mut details = Vec::new();
    if let Some(author) = player.author {
        details.push(author);
    }
    if let Some(duration) = player.duration {
        details.push(duration);
    }
    if let Some(views) = player.views {
        details.push(format!("{views} vistas"));
    }
    if let Some(status) = player.status {
        details.push(status);
    }
    if !details.is_empty() {
        push_fallback_fragment(
            fragments,
            &details.join(" / "),
            14.0,
            false,
            21.0,
            8.0,
            text_color,
            true,
        );
    }

    if player.direct_streams.is_empty() {
        push_fallback_fragment(
            fragments,
            &format!(
                "{} formatos detectados; requieren descifrar el player JS de YouTube para URL directa.",
                player.protected_formats
            ),
            14.0,
            false,
            21.0,
            6.0,
            text_color,
            true,
        );
    } else {
        for stream in player.direct_streams {
            push_link_fragment(
                fragments,
                &format!("Stream directo {}", stream.label),
                &stream.url,
            );
        }
    }
}

fn push_link_fragment(fragments: &mut Vec<LayoutFragment>, text: &str, href: &str) {
    fragments.push(LayoutFragment::Text(TextFragment::new_text(
        text.to_string(),
        15.0,
        false,
        22.0,
        4.0,
        true,
        FragmentLayout {
            max_width: Some("860px".to_string()),
            ..FragmentLayout::default()
        },
        [0.478, 0.635, 0.968, 1.0],
        Some(href.to_string()),
    )));
}

fn is_youtube_home_shell(page_url: &str, raw_html: &str) -> bool {
    if let Ok(url) = Url::parse(page_url) {
        if let Some(host) = url.host_str() {
            if host.contains("youtube.com") {
                let path = url.path();
                return (path == "/" || path.is_empty() || path.starts_with("/feed") || path.starts_with("/results"))
                    && !raw_html.contains("\"videoId\"")
                    && !url.query().unwrap_or("").contains("v=");
            }
        }
    }
    false
}

fn is_youtube_watch_shell(page_url: &str, raw_html: &str) -> bool {
    Url::parse(page_url).ok().is_some_and(|url| {
        url.host_str()
            .is_some_and(|host| host.contains("youtube.com"))
            && url.path().contains("watch")
    }) || raw_html.contains("ytInitialPlayerResponse")
}

fn push_video_card_fragment(fragments: &mut Vec<LayoutFragment>, video: VideoCard) {
    let mut text = video.title;
    let mut details = Vec::new();
    if let Some(duration) = video.duration.filter(|value| !value.is_empty()) {
        details.push(duration);
    }
    if let Some(subtitle) = video.subtitle.filter(|value| !value.is_empty()) {
        details.push(subtitle);
    }
    if !details.is_empty() {
        text.push_str(" - ");
        text.push_str(&details.join(" / "));
    }

    fragments.push(LayoutFragment::Text(TextFragment::new_text(
        normalize_text(&text),
        15.0,
        false,
        22.0,
        5.0,
        true,
        FragmentLayout {
            max_width: Some("920px".to_string()),
            ..FragmentLayout::default()
        },
        [0.478, 0.635, 0.968, 1.0],
        Some(video.url),
    )));
}

fn push_fallback_fragment(
    fragments: &mut Vec<LayoutFragment>,
    text: &str,
    px_size: f32,
    is_bold: bool,
    line_height: f32,
    margin_after: f32,
    color: [f32; 4],
    line_break_after: bool,
) {
    let text = normalize_text(text);
    if text.len() <= 2 {
        return;
    }

    fragments.push(LayoutFragment::Text(TextFragment::new_text(
        text,
        px_size,
        is_bold,
        line_height,
        margin_after,
        line_break_after,
        FragmentLayout {
            max_width: Some("860px".to_string()),
            ..FragmentLayout::default()
        },
        color,
        None,
    )));
}

fn collect_page_metadata(nodes: &[DomNode]) -> PageMetadata {
    let mut metadata = PageMetadata::default();
    collect_page_metadata_inner(nodes, &mut metadata);
    metadata
}

fn collect_page_metadata_inner(nodes: &[DomNode], metadata: &mut PageMetadata) {
    for node in nodes {
        let DomNode::Element {
            tag,
            attributes,
            children,
        } = node
        else {
            continue;
        };

        if matches!(tag, HtmlTag::Custom(name) if name == "title") {
            let title = collect_node_text(children);
            if !title.trim().is_empty() {
                metadata.title = Some(normalize_text(&title));
            }
        }

        if matches!(tag, HtmlTag::Custom(name) if name == "meta") {
            let key = attributes
                .get("name")
                .or_else(|| attributes.get("property"))
                .map(|value| value.to_ascii_lowercase());
            if let (Some(key), Some(content)) = (key, attributes.get("content")) {
                let content = normalize_text(content);
                match key.as_str() {
                    "description" | "og:description" | "twitter:description"
                        if metadata.description.is_none() =>
                    {
                        metadata.description = Some(content)
                    }
                    "og:title" | "twitter:title" | "title" if metadata.title.is_none() => {
                        metadata.title = Some(content)
                    }
                    "og:site_name" | "application-name" if metadata.site_name.is_none() => {
                        metadata.site_name = Some(content)
                    }
                    _ => {}
                }
            }
        }

        if matches!(tag, HtmlTag::Custom(name) if name == "link")
            && attributes
                .get("rel")
                .is_some_and(|rel| rel.to_ascii_lowercase().contains("canonical"))
            && metadata.canonical_url.is_none()
        {
            metadata.canonical_url = attributes.get("href").cloned();
        }

        collect_page_metadata_inner(children, metadata);
    }
}

fn collect_node_text(nodes: &[DomNode]) -> String {
    let mut out = String::new();
    for node in nodes {
        match node {
            DomNode::Text(text) => {
                out.push_str(text);
                out.push(' ');
            }
            DomNode::Element { children, .. } => out.push_str(&collect_node_text(children)),
        }
    }
    out
}

fn extract_embedded_app_text(raw_html: &str, limit: usize, metadata: &PageMetadata) -> Vec<String> {
    let mut out = Vec::new();
    for marker in [
        "\"label\":\"",
        "\"simpleText\":\"",
        "\"text\":\"",
        "\"title\":\"",
        "\"ariaLabel\":\"",
    ] {
        collect_json_string_values(raw_html, marker, limit, metadata, &mut out);
        if out.len() >= limit {
            break;
        }
    }
    out
}

fn extract_embedded_video_cards(raw_html: &str, limit: usize) -> Vec<VideoCard> {
    let Some(json) = extract_assigned_json(raw_html, "ytInitialData") else {
        return Vec::new();
    };
    let Ok(value) = serde_json::from_str::<Value>(&json) else {
        return Vec::new();
    };

    let mut videos = Vec::new();
    collect_video_cards(&value, limit, &mut videos);
    videos
}

fn extract_embedded_player_shell(raw_html: &str) -> Option<PlayerShell> {
    let json = extract_assigned_json(raw_html, "ytInitialPlayerResponse")?;
    let value = serde_json::from_str::<Value>(&json).ok()?;
    let details = value.get("videoDetails");
    let streaming = value.get("streamingData");
    let playability = value.get("playabilityStatus");

    let formats = streaming
        .and_then(|streaming| streaming.get("formats"))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .chain(
            streaming
                .and_then(|streaming| streaming.get("adaptiveFormats"))
                .and_then(Value::as_array)
                .into_iter()
                .flatten(),
        )
        .collect::<Vec<_>>();

    let mut direct_streams = Vec::new();
    for format in &formats {
        let Some(url) = format.get("url").and_then(Value::as_str) else {
            continue;
        };
        let label = format_label(format);
        if !direct_streams
            .iter()
            .any(|stream: &StreamLink| stream.url == url)
        {
            direct_streams.push(StreamLink {
                label,
                url: url.to_string(),
            });
        }
        if direct_streams.len() >= 4 {
            break;
        }
    }

    Some(PlayerShell {
        title: details
            .and_then(|details| details.get("title"))
            .and_then(Value::as_str)
            .map(normalize_text),
        author: details
            .and_then(|details| details.get("author"))
            .and_then(Value::as_str)
            .map(normalize_text),
        duration: details
            .and_then(|details| details.get("lengthSeconds"))
            .and_then(Value::as_str)
            .and_then(format_duration),
        views: details
            .and_then(|details| details.get("viewCount"))
            .and_then(Value::as_str)
            .map(format_number_grouped),
        status: playability
            .and_then(|status| status.get("status"))
            .and_then(Value::as_str)
            .map(str::to_string),
        direct_streams,
        protected_formats: formats.len(),
    })
}

fn format_label(format: &Value) -> String {
    let mime = format
        .get("mimeType")
        .and_then(Value::as_str)
        .and_then(|mime| mime.split(';').next())
        .unwrap_or("media");
    let quality = format
        .get("qualityLabel")
        .or_else(|| format.get("audioQuality"))
        .or_else(|| format.get("quality"))
        .and_then(Value::as_str)
        .unwrap_or("auto");
    format!("{quality} {mime}")
}

fn format_duration(seconds: &str) -> Option<String> {
    let total = seconds.parse::<u64>().ok()?;
    let hours = total / 3600;
    let minutes = (total % 3600) / 60;
    let seconds = total % 60;
    if hours > 0 {
        Some(format!("{hours}:{minutes:02}:{seconds:02}"))
    } else {
        Some(format!("{minutes}:{seconds:02}"))
    }
}

fn format_number_grouped(value: &str) -> String {
    let mut out = String::new();
    for (idx, ch) in value.chars().rev().enumerate() {
        if idx > 0 && idx % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    out.chars().rev().collect()
}

fn collect_video_cards(value: &Value, limit: usize, out: &mut Vec<VideoCard>) {
    if out.len() >= limit {
        return;
    }

    match value {
        Value::Object(map) => {
            for key in [
                "videoRenderer",
                "compactVideoRenderer",
                "gridVideoRenderer",
                "playlistVideoRenderer",
                "reelItemRenderer",
            ] {
                if let Some(renderer) = map.get(key) {
                    if let Some(video) = video_card_from_renderer(renderer) {
                        if !out.iter().any(|existing| existing.url == video.url) {
                            out.push(video);
                            if out.len() >= limit {
                                return;
                            }
                        }
                    }
                }
            }

            for child in map.values() {
                collect_video_cards(child, limit, out);
                if out.len() >= limit {
                    return;
                }
            }
        }
        Value::Array(items) => {
            for child in items {
                collect_video_cards(child, limit, out);
                if out.len() >= limit {
                    return;
                }
            }
        }
        _ => {}
    }
}

fn video_card_from_renderer(renderer: &Value) -> Option<VideoCard> {
    let video_id = renderer.get("videoId")?.as_str()?;
    if video_id.len() < 6 {
        return None;
    }

    let title = text_from_json_text(renderer.get("title")?)
        .or_else(|| renderer.get("headline").and_then(text_from_json_text))
        .filter(|title| is_useful_video_title(title))?;
    let subtitle = renderer
        .get("ownerText")
        .or_else(|| renderer.get("longBylineText"))
        .or_else(|| renderer.get("shortBylineText"))
        .and_then(text_from_json_text);
    let duration = renderer
        .get("lengthText")
        .or_else(|| renderer.get("thumbnailOverlayTimeStatusRenderer"))
        .and_then(text_from_json_text);

    Some(VideoCard {
        title,
        url: format!("https://www.youtube.com/watch?v={video_id}"),
        subtitle,
        duration,
    })
}

fn text_from_json_text(value: &Value) -> Option<String> {
    if let Some(text) = value.get("simpleText").and_then(Value::as_str) {
        return Some(normalize_text(text));
    }

    let runs = value.get("runs")?.as_array()?;
    let text = runs
        .iter()
        .filter_map(|run| run.get("text").and_then(Value::as_str))
        .collect::<Vec<_>>()
        .join("");
    if text.trim().is_empty() {
        None
    } else {
        Some(normalize_text(&text))
    }
}

fn is_useful_video_title(title: &str) -> bool {
    let lower = title.to_ascii_lowercase();
    title.len() >= 3
        && !is_noisy_app_text(&lower)
        && !lower.contains("youtube")
        && !lower.contains("busca algo")
}

fn extract_assigned_json(raw_html: &str, variable: &str) -> Option<String> {
    let marker = format!("{variable} = ");
    let start = raw_html.find(&marker)? + marker.len();
    let json_start = raw_html[start..].find('{')? + start;
    extract_balanced_json_object(&raw_html[json_start..])
}

fn extract_balanced_json_object(text: &str) -> Option<String> {
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    for (idx, ch) in text.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }

        match ch {
            '"' => in_string = true,
            '{' => depth += 1,
            '}' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(text[..=idx].to_string());
                }
            }
            _ => {}
        }
    }
    None
}

fn collect_json_string_values(
    raw_html: &str,
    marker: &str,
    limit: usize,
    metadata: &PageMetadata,
    out: &mut Vec<String>,
) {
    let mut start = 0;
    while out.len() < limit {
        let Some(pos) = raw_html[start..].find(marker) else {
            break;
        };
        let value_start = start + pos + marker.len();
        let Some((value, consumed)) = read_json_string_fragment(&raw_html[value_start..]) else {
            start = value_start;
            continue;
        };
        start = value_start + consumed;

        let value = normalize_text(&decode_json_text(&value));
        if is_useful_app_text(&value)
            && !matches_metadata_text(&value, metadata)
            && !out
                .iter()
                .any(|existing| existing.eq_ignore_ascii_case(&value))
        {
            out.push(value);
        }
    }
}

fn matches_metadata_text(text: &str, metadata: &PageMetadata) -> bool {
    let text = text.trim();
    [metadata.title.as_deref(), metadata.description.as_deref()]
        .into_iter()
        .flatten()
        .any(|metadata_text| {
            metadata_text.eq_ignore_ascii_case(text)
                || metadata_text
                    .to_ascii_lowercase()
                    .contains(&text.to_ascii_lowercase())
        })
}

fn read_json_string_fragment(text: &str) -> Option<(String, usize)> {
    let mut value = String::new();
    let mut escaped = false;
    for (idx, ch) in text.char_indices() {
        if escaped {
            value.push('\\');
            value.push(ch);
            escaped = false;
            continue;
        }
        match ch {
            '\\' => escaped = true,
            '"' => return Some((value, idx + 1)),
            _ => value.push(ch),
        }
    }
    None
}

fn decode_json_text(text: &str) -> String {
    let mut out = String::new();
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            out.push(ch);
            continue;
        }

        match chars.next() {
            Some('n') | Some('r') | Some('t') => out.push(' '),
            Some('"') => out.push('"'),
            Some('\\') => out.push('\\'),
            Some('/') => out.push('/'),
            Some('u') => {
                let hex = chars.by_ref().take(4).collect::<String>();
                if let Ok(code) = u32::from_str_radix(&hex, 16) {
                    if let Some(decoded) = char::from_u32(code) {
                        out.push(decoded);
                    }
                }
            }
            Some(other) => out.push(other),
            None => {}
        }
    }
    out
}

fn is_useful_app_text(text: &str) -> bool {
    let trimmed = text.trim();
    if trimmed.len() < 3 || trimmed.len() > 120 {
        return false;
    }
    let lower = trimmed.to_ascii_lowercase();
    if lower.starts_with("http")
        || lower.contains(".js")
        || lower.contains(".css")
        || lower.contains("sprite")
        || lower.contains("endpoint")
        || is_noisy_app_text(&lower)
    {
        return false;
    }
    let words = trimmed.split_whitespace().count();
    if words < 2 && trimmed.chars().count() < 14 {
        return false;
    }
    let letters = trimmed.chars().filter(|ch| ch.is_alphabetic()).count();
    letters >= 2
}

fn is_noisy_app_text(lower: &str) -> bool {
    const NOISE_PARTS: &[&str] = &[
        "acceder",
        "activar o desactivar",
        "adelantar",
        "aria",
        "atajo",
        "aumentar velocidad",
        "avanzar",
        "borrar busqueda",
        "borrar búsqueda",
        "cancelar",
        "capitulo",
        "capítulo",
        "combinaciones de teclas",
        "configuracion",
        "configuración",
        "cuadro anterior",
        "desplazarse",
        "disminuir velocidad",
        "fuente",
        "niveles de opacidad",
        "pantalla completa",
        "pausa",
        "principal",
        "realiza busquedas con la voz",
        "realiza búsquedas con la voz",
        "reproduccion",
        "reproducción",
        "retroceder",
        "saltar al",
        "siguiente cuadro",
        "siguiente video",
        "tecla",
        "video anterior",
        "videos esfericos",
        "videos esfÃ©ricos",
    ];

    if NOISE_PARTS.iter().any(|part| lower.contains(part)) {
        return true;
    }

    matches!(
        lower.trim(),
        "buscar" | "coma" | "general" | "menos" | "punto" | "visitar la fuente"
    )
}
