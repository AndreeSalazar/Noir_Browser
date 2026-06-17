use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

#[derive(Clone, Debug)]
pub struct DecodedImage {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

static IMAGE_CACHE: OnceLock<Mutex<HashMap<String, Arc<DecodedImage>>>> = OnceLock::new();

fn get_cache() -> &'static Mutex<HashMap<String, Arc<DecodedImage>>> {
    IMAGE_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn decode_image_bytes(bytes: &[u8]) -> Option<DecodedImage> {
    match image::load_from_memory(bytes) {
        Ok(img) => {
            let rgba = img.to_rgba8();
            let (w, h) = rgba.dimensions();
            Some(DecodedImage {
                width: w,
                height: h,
                rgba: rgba.to_vec(),
            })
        }
        Err(e) => {
            tracing::warn!("Image decode error: {}", e);
            None
        }
    }
}

pub fn get_cached_image(url: &str) -> Option<Arc<DecodedImage>> {
    get_cache().lock().unwrap().get(url).cloned()
}

pub fn cache_image(url: &str, img: DecodedImage) -> Arc<DecodedImage> {
    let arc = Arc::new(img);
    get_cache().lock().unwrap().insert(url.to_string(), arc.clone());
    arc
}

pub fn draw_image_to_buffer(
    buf: &mut [u32],
    stride: usize,
    img: &DecodedImage,
    dest_x: i32,
    dest_y: i32,
    dest_w: i32,
    dest_h: i32,
    screen_w: i32,
    screen_h: i32,
) {
    if dest_y + dest_h < 0 || dest_y > screen_h {
        return;
    }
    if dest_x + dest_w < 0 || dest_x > screen_w {
        return;
    }

    let sx = img.width as f32 / dest_w as f32;
    let sy = img.height as f32 / dest_h as f32;

    for dy in 0..dest_h {
        let screen_y = dest_y + dy;
        if screen_y < 0 || screen_y >= screen_h {
            continue;
        }
        let iy = ((dy as f32 * sy) as u32).min(img.height - 1);

        for dx in 0..dest_w {
            let screen_x = dest_x + dx;
            if screen_x < 0 || screen_x >= screen_w {
                continue;
            }
            let ix = ((dx as f32 * sx) as u32).min(img.width - 1);

            let src_idx = ((iy * img.width + ix) * 4) as usize;
            if src_idx + 3 < img_rgba_len(img) {
                let r = img.rgba[src_idx] as u32;
                let g = img.rgba[src_idx + 1] as u32;
                let b = img.rgba[src_idx + 2] as u32;
                let a = img.rgba[src_idx + 3] as u32;

                let dst_idx = screen_y as usize * stride + screen_x as usize;
                if dst_idx < buf.len() {
                    if a > 128 {
                        buf[dst_idx] = (255 << 24) | (r << 16) | (g << 8) | b;
                    }
                }
            }
        }
    }
}

fn img_rgba_len(img: &DecodedImage) -> usize {
    img.rgba.len()
}

pub async fn fetch_image(url: &str) -> Option<Arc<DecodedImage>> {
    if let Some(cached) = get_cached_image(url) {
        return Some(cached);
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .ok()?;

    let bytes = client.get(url).send().await.ok()?.bytes().await.ok()?;
    let decoded = decode_image_bytes(&bytes)?;
    Some(cache_image(url, decoded))
}
