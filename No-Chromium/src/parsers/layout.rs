use crate::parsers::css_simple::{CssCascade, CssDeclarations, parse_color, parse_px};
use crate::parsers::page_document::{PageDocument, TextBlock};

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

pub fn layout_page(doc: &PageDocument, viewport_w: f32) -> Vec<LayoutBlock> {
    let content_x = 40.0;
    let content_w = (viewport_w - 80.0).max(200.0);

    let mut ctx = LayoutContext::new(viewport_w, content_x, content_w)
        .with_css(&doc.style_blocks);

    let mut blocks = Vec::new();

    for text_block in &doc.text_blocks {
        let styled = apply_css_to_block(text_block, &ctx.css);
        layout_block(text_block, &styled, &mut ctx, &mut blocks);
    }

    blocks
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

    match block.tag.as_str() {
        "h1" => {
            styled.font_size = 28.0;
            styled.bold = true;
            styled.margin_top = 20.0;
            styled.margin_bottom = 12.0;
            styled.color = [1.0, 1.0, 1.0, 1.0];
        }
        "h2" => {
            styled.font_size = 22.0;
            styled.bold = true;
            styled.margin_top = 16.0;
            styled.margin_bottom = 8.0;
            styled.color = [1.0, 1.0, 1.0, 1.0];
        }
        "h3" => {
            styled.font_size = 18.0;
            styled.bold = true;
            styled.margin_top = 14.0;
            styled.margin_bottom = 6.0;
            styled.color = [0.95, 0.95, 0.95, 1.0];
        }
        "h4" => {
            styled.font_size = 16.0;
            styled.bold = true;
            styled.margin_top = 12.0;
            styled.margin_bottom = 4.0;
            styled.color = [0.9, 0.9, 0.9, 1.0];
        }
        "p" => {
            styled.font_size = 14.0;
            styled.margin_bottom = 8.0;
            styled.color = [0.82, 0.82, 0.82, 1.0];
        }
        "a" => {
            styled.font_size = 14.0;
            styled.color = [0.4, 0.6, 1.0, 1.0];
            styled.margin_bottom = 2.0;
        }
        "b" => {
            styled.font_size = 14.0;
            styled.bold = true;
            styled.color = [1.0, 1.0, 1.0, 1.0];
        }
        "li" => {
            styled.font_size = 14.0;
            styled.margin_bottom = 2.0;
            styled.indent += 16.0;
            styled.color = [0.82, 0.82, 0.82, 1.0];
        }
        "code" => {
            styled.font_size = 12.0;
            styled.bg_color = Some([0.12, 0.12, 0.14, 1.0]);
            styled.padding_top = 4.0;
            styled.padding_bottom = 4.0;
            styled.padding_left = 8.0;
            styled.margin_bottom = 6.0;
            styled.color = [0.8, 0.9, 0.8, 1.0];
        }
        _ => {}
    }

    styled
}

fn layout_block(block: &TextBlock, styled: &StyledBlock, ctx: &mut LayoutContext, out: &mut Vec<LayoutBlock>) {
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
        out.push(LayoutBlock {
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
        });
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
                out.push(LayoutBlock {
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
                });
                ctx.cursor_y += h;
                line = word.to_string();
            } else {
                line = test;
            }
        }
        if !line.is_empty() {
            let h = styled.font_size * ctx.line_height;
            out.push(LayoutBlock {
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
            });
            ctx.cursor_y += h;
        }
    }

    ctx.cursor_y += styled.margin_bottom;
}

pub fn measure_text_width_approx(text: &str, font_size: f32) -> f32 {
    text.len() as f32 * font_size * 0.58
}

pub fn total_content_height(blocks: &[LayoutBlock]) -> f32 {
    blocks.iter().map(|b| b.y + b.h).fold(0.0f32, f32::max)
}

pub fn hit_test_link(blocks: &[LayoutBlock], mx: f32, my: f32, scroll_y: f32) -> Option<String> {
    let adjusted_y = my + scroll_y;
    for block in blocks {
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
    None
}
