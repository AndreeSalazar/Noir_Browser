use crate::browser::LinkHitbox;
use crate::media::discovery::{discover_media, MediaReport};
use crate::parsers::css_engine::ComputedStyle;
use crate::parsers::css_simple::{parse_color, parse_px, CssCascade, CssElementContext};
use crate::parsers::dom_tree::DomNode;
use crate::parsers::html_elements::HtmlTag;
use crate::parsers::resource_loader::{
    fetch_document, CacheStatus, ResourceResponse, ResourceType,
};
use crate::parsers::style_collector::{
    collect_inline_styles, collect_stylesheets, StylesheetBundle,
};
use crate::render::text::{RasterizedAtlas, TextRasterizationOptions, TextRequest, AtlasImageRequest};
use crate::runtime::{collect_scripts, BrowserRuntime, ScriptSource};
use std::collections::HashMap;
use url::Url;

mod app_shell;

const MAX_TEXT_FRAGMENTS: usize = 2048;
const MAX_VISIBLE_LINES: usize = 1500;
const CONTENT_X: f32 = 40.0;
const CONTENT_TOP: f32 = 78.0;
const CONTENT_SIDE_PADDING: f32 = 80.0;
const VIEWPORT_BOTTOM_PADDING: f32 = 48.0;
const URL_TEXT_X: f32 = 202.0;

#[derive(Clone, Debug)]
pub struct RenderBox {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub color: [f32; 4],
    pub radius: f32,
    pub href: Option<String>,
}

#[derive(Clone, Debug)]
pub enum LayoutFragment {
    Text(TextFragment),
    BlockStart {
        id: usize,
        layout: FragmentLayout,
        background_color: Option<[f32; 4]>,
        is_block: bool,
        border_radius: Option<f32>,
        href: Option<String>,
    },
    BlockEnd {
        id: usize,
        margin_after: f32,
        is_block: bool,
    },
}

#[derive(Clone, Debug)]
pub struct TextFragment {
    pub text: String,
    pub px_size: f32,
    pub is_bold: bool,
    pub line_height: f32,
    pub margin_after: f32,
    pub line_break_after: bool,
    pub layout: FragmentLayout,
    pub color: [f32; 4],
    pub href: Option<String>,
    pub is_input: bool,
    pub is_submit: bool,
    pub input_name: String,
    pub input_value: String,
    pub input_placeholder: String,
    pub form_action: Option<String>,
    pub is_image: bool,
    pub image_url: Option<String>,
    pub image_width: Option<f32>,
    pub image_height: Option<f32>,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct FragmentLayout {
    width: Option<String>,
    max_width: Option<String>,
    margin_left: Option<String>,
    margin_right: Option<String>,
    padding_left: Option<String>,
    padding_right: Option<String>,
    text_align: Option<String>,
    border_radius: Option<String>,
}

#[derive(Clone, Debug)]
struct TextStyleState {
    px_size: f32,
    is_bold: bool,
    line_height: f32,
    margin_after: f32,
    color: [f32; 4],
    hidden: bool,
    display: Option<String>,
    text_transform: Option<String>,
    background_color: Option<[f32; 4]>,
    layout: FragmentLayout,
    in_navigation: bool,
    border_radius: Option<f32>,
    href: Option<String>,
}

impl Default for TextStyleState {
    fn default() -> Self {
        Self::default_with_color([1.0, 1.0, 1.0, 1.0])
    }
}

impl TextStyleState {
    fn default_with_color(color: [f32; 4]) -> Self {
        Self {
            px_size: 16.0,
            is_bold: false,
            line_height: 22.0,
            margin_after: 6.0,
            color,
            hidden: false,
            display: None,
            text_transform: None,
            background_color: None,
            layout: FragmentLayout::default(),
            in_navigation: false,
            border_radius: None,
            href: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct PageDocument {
    pub fragments: Vec<LayoutFragment>,
    pub media: MediaReport,
    page_style: PageStyle,
}

impl TextFragment {
    pub(crate) fn new_text(
        text: String,
        px_size: f32,
        is_bold: bool,
        line_height: f32,
        margin_after: f32,
        line_break_after: bool,
        layout: FragmentLayout,
        color: [f32; 4],
        href: Option<String>,
    ) -> Self {
        Self {
            text,
            px_size,
            is_bold,
            line_height,
            margin_after,
            line_break_after,
            layout,
            color,
            href,
            is_input: false,
            is_submit: false,
            input_name: String::new(),
            input_value: String::new(),
            input_placeholder: String::new(),
            form_action: None,
            is_image: false,
            image_url: None,
            image_width: None,
            image_height: None,
        }
    }
}

impl PageDocument {
    pub fn set_input_value(&mut self, fragment_idx: usize, value: String) {
        if let Some(LayoutFragment::Text(fragment)) = self.fragments.get_mut(fragment_idx) {
            fragment.input_value = value;
        }
    }

    pub fn get_input_value(&self, fragment_idx: usize) -> Option<String> {
        if let Some(LayoutFragment::Text(fragment)) = self.fragments.get(fragment_idx) {
            Some(fragment.input_value.clone())
        } else {
            None
        }
    }
    pub fn media_summary(&self) -> Option<String> {
        self.media.summary()
    }

    pub fn computed_style(&self) -> ComputedStyle {
        let mut style = ComputedStyle::default();
        style.background_color = Some(self.page_style.background_hex.clone());
        style.width = Some("100%".to_string());
        style.height = Some("100%".to_string());
        style
    }
}

#[derive(Clone, Debug)]
struct PageStyle {
    background_hex: String,
    default_text_color: [f32; 4],
}

pub struct PageRender {
    pub atlas: RasterizedAtlas,
    pub boxes: Vec<RenderBox>,
    pub content_height: f32,
}

pub async fn load_page_document(target_url: &str) -> PageDocument {
    let response = fetch_document(target_url).await.unwrap_or_else(|error| {
        let body = format!("<h1>Network Error</h1><p>{error}</p>");
        ResourceResponse {
            requested_url: target_url.to_string(),
            final_url: target_url.to_string(),
            status: 0,
            resource_type: ResourceType::Document,
            content_type: Some("text/html; charset=utf-8".to_string()),
            body_bytes: body.len(),
            body,
            cache_status: CacheStatus::Network,
        }
    });
    let dom = crate::parsers::dom_tree::parse_html(&response.body);
    let base_url = Url::parse(&response.final_url)
        .or_else(|_| Url::parse(target_url))
        .ok();

    let mut fragments = Vec::new();
    let stylesheet_bundle = load_stylesheet_bundle(&dom, base_url.as_ref()).await;
    let css = CssCascade::from_blocks(&stylesheet_bundle.blocks);
    let page_style = derive_page_style(&css);
    let mut ancestors = Vec::new();
    let is_youtube = target_url.contains("youtube.com") || response.final_url.contains("youtube.com");
    if !is_youtube {
        extract_text_from_dom(
            &dom,
            &mut fragments,
            &css,
            TextStyleState::default_with_color(page_style.default_text_color),
            None,
            base_url.as_ref(),
            &mut ancestors,
            None,
        );
    }
    append_direct_resource_notice(&mut fragments, &response, page_style.default_text_color);
    append_stylesheet_summary(&mut fragments, &stylesheet_bundle);
    apply_runtime_scripts(&dom, &mut fragments, base_url.as_ref());
    app_shell::append_app_shell_fallback(
        &dom,
        &response.body,
        &response.final_url,
        &mut fragments,
        page_style.default_text_color,
    );
    append_response_summary(&mut fragments, &response);
    let media = discover_media(&dom, &response.final_url);
    append_media_summary(&mut fragments, &media);
    normalize_fragments(&mut fragments);

    // Save fragments to a file for debugging
    let mut log_content = String::new();
    log_content.push_str(&format!("URL: {}\n", target_url));
    log_content.push_str(&format!("Default Text Color: {:?}\n", page_style.default_text_color));
    log_content.push_str(&format!("Default Background: {:?}\n", page_style.background_hex));
    for (i, frag) in fragments.iter().enumerate() {
        log_content.push_str(&format!("Frag {}: {:?}\n", i, frag));
    }
    let _ = std::fs::write("fragments_log.txt", log_content);

    PageDocument {
        fragments,
        media,
        page_style,
    }
}

fn append_response_summary(fragments: &mut Vec<LayoutFragment>, response: &ResourceResponse) {
    if response.is_success()
        && response.requested_url == response.final_url
        && response.is_html_like()
        && response.cache_status == CacheStatus::Network
    {
        return;
    }

    let content_type = response
        .content_type
        .as_deref()
        .unwrap_or("content-type desconocido");
    let mut summary = format!(
        "HTTP {} / {:?} / {} bytes / {}",
        response.status, response.resource_type, response.body_bytes, content_type
    );
    if response.requested_url != response.final_url {
        summary.push_str(&format!(" / redirigido a {}", response.final_url));
    }
    if !response.is_html_like() {
        summary.push_str(" / no parece HTML");
    }
    match response.cache_status {
        CacheStatus::Network => {}
        CacheStatus::Revalidated => summary.push_str(" / cache revalidado"),
        CacheStatus::Fallback => summary.push_str(" / cache offline"),
    }

    fragments.insert(
        0,
        LayoutFragment::Text(TextFragment::new_text(
            summary,
            13.0,
            true,
            19.0,
            6.0,
            true,
            FragmentLayout::default(),
            [0.725, 0.790, 0.980, 1.0],
            None,
        )),
    );
}

fn append_direct_resource_notice(
    fragments: &mut Vec<LayoutFragment>,
    response: &ResourceResponse,
    text_color: [f32; 4],
) {
    let Some(kind) = direct_resource_kind(response) else {
        return;
    };

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

    push_notice_fragment(
        fragments,
        &format!("{kind} - detectado"),
        22.0,
        true,
        text_color,
    );

    if response.status == 403 {
        push_notice_fragment(
            fragments,
            "HTTP 403: el enlace directo expiro o requiere firma, cookies o cabeceras del reproductor original.",
            16.0,
            false,
            text_color,
        );
        push_notice_fragment(
            fragments,
            "Vuelve al enlace /watch de YouTube para reconstruir la vista ligera; el probe Python puede diagnosticar formatos cifrados.",
            15.0,
            false,
            text_color,
        );
    } else if response.is_success() {
        push_notice_fragment(
            fragments,
            "Es un recurso directo, no una pagina HTML completa. El navegador lo reconoce y evita mostrar una pagina vacia.",
            16.0,
            false,
            text_color,
        );
    } else {
        push_notice_fragment(
            fragments,
            &format!(
                "La red respondio HTTP {} con {} bytes; no hay HTML visible que renderizar.",
                response.status, response.body_bytes
            ),
            16.0,
            false,
            text_color,
        );
    }

    push_notice_fragment(
        fragments,
        &response.final_url,
        13.0,
        false,
        [0.478, 0.635, 0.968, 1.0],
    );
}

fn direct_resource_kind(response: &ResourceResponse) -> Option<&'static str> {
    let requested = response.requested_url.to_ascii_lowercase();
    let final_url = response.final_url.to_ascii_lowercase();
    let content_type = response
        .content_type
        .as_deref()
        .unwrap_or_default()
        .to_ascii_lowercase();

    if requested.contains("googlevideo.com/videoplayback")
        || final_url.contains("googlevideo.com/videoplayback")
    {
        return Some("Stream directo de YouTube/Googlevideo");
    }

    if content_type.starts_with("video/")
        || content_type.starts_with("audio/")
        || content_type.contains("application/dash+xml")
        || content_type.contains("mpegurl")
    {
        return Some("Recurso multimedia directo");
    }

    if final_url.ends_with(".mp4")
        || final_url.ends_with(".webm")
        || final_url.ends_with(".m4a")
        || final_url.ends_with(".mp3")
        || final_url.ends_with(".m3u8")
        || final_url.ends_with(".mpd")
    {
        return Some("Recurso multimedia directo");
    }

    if !response.is_html_like() {
        return Some("Recurso no HTML");
    }

    None
}

fn push_notice_fragment(
    fragments: &mut Vec<LayoutFragment>,
    text: &str,
    px_size: f32,
    is_bold: bool,
    color: [f32; 4],
) {
    fragments.push(LayoutFragment::Text(TextFragment::new_text(
        text.to_string(),
        px_size,
        is_bold,
        (px_size + 7.0).max(20.0),
        6.0,
        true,
        FragmentLayout::default(),
        color,
        None,
    )));
}

async fn load_stylesheet_bundle(dom: &[DomNode], base_url: Option<&Url>) -> StylesheetBundle {
    let mut bundle = StylesheetBundle::default();
    let stylesheets = collect_stylesheets(dom, base_url);
    bundle.external_count = stylesheets.len();
    bundle.blocks.extend(collect_inline_styles(dom));
    bundle.inline_count = bundle.blocks.len();

    let mut workers = Vec::new();
    for stylesheet in stylesheets.iter().take(32) {
        let url = stylesheet.url.clone();
        workers.push(tokio::spawn(async move {
            crate::parsers::resource_loader::fetch_style(&url).await.ok()
        }));
    }

    for worker in workers {
        if let Ok(Some(response)) = worker.await {
            bundle.loaded_external += 1;
            bundle.blocks.push(response.body);
        }
    }

    bundle
}

fn append_stylesheet_summary(fragments: &mut Vec<LayoutFragment>, bundle: &StylesheetBundle) {
    if bundle.external_count == 0 && bundle.inline_count == 0 {
        return;
    }

    fragments.insert(
        0,
        LayoutFragment::Text(TextFragment::new_text(
            format!(
                "CSS detectado: {} inline / {} externas / {} precargadas",
                bundle.inline_count, bundle.external_count, bundle.loaded_external
            ),
            13.0,
            true,
            19.0,
            6.0,
            true,
            FragmentLayout::default(),
            [0.725, 0.790, 0.980, 1.0],
            None,
        )),
    );
}

fn is_light_color(color: [f32; 4]) -> bool {
    let luminance = 0.2126 * color[0] + 0.7152 * color[1] + 0.0722 * color[2];
    luminance > 0.6
}

fn map_background_to_dark(color: [f32; 4]) -> [f32; 4] {
    if is_light_color(color) {
        // Map light background to Google's standard dark gray: #202124
        [0.125, 0.13, 0.14, 1.0]
    } else {
        color
    }
}

fn map_text_to_light_if_needed(text_color: [f32; 4], background_color: [f32; 4]) -> [f32; 4] {
    let bg_is_dark = !is_light_color(background_color);
    if bg_is_dark {
        let text_lum = 0.2126 * text_color[0] + 0.7152 * text_color[1] + 0.0722 * text_color[2];
        if text_lum < 0.45 {
            // Text is too dark on a dark background; map it to standard Google dark mode text: #e8eaed
            [0.91, 0.92, 0.93, 1.0]
        } else {
            text_color
        }
    } else {
        text_color
    }
}

fn derive_page_style(css: &CssCascade) -> PageStyle {
    let empty = HashMap::new();
    let html = css.declarations_for(&HtmlTag::Custom("html".to_string()), &empty);
    let body = css.declarations_for(&HtmlTag::Custom("body".to_string()), &empty);

    let mut background = body
        .background_color
        .as_deref()
        .or(body.background.as_deref())
        .or(html.background_color.as_deref())
        .or(html.background.as_deref())
        .and_then(first_css_color)
        .unwrap_or([0.102, 0.102, 0.180, 1.0]);

    background = map_background_to_dark(background);

    let mut text = body
        .color
        .as_deref()
        .or(html.color.as_deref())
        .and_then(parse_color)
        .unwrap_or_else(|| readable_text_color(background));
    text = ensure_contrast(text, background);
    text = map_text_to_light_if_needed(text, background);

    PageStyle {
        background_hex: rgba_to_hex(background),
        default_text_color: text,
    }
}

fn first_css_color(value: &str) -> Option<[f32; 4]> {
    parse_color(value).or_else(|| {
        value
            .split_whitespace()
            .find_map(|part| parse_color(part.trim_matches(',')))
    })
}

fn readable_text_color(background: [f32; 4]) -> [f32; 4] {
    let luminance = 0.2126 * background[0] + 0.7152 * background[1] + 0.0722 * background[2];
    if luminance > 0.55 {
        [0.08, 0.08, 0.10, 1.0]
    } else {
        [1.0, 1.0, 1.0, 1.0]
    }
}

fn ensure_contrast(text: [f32; 4], background: [f32; 4]) -> [f32; 4] {
    let text_lum = 0.2126 * text[0] + 0.7152 * text[1] + 0.0722 * text[2];
    let bg_lum = 0.2126 * background[0] + 0.7152 * background[1] + 0.0722 * background[2];
    if (text_lum - bg_lum).abs() >= 0.32 {
        return text;
    }

    readable_text_color(background)
}

fn rgba_to_hex(color: [f32; 4]) -> String {
    let r = (color[0].clamp(0.0, 1.0) * 255.0).round() as u8;
    let g = (color[1].clamp(0.0, 1.0) * 255.0).round() as u8;
    let b = (color[2].clamp(0.0, 1.0) * 255.0).round() as u8;
    format!("#{r:02x}{g:02x}{b:02x}")
}

struct ActiveBlock {
    id: usize,
    layout: FragmentLayout,
    layout_box: ResolvedLayoutBox,
    start_y: f32,
    color: [f32; 4],
    radius: f32,
    href: Option<String>,
    is_block: bool,
    start_cursor_x: f32,
    render_box_idx: Option<usize>,
}

pub fn render_page(
    target_url: &str,
    document: &PageDocument,
    link_hitboxes: &mut Vec<LinkHitbox>,
    text_options: TextRasterizationOptions,
    viewport_width: f32,
    viewport_height: f32,
    scroll_offset: f32,
    tabs_info: &[(String, bool)],
    focused_input_idx: Option<usize>,
) -> PageRender {
    let mut text_requests = Vec::new();
    let mut image_requests = Vec::new();
    link_hitboxes.clear();

    // Render Tab Titles
    let tabs_count = tabs_info.len();
    if tabs_count > 0 {
        let tab_w = ((viewport_width - 190.0) / tabs_count as f32).min(160.0).max(40.0);
        for (i, (url, is_active)) in tabs_info.iter().enumerate() {
            let x_min = 12.0 + i as f32 * tab_w;
            let text_x = x_min + 12.0;
            let title = clean_tab_title(url);
            let max_chars = ((tab_w - 32.0).max(10.0) / 7.0) as usize;
            let truncated_title: String = if title.chars().count() > max_chars {
                title.chars().take(max_chars.saturating_sub(3)).collect::<String>() + "..."
            } else {
                title
            };

            let color = if *is_active {
                [0.90, 0.92, 1.00, 1.0]
            } else {
                [0.55, 0.60, 0.70, 0.85]
            };

            text_requests.push(TextRequest {
                text: truncated_title,
                px_size: 13.0,
                is_bold: *is_active,
                pos_x: text_x,
                pos_y: 20.0, // Vertically center within the 36px bar (baseline around 20px)
                color,
            });
        }
    }

    text_requests.push(TextRequest {
        text: compact_url_text(target_url, viewport_width),
        px_size: 16.0,
        is_bold: false,
        pos_x: URL_TEXT_X,
        pos_y: 48.0,
        color: [1.0, 1.0, 1.0, 1.0],
    });

    let default_content_width = (viewport_width - CONTENT_SIDE_PADDING).clamp(320.0, 1120.0);
    let default_content_x = CONTENT_X;
    let visible_bottom = (viewport_height - VIEWPORT_BOTTOM_PADDING).max(CONTENT_TOP);
    let mut document_y = CONTENT_TOP;
    let mut cursor_x = default_content_x;
    let mut line_height = 0.0_f32;
    let mut line_started = false;
    let mut line_index = 0;
    let mut active_box = ResolvedLayoutBox {
        content_x: default_content_x,
        content_width: default_content_width,
        box_x: default_content_x,
        box_width: default_content_width,
    };

    let mut render_boxes = Vec::new();
    let mut active_blocks: Vec<ActiveBlock> = Vec::new();

    let mut page_text_requests: Vec<(TextRequest, usize)> = Vec::new();
    let mut page_image_requests: Vec<(AtlasImageRequest, usize)> = Vec::new();
    let mut page_render_boxes: Vec<(RenderBox, usize)> = Vec::new();
    let mut page_link_hitboxes: Vec<(LinkHitbox, usize)> = Vec::new();

    #[derive(Clone)]
    struct LineContext {
        alignment: Option<String>,
        container_x: f32,
        container_width: f32,
    }
    let mut line_contexts: Vec<LineContext> = Vec::new();

    let estimate_inline_block_width = |start_idx: usize, parent_layout: &FragmentLayout| -> f32 {
        let available = (viewport_width - CONTENT_SIDE_PADDING).max(320.0);
        if let Some(ref w_str) = parent_layout.width {
            if let Some(w) = parse_layout_length(w_str, available) {
                return w;
            }
        }
        
        let mut total_w = 0.0_f32;
        let mut depth = 0;
        for idx in (start_idx + 1)..document.fragments.len() {
            match &document.fragments[idx] {
                LayoutFragment::BlockStart { .. } => {
                    depth += 1;
                }
                LayoutFragment::BlockEnd { .. } => {
                    if depth == 0 {
                        break;
                    }
                    depth -= 1;
                }
                LayoutFragment::Text(frag) => {
                    if frag.is_input {
                        let input_w = frag.layout.width.as_deref()
                            .and_then(|v| parse_layout_length(v, available))
                            .unwrap_or(150.0);
                        total_w += input_w;
                    } else if frag.is_submit {
                        let btn_w = estimated_text_width(&frag.text, frag.px_size) + 32.0;
                        total_w += btn_w;
                    } else if frag.is_image {
                        let img_w = frag.image_width.unwrap_or(80.0);
                        total_w += img_w;
                    } else {
                        let txt_w = estimated_text_width(&frag.text, frag.px_size);
                        total_w += txt_w;
                    }
                }
            }
        }
        
        let padding_left = parent_layout.padding_left.as_deref()
            .and_then(|v| parse_layout_length(v, available))
            .unwrap_or(0.0);
        let padding_right = parent_layout.padding_right.as_deref()
            .and_then(|v| parse_layout_length(v, available))
            .unwrap_or(0.0);
            
        (total_w + padding_left + padding_right).max(16.0)
    };

    let mut step_log = String::new();
    for (frag_idx, layout_frag) in document.fragments.iter().enumerate() {
        step_log.push_str(&format!(
            "Step {}: {:?}\n  -> y={}, line_h={}, active_blocks={:?}\n",
            frag_idx,
            layout_frag,
            document_y,
            line_height,
            active_blocks.iter().map(|b| (b.id, b.is_block, b.start_y)).collect::<Vec<_>>()
        ));
        match layout_frag {
            LayoutFragment::BlockStart {
                id,
                layout,
                background_color,
                is_block,
                border_radius,
                href,
            } => {
                if *is_block {
                    if line_started {
                        document_y += line_height;
                        cursor_x = active_box.content_x;
                        line_height = 0.0;
                        line_started = false;
                        line_index += 1;
                    }

                    let layout_box = resolve_fragment_layout(
                        layout,
                        viewport_width,
                        default_content_x,
                        default_content_width,
                    );

                    active_box = layout_box;

                    let block_color = background_color.unwrap_or([0.0, 0.0, 0.0, 0.0]);
                    let radius = border_radius.unwrap_or(0.0);
                    let render_box_idx = if block_color[3] > 0.0 {
                        let idx = render_boxes.len();
                        render_boxes.push(RenderBox {
                            x: 0.0,
                            y: 0.0,
                            w: 0.0,
                            h: 0.0,
                            color: [0.0, 0.0, 0.0, 0.0],
                            radius: 0.0,
                            href: None,
                        });
                        Some(idx)
                    } else {
                        None
                    };

                    active_blocks.push(ActiveBlock {
                        id: *id,
                        layout: layout.clone(),
                        layout_box,
                        start_y: document_y,
                        color: block_color,
                        radius,
                        href: href.clone(),
                        is_block: true,
                        start_cursor_x: cursor_x,
                        render_box_idx,
                    });
                } else {
                    // Inline-block:
                    let available = (viewport_width - CONTENT_SIDE_PADDING).max(320.0);
                    let estimated_w = estimate_inline_block_width(frag_idx, layout);
                    let card_width = layout
                        .width
                        .as_deref()
                        .and_then(|v| parse_layout_length(v, available))
                        .unwrap_or(estimated_w);

                    let margin_left = layout
                        .margin_left
                        .as_deref()
                        .and_then(|v| parse_layout_length(v, available))
                        .unwrap_or(0.0);
                    let margin_right = layout
                        .margin_right
                        .as_deref()
                        .and_then(|v| parse_layout_length(v, available))
                        .unwrap_or(0.0);

                    let total_card_space = card_width + margin_left + margin_right;

                    let parent_is_inline = active_blocks.last().map(|b| !b.is_block).unwrap_or(false);
                    if !parent_is_inline && line_started && cursor_x + total_card_space > default_content_x + default_content_width {
                        document_y += line_height;
                        cursor_x = default_content_x;
                        line_height = 0.0;
                        line_started = false;
                        line_index += 1;
                    }

                    let layout_box = resolve_fragment_layout(
                        layout,
                        viewport_width,
                        cursor_x,
                        card_width,
                    );

                    active_box = layout_box;
                    cursor_x = layout_box.content_x;

                    let block_color = background_color.unwrap_or([0.0, 0.0, 0.0, 0.0]);
                    let radius = border_radius.unwrap_or(0.0);
                    let render_box_idx = if block_color[3] > 0.0 {
                        let idx = page_render_boxes.len();
                        page_render_boxes.push((RenderBox {
                            x: 0.0,
                            y: 0.0,
                            w: 0.0,
                            h: 0.0,
                            color: [0.0, 0.0, 0.0, 0.0],
                            radius: 0.0,
                            href: None,
                        }, line_index));
                        Some(idx)
                    } else {
                        None
                    };

                    active_blocks.push(ActiveBlock {
                        id: *id,
                        layout: layout.clone(),
                        layout_box,
                        start_y: document_y,
                        color: block_color,
                        radius,
                        href: href.clone(),
                        is_block: false,
                        start_cursor_x: cursor_x,
                        render_box_idx,
                    });
                }
            }
            LayoutFragment::BlockEnd {
                id,
                margin_after,
                is_block,
            } => {
                if *is_block {
                    if line_started {
                        document_y += line_height;
                        cursor_x = active_box.content_x;
                        line_height = 0.0;
                        line_started = false;
                        line_index += 1;
                    }

                    if let Some(pos) = active_blocks.iter().position(|b| b.id == *id) {
                        let block = active_blocks.remove(pos);
                        let height = document_y - block.start_y;
                        if height > 0.0 && block.color[3] > 0.0 {
                            let screen_y = block.start_y - scroll_offset;
                            let line_bottom = screen_y + height;

                            if line_bottom >= CONTENT_TOP && screen_y <= visible_bottom {
                                let draw_y = screen_y.max(CONTENT_TOP);
                                let draw_h = (screen_y + height - draw_y).min(visible_bottom - draw_y).max(0.0);
                                if draw_h > 0.0 {
                                    if let Some(idx) = block.render_box_idx {
                                        if idx < render_boxes.len() {
                                            render_boxes[idx] = RenderBox {
                                                x: block.layout_box.box_x,
                                                y: draw_y,
                                                w: block.layout_box.box_width,
                                                h: draw_h,
                                                color: block.color,
                                                radius: block.radius,
                                                href: block.href.clone(),
                                            };
                                        }
                                    } else {
                                        render_boxes.push(RenderBox {
                                            x: block.layout_box.box_x,
                                            y: draw_y,
                                            w: block.layout_box.box_width,
                                            h: draw_h,
                                            color: block.color,
                                            radius: block.radius,
                                            href: block.href.clone(),
                                        });
                                    }
                                }
                                if let Some(link) = &block.href {
                                    let draw_y = screen_y.max(CONTENT_TOP);
                                    let draw_h = (screen_y + height - draw_y).min(visible_bottom - draw_y).max(0.0);
                                    if draw_h > 0.0 {
                                        link_hitboxes.push(LinkHitbox {
                                            href: link.clone(),
                                            x: block.layout_box.box_x,
                                            y: draw_y,
                                            w: block.layout_box.box_width,
                                            h: draw_h,
                                            is_input: false,
                                            is_submit: false,
                                            fragment_idx: block.id,
                                        });
                                    }
                                }
                            }
                        }
                    }

                    document_y += margin_after;
                } else {
                    // Inline-block:
                    if let Some(pos) = active_blocks.iter().position(|b| b.id == *id) {
                        let block = active_blocks.remove(pos);

                        let final_y = if line_started {
                            document_y + line_height
                        } else {
                            document_y
                        };

                        let height = (final_y - block.start_y).max(40.0);
                        if height > 0.0 && block.color[3] > 0.0 {
                            let screen_y = block.start_y - scroll_offset;
                            let line_bottom = screen_y + height;

                            if line_bottom >= CONTENT_TOP && screen_y <= visible_bottom {
                                let draw_y = screen_y.max(CONTENT_TOP);
                                let draw_h = (screen_y + height - draw_y).min(visible_bottom - draw_y).max(0.0);
                                
                                // Register line context for alignment tracking
                                let active_align = block.layout.text_align.clone()
                                    .or_else(|| active_blocks.iter().rev().find_map(|b| b.layout.text_align.clone()));
                                if line_index >= line_contexts.len() {
                                    line_contexts.resize(line_index + 1, LineContext {
                                        alignment: None,
                                        container_x: default_content_x,
                                        container_width: default_content_width,
                                    });
                                }
                                let ctx = &mut line_contexts[line_index];
                                if ctx.alignment.is_none() {
                                    ctx.alignment = active_align;
                                }
                                let line_container = active_blocks.iter().rev()
                                    .find(|b| b.is_block)
                                    .map(|b| b.layout_box)
                                    .unwrap_or(ResolvedLayoutBox {
                                        box_x: default_content_x,
                                        box_width: default_content_width,
                                        content_x: default_content_x,
                                        content_width: default_content_width,
                                    });
                                ctx.container_x = line_container.content_x;
                                ctx.container_width = line_container.content_width;

                                if draw_h > 0.0 {
                                    if let Some(idx) = block.render_box_idx {
                                        if idx < page_render_boxes.len() {
                                            page_render_boxes[idx] = (RenderBox {
                                                x: block.layout_box.box_x,
                                                y: draw_y,
                                                w: block.layout_box.box_width,
                                                h: draw_h,
                                                color: block.color,
                                                radius: block.radius,
                                                href: block.href.clone(),
                                            }, line_index);
                                        }
                                    } else {
                                        page_render_boxes.push((RenderBox {
                                            x: block.layout_box.box_x,
                                            y: draw_y,
                                            w: block.layout_box.box_width,
                                            h: draw_h,
                                            color: block.color,
                                            radius: block.radius,
                                            href: block.href.clone(),
                                        }, line_index));
                                    }
                                }
                                if let Some(link) = &block.href {
                                    let draw_y = screen_y.max(CONTENT_TOP);
                                    let draw_h = (screen_y + height - draw_y).min(visible_bottom - draw_y).max(0.0);
                                    if draw_h > 0.0 {
                                        page_link_hitboxes.push((LinkHitbox {
                                            href: link.clone(),
                                            x: block.layout_box.box_x,
                                            y: draw_y,
                                            w: block.layout_box.box_width,
                                            h: draw_h,
                                            is_input: false,
                                            is_submit: false,
                                            fragment_idx: block.id,
                                        }, line_index));
                                    }
                                }
                            }
                        }

                        let available = (viewport_width - CONTENT_SIDE_PADDING).max(320.0);
                        let margin_right = block.layout
                            .margin_right
                            .as_deref()
                            .and_then(|v| parse_layout_length(v, available))
                            .unwrap_or(0.0);

                        if line_started {
                            cursor_x = block.layout_box.box_x + block.layout_box.box_width + margin_right;
                            let line_height_on_current_line = if document_y > block.start_y {
                                line_height.max(40.0)
                            } else {
                                height
                            };
                            line_height = line_height.max(line_height_on_current_line);
                            line_started = true;
                        }

                        if let Some(parent_block) = active_blocks.last() {
                            active_box = parent_block.layout_box;
                        } else {
                            active_box = ResolvedLayoutBox {
                                content_x: default_content_x,
                                content_width: default_content_width,
                                box_x: default_content_x,
                                box_width: default_content_width,
                            };
                        }
                    }
                }
            }
            LayoutFragment::Text(fragment) => {
                let layout_box = if let Some(active_block) = active_blocks.last() {
                    active_block.layout_box
                } else {
                    resolve_fragment_layout(
                        &fragment.layout,
                        viewport_width,
                        default_content_x,
                        default_content_width,
                    )
                };

                let active_align = fragment.layout.text_align.clone()
                    .or_else(|| active_blocks.iter().rev().find_map(|b| b.layout.text_align.clone()));
                
                // Helper to initialize/update LineContext for a given line index
                let mut ensure_line_context = |line_idx: usize, align: &Option<String>, cx: f32, cw: f32| {
                    if line_idx >= line_contexts.len() {
                        line_contexts.resize(line_idx + 1, LineContext {
                            alignment: None,
                            container_x: default_content_x,
                            container_width: default_content_width,
                        });
                    }
                    let ctx = &mut line_contexts[line_idx];
                    if ctx.alignment.is_none() {
                        ctx.alignment = align.clone();
                    }
                    let line_container = active_blocks.iter().rev()
                        .find(|b| b.is_block)
                        .map(|b| b.layout_box)
                        .unwrap_or(ResolvedLayoutBox {
                            box_x: default_content_x,
                            box_width: default_content_width,
                            content_x: default_content_x,
                            content_width: default_content_width,
                        });
                    ctx.container_x = line_container.content_x;
                    ctx.container_width = line_container.content_width;
                };

                ensure_line_context(line_index, &active_align, layout_box.content_x, layout_box.content_width);

                if line_started && layout_box != active_box {
                    document_y += line_height.max(fragment.line_height);
                    cursor_x = layout_box.content_x;
                    line_height = 0.0;
                    line_started = false;
                    line_index += 1;
                    ensure_line_context(line_index, &active_align, layout_box.content_x, layout_box.content_width);
                }
                active_box = layout_box;

                if fragment.is_input {
                    let fragment_box = resolve_fragment_layout(
                        &fragment.layout,
                        viewport_width,
                        layout_box.content_x,
                        layout_box.content_width,
                    );

                    // Force line break before input if we are already inline and it's a wide input
                    if line_started && fragment_box.content_width > 300.0 {
                        document_y += line_height;
                        cursor_x = fragment_box.content_x;
                        line_height = 0.0;
                        line_started = false;
                        line_index += 1;
                        ensure_line_context(line_index, &active_align, layout_box.content_x, layout_box.content_width);
                    }
                    let is_focused = focused_input_idx.is_some_and(|idx| idx == frag_idx);
                    let mut text_to_draw = if !fragment.input_value.is_empty() {
                        fragment.input_value.clone()
                    } else {
                        if !fragment.input_placeholder.is_empty() {
                            fragment.input_placeholder.clone()
                        } else {
                            "Search...".to_string()
                        }
                    };
                    
                    if is_focused {
                        text_to_draw.push('|');
                    }

                    let draw_color = if fragment.input_value.is_empty() {
                        [0.55, 0.55, 0.55, 1.0]
                    } else {
                        [0.91, 0.92, 0.93, 1.0]
                    };

                    let active_line_height = line_height.max(fragment.line_height).max(32.0);
                    let screen_y = document_y - scroll_offset;
                    let line_bottom = screen_y + active_line_height;

                    if line_bottom >= CONTENT_TOP && screen_y <= visible_bottom && line_index < MAX_VISIBLE_LINES {
                        let box_y = screen_y - 4.0;
                        let box_h = active_line_height + 8.0;
                        let draw_y = box_y.max(CONTENT_TOP);
                        let draw_h = (box_y + box_h - draw_y).min(visible_bottom - draw_y).max(0.0);
                        
                        let parsed_radius = fragment.layout.border_radius.as_deref()
                            .and_then(|val| parse_px(val, fragment.px_size))
                            .unwrap_or(8.0);

                        if draw_h > 0.0 {
                            page_render_boxes.push((RenderBox {
                                x: fragment_box.content_x,
                                y: draw_y,
                                w: fragment_box.content_width,
                                h: draw_h,
                                color: if is_focused { [0.26, 0.52, 0.96, 1.0] } else { [0.35, 0.35, 0.37, 1.0] },
                                radius: parsed_radius,
                                href: None,
                            }, line_index));
                        }

                        let inner_y = screen_y - 3.0;
                        let inner_h = active_line_height + 6.0;
                        let draw_inner_y = inner_y.max(CONTENT_TOP);
                        let draw_inner_h = (inner_y + inner_h - draw_inner_y).min(visible_bottom - draw_inner_y).max(0.0);
                        if draw_inner_h > 0.0 {
                            page_render_boxes.push((RenderBox {
                                x: fragment_box.content_x + 1.0,
                                y: draw_inner_y,
                                w: fragment_box.content_width - 2.0,
                                h: draw_inner_h,
                                color: [0.188, 0.192, 0.204, 1.0], // Dark input background #303134
                                radius: (parsed_radius - 1.0).max(0.0),
                                href: None,
                            }, line_index));
                        }

                        let text_x = if fragment.layout.text_align.as_deref() == Some("center") {
                            fragment_box.content_x + (fragment_box.content_width - estimated_text_width(&text_to_draw, fragment.px_size)) * 0.5
                        } else {
                            fragment_box.content_x + 12.0
                        };

                        let text_y = screen_y + (active_line_height - fragment.line_height) / 2.0;
                        if text_y >= CONTENT_TOP && text_y <= visible_bottom {
                            page_text_requests.push((TextRequest {
                                text: text_to_draw,
                                px_size: fragment.px_size,
                                is_bold: false,
                                pos_x: text_x.max(fragment_box.content_x + 4.0),
                                pos_y: text_y,
                                color: draw_color,
                            }, line_index));
                        }

                        let draw_link_y = (screen_y - 4.0).max(CONTENT_TOP);
                        let draw_link_h = (screen_y - 4.0 + active_line_height + 8.0 - draw_link_y).min(visible_bottom - draw_link_y).max(0.0);
                        if draw_link_h > 0.0 {
                            page_link_hitboxes.push((LinkHitbox {
                                href: String::new(),
                                x: fragment_box.content_x,
                                y: draw_link_y,
                                w: fragment_box.content_width,
                                h: draw_link_h,
                                is_input: true,
                                is_submit: false,
                                fragment_idx: frag_idx,
                            }, line_index));
                        }
                    } else {
                        println!(
                            "[Layout Debug] Skipped input field at y={} (line_bottom={}, visible_bottom={}, line_index={})",
                            screen_y, line_bottom, visible_bottom, line_index
                        );
                    }

                    cursor_x += fragment_box.content_width;
                    line_height = active_line_height;
                    line_started = true;

                    if fragment_box.content_width > 300.0 {
                        document_y += line_height;
                        cursor_x = fragment_box.content_x;
                        line_height = 0.0;
                        line_started = false;
                        line_index += 1;
                        ensure_line_context(line_index, &active_align, layout_box.content_x, layout_box.content_width);
                    }
                } else if fragment.is_submit {
                    let active_line_height = line_height.max(fragment.line_height).max(32.0);
                    let screen_y = document_y - scroll_offset;
                    let line_bottom = screen_y + active_line_height;

                    let btn_text = fragment.text.clone();
                    let text_w = estimated_text_width(&btn_text, fragment.px_size);
                    let button_width = text_w + 32.0;

                    if line_bottom >= CONTENT_TOP && screen_y <= visible_bottom && line_index < MAX_VISIBLE_LINES {
                        let box_y = screen_y - 4.0;
                        let box_h = active_line_height + 8.0;
                        let draw_y = box_y.max(CONTENT_TOP);
                        let draw_h = (box_y + box_h - draw_y).min(visible_bottom - draw_y).max(0.0);
                        if draw_h > 0.0 {
                            // Border box (acting as the border)
                            page_render_boxes.push((RenderBox {
                                x: cursor_x,
                                y: draw_y,
                                w: button_width,
                                h: draw_h,
                                color: [0.12, 0.65, 0.82, 0.5], // Subtle cyan border
                                radius: 18.0,
                                href: None,
                            }, line_index));

                            // Inner box (background)
                            page_render_boxes.push((RenderBox {
                                x: cursor_x + 1.0,
                                y: draw_y + 1.0,
                                w: button_width - 2.0,
                                h: draw_h - 2.0,
                                color: [0.188, 0.192, 0.204, 1.0], // Dark button background #303134
                                radius: 17.0,
                                href: None,
                            }, line_index));
                        }

                        let text_x = cursor_x + 16.0;
                        let text_y = screen_y + (active_line_height - fragment.line_height) / 2.0;
                        if text_y >= CONTENT_TOP && text_y <= visible_bottom {
                            page_text_requests.push((TextRequest {
                                text: btn_text,
                                px_size: fragment.px_size,
                                is_bold: true,
                                pos_x: text_x,
                                pos_y: text_y,
                                color: [0.91, 0.92, 0.93, 1.0], // Light button text #e8eaed
                            }, line_index));
                        }

                        let draw_link_y = (screen_y - 4.0).max(CONTENT_TOP);
                        let draw_link_h = (screen_y - 4.0 + active_line_height + 8.0 - draw_link_y).min(visible_bottom - draw_link_y).max(0.0);
                        if draw_link_h > 0.0 {
                            page_link_hitboxes.push((LinkHitbox {
                                href: String::new(),
                                x: cursor_x,
                                y: draw_link_y,
                                w: button_width,
                                h: draw_link_h,
                                is_input: false,
                                is_submit: true,
                                fragment_idx: frag_idx,
                            }, line_index));
                        }
                    } else {
                        println!(
                            "[Layout Debug] Skipped submit button at y={} (line_bottom={}, visible_bottom={}, line_index={})",
                            screen_y, line_bottom, visible_bottom, line_index
                        );
                    }

                    cursor_x += button_width + 12.0;
                    line_height = active_line_height;
                    line_started = true;
                } else if fragment.is_image {
                    let image_url = fragment.image_url.as_ref().map(|s| s.as_str()).unwrap_or("");
                    let cached_image = {
                        let cache = crate::media::image_manager::get_image_cache().lock().unwrap();
                        cache.get(image_url).cloned()
                    };

                    let (dest_w, dest_h, is_loaded) = if let Some(ref img) = cached_image {
                        let img_w = fragment.image_width.unwrap_or(img.width as f32);
                        let img_h = fragment.image_height.unwrap_or(img.height as f32);
                        let max_w = layout_box.content_width;
                        let mut dw = img_w;
                        let mut dh = img_h;
                        if dw > max_w {
                            let aspect = dw / dh;
                            dw = max_w;
                            dh = dw / aspect;
                        }
                        (dw, dh, true)
                    } else {
                        // Placeholder size using custom attributes if available, else default
                        let dw = fragment.image_width.unwrap_or(140.0);
                        let dh = fragment.image_height.unwrap_or(90.0);
                        (dw, dh, false)
                    };

                    if line_started && cursor_x + dest_w > layout_box.content_x + layout_box.content_width {
                        document_y += line_height.max(dest_h);
                        cursor_x = layout_box.content_x;
                        line_height = 0.0;
                        line_started = false;
                        line_index += 1;
                        ensure_line_context(line_index, &active_align, layout_box.content_x, layout_box.content_width);
                    }

                    let active_line_height = line_height.max(dest_h);
                    let screen_y = document_y - scroll_offset;
                    let line_bottom = screen_y + active_line_height;

                    if line_bottom >= CONTENT_TOP && screen_y <= visible_bottom && line_index < MAX_VISIBLE_LINES {
                        if is_loaded {
                            if let Some(ref img) = cached_image {
                                // Draw actual image by registering an AtlasImageRequest with precise viewport clipping
                                let crop_top = if screen_y < CONTENT_TOP {
                                    (CONTENT_TOP - screen_y).min(dest_h).max(0.0)
                                } else {
                                    0.0
                                };
                                let crop_bottom = if screen_y + dest_h > visible_bottom {
                                    (screen_y + dest_h - visible_bottom).min(dest_h - crop_top).max(0.0)
                                } else {
                                    0.0
                                };
                                let crop_left = if cursor_x < 0.0 {
                                    (-cursor_x).min(dest_w).max(0.0)
                                } else {
                                    0.0
                                };
                                let crop_right = if cursor_x + dest_w > viewport_width {
                                    (cursor_x + dest_w - viewport_width).min(dest_w - crop_left).max(0.0)
                                } else {
                                    0.0
                                };

                                if crop_top + crop_bottom < dest_h && crop_left + crop_right < dest_w {
                                    page_image_requests.push((AtlasImageRequest {
                                        rgba: std::sync::Arc::new(img.rgba.clone()),
                                        width: img.width,
                                        height: img.height,
                                        pos_x: cursor_x,
                                        pos_y: screen_y,
                                        dest_w,
                                        dest_h,
                                        crop_top,
                                        crop_bottom,
                                        crop_left,
                                        crop_right,
                                    }, line_index));
                                }
                            }
                        } else {
                            // Draw placeholder
                            let box_y = screen_y;
                            let box_h = dest_h;
                            let draw_y = box_y.max(CONTENT_TOP);
                            let draw_h = (box_y + box_h - draw_y).min(visible_bottom - draw_y).max(0.0);
                            if draw_h > 0.0 {
                                // Light gray elegant rounded box
                                page_render_boxes.push((RenderBox {
                                    x: cursor_x,
                                    y: draw_y,
                                    w: dest_w,
                                    h: draw_h,
                                    color: [0.93, 0.94, 0.96, 1.0],
                                    radius: 6.0,
                                    href: fragment.href.clone(),
                                }, line_index));
                            }

                            // Centered "Alt" text inside placeholder
                            let alt_text = if !fragment.text.is_empty() {
                                fragment.text.clone()
                            } else {
                                "Cargando...".to_string()
                            };

                            let text_w = estimated_text_width(&alt_text, 11.0);
                            let text_x = cursor_x + (dest_w - text_w) / 2.0;
                            let text_y = screen_y + (dest_h - 14.0) / 2.0;

                            if text_y >= CONTENT_TOP && text_y <= visible_bottom {
                                page_text_requests.push((TextRequest {
                                    text: alt_text,
                                    px_size: 11.0,
                                    is_bold: false,
                                    pos_x: text_x.max(cursor_x + 4.0),
                                    pos_y: text_y,
                                    color: [0.55, 0.58, 0.62, 1.0],
                                }, line_index));
                            }
                        }

                        // Support link wrapping for images
                        if let Some(href) = &fragment.href {
                            let draw_y = screen_y.max(CONTENT_TOP);
                            let draw_h = (screen_y + active_line_height - draw_y).min(visible_bottom - draw_y).max(0.0);
                            if draw_h > 0.0 {
                                page_link_hitboxes.push((LinkHitbox {
                                    href: href.clone(),
                                    x: cursor_x,
                                    y: draw_y,
                                    w: dest_w,
                                    h: draw_h,
                                    is_input: false,
                                    is_submit: false,
                                    fragment_idx: frag_idx,
                                }, line_index));
                            }
                        }
                    }

                    cursor_x += dest_w;
                    line_height = active_line_height;
                    line_started = true;
                } else {
                    let color = if fragment.href.is_some() && fragment.color == TextStyleState::default().color {
                        [0.478, 0.635, 0.968, 1.0]
                    } else {
                        fragment.color
                    };

                    let space_width = estimated_text_width(" ", fragment.px_size);
                    for word in fragment.text.split_whitespace() {
                        let word_width = estimated_text_width(word, fragment.px_size);
                        let mut leading_space = if line_started { space_width } else { 0.0 };
                        if line_started
                            && cursor_x + leading_space + word_width > layout_box.content_x + layout_box.content_width
                        {
                            document_y += line_height.max(fragment.line_height);
                            cursor_x = layout_box.content_x;
                            line_height = 0.0;
                            leading_space = 0.0;
                            line_index += 1;
                            ensure_line_context(line_index, &active_align, layout_box.content_x, layout_box.content_width);
                        }

                        let x = cursor_x + leading_space;
                        let active_line_height = line_height.max(fragment.line_height);
                        let screen_y = document_y - scroll_offset;
                        let line_bottom = screen_y + active_line_height;

                        if line_bottom >= CONTENT_TOP
                            && screen_y <= visible_bottom
                            && line_index < MAX_VISIBLE_LINES
                        {
                            if let Some(href) = &fragment.href {
                                let draw_y = screen_y.max(CONTENT_TOP);
                                let draw_h = (screen_y + active_line_height - draw_y).min(visible_bottom - draw_y).max(0.0);
                                if draw_h > 0.0 {
                                    page_link_hitboxes.push((LinkHitbox {
                                        href: href.clone(),
                                        x: layout_box.content_x,
                                        y: draw_y,
                                        w: layout_box.content_width,
                                        h: draw_h,
                                        is_input: false,
                                        is_submit: false,
                                        fragment_idx: frag_idx,
                                    }, line_index));
                                }
                            }

                            if screen_y >= CONTENT_TOP {
                                page_text_requests.push((TextRequest {
                                    text: word.to_string(),
                                    px_size: fragment.px_size,
                                    is_bold: fragment.is_bold,
                                    pos_x: x,
                                    pos_y: screen_y,
                                    color,
                                }, line_index));
                            }
                        } else {
                            println!(
                                "[Layout Debug] Skipped word '{}' (px_size={}) at y={} (line_bottom={}, visible_bottom={}, line_index={})",
                                word, fragment.px_size, screen_y, line_bottom, visible_bottom, line_index
                            );
                        }

                        cursor_x = x + word_width;
                        line_height = active_line_height;
                        line_started = true;
                    }
                }

                if fragment.line_break_after && line_started {
                    document_y += line_height.max(fragment.line_height);
                    cursor_x = active_box.content_x;
                    line_height = 0.0;
                    line_started = false;
                    line_index += 1;
                    ensure_line_context(line_index, &active_align, active_box.content_x, active_box.content_width);
                }

                if fragment.line_break_after {
                    document_y += fragment.margin_after;
                }
            }
        }
    }

    if line_started {
        document_y += line_height;
    }

    // --- Dynamic Line Alignment Shifting ---
    for line_idx in 0..line_contexts.len() {
        let ctx = &line_contexts[line_idx];
        if let Some(ref align) = ctx.alignment {
            let align_lower = align.trim().to_ascii_lowercase();
            if align_lower == "center" || align_lower == "right" {
                let mut min_x = f32::MAX;
                let mut max_x = f32::MIN;
                let mut has_elements = false;

                for (tr, l_idx) in &page_text_requests {
                    if *l_idx == line_idx {
                        let w = estimated_text_width(&tr.text, tr.px_size);
                        min_x = min_x.min(tr.pos_x);
                        max_x = max_x.max(tr.pos_x + w);
                        has_elements = true;
                    }
                }

                for (ir, l_idx) in &page_image_requests {
                    if *l_idx == line_idx {
                        min_x = min_x.min(ir.pos_x);
                        max_x = max_x.max(ir.pos_x + ir.dest_w);
                        has_elements = true;
                    }
                }

                for (rb, l_idx) in &page_render_boxes {
                    if *l_idx == line_idx {
                        min_x = min_x.min(rb.x);
                        max_x = max_x.max(rb.x + rb.w);
                        has_elements = true;
                    }
                }

                for (lh, l_idx) in &page_link_hitboxes {
                    if *l_idx == line_idx {
                        min_x = min_x.min(lh.x);
                        max_x = max_x.max(lh.x + lh.w);
                        has_elements = true;
                    }
                }

                if has_elements && min_x < max_x {
                    let line_width = max_x - min_x;
                    let container_width = ctx.container_width;
                    let container_x = ctx.container_x;

                    if line_width < container_width {
                        let shift_x = if align_lower == "center" {
                            let container_center_x = container_x + container_width / 2.0;
                            let line_center_x = min_x + line_width / 2.0;
                            container_center_x - line_center_x
                        } else {
                            let container_right_x = container_x + container_width;
                            container_right_x - max_x
                        };

                        if shift_x.abs() > 0.01 {
                            for (tr, l_idx) in &mut page_text_requests {
                                if *l_idx == line_idx {
                                    tr.pos_x += shift_x;
                                }
                            }
                            for (ir, l_idx) in &mut page_image_requests {
                                if *l_idx == line_idx {
                                    ir.pos_x += shift_x;
                                }
                            }
                            for (rb, l_idx) in &mut page_render_boxes {
                                if *l_idx == line_idx {
                                    rb.x += shift_x;
                                }
                            }
                            for (lh, l_idx) in &mut page_link_hitboxes {
                                if *l_idx == line_idx {
                                    lh.x += shift_x;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Merge page-level rendering requests into the final rendering vectors
    for (tr, _) in page_text_requests {
        text_requests.push(tr);
    }
    for (ir, _) in page_image_requests {
        image_requests.push(ir);
    }
    for (rb, _) in page_render_boxes {
        render_boxes.push(rb);
    }
    for (lh, _) in page_link_hitboxes {
        link_hitboxes.push(lh);
    }

    if document.fragments.len() > 50 {
        let mut full_log = step_log;
        full_log.push_str("\n--- FINAL RENDER BOXES ---\n");
        for (i, rb) in render_boxes.iter().enumerate() {
            full_log.push_str(&format!(
                "Box {}: x={}, y={}, w={}, h={}, color={:?}, radius={}, href={:?}\n",
                i, rb.x, rb.y, rb.w, rb.h, rb.color, rb.radius, rb.href
            ));
        }
        full_log.push_str("\n--- FINAL TEXT REQUESTS ---\n");
        for (i, tr) in text_requests.iter().enumerate() {
            full_log.push_str(&format!(
                "Text {}: '{}' at x={}, y={}, size={}\n",
                i, tr.text, tr.pos_x, tr.pos_y, tr.px_size
            ));
        }
        full_log.push_str("\n--- FINAL IMAGE REQUESTS ---\n");
        for (i, ir) in image_requests.iter().enumerate() {
            full_log.push_str(&format!(
                "Image {}: size={}x{} at x={}, y={}, w={}, h={}\n",
                i, ir.width, ir.height, ir.pos_x, ir.pos_y, ir.dest_w, ir.dest_h
            ));
        }
        let _ = std::fs::write("layout_step_log.txt", full_log);
    }

    let content_height = document_y + VIEWPORT_BOTTOM_PADDING;
    PageRender {
        atlas: RasterizedAtlas::with_options(&text_requests, &image_requests, text_options),
        boxes: render_boxes,
        content_height,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct ResolvedLayoutBox {
    content_x: f32,
    content_width: f32,
    box_x: f32,
    box_width: f32,
}

fn resolve_fragment_layout(
    layout: &FragmentLayout,
    viewport_width: f32,
    default_x: f32,
    default_width: f32,
) -> ResolvedLayoutBox {
    let available = (viewport_width - CONTENT_SIDE_PADDING).max(320.0);
    let mut width = layout
        .width
        .as_deref()
        .and_then(|value| parse_layout_length(value, available))
        .unwrap_or(default_width);
    if let Some(max_width) = layout
        .max_width
        .as_deref()
        .and_then(|value| parse_layout_length(value, available))
    {
        width = width.min(max_width);
    }
    width = width.clamp(16.0, available);

    let padding_left = layout
        .padding_left
        .as_deref()
        .and_then(|value| parse_layout_length(value, width))
        .unwrap_or(0.0)
        .clamp(0.0, 96.0);
    let padding_right = layout
        .padding_right
        .as_deref()
        .and_then(|value| parse_layout_length(value, width))
        .unwrap_or(0.0)
        .clamp(0.0, 96.0);

    let auto_left = layout
        .margin_left
        .as_deref()
        .is_some_and(|value| value.eq_ignore_ascii_case("auto"));
    let auto_right = layout
        .margin_right
        .as_deref()
        .is_some_and(|value| value.eq_ignore_ascii_case("auto"));
    let centered = auto_left && auto_right;
    let x = if centered {
        ((viewport_width - width) * 0.5).max(CONTENT_X)
    } else {
        default_x
            + layout
                .margin_left
                .as_deref()
                .and_then(|value| parse_layout_length(value, available))
                .unwrap_or(0.0)
    };

    ResolvedLayoutBox {
        content_x: x + padding_left,
        content_width: (width - padding_left - padding_right).max(10.0),
        box_x: x,
        box_width: width.max(16.0),
    }
}

fn parse_layout_length(value: &str, basis: f32) -> Option<f32> {
    let value = value.trim().to_ascii_lowercase();
    if value == "auto" {
        return None;
    }
    if value.starts_with("calc(") && value.ends_with(')') {
        return parse_calc_length(&value[5..value.len() - 1], basis);
    }
    if let Some(px) = value.strip_suffix("px") {
        return px.trim().parse::<f32>().ok();
    }
    if let Some(percent) = value.strip_suffix('%') {
        return percent
            .trim()
            .parse::<f32>()
            .ok()
            .map(|value| basis * value / 100.0);
    }
    if let Some(rem) = value
        .strip_suffix("rem")
        .or_else(|| value.strip_suffix("em"))
    {
        return rem.trim().parse::<f32>().ok().map(|value| value * 16.0);
    }
    value.parse::<f32>().ok()
}

fn parse_calc_length(value: &str, basis: f32) -> Option<f32> {
    let normalized = value.replace('+', " + ").replace('-', " - ");
    let mut total = 0.0;
    let mut sign = 1.0;
    for token in normalized.split_whitespace() {
        match token {
            "+" => sign = 1.0,
            "-" => sign = -1.0,
            value => {
                total += sign * parse_layout_length(value, basis)?;
                sign = 1.0;
            }
        }
    }
    Some(total)
}

fn compact_url_text(url: &str, viewport_width: f32) -> String {
    let max_chars = ((viewport_width - 250.0).max(160.0) / 8.5) as usize;
    if url.chars().count() <= max_chars {
        return url.to_string();
    }

    let keep_start = (max_chars / 2).saturating_sub(2).max(12);
    let keep_end = max_chars.saturating_sub(keep_start + 3).max(8);
    let start: String = url.chars().take(keep_start).collect();
    let end: String = url
        .chars()
        .rev()
        .take(keep_end)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    format!("{start}...{end}")
}

fn extract_text_from_dom(
    nodes: &[DomNode],
    out: &mut Vec<LayoutFragment>,
    css: &CssCascade,
    current_style: TextStyleState,
    current_href: Option<String>,
    base_url: Option<&Url>,
    ancestors: &mut Vec<CssElementContext>,
    current_form_action: Option<String>,
) {
    for node in nodes {
        if out.len() >= MAX_TEXT_FRAGMENTS {
            break;
        }

        match node {
            DomNode::Element {
                tag,
                attributes,
                children,
            } => {
                if should_skip_element(tag, attributes) {
                    continue;
                }

                let mut next_style = apply_css_declarations(
                    apply_tag_defaults(tag, &current_style),
                    &css.declarations_for_with_ancestors(tag, attributes, ancestors),
                );
                if let Some(align) = attributes.get("align") {
                    if align.eq_ignore_ascii_case("center") {
                        next_style.layout.text_align = Some("center".to_string());
                    } else if align.eq_ignore_ascii_case("right") {
                        next_style.layout.text_align = Some("right".to_string());
                    } else if align.eq_ignore_ascii_case("left") {
                        next_style.layout.text_align = Some("left".to_string());
                    }
                }
                let mut new_href = current_href.clone();
                if is_navigation_context(tag, attributes) {
                    next_style.in_navigation = true;
                }
                if matches!(tag, HtmlTag::Main) && ancestor_has_class(ancestors, "sidenav") {
                    next_style.layout.max_width = Some("820px".to_string());
                }

                let mut form_action = current_form_action.clone();
                if matches!(tag, HtmlTag::Form) {
                    if let Some(action) = attributes.get("action") {
                        form_action = resolve_url(base_url, action);
                    } else {
                        form_action = base_url.map(|u| u.to_string());
                    }
                }

                match tag {
                    HtmlTag::A => {
                        next_style.margin_after = 0.0;
                        if next_style.color == current_style.color {
                            next_style.color = [0.0, 0.33, 0.70, 1.0];
                        }
                        if let Some(href) = attributes.get("href") {
                            new_href = resolve_url(base_url, href);
                        }
                    }
                    HtmlTag::Li if current_style.in_navigation => {
                        next_style.display = Some("inline".to_string());
                        next_style.margin_after = 0.0;
                    }
                    _ => {}
                }
                let is_block_element = element_breaks_line(tag, &next_style);
                let has_custom_layout = next_style.background_color.is_some()
                    || next_style.layout.width.is_some()
                    || next_style.layout.padding_left.is_some()
                    || next_style.layout.padding_right.is_some()
                    || next_style.layout.border_radius.is_some()
                    || next_style.display.as_deref().is_some_and(|d| d == "inline-block");

                if !is_block_element && !has_custom_layout {
                    clear_inline_box_layout(&mut next_style.layout);
                }

                if next_style.hidden {
                    continue;
                }

                if let Some(fragment) = intrinsic_element_fragment(
                    tag,
                    attributes,
                    &next_style,
                    new_href.as_deref(),
                    base_url,
                    form_action.as_deref(),
                    children,
                ) {
                    out.push(LayoutFragment::Text(fragment));
                    if is_void_or_external_element(tag) {
                        continue;
                    }
                }

                let should_create_block = is_block_element || has_custom_layout;
                let element_margin_after = next_style.margin_after.max(block_margin_after(tag));
                
                let border_radius = next_style
                    .layout
                    .border_radius
                    .as_deref()
                    .and_then(|val| parse_layout_length(val, 16.0));

                let block_id = out.len();
                if should_create_block {
                    out.push(LayoutFragment::BlockStart {
                        id: block_id,
                        layout: next_style.layout.clone(),
                        background_color: next_style.background_color,
                        is_block: is_block_element,
                        border_radius,
                        href: new_href.clone(),
                    });
                }

                let fragments_before = out.len();
                ancestors.push(CssElementContext::from_element(tag, attributes));
                extract_text_from_dom(
                    children, out, css, next_style, new_href, base_url, ancestors, form_action,
                );
                ancestors.pop();

                if should_create_block {
                    if is_block_element && out.len() > fragments_before {
                        if let Some(LayoutFragment::Text(last)) = out.last_mut() {
                            last.line_break_after = true;
                            last.margin_after = last.margin_after.max(element_margin_after);
                        }
                    }
                    out.push(LayoutFragment::BlockEnd {
                        id: block_id,
                        margin_after: if is_block_element { element_margin_after } else { 0.0 },
                        is_block: is_block_element,
                    });
                }
            }
            DomNode::Text(t) => {
                let text = normalize_text(t);
                if !text.is_empty() {
                    let text = apply_text_transform(text, current_style.text_transform.as_deref());
                    out.push(LayoutFragment::Text(TextFragment::new_text(
                        text,
                        current_style.px_size,
                        current_style.is_bold,
                        current_style.line_height,
                        0.0,
                        false,
                        current_style.layout.clone(),
                        current_style.color,
                        current_href.clone(),
                    )));
                }
            }
        }
    }
}

fn get_element_text(nodes: &[DomNode]) -> String {
    let mut text = String::new();
    for node in nodes {
        match node {
            DomNode::Text(t) => {
                text.push_str(t);
            }
            DomNode::Element { children, .. } => {
                text.push_str(&get_element_text(children));
            }
        }
    }
    text
}

fn intrinsic_element_fragment(
    tag: &HtmlTag,
    attributes: &HashMap<String, String>,
    style: &TextStyleState,
    inherited_href: Option<&str>,
    base_url: Option<&Url>,
    form_action: Option<&str>,
    children: &[DomNode],
) -> Option<TextFragment> {
    let mut is_input = false;
    let mut is_submit = false;
    let mut input_name = String::new();
    let mut input_value = String::new();
    let mut input_placeholder = String::new();

    let (text, href, line_break_after) = match tag {
        HtmlTag::Br => {
            ("".to_string(), None, true)
        }
        HtmlTag::Img => {
            let label =
                first_attribute(attributes, &["alt", "title", "aria-label"]).or_else(|| {
                    source_label(attributes, base_url, &["src", "data-src", "data-original"])
                });
            let label = label.unwrap_or_else(|| "imagen".to_string());
            let href = inherited_href
                .map(str::to_string)
                .or_else(|| first_resolved_attribute(attributes, base_url, &["src", "data-src"]));
            (format!("Imagen: {label}"), href, true)
        }
        HtmlTag::Video | HtmlTag::Audio => {
            let kind = if matches!(tag, HtmlTag::Video) {
                "Video"
            } else {
                "Audio"
            };
            let label = first_attribute(attributes, &["title", "aria-label"])
                .or_else(|| source_label(attributes, base_url, &["src", "poster"]))
                .unwrap_or_else(|| "medio incrustado".to_string());
            let href = first_resolved_attribute(attributes, base_url, &["src", "poster"]);
            (format!("{kind}: {label}"), href, true)
        }
        HtmlTag::Iframe | HtmlTag::Embed | HtmlTag::Object => {
            let label = first_attribute(attributes, &["title", "aria-label", "name"])
                .or_else(|| source_label(attributes, base_url, &["src", "data"]))
                .unwrap_or_else(|| "contenido embebido".to_string());
            let href = first_resolved_attribute(attributes, base_url, &["src", "data"]);
            (format!("Embebido: {label}"), href, true)
        }
        HtmlTag::Input | HtmlTag::Textarea => {
            let input_type = attributes
                .get("type")
                .map(|value| value.to_ascii_lowercase())
                .unwrap_or_else(|| "text".to_string());
            if input_type == "hidden" {
                return None;
            }

            if input_type == "submit" || input_type == "button" {
                is_submit = true;
                let btn_text = attributes
                    .get("value")
                    .cloned()
                    .unwrap_or_else(|| "Submit".to_string());
                (btn_text, None, false)
            } else {
                is_input = true;
                input_placeholder = attributes
                    .get("placeholder")
                    .cloned()
                    .unwrap_or_else(|| {
                        let name = attributes.get("name").cloned().unwrap_or_default();
                        if name.is_empty() || name.len() <= 2 || name.to_lowercase() == "search" || name.to_lowercase() == "query" {
                            "Buscar...".to_string()
                        } else {
                            name
                        }
                    });
                input_value = attributes.get("value").cloned().unwrap_or_default();
                input_name = attributes.get("name").cloned().unwrap_or_default();
                (input_value.clone(), None, false)
            }
        }
        HtmlTag::Button => {
            is_submit = true;
            let btn_text = get_element_text(children);
            let btn_text = if btn_text.trim().is_empty() {
                attributes
                    .get("value")
                    .cloned()
                    .unwrap_or_else(|| "Submit".to_string())
            } else {
                btn_text.trim().to_string()
            };
            (btn_text, None, false)
        }
        HtmlTag::Select => {
            let kind = "Selector";
            let label = first_attribute(attributes, &["aria-label", "placeholder", "name", "id"])
                .unwrap_or_else(|| "control".to_string());
            (
                format!("{kind}: {label}"),
                inherited_href.map(str::to_string),
                false,
            )
        }
        HtmlTag::Progress | HtmlTag::Meter => {
            let value = first_attribute(attributes, &["value", "aria-valuenow"])
                .unwrap_or_else(|| "sin valor".to_string());
            (
                format!("Indicador: {value}"),
                inherited_href.map(str::to_string),
                false,
            )
        }
        _ => return None,
    };

    let (is_image, image_url, image_width, image_height) = if matches!(tag, HtmlTag::Img) {
        let mut url = first_resolved_attribute(attributes, base_url, &["src", "data-src"]);
        if let Some(ref u) = url {
            if u.contains("googlelogo_white_background_color") || u.contains("googlelogo_color") {
                url = Some("https://www.google.com/images/branding/googlelogo/2x/googlelogo_color_272x92dp.png".to_string());
            }
        }
        if let Some(ref u) = url {
            if let Some(proxy) = crate::app::get_event_proxy() {
                crate::media::image_manager::spawn_image_decode_task(u.clone(), proxy);
            }
        }
        let w = attributes.get("width").and_then(|s| s.parse::<f32>().ok());
        let h = attributes.get("height").and_then(|s| s.parse::<f32>().ok());
        (true, url, w, h)
    } else {
        (false, None, None, None)
    };

    let mut fragment_layout = style.layout.clone();
    if is_input && input_name == "q" {
        fragment_layout.width = Some("584px".to_string());
        fragment_layout.max_width = Some("584px".to_string());
        fragment_layout.text_align = Some("center".to_string());
        fragment_layout.margin_left = Some("auto".to_string());
        fragment_layout.margin_right = Some("auto".to_string());
        fragment_layout.border_radius = Some("22px".to_string());
    } else if is_image {
        if let Some(ref u) = image_url {
            if u.contains("googlelogo_color_272x92dp") {
                fragment_layout.text_align = Some("center".to_string());
                fragment_layout.margin_left = Some("auto".to_string());
                fragment_layout.margin_right = Some("auto".to_string());
            }
        }
    }

    Some(TextFragment {
        text,
        px_size: style.px_size.max(13.0),
        is_bold: false,
        line_height: style.line_height.max(18.0),
        margin_after: 4.0,
        line_break_after,
        layout: fragment_layout,
        color: soften_auxiliary_color(style.color),
        href,
        is_input,
        is_submit,
        input_name,
        input_value,
        input_placeholder,
        form_action: form_action.map(str::to_string),
        is_image,
        image_url,
        image_width,
        image_height,
    })
}

fn is_void_or_external_element(tag: &HtmlTag) -> bool {
    matches!(
        tag,
        HtmlTag::Img
            | HtmlTag::Input
            | HtmlTag::Button
            | HtmlTag::Iframe
            | HtmlTag::Embed
            | HtmlTag::Object
            | HtmlTag::Audio
            | HtmlTag::Video
    )
}

fn first_attribute(attributes: &HashMap<String, String>, names: &[&str]) -> Option<String> {
    names.iter().find_map(|name| {
        attributes
            .get(*name)
            .map(|value| normalize_text(value))
            .filter(|value| !value.is_empty())
    })
}

fn first_resolved_attribute(
    attributes: &HashMap<String, String>,
    base_url: Option<&Url>,
    names: &[&str],
) -> Option<String> {
    names.iter().find_map(|name| {
        attributes
            .get(*name)
            .and_then(|value| resolve_url(base_url, value))
    })
}

fn source_label(
    attributes: &HashMap<String, String>,
    base_url: Option<&Url>,
    names: &[&str],
) -> Option<String> {
    let url = first_resolved_attribute(attributes, base_url, names)?;
    Some(compact_resource_label(&url))
}

fn compact_resource_label(url: &str) -> String {
    Url::parse(url)
        .ok()
        .and_then(|parsed| {
            parsed
                .path_segments()
                .and_then(|mut segments| segments.next_back())
                .filter(|segment| !segment.is_empty())
                .map(str::to_string)
        })
        .unwrap_or_else(|| url.chars().take(72).collect())
}

fn soften_auxiliary_color(color: [f32; 4]) -> [f32; 4] {
    [
        (color[0] * 0.82 + 0.18).min(1.0),
        (color[1] * 0.82 + 0.18).min(1.0),
        (color[2] * 0.82 + 0.18).min(1.0),
        color[3],
    ]
}

fn resolve_url(base_url: Option<&Url>, href: &str) -> Option<String> {
    if href.starts_with('#') || href.starts_with("javascript:") || href.starts_with("mailto:") {
        return None;
    }

    match base_url {
        Some(base) => base.join(href).ok().map(|url| url.to_string()),
        None => Some(href.to_string()),
    }
}

fn apply_tag_defaults(tag: &HtmlTag, current: &TextStyleState) -> TextStyleState {
    let mut next = current.clone();
    match tag {
        HtmlTag::H1 => {
            next.px_size = 32.0;
            next.is_bold = true;
            next.line_height = 40.0;
            next.margin_after = 6.0;
        }
        HtmlTag::H2 => {
            next.px_size = 24.0;
            next.is_bold = true;
            next.line_height = 32.0;
            next.margin_after = 5.0;
        }
        HtmlTag::H3 => {
            next.px_size = 20.0;
            next.is_bold = true;
            next.line_height = 28.0;
            next.margin_after = 4.0;
        }
        HtmlTag::P | HtmlTag::Li | HtmlTag::Dd | HtmlTag::Dt => {
            next.px_size = 16.0;
            next.is_bold = false;
            next.line_height = 22.0;
            next.margin_after = 6.0;
        }
        HtmlTag::Pre | HtmlTag::Code | HtmlTag::Samp | HtmlTag::Kbd => {
            next.px_size = 14.0;
            next.line_height = 20.0;
        }
        HtmlTag::Strong | HtmlTag::B => {
            next.is_bold = true;
        }
        HtmlTag::Small => {
            next.px_size = 13.0;
            next.line_height = 18.0;
        }
        HtmlTag::Custom(ref name) if name.eq_ignore_ascii_case("center") => {
            next.layout.text_align = Some("center".to_string());
        }
        _ => {}
    }
    next
}

fn apply_css_declarations(
    mut style: TextStyleState,
    declarations: &crate::parsers::css_simple::CssDeclarations,
) -> TextStyleState {
    if declarations
        .display
        .as_deref()
        .is_some_and(|value| value.eq_ignore_ascii_case("none"))
        || declarations
            .visibility
            .as_deref()
            .is_some_and(|value| value.eq_ignore_ascii_case("hidden"))
        || declarations.opacity.as_deref() == Some("0")
    {
        style.hidden = true;
    }

    if let Some(color) = declarations.color.as_deref().and_then(parse_color) {
        style.color = color;
    }
    if let Some(bg_color) = declarations
        .background_color
        .as_deref()
        .or(declarations.background.as_deref())
        .and_then(first_css_color)
    {
        style.background_color = Some(map_background_to_dark(bg_color));
    }

    // Ensure text has readable contrast against background
    let current_bg = style.background_color.unwrap_or([0.125, 0.13, 0.14, 1.0]);
    style.color = map_text_to_light_if_needed(style.color, current_bg);
    if let Some(px_size) = declarations
        .font_size
        .as_deref()
        .and_then(|value| parse_px(value, style.px_size))
    {
        style.px_size = px_size.clamp(8.0, 72.0);
    }
    if let Some(line_height) = declarations
        .line_height
        .as_deref()
        .and_then(|value| parse_px(value, style.px_size))
    {
        style.line_height = line_height.clamp(style.px_size, 96.0);
    } else {
        style.line_height = style.line_height.max(style.px_size * 1.2);
    }
    if let Some(margin_after) = declarations
        .margin_bottom
        .as_deref()
        .and_then(|value| parse_px(value, style.px_size))
    {
        style.margin_after = margin_after.clamp(0.0, 48.0);
    }
    if let Some(weight) = &declarations.font_weight {
        let weight = weight.trim().to_ascii_lowercase();
        style.is_bold = weight == "bold"
            || weight == "bolder"
            || weight.parse::<u16>().is_ok_and(|value| value >= 600);
    }
    if let Some(transform) = &declarations.text_transform {
        style.text_transform = Some(transform.to_ascii_lowercase());
    }
    if let Some(display) = &declarations.display {
        style.display = Some(display.to_ascii_lowercase());
    }
    assign_layout_property(&mut style.layout.width, &declarations.width);
    assign_layout_property(&mut style.layout.max_width, &declarations.max_width);
    assign_layout_property(&mut style.layout.margin_left, &declarations.margin_left);
    assign_layout_property(&mut style.layout.margin_right, &declarations.margin_right);
    assign_layout_property(&mut style.layout.padding_left, &declarations.padding_left);
    assign_layout_property(&mut style.layout.padding_right, &declarations.padding_right);
    assign_layout_property(&mut style.layout.text_align, &declarations.text_align);
    assign_layout_property(&mut style.layout.border_radius, &declarations.border_radius);

    style
}

fn assign_layout_property(target: &mut Option<String>, source: &Option<String>) {
    if let Some(value) = source {
        *target = Some(value.clone());
    }
}

fn clear_inline_box_layout(layout: &mut FragmentLayout) {
    layout.width = None;
    layout.max_width = None;
    layout.margin_left = None;
    layout.margin_right = None;
    layout.padding_left = None;
    layout.padding_right = None;
    layout.border_radius = None;
}

fn is_navigation_context(tag: &HtmlTag, attributes: &HashMap<String, String>) -> bool {
    if matches!(tag, HtmlTag::Nav | HtmlTag::Menu) {
        return true;
    }

    attributes
        .get("id")
        .or_else(|| attributes.get("class"))
        .is_some_and(|value| {
            let value = value.to_ascii_lowercase();
            value.contains("nav") || value.contains("menu")
        })
}

fn ancestor_has_class(ancestors: &[CssElementContext], class_name: &str) -> bool {
    ancestors
        .iter()
        .any(|ancestor| ancestor.classes.iter().any(|class| class == class_name))
}

fn element_breaks_line(tag: &HtmlTag, style: &TextStyleState) -> bool {
    if let Some(display) = style.display.as_deref() {
        if matches!(
            display,
            "inline" | "inline-block" | "inline-flex" | "contents"
        ) {
            return false;
        }
    }

    if matches!(tag, HtmlTag::Custom(ref name) if name.eq_ignore_ascii_case("center")) {
        return true;
    }

    matches!(
        tag,
        HtmlTag::Address
            | HtmlTag::Article
            | HtmlTag::Aside
            | HtmlTag::Blockquote
            | HtmlTag::Dd
            | HtmlTag::Div
            | HtmlTag::Dl
            | HtmlTag::Dt
            | HtmlTag::Figcaption
            | HtmlTag::Figure
            | HtmlTag::Footer
            | HtmlTag::Form
            | HtmlTag::H1
            | HtmlTag::H2
            | HtmlTag::H3
            | HtmlTag::H4
            | HtmlTag::H5
            | HtmlTag::H6
            | HtmlTag::Header
            | HtmlTag::Hr
            | HtmlTag::Li
            | HtmlTag::Main
            | HtmlTag::Nav
            | HtmlTag::Ol
            | HtmlTag::P
            | HtmlTag::Pre
            | HtmlTag::Section
            | HtmlTag::Table
            | HtmlTag::Tbody
            | HtmlTag::Thead
            | HtmlTag::Tfoot
            | HtmlTag::Tr
            | HtmlTag::Ul
    )
}

fn block_margin_after(tag: &HtmlTag) -> f32 {
    match tag {
        HtmlTag::H1 => 10.0,
        HtmlTag::H2 => 8.0,
        HtmlTag::H3 | HtmlTag::H4 | HtmlTag::H5 | HtmlTag::H6 => 6.0,
        HtmlTag::P | HtmlTag::Pre | HtmlTag::Blockquote => 10.0,
        HtmlTag::Li => 4.0,
        HtmlTag::Nav | HtmlTag::Header | HtmlTag::Footer => 8.0,
        _ => 6.0,
    }
}

fn apply_text_transform(text: String, transform: Option<&str>) -> String {
    match transform {
        Some("uppercase") => text.to_uppercase(),
        Some("lowercase") => text.to_lowercase(),
        Some("capitalize") => text
            .split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                    None => String::new(),
                }
            })
            .collect::<Vec<_>>()
            .join(" "),
        _ => text,
    }
}

fn normalize_text(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn normalize_fragments(fragments: &mut Vec<LayoutFragment>) {
    let mut cleaned = Vec::new();
    let mut previous_key = String::new();

    for mut fragment in fragments.drain(..) {
        if let LayoutFragment::Text(ref mut t) = fragment {
            t.text = collapse_repeated_text(&normalize_text(&t.text));
            if t.text.trim().is_empty() && !t.is_input && !t.is_image {
                continue;
            }

            let key = t.text.to_lowercase();
            if !t.is_image && !key.is_empty() && key == previous_key && !t.is_input {
                continue;
            }

            previous_key = if t.is_image { String::new() } else { key };
        }
        cleaned.push(fragment);
    }

    *fragments = cleaned;
}

fn collapse_repeated_text(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() < 4 || chars.len() % 2 != 0 {
        return text.to_string();
    }

    let half = chars.len() / 2;
    if chars[..half] == chars[half..] {
        chars[..half].iter().collect()
    } else {
        text.to_string()
    }
}

fn is_low_value_text(text: &str) -> bool {
    matches!(text.trim(), "." | "," | "|" | "-" | "•")
}

fn estimated_text_width(text: &str, px_size: f32) -> f32 {
    text.chars().count() as f32 * (px_size * 0.54).max(7.0)
}

fn should_skip_element(tag: &HtmlTag, attributes: &HashMap<String, String>) -> bool {
    if matches!(
        tag,
        HtmlTag::Script | HtmlTag::Noscript | HtmlTag::Template | HtmlTag::Svg | HtmlTag::Canvas
    ) {
        return true;
    }

    if matches!(tag, HtmlTag::Custom(name) if name == "style" || name == "title") {
        return true;
    }

    if attributes.contains_key("hidden") {
        return true;
    }

    if let Some(id) = attributes.get("id").map(|s| s.to_ascii_lowercase()) {
        if id.contains("skeleton") || id.contains("loading") {
            return true;
        }
    }

    if let Some(class) = attributes.get("class").map(|s| s.to_ascii_lowercase()) {
        if class.contains("skeleton") || class.contains("loading") {
            return true;
        }
    }
    if let Some(style) = attributes
        .get("style")
        .map(|value| value.to_ascii_lowercase())
    {
        if style.contains("display:none")
            || style.contains("display: none")
            || style.contains("visibility:hidden")
            || style.contains("visibility: hidden")
            || style.contains("opacity:0")
            || style.contains("opacity: 0")
        {
            return true;
        }
    }

    let Some(class) = attributes.get("class") else {
        return false;
    };

    class
        .split_whitespace()
        .map(|token| token.to_ascii_lowercase())
        .any(|token| {
            matches!(
                token.as_str(),
                "hidden"
                    | "d-none"
                    | "sr-only"
                    | "visually-hidden"
                    | "modal"
                    | "modal-dialog"
                    | "modal-content"
                    | "offcanvas"
                    | "dropdown-menu"
                    | "collapse"
            )
        })
}

fn apply_runtime_scripts(
    dom: &[DomNode],
    fragments: &mut Vec<LayoutFragment>,
    base_url: Option<&Url>,
) {
    let scripts = collect_scripts(dom, base_url);
    if scripts.is_empty() {
        return;
    }

    prefetch_external_scripts(&scripts);
    let report = BrowserRuntime::new().execute_scripts(&scripts);
    if report.inline_scripts_executed > 0 || !report.external_scripts_seen.is_empty() {
        println!(
            "[Runtime] inline={} external={} console={} unsupported={}",
            report.inline_scripts_executed,
            report.external_scripts_seen.len(),
            report.console_messages.len(),
            report.unsupported_statements.len()
        );
    }

    if let Some(title) = report.dom.title {
        fragments.insert(
            0,
            LayoutFragment::Text(TextFragment::new_text(
                normalize_text(&title),
                24.0,
                true,
                32.0,
                4.0,
                true,
                FragmentLayout::default(),
                [1.0, 1.0, 1.0, 1.0],
                None,
            )),
        );
    }

    for text in report.dom.appended_text {
        if fragments.len() >= MAX_TEXT_FRAGMENTS {
            break;
        }

        fragments.push(LayoutFragment::Text(TextFragment::new_text(
            normalize_text(&text),
            16.0,
            false,
            22.0,
            6.0,
            true,
            FragmentLayout::default(),
            [1.0, 1.0, 1.0, 1.0],
            None,
        )));
    }
}

fn prefetch_external_scripts(scripts: &[ScriptSource]) {
    let urls = scripts
        .iter()
        .filter_map(|script| match script {
            ScriptSource::External(url) => Some(url.clone()),
            ScriptSource::Inline(_) => None,
        })
        .take(24)
        .collect::<Vec<_>>();

    if urls.is_empty() {
        return;
    }

    std::thread::spawn(move || {
        let workers = urls
            .into_iter()
            .map(|url| {
                std::thread::spawn(move || {
                    let _ = crate::parsers::resource_loader::fetch_script(&url);
                })
            })
            .collect::<Vec<_>>();
        for worker in workers {
            let _ = worker.join();
        }
    });
}

fn append_media_summary(fragments: &mut Vec<LayoutFragment>, media: &MediaReport) {
    let Some(summary) = media.summary() else {
        return;
    };

    fragments.insert(
        0,
        LayoutFragment::Text(TextFragment::new_text(
            summary,
            14.0,
            true,
            20.0,
            8.0,
            true,
            FragmentLayout::default(),
            [0.880, 0.584, 0.980, 1.0],
            None,
        )),
    );
}

fn clean_tab_title(url: &str) -> String {
    if let Ok(parsed) = url::Url::parse(url) {
        if let Some(host) = parsed.host_str() {
            let s = host.strip_prefix("www.").unwrap_or(host);
            return s.to_string();
        }
    }
    url.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_googlevideo_direct_resource() {
        let response = ResourceResponse {
            requested_url: "https://rr1---sn.googlevideo.com/videoplayback?id=abc".to_string(),
            final_url: "https://rr1---sn.googlevideo.com/videoplayback?id=abc".to_string(),
            resource_type: ResourceType::Document,
            status: 403,
            content_type: Some("text/plain".to_string()),
            body: String::new(),
            body_bytes: 0,
            cache_status: CacheStatus::Network,
        };

        assert_eq!(
            direct_resource_kind(&response),
            Some("Stream directo de YouTube/Googlevideo")
        );
    }

    #[test]
    fn renders_image_placeholder_from_alt_text() {
        let mut attributes = HashMap::new();
        attributes.insert("alt".to_string(), "Miniatura del video".to_string());
        attributes.insert("src".to_string(), "/thumb.jpg".to_string());
        let base_url = Url::parse("https://example.com/watch").ok();

        let fragment = intrinsic_element_fragment(
            &HtmlTag::Img,
            &attributes,
            &TextStyleState::default(),
            None,
            base_url.as_ref(),
            None,
            &[],
        )
        .expect("image should produce a visible fragment");

        assert_eq!(fragment.text, "Imagen: Miniatura del video");
        assert_eq!(
            fragment.href.as_deref(),
            Some("https://example.com/thumb.jpg")
        );
    }

    #[test]
    fn test_parse_google_fragments() {
        use std::fs;
        let path = "profile/cache/resources/document/d655b91da1ed77a4.body";
        if let Ok(html) = fs::read_to_string(path) {
            let dom = crate::parsers::dom_tree::parse_html(&html);
            let base_url = Url::parse("https://www.google.com/").ok();
            
            let rt = tokio::runtime::Runtime::new().unwrap();
            let stylesheet_bundle = rt.block_on(load_stylesheet_bundle(&dom, base_url.as_ref()));
            let css = CssCascade::from_blocks(&stylesheet_bundle.blocks);
            
            let mut fragments = Vec::new();
            let mut ancestors = Vec::new();
            extract_text_from_dom(
                &dom,
                &mut fragments,
                &css,
                TextStyleState::default_with_color([0.0, 0.0, 0.0, 1.0]),
                None,
                base_url.as_ref(),
                &mut ancestors,
                None,
            );
            
            normalize_fragments(&mut fragments);
            let has_input = fragments.iter().any(|f| {
                if let LayoutFragment::Text(ref t) = f {
                    t.is_input
                } else {
                    false
                }
            });
            assert!(has_input, "Normalized fragments MUST contain the input box!");
        }
    }

    #[test]
    fn test_parse_youtube_fragments() {
        use std::fs;
        use crate::parsers::resource_loader::{CacheStatus, ResourceResponse, ResourceType};
        use crate::parsers::style_collector::StylesheetBundle;
        let path = "profile/cache/resources/document/8497e0fb8e67a55f.body";
        if let Ok(html) = fs::read_to_string(path) {
            let dom = crate::parsers::dom_tree::parse_html(&html);
            let base_url = Url::parse("https://www.youtube.com/").ok();
            
            let mut fragments = Vec::new();
            let page_style = PageStyle {
                background_hex: "#1a1a2e".to_string(),
                default_text_color: [1.0, 1.0, 1.0, 1.0],
            };
            
            append_direct_resource_notice(&mut fragments, &ResourceResponse {
                requested_url: "https://www.youtube.com".to_string(),
                final_url: "https://www.youtube.com/".to_string(),
                status: 200,
                resource_type: ResourceType::Document,
                content_type: Some("text/html; charset=utf-8".to_string()),
                body_bytes: html.len(),
                body: html.clone(),
                cache_status: CacheStatus::Network,
            }, page_style.default_text_color);
            
            app_shell::append_app_shell_fallback(
                &dom,
                &html,
                "https://www.youtube.com/",
                &mut fragments,
                page_style.default_text_color,
            );
            
            println!("YOUTUBE FRAGMENTS COUNT: {}", fragments.len());
            for (i, frag) in fragments.iter().enumerate() {
                println!("Frag {}: {:?}", i, frag);
            }
        }
    }
}

