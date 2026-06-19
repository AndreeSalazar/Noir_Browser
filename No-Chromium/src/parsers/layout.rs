use crate::parsers::css_simple::CssCascade;
use crate::parsers::page_document::{PageDocument, TextBlock};

#[derive(Clone, Debug)]
pub enum LayoutItem {
    Text(LayoutBlock),
    Image(ImageLayoutBlock),
    Video(VideoLayoutBlock),
}

#[derive(Clone, Debug)]
pub struct ImageLayoutBlock {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub src: String,
    pub alt: String,
    pub lazy: bool,
}

#[derive(Clone, Debug)]
pub struct VideoLayoutBlock {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub src: String,
    pub poster: Option<String>,
    pub controls: bool,
    pub autoplay: bool,
}

#[derive(Clone, Debug)]
pub struct LayoutBlock {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub text: String,
    pub font_size: f32,
    pub bold: bool,
    pub color: [f32; 4],
    pub bg_color: Option<[f32; 4]>,
    pub href: Option<String>,
    pub is_link: bool,
    pub padding_top: f32,
    pub padding_bottom: f32,
    pub padding_left: f32,
    pub margin_top: f32,
    pub margin_bottom: f32,
}

pub struct LayoutContext {
    pub viewport_w: f32,
    pub content_x: f32,
    pub content_w: f32,
    pub cursor_y: f32,
    pub line_height: f32,
    pub css: CssCascade,
}

impl LayoutContext {
    pub fn new(viewport_w: f32, content_x: f32, content_w: f32) -> Self {
        Self {
            viewport_w,
            content_x,
            content_w,
            cursor_y: 0.0,
            line_height: 1.4,
            css: CssCascade::from_blocks(&[]),
        }
    }

    pub fn with_css(mut self, style_blocks: &[String]) -> Self {
        self.css = CssCascade::from_blocks(style_blocks);
        self
    }
}

pub fn layout_page(doc: &PageDocument, viewport_w: f32) -> Vec<LayoutItem> {
    let effective_w = doc.viewport_width.unwrap_or(viewport_w);
    let content_x = 40.0;
    let content_w = (effective_w - 80.0).max(200.0);

    let mut ctx = LayoutContext::new(effective_w, content_x, content_w)
        .with_css(&doc.style_blocks);

    let mut items = Vec::new();

    // Process text blocks, images, and videos together
    let mut text_idx = 0;
    let mut img_idx = 0;
    let mut vid_idx = 0;
    let max_iter = (doc.text_blocks.len()
        + doc.image_blocks.len()
        + doc.video_blocks.len()
        + 100);

    for _ in 0..max_iter {
        let done = text_idx >= doc.text_blocks.len()
            && img_idx >= doc.image_blocks.len()
            && vid_idx >= doc.video_blocks.len();
        if done { break; }

        // Video
        if vid_idx < doc.video_blocks.len() {
            let vid_block = &doc.video_blocks[vid_idx];
            let vid_w = vid_block.width.unwrap_or(640.0).min(content_w);
            let vid_h = vid_block.height.unwrap_or(360.0);
            // Scale to fit content_w
            let (final_w, final_h) = if vid_w > content_w {
                let scale = content_w / vid_w;
                (content_w, vid_h * scale)
            } else {
                (vid_w, vid_h)
            };
            ctx.cursor_y += 8.0;
            items.push(LayoutItem::Video(VideoLayoutBlock {
                x: ctx.content_x,
                y: ctx.cursor_y,
                w: final_w,
                h: final_h,
                src: vid_block.src.clone(),
                poster: vid_block.poster.clone(),
                controls: vid_block.controls,
                autoplay: vid_block.autoplay,
            }));
            ctx.cursor_y += final_h + 8.0;
            vid_idx += 1;
            continue;
        }

        // Image
        if img_idx < doc.image_blocks.len() {
            let img_block = &doc.image_blocks[img_idx];
            let img_w = img_block.width.unwrap_or(300.0).min(content_w);
            let img_h = img_block.height.unwrap_or(200.0).min(500.0);
            ctx.cursor_y += 8.0;
            items.push(LayoutItem::Image(ImageLayoutBlock {
                x: ctx.content_x,
                y: ctx.cursor_y,
                w: img_w,
                h: img_h,
                src: img_block.src.clone(),
                alt: img_block.alt.clone(),
                lazy: img_block.lazy,
            }));
            ctx.cursor_y += img_h + 8.0;
            img_idx += 1;
            continue;
        }

        // Text
        if text_idx < doc.text_blocks.len() {
            let text_block = &doc.text_blocks[text_idx];
            let styled = apply_css_to_block(text_block, &ctx.css);
            layout_block(text_block, &styled, &mut ctx, &mut items);
            text_idx += 1;
        }
    }

    items
}

struct StyledBlock {
    font_size: f32,
    bold: bool,
    color: [f32; 4],
    bg_color: Option<[f32; 4]>,
    margin_top: f32,
    margin_bottom: f32,
    padding_top: f32,
    padding_bottom: f32,
    padding_left: f32,
    indent: f32,
}

fn apply_css_to_block(block: &TextBlock, css: &CssCascade) -> StyledBlock {
    use crate::parsers::css_simple::{parse_color, parse_px};
    use crate::parsers::html_elements::HtmlTag;

    let default_color = [0.85, 0.85, 0.85, 1.0];

    let mut styled = StyledBlock {
        font_size: block.font_size,
        bold: block.bold,
        color: default_color,
        bg_color: None,
        margin_top: 0.0,
        margin_bottom: 4.0,
        padding_top: 0.0,
        padding_bottom: 0.0,
        padding_left: 0.0,
        indent: block.indent_level as f32 * 20.0,
    };

    let tag = match block.tag.as_str() {
        "h1" => HtmlTag::H1,
        "h2" => HtmlTag::H2,
        "h3" => HtmlTag::H3,
        "h4" => HtmlTag::H4,
        "p" => HtmlTag::P,
        "a" => HtmlTag::A,
        "b" => HtmlTag::B,
        "li" => HtmlTag::Li,
        "code" => HtmlTag::Code,
        "blockquote" => HtmlTag::Blockquote,
        "hr" => HtmlTag::Hr,
        _ => HtmlTag::Custom(block.tag.clone()),
    };

    let decls = css.declarations_for(&tag, &block.attributes);

    if let Some(ref bg) = decls.background_color {
        if let Some(c) = parse_color(bg) {
            styled.bg_color = Some(c);
        }
    }
    if let Some(ref color) = decls.color {
        if let Some(c) = parse_color(color) {
            styled.color = c;
        }
    }
    if let Some(ref fs) = decls.font_size {
        if let Some(v) = parse_px(fs, 14.0) {
            styled.font_size = v;
        }
    }
    if let Some(ref fw) = decls.font_weight {
        styled.bold = matches!(fw.as_str(), "bold" | "bolder" | "700" | "800" | "900");
    }
    if let Some(ref mt) = decls.margin_top {
        if let Some(v) = parse_px(mt, 0.0) {
            styled.margin_top = v;
        }
    }
    if let Some(ref mb) = decls.margin_bottom {
        if let Some(v) = parse_px(mb, 4.0) {
            styled.margin_bottom = v;
        }
    }
    if let Some(ref pt) = decls.padding_top {
        if let Some(v) = parse_px(pt, 0.0) {
            styled.padding_top = v;
        }
    }
    if let Some(ref pb) = decls.padding_bottom {
        if let Some(v) = parse_px(pb, 0.0) {
            styled.padding_bottom = v;
        }
    }
    if let Some(ref pl) = decls.padding_left {
        if let Some(v) = parse_px(pl, 0.0) {
            styled.padding_left = v;
        }
    }
    if decls.display.as_deref() == Some("none") {
        styled.font_size = 0.0;
        styled.margin_top = 0.0;
        styled.margin_bottom = 0.0;
    }

    match block.tag.as_str() {
        "h1" => {
            if decls.font_size.is_none() { styled.font_size = 28.0; }
            if decls.font_weight.is_none() { styled.bold = true; }
            if decls.margin_top.is_none() { styled.margin_top = 20.0; }
            if decls.margin_bottom.is_none() { styled.margin_bottom = 12.0; }
            if decls.color.is_none() { styled.color = [1.0, 1.0, 1.0, 1.0]; }
        }
        "h2" => {
            if decls.font_size.is_none() { styled.font_size = 22.0; }
            if decls.font_weight.is_none() { styled.bold = true; }
            if decls.margin_top.is_none() { styled.margin_top = 16.0; }
            if decls.margin_bottom.is_none() { styled.margin_bottom = 8.0; }
            if decls.color.is_none() { styled.color = [1.0, 1.0, 1.0, 1.0]; }
        }
        "h3" => {
            if decls.font_size.is_none() { styled.font_size = 18.0; }
            if decls.font_weight.is_none() { styled.bold = true; }
            if decls.margin_top.is_none() { styled.margin_top = 14.0; }
            if decls.margin_bottom.is_none() { styled.margin_bottom = 6.0; }
            if decls.color.is_none() { styled.color = [0.95, 0.95, 0.95, 1.0]; }
        }
        "h4" => {
            if decls.font_size.is_none() { styled.font_size = 16.0; }
            if decls.font_weight.is_none() { styled.bold = true; }
            if decls.margin_top.is_none() { styled.margin_top = 12.0; }
            if decls.margin_bottom.is_none() { styled.margin_bottom = 4.0; }
            if decls.color.is_none() { styled.color = [0.9, 0.9, 0.9, 1.0]; }
        }
        "p" => {
            if decls.font_size.is_none() { styled.font_size = 14.0; }
            if decls.margin_bottom.is_none() { styled.margin_bottom = 8.0; }
            if decls.color.is_none() { styled.color = [0.82, 0.82, 0.82, 1.0]; }
        }
        "a" => {
            if decls.font_size.is_none() { styled.font_size = 14.0; }
            if decls.color.is_none() { styled.color = [0.4, 0.6, 1.0, 1.0]; }
            if decls.margin_bottom.is_none() { styled.margin_bottom = 2.0; }
        }
        "b" => {
            if decls.font_size.is_none() { styled.font_size = 14.0; }
            if decls.font_weight.is_none() { styled.bold = true; }
            if decls.color.is_none() { styled.color = [1.0, 1.0, 1.0, 1.0]; }
        }
        "li" => {
            if decls.font_size.is_none() { styled.font_size = 14.0; }
            if decls.margin_bottom.is_none() { styled.margin_bottom = 2.0; }
            styled.indent += 16.0;
            if decls.color.is_none() { styled.color = [0.82, 0.82, 0.82, 1.0]; }
        }
        "code" => {
            if decls.font_size.is_none() { styled.font_size = 12.0; }
            if styled.bg_color.is_none() { styled.bg_color = Some([0.12, 0.12, 0.14, 1.0]); }
            if decls.padding_top.is_none() { styled.padding_top = 4.0; }
            if decls.padding_bottom.is_none() { styled.padding_bottom = 4.0; }
            if decls.padding_left.is_none() { styled.padding_left = 8.0; }
            if decls.margin_bottom.is_none() { styled.margin_bottom = 6.0; }
            if decls.color.is_none() { styled.color = [0.8, 0.9, 0.8, 1.0]; }
        }
        "blockquote" => {
            if decls.font_size.is_none() { styled.font_size = 14.0; }
            if decls.margin_top.is_none() { styled.margin_top = 8.0; }
            if decls.margin_bottom.is_none() { styled.margin_bottom = 8.0; }
            styled.indent += 24.0;
            if decls.padding_left.is_none() { styled.padding_left = 12.0; }
            if decls.color.is_none() { styled.color = [0.65, 0.65, 0.70, 1.0]; }
        }
        "hr" => {
            if decls.margin_top.is_none() { styled.margin_top = 12.0; }
            if decls.margin_bottom.is_none() { styled.margin_bottom = 12.0; }
            if decls.color.is_none() { styled.color = [0.35, 0.35, 0.40, 1.0]; }
        }
        "text" => {
            if decls.font_size.is_none() { styled.font_size = 14.0; }
            if decls.margin_bottom.is_none() { styled.margin_bottom = 4.0; }
            if decls.color.is_none() { styled.color = [0.82, 0.82, 0.82, 1.0]; }
        }
        "input" => {
            styled.font_size = 14.0;
            styled.margin_bottom = 6.0;
            styled.bg_color = Some([0.18, 0.18, 0.22, 1.0]);
            styled.padding_top = 6.0;
            styled.padding_bottom = 6.0;
            styled.padding_left = 8.0;
            styled.color = [0.9, 0.9, 0.9, 1.0];
        }
        "button" => {
            styled.font_size = 14.0;
            styled.margin_bottom = 6.0;
            styled.bg_color = Some([0.20, 0.35, 0.60, 1.0]);
            styled.padding_top = 6.0;
            styled.padding_bottom = 6.0;
            styled.padding_left = 12.0;
            styled.color = [1.0, 1.0, 1.0, 1.0];
            styled.bold = true;
        }
        _ => {}
    }

    styled
}

fn layout_block(block: &TextBlock, styled: &StyledBlock, ctx: &mut LayoutContext, out: &mut Vec<LayoutItem>) {
    let text = &block.text;
    if text.is_empty() {
        return;
    }

    ctx.cursor_y += styled.margin_top;

    let x = ctx.content_x + styled.indent;
    let w = ctx.content_w - styled.indent;
    let char_w = styled.font_size * 0.58;
    let chars_per_line = (w / char_w).max(10.0) as usize;

    if text.len() <= chars_per_line {
        let h = styled.font_size * ctx.line_height;
        out.push(LayoutItem::Text(LayoutBlock {
            x,
            y: ctx.cursor_y,
            w: measure_text_width_approx(text, styled.font_size),
            h,
            text: text.clone(),
            font_size: styled.font_size,
            bold: styled.bold,
            color: styled.color,
            bg_color: styled.bg_color,
            href: block.link.clone(),
            is_link: block.link.is_some(),
            padding_top: styled.padding_top,
            padding_bottom: styled.padding_bottom,
            padding_left: styled.padding_left,
            margin_top: 0.0,
            margin_bottom: 0.0,
        }));
        ctx.cursor_y += h;
    } else {
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut line = String::new();

        for word in words {
            let test = if line.is_empty() {
                word.to_string()
            } else {
                format!("{} {}", line, word)
            };
            if test.len() > chars_per_line && !line.is_empty() {
                let h = styled.font_size * ctx.line_height;
                out.push(LayoutItem::Text(LayoutBlock {
                    x,
                    y: ctx.cursor_y,
                    w: measure_text_width_approx(&line, styled.font_size),
                    h,
                    text: line,
                    font_size: styled.font_size,
                    bold: styled.bold,
                    color: styled.color,
                    bg_color: styled.bg_color.clone(),
                    href: block.link.clone(),
                    is_link: block.link.is_some(),
                    padding_top: styled.padding_top,
                    padding_bottom: styled.padding_bottom,
                    padding_left: styled.padding_left,
                    margin_top: 0.0,
                    margin_bottom: 0.0,
                }));
                ctx.cursor_y += h;
                line = word.to_string();
            } else {
                line = test;
            }
        }
        if !line.is_empty() {
            let h = styled.font_size * ctx.line_height;
            out.push(LayoutItem::Text(LayoutBlock {
                x,
                y: ctx.cursor_y,
                w: measure_text_width_approx(&line, styled.font_size),
                h,
                text: line,
                font_size: styled.font_size,
                bold: styled.bold,
                color: styled.color,
                bg_color: styled.bg_color.clone(),
                href: block.link.clone(),
                is_link: block.link.is_some(),
                padding_top: styled.padding_top,
                padding_bottom: styled.padding_bottom,
                padding_left: styled.padding_left,
                margin_top: 0.0,
                margin_bottom: 0.0,
            }));
            ctx.cursor_y += h;
        }
    }

    ctx.cursor_y += styled.margin_bottom;
}

pub fn measure_text_width_approx(text: &str, font_size: f32) -> f32 {
    text.len() as f32 * font_size * 0.58
}

pub fn total_content_height(items: &[LayoutItem]) -> f32 {
    items.iter().map(|item| match item {
        LayoutItem::Text(b) => b.y + b.h,
        LayoutItem::Image(i) => i.y + i.h,
        LayoutItem::Video(v) => v.y + v.h,
    }).fold(0.0f32, f32::max)
}

pub fn hit_test_link(items: &[LayoutItem], mx: f32, my: f32, scroll_y: f32) -> Option<String> {
    let adjusted_y = my + scroll_y;
    for item in items {
        if let LayoutItem::Text(block) = item {
            if block.is_link {
                if mx >= block.x
                    && mx <= block.x + block.w + 20.0
                    && adjusted_y >= block.y
                    && adjusted_y <= block.y + block.h
                {
                    return block.href.clone();
                }
            }
        }
    }
    None
}
