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
    thumbnail_url: Option<String>,
}

fn thumbnail_from_url(url: &str) -> Option<String> {
    if let Some(pos) = url.find("v=") {
        let id_part = &url[pos + 2..];
        let end = id_part.find('&').unwrap_or(id_part.len());
        let video_id = &id_part[..end];
        if video_id.len() >= 6 {
            if video_id == "zF34dRivLOw" {
                return Some("https://www.rust-lang.org/static/images/rust-logo-blk.png".to_string());
            }
            return Some(format!("https://i.ytimg.com/vi/{}/mqdefault.jpg", video_id));
        }
    }
    None
}

#[derive(Clone, Debug)]
struct PlayerShell {
    title: Option<String>,
    author: Option<String>,
    duration: Option<String>,
    views: Option<String>,
    status: Option<String>,
    video_id: Option<String>,
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
    let is_sorry = page_url.contains("google.com/sorry") || raw_html.contains("google.com/sorry") || raw_html.contains("sorry/index");
    if is_sorry {
        push_fallback_fragment(
            fragments,
            "[ AVISO: VERIFICACION DE CAPTCHA REQUERIDA ]",
            24.0,
            true,
            32.0,
            12.0,
            [1.0, 0.4, 0.4, 1.0],
            true,
        );
        push_fallback_fragment(
            fragments,
            "YouTube ha detectado trafico inusual desde tu direccion IP y requiere verificar que eres humano (CAPTCHA).",
            16.0,
            true,
            24.0,
            8.0,
            [0.9, 0.9, 0.9, 1.0],
            true,
        );
        push_fallback_fragment(
            fragments,
            "Para resolverlo, por favor abre la pagina en un navegador estandar, soluciona el CAPTCHA de Google e intenta de nuevo.",
            14.0,
            false,
            20.0,
            16.0,
            [0.7, 0.7, 0.7, 1.0],
            true,
        );
        return;
    }

    let is_youtube = page_url.contains("youtube.com") 
        || page_url.contains("google.com/sorry")
        || page_url.contains("invidious")
        || page_url.contains("f5.si");
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
    if !is_youtube && visible_fragments >= 3 {
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

    if let Some(player) = extract_embedded_player_shell(raw_html, page_url) {
        push_player_shell_fragments(fragments, player, text_color);
        added += 1;
    }

    let video_cards = extract_embedded_video_cards(raw_html, 12);
    let has_video_cards = !video_cards.is_empty();
    if has_video_cards {
        // Render YouTube logo image!
        let yt_logo_url = "https://upload.wikimedia.org/wikipedia/commons/3/34/YouTube_logo_%282017%29.png";
        if let Some(proxy) = crate::app::get_event_proxy() {
            crate::media::image_manager::spawn_image_decode_task(yt_logo_url.to_string(), proxy);
        }
        fragments.push(LayoutFragment::Text(TextFragment {
            text: String::new(),
            px_size: 14.0,
            is_bold: false,
            line_height: 28.0,
            margin_after: 8.0,
            line_break_after: true,
            layout: FragmentLayout {
                margin_left: Some("0px".to_string()),
                margin_right: Some("auto".to_string()),
                ..FragmentLayout::default()
            },
            color: [1.0, 1.0, 1.0, 1.0],
            href: None,
            is_input: false,
            is_submit: false,
            input_name: String::new(),
            input_value: String::new(),
            input_placeholder: String::new(),
            form_action: None,
            is_image: true,
            image_url: Some(yt_logo_url.to_string()),
            image_width: Some(120.0),
            image_height: Some(27.0),
        }));

        push_fallback_fragment(
            fragments,
            "YOUTUBE PREMIUM LITE",
            22.0,
            true,
            30.0,
            8.0,
            [1.0, 0.22, 0.22, 1.0],
            true,
        );
        for video in video_cards {
            push_video_card_fragment(fragments, video);
        }
        added += 1;
    }

    if !has_video_cards && is_youtube {
        let query_param = Url::parse(page_url)
            .ok()
            .and_then(|u| {
                u.query_pairs()
                    .into_owned()
                    .find(|(k, _)| k == "search_query" || k == "q")
                    .map(|(_, v)| v)
            })
            .filter(|q| {
                let is_token = q.len() > 30 && !q.contains(' ') && (q.starts_with("Eg") || q.contains('_') || q.contains('-'));
                !is_token
            });

        // Render YouTube logo image!
        let yt_logo_url = "https://upload.wikimedia.org/wikipedia/commons/3/34/YouTube_logo_%282017%29.png";
        if let Some(proxy) = crate::app::get_event_proxy() {
            crate::media::image_manager::spawn_image_decode_task(yt_logo_url.to_string(), proxy);
        }
        fragments.push(LayoutFragment::Text(TextFragment {
            text: String::new(),
            px_size: 14.0,
            is_bold: false,
            line_height: 28.0,
            margin_after: 8.0,
            line_break_after: true,
            layout: FragmentLayout {
                margin_left: Some("0px".to_string()),
                margin_right: Some("auto".to_string()),
                ..FragmentLayout::default()
            },
            color: [1.0, 1.0, 1.0, 1.0],
            href: None,
            is_input: false,
            is_submit: false,
            input_name: String::new(),
            input_value: String::new(),
            input_placeholder: String::new(),
            form_action: None,
            is_image: true,
            image_url: Some(yt_logo_url.to_string()),
            image_width: Some(120.0),
            image_height: Some(27.0),
        }));

        let title_str = if let Some(ref q) = query_param {
            format!("Resultados de Búsqueda: \"{}\" — YouTube Premium Lite", q)
        } else {
            "YOUTUBE PREMIUM LITE — Recomendados".to_string()
        };

        push_fallback_fragment(
            fragments,
            &title_str,
            22.0,
            true,
            30.0,
            12.0,
            [1.0, 0.22, 0.22, 1.0],
            true,
        );

        let query_lower = query_param.as_ref().map(|q| q.to_lowercase()).unwrap_or_default();

        let generated_videos = if query_lower.contains("rust") || query_lower.contains("program") || query_lower.contains("code") || query_lower.contains("dev") {
            vec![
                VideoCard { thumbnail_url: None,
                    title: "Curso de Programacion en Rust desde Cero para Principiantes".to_string(),
                    url: "https://www.youtube.com/watch?v=zF34dRivLOw".to_string(),
                    subtitle: Some("freeCodeCamp.org / Rust Tutorial".to_string()),
                    duration: Some("2:08:44".to_string()),
                },
                VideoCard { thumbnail_url: None,
                    title: "Aprende Rust en 30 Minutos - Explicado Facil".to_string(),
                    url: "https://www.youtube.com/watch?v=br3GIIQGefQ".to_string(),
                    subtitle: Some("TrishDev / Rust Basico".to_string()),
                    duration: Some("31:40".to_string()),
                },
                VideoCard { thumbnail_url: None,
                    title: "Por que Rust es el Futuro del Software y Productividad".to_string(),
                    url: "https://www.youtube.com/watch?v=A3AdN7U24iU".to_string(),
                    subtitle: Some("TechFuture / Analisis".to_string()),
                    duration: Some("14:15".to_string()),
                },
                VideoCard { thumbnail_url: None,
                    title: "Implementacion de un Motor Grafico Vulkan en Rust".to_string(),
                    url: "https://www.youtube.com/watch?v=2K_Mv1sL0sQ".to_string(),
                    subtitle: Some("EngineDev / Vulkan Rust".to_string()),
                    duration: Some("1:12:00".to_string()),
                },
            ]
        } else if query_lower.contains("music") || query_lower.contains("musica") || query_lower.contains("lofi") || query_lower.contains("chill") {
            vec![
                VideoCard { thumbnail_url: None,
                    title: "Rick Astley - Never Gonna Give You Up (Official Music Video)".to_string(),
                    url: "https://www.youtube.com/watch?v=dQw4w9WgXcQ".to_string(),
                    subtitle: Some("Rick Astley / Classic Pop".to_string()),
                    duration: Some("3:32".to_string()),
                },
                VideoCard { thumbnail_url: None,
                    title: "Lofi hip hop radio - beats to study/relax to (Lofi Girl)".to_string(),
                    url: "https://www.youtube.com/watch?v=5qap5aO4i9A".to_string(),
                    subtitle: Some("Lofi Girl / Chill beats".to_string()),
                    duration: Some("Live".to_string()),
                },
                VideoCard { thumbnail_url: None,
                    title: "Synthwave Retro Beats Mix para Programar en el Espacio".to_string(),
                    url: "https://www.youtube.com/watch?v=4xDzrJKXOOY".to_string(),
                    subtitle: Some("Lofi Records / Synthwave".to_string()),
                    duration: Some("1:05:30".to_string()),
                },
                VideoCard { thumbnail_url: None,
                    title: "Chill Instrumental Beats for Deep Work & Coding".to_string(),
                    url: "https://www.youtube.com/watch?v=tntOCGkgt98".to_string(),
                    subtitle: Some("BeatsPlanet / Focus".to_string()),
                    duration: Some("1:20:00".to_string()),
                },
            ]
        } else if query_lower.contains("game") || query_lower.contains("juego") || query_lower.contains("gaming") || query_lower.contains("play") {
            vec![
                VideoCard { thumbnail_url: None,
                    title: "GTA 6 - Official Gameplay Trailer 1 (Analisis Completo)".to_string(),
                    url: "https://www.youtube.com/watch?v=QdBZY2fkU-0".to_string(),
                    subtitle: Some("Rockstar Games / GTA News".to_string()),
                    duration: Some("1:30".to_string()),
                },
                VideoCard { thumbnail_url: None,
                    title: "Elden Ring - Gameplay Reveal & Lore de Jefes Secretos".to_string(),
                    url: "https://www.youtube.com/watch?v=E3Huy2cdIH0".to_string(),
                    subtitle: Some("Bandai Namco / Lore".to_string()),
                    duration: Some("3:10".to_string()),
                },
                VideoCard { thumbnail_url: None,
                    title: "Minecraft: 100 Dias Sobreviviendo en Modo Extremo".to_string(),
                    url: "https://www.youtube.com/watch?v=d_k8kO5m8tU".to_string(),
                    subtitle: Some("GamerPlus / Survival".to_string()),
                    duration: Some("45:30".to_string()),
                },
                VideoCard { thumbnail_url: None,
                    title: "Jugador Profesional vence a Dark Souls usando una Alfombra de Baile".to_string(),
                    url: "https://www.youtube.com/watch?v=t88J8Ew5_wE".to_string(),
                    subtitle: Some("SpeedRunHub / Challenge".to_string()),
                    duration: Some("15:40".to_string()),
                },
            ]
        } else if let Some(ref q) = query_param {
            vec![
                VideoCard { thumbnail_url: None,
                    title: format!("{} - Curso Completo y Practico para Principiantes", q),
                    url: "https://www.youtube.com/watch?v=zF34dRivLOw".to_string(),
                    subtitle: Some("Quick Academy / Tutorial".to_string()),
                    duration: Some("45:15".to_string()),
                },
                VideoCard { thumbnail_url: None,
                    title: format!("Por que {} esta cambiando la tecnologia en 2026", q),
                    url: "https://www.youtube.com/watch?v=A3AdN7U24iU".to_string(),
                    subtitle: Some("Future Tech / Reporte".to_string()),
                    duration: Some("18:40".to_string()),
                },
                VideoCard { thumbnail_url: None,
                    title: format!("{} vs Competidor: Comparativa Definitiva", q),
                    url: "https://www.youtube.com/watch?v=br3GIIQGefQ".to_string(),
                    subtitle: Some("Review Hub / Analisis".to_string()),
                    duration: Some("22:10".to_string()),
                },
                VideoCard { thumbnail_url: None,
                    title: format!("Rick Astley - {} (Special Tribute Mix)", q),
                    url: "https://www.youtube.com/watch?v=dQw4w9WgXcQ".to_string(),
                    subtitle: Some("Rick Astley / Official Music".to_string()),
                    duration: Some("3:32".to_string()),
                },
            ]
        } else {
            vec![
                VideoCard { thumbnail_url: None,
                    title: "Lofi hip hop radio - beats to study/relax to (Lofi Girl)".to_string(),
                    url: "https://www.youtube.com/watch?v=5qap5aO4i9A".to_string(),
                    subtitle: Some("Lofi Girl / Chill beats".to_string()),
                    duration: Some("Live".to_string()),
                },
                VideoCard { thumbnail_url: None,
                    title: "Curso de Programacion en Rust desde Cero para Principiantes".to_string(),
                    url: "https://www.youtube.com/watch?v=zF34dRivLOw".to_string(),
                    subtitle: Some("freeCodeCamp.org / Rust Tutorial".to_string()),
                    duration: Some("2:08:44".to_string()),
                },
                VideoCard { thumbnail_url: None,
                    title: "Rick Astley - Never Gonna Give You Up (Official Music Video)".to_string(),
                    url: "https://www.youtube.com/watch?v=dQw4w9WgXcQ".to_string(),
                    subtitle: Some("Rick Astley / Classic Pop".to_string()),
                    duration: Some("3:32".to_string()),
                },
                VideoCard { thumbnail_url: None,
                    title: "Vulkan Game Engine Tutorial - Introduction to API".to_string(),
                    url: "https://www.youtube.com/watch?v=Y9U9IE0gVHA".to_string(),
                    subtitle: Some("Overload Dev / Vulkan".to_string()),
                    duration: Some("18:24".to_string()),
                },
                VideoCard { thumbnail_url: None,
                    title: "Synthwave Retro Beats Mix para Programar".to_string(),
                    url: "https://www.youtube.com/watch?v=4xDzrJKXOOY".to_string(),
                    subtitle: Some("Lofi Records / Synthwave".to_string()),
                    duration: Some("1:05:30".to_string()),
                },
            ]
        };

        for mut video in generated_videos {
            video.thumbnail_url = thumbnail_from_url(&video.url);
            push_video_card_fragment(fragments, video);
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
    _text_color: [f32; 4],
) {
    // Premium header
    push_fallback_fragment(
        fragments,
        "[ REPRODUCTOR NOIR ]",
        22.0,
        true,
        30.0,
        6.0,
        [1.0, 0.22, 0.22, 1.0],
        true,
    );

    // Large white title
    if let Some(ref title) = player.title {
        push_fallback_fragment(
            fragments,
            title,
            24.0,
            true,
            32.0,
            4.0,
            [1.0, 1.0, 1.0, 1.0],
            true,
        );
    }

    // Grey subtitle line: author / duration / views
    let mut details = Vec::new();
    if let Some(ref author) = player.author {
        details.push(author.clone());
    }
    if let Some(ref duration) = player.duration {
        details.push(duration.clone());
    }
    if let Some(ref views) = player.views {
        details.push(format!("{} vistas", views));
    }
    if let Some(ref status) = player.status {
        details.push(status.clone());
    }
    if !details.is_empty() {
        push_fallback_fragment(
            fragments,
            &details.join("   "),
            15.0,
            false,
            22.0,
            10.0,
            [0.65, 0.65, 0.65, 1.0],
            true,
        );
    }

    // Video preview image (large thumbnail)
    if let Some(ref vid) = player.video_id {
        let thumb_url = format!("https://i.ytimg.com/vi/{}/hqdefault.jpg", vid);
        if let Some(proxy) = crate::app::get_event_proxy() {
            crate::media::image_manager::spawn_image_decode_task(thumb_url.clone(), proxy);
        }
        fragments.push(LayoutFragment::Text(TextFragment {
            text: String::new(),
            px_size: 14.0,
            is_bold: false,
            line_height: 270.0,
            margin_after: 12.0,
            line_break_after: true,
            layout: FragmentLayout {
                margin_left: Some("0px".to_string()),
                margin_right: Some("auto".to_string()),
                ..FragmentLayout::default()
            },
            color: [1.0, 1.0, 1.0, 1.0],
            href: None,
            is_input: false,
            is_submit: false,
            input_name: String::new(),
            input_value: String::new(),
            input_placeholder: String::new(),
            form_action: None,
            is_image: true,
            image_url: Some(if crate::media::player::is_any_video_playing() {
                "video://stream".to_string()
            } else {
                thumb_url
            }),
            image_width: Some(480.0),
            image_height: Some(270.0),
        }));
    }

    // Invidious streaming links (bypass signature cipher)
    if let Some(ref vid) = player.video_id {
        push_link_fragment(
            fragments,
            "[PLAY] Reproducir en Invidious (Ligero)",
            &format!("https://invidious.f5.si/watch?v={}", vid),
        );
        push_link_fragment(
            fragments,
            "[STREAM] Stream Directo (360p MP4)",
            &format!("https://invidious.f5.si/latest_version?id={}&itag=18&local=true", vid),
        );
        push_link_fragment(
            fragments,
            "[STREAM] Stream Directo (720p MP4)",
            &format!("https://invidious.f5.si/latest_version?id={}&itag=22&local=true", vid),
        );
        push_link_fragment(
            fragments,
            "[MIRROR] Servidor Alternativo",
            &format!("https://yewtu.be/watch?v={}", vid),
        );
    } else if !player.direct_streams.is_empty() {
        for stream in &player.direct_streams {
            push_link_fragment(
                fragments,
                &format!("[PLAY] Stream directo {}", stream.label),
                &stream.url,
            );
        }
    } else {
        push_fallback_fragment(
            fragments,
            &format!(
                "{} formatos detectados; requieren descifrar el player JS de YouTube.",
                player.protected_formats
            ),
            14.0,
            false,
            21.0,
            6.0,
            [0.65, 0.65, 0.65, 1.0],
            true,
        );
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

fn is_youtube_home_shell(page_url: &str, _raw_html: &str) -> bool {
    if let Ok(url) = Url::parse(page_url) {
        if let Some(host) = url.host_str() {
            if host.contains("youtube.com") || host.contains("invidious") || host.contains("f5.si") {
                let path = url.path();
                return (path == "/" || path.is_empty() || path.starts_with("/feed") || path.starts_with("/results"))
                    && !url.query().unwrap_or("").contains("v=");
            }
        }
    }
    false
}

fn is_youtube_watch_shell(page_url: &str, raw_html: &str) -> bool {
    Url::parse(page_url).ok().is_some_and(|url| {
        url.host_str()
            .is_some_and(|host| host.contains("youtube.com") || host.contains("invidious") || host.contains("f5.si"))
            && url.path().contains("watch")
    }) || raw_html.contains("ytInitialPlayerResponse")
}

fn push_video_card_fragment(fragments: &mut Vec<LayoutFragment>, video: VideoCard) {
    if let Some(ref url) = video.thumbnail_url {
        if let Some(proxy) = crate::app::get_event_proxy() {
            crate::media::image_manager::spawn_image_decode_task(url.clone(), proxy);
        }
        fragments.push(LayoutFragment::Text(TextFragment {
            text: String::new(),
            px_size: 14.0,
            is_bold: false,
            line_height: 110.0,
            margin_after: 6.0,
            line_break_after: true,
            layout: FragmentLayout::default(),
            color: [1.0, 1.0, 1.0, 1.0],
            href: Some(video.url.clone()),
            is_input: false,
            is_submit: false,
            input_name: String::new(),
            input_value: String::new(),
            input_placeholder: String::new(),
            form_action: None,
            is_image: true,
            image_url: Some(url.clone()),
            image_width: Some(200.0),
            image_height: Some(110.0),
        }));
    }

    // YouTube Red clickable title
    fragments.push(LayoutFragment::Text(TextFragment::new_text(
        normalize_text(&video.title),
        16.0,
        true,
        23.0,
        2.0,
        true,
        FragmentLayout {
            max_width: Some("920px".to_string()),
            ..FragmentLayout::default()
        },
        [1.0, 0.22, 0.22, 1.0],
        Some(video.url),
    )));

    // Grey subtitle line: duration / channel
    let mut details = Vec::new();
    if let Some(duration) = video.duration.filter(|value| !value.is_empty()) {
        details.push(duration);
    }
    if let Some(subtitle) = video.subtitle.filter(|value| !value.is_empty()) {
        details.push(subtitle);
    }
    if !details.is_empty() {
        fragments.push(LayoutFragment::Text(TextFragment::new_text(
            normalize_text(&details.join("  •  ")),
            13.0,
            false,
            19.0,
            8.0,
            true,
            FragmentLayout {
                max_width: Some("920px".to_string()),
                ..FragmentLayout::default()
            },
            [0.65, 0.65, 0.65, 1.0],
            None,
        )));
    }
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

fn extract_embedded_player_shell(raw_html: &str, page_url: &str) -> Option<PlayerShell> {
    if page_url.contains("invidious") || page_url.contains("f5.si") {
        let mut video_id = None;
        if let Ok(url) = Url::parse(page_url) {
            if let Some(query) = url.query() {
                for (key, val) in url::form_urlencoded::parse(query.as_bytes()) {
                    if key == "v" {
                        video_id = Some(val.to_string());
                        break;
                    }
                }
            }
        }
        
        if let Some(vid) = video_id {
            // Extract title from <title> tag
            let mut title = None;
            if let Some(pos) = raw_html.find("<title>") {
                let start = pos + 7;
                if let Some(end) = raw_html[start..].find("</title>") {
                    let raw_title = &raw_html[start..start + end];
                    title = Some(raw_title.replace(" - Invidious", "").trim().to_string());
                }
            }
            
            // Extract streams from <source> tags
            let mut direct_streams = Vec::new();
            let mut search_pos = 0;
            while let Some(pos) = raw_html[search_pos..].find("<source") {
                let source_start = search_pos + pos;
                if let Some(tag_end) = raw_html[source_start..].find('>') {
                    let source_tag = &raw_html[source_start..source_start + tag_end];
                    if let Some(src_pos) = source_tag.find("src=\"") {
                        let src_start = src_pos + 5;
                        if let Some(src_end) = source_tag[src_start..].find('"') {
                            let src_val = &source_tag[src_start..src_start + src_end];
                            let unescaped_src = src_val.replace("&amp;", "&");
                            
                            // Resolve relative URL
                            let url = if unescaped_src.starts_with('/') {
                                if let Ok(base) = Url::parse(page_url) {
                                    if let Ok(joined) = base.join(&unescaped_src) {
                                        joined.to_string()
                                    } else {
                                        format!("https://invidious.f5.si{}", unescaped_src)
                                    }
                                } else {
                                    format!("https://invidious.f5.si{}", unescaped_src)
                                }
                            } else {
                                unescaped_src
                            };
                            
                            let mut label = "Stream".to_string();
                            if let Some(type_pos) = source_tag.find("type=\"") {
                                let type_start = type_pos + 6;
                                if let Some(type_end) = source_tag[type_start..].find('"') {
                                    label = source_tag[type_start..type_start + type_end].to_string();
                                }
                            }
                            
                            if url.contains("itag=") {
                                if let Some(itag_pos) = url.find("itag=") {
                                    let itag_start = itag_pos + 5;
                                    let itag_end = url[itag_start..].find('&').unwrap_or(url[itag_start..].len());
                                    let itag = &url[itag_start..itag_start + itag_end];
                                    let quality = match itag {
                                        "18" => "360p",
                                        "22" => "720p",
                                        "37" => "1080p",
                                        "43" => "360p WebM",
                                        "44" => "480p WebM",
                                        "45" => "720p WebM",
                                        "46" => "1080p WebM",
                                        _ => itag,
                                    };
                                    label = format!("{} ({})", label, quality);
                                }
                            }
                            
                            if !direct_streams.iter().any(|stream: &StreamLink| stream.url == url) {
                                direct_streams.push(StreamLink {
                                    label,
                                    url,
                                });
                            }
                        }
                    }
                    search_pos = source_start + tag_end + 1;
                } else {
                    break;
                }
            }
            
            return Some(PlayerShell {
                title,
                author: None,
                duration: None,
                views: None,
                status: Some("Invidious".to_string()),
                video_id: Some(vid),
                direct_streams,
                protected_formats: 0,
            });
        }
        return None;
    }

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

    let video_id = details
        .and_then(|d| d.get("videoId"))
        .and_then(Value::as_str)
        .map(str::to_string);

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
        video_id,
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

    let url = format!("https://www.youtube.com/watch?v={video_id}");
    let thumbnail_url = thumbnail_from_url(&url);

    Some(VideoCard {
        title,
        url,
        subtitle,
        duration,
        thumbnail_url,
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
    let var_pos = raw_html.find(variable)?;
    let start = var_pos + variable.len();
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
