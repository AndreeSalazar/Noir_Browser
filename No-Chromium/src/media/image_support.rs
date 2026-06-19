use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Instant;

#[derive(Clone, Debug)]
pub struct DecodedImage {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
    pub format: ImageFormat,
    pub last_accessed: Instant,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ImageFormat {
    Png,
    Jpeg,
    Gif,
    WebP,
    Bmp,
    Unknown,
}

#[derive(Clone, Debug)]
pub struct ImageLoadState {
    pub url: String,
    pub status: LoadStatus,
    pub attempts: u32,
    pub last_attempt: Option<Instant>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum LoadStatus {
    Pending,
    Loading,
    Loaded,
    Failed,
}

static IMAGE_CACHE: OnceLock<Mutex<ImageCache>> = OnceLock::new();
static IMAGE_DIRTY: OnceLock<Mutex<bool>> = OnceLock::new();
static IMAGE_STATS: OnceLock<Mutex<ImageStats>> = OnceLock::new();

const MAX_CACHE_SIZE: usize = 100;
const MAX_CACHE_BYTES: u64 = 50 * 1024 * 1024;
const MAX_RETRIES: u32 = 3;
const RETRY_DELAY_MS: u64 = 1000;

#[derive(Default, Clone)]
pub struct ImageStats {
    pub total_loads: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub failed_loads: u64,
    pub total_bytes: u64,
}

pub struct ImageCache {
    images: HashMap<String, Arc<DecodedImage>>,
    access_order: VecDeque<String>,
    current_bytes: u64,
}

impl ImageCache {
    fn new() -> Self {
        Self {
            images: HashMap::new(),
            access_order: VecDeque::new(),
            current_bytes: 0,
        }
    }

    fn get(&mut self, url: &str) -> Option<Arc<DecodedImage>> {
        let img = self.images.get(url)?.clone();
        self.touch(url);
        Some(img)
    }

    fn touch(&mut self, url: &str) {
        self.access_order.retain(|u| u != url);
        self.access_order.push_back(url.to_string());
    }

    fn insert(&mut self, url: String, img: DecodedImage) -> Arc<DecodedImage> {
        let size = (img.width * img.height * 4) as u64;
        let arc = Arc::new(img);
        self.current_bytes += size;

        if let Some(old) = self.images.insert(url.clone(), arc.clone()) {
            let old_size = (old.width * old.height * 4) as u64;
            self.current_bytes -= old_size;
        } else {
            self.access_order.push_back(url.clone());
        }

        self.evict_if_needed();
        arc
    }

    fn evict_if_needed(&mut self) {
        while self.images.len() > MAX_CACHE_SIZE || self.current_bytes > MAX_CACHE_BYTES {
            if let Some(oldest_url) = self.access_order.pop_front() {
                if let Some(old) = self.images.remove(&oldest_url) {
                    let old_size = (old.width * old.height * 4) as u64;
                    self.current_bytes -= old_size;
                }
            } else {
                break;
            }
        }
    }
}

fn get_cache() -> &'static Mutex<ImageCache> {
    IMAGE_CACHE.get_or_init(|| Mutex::new(ImageCache::new()))
}

fn get_dirty() -> &'static Mutex<bool> {
    IMAGE_DIRTY.get_or_init(|| Mutex::new(false))
}

fn get_stats() -> &'static Mutex<ImageStats> {
    IMAGE_STATS.get_or_init(|| Mutex::new(ImageStats::default()))
}

pub fn detect_format(bytes: &[u8]) -> ImageFormat {
    if bytes.len() < 8 {
        return ImageFormat::Unknown;
    }
    if bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {
        ImageFormat::Png
    } else if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        ImageFormat::Jpeg
    } else if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        ImageFormat::Gif
    } else if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        ImageFormat::WebP
    } else if bytes.starts_with(b"BM") {
        ImageFormat::Bmp
    } else if bytes.len() >= 4 && &bytes[0..4] == b"\x00\x00\x00\x20" {
        ImageFormat::Unknown
    } else {
        ImageFormat::Unknown
    }
}

pub fn decode_image_bytes(bytes: &[u8]) -> Option<DecodedImage> {
    let format = detect_format(bytes);
    match image::load_from_memory(bytes) {
        Ok(img) => {
            let rgba = img.to_rgba8();
            let (w, h) = rgba.dimensions();
            Some(DecodedImage {
                width: w,
                height: h,
                rgba: rgba.to_vec(),
                format,
                last_accessed: Instant::now(),
            })
        }
        Err(e) => {
            tracing::warn!("Image decode error: {}", e);
            None
        }
    }
}

pub fn get_cached_image(url: &str) -> Option<Arc<DecodedImage>> {
    let mut stats = get_stats().lock().unwrap();
    if let Some(img) = get_cache().lock().unwrap().get(url) {
        stats.cache_hits += 1;
        return Some(img);
    }
    stats.cache_misses += 1;
    None
}

pub fn cache_image(url: &str, img: DecodedImage) -> Arc<DecodedImage> {
    let size = (img.width * img.height * 4) as u64;
    let arc = get_cache().lock().unwrap().insert(url.to_string(), img);
    get_stats().lock().unwrap().total_bytes += size;
    if let Ok(mut dirty) = get_dirty().lock() {
        *dirty = true;
    }
    arc
}

pub fn take_image_dirty() -> bool {
    get_dirty()
        .lock()
        .map(|mut f| std::mem::replace(&mut *f, false))
        .unwrap_or(false)
}

pub fn get_image_stats() -> ImageStats {
    get_stats().lock().unwrap().clone()
}

pub fn clear_cache() {
    get_cache().lock().unwrap().images.clear();
    get_cache().lock().unwrap().access_order.clear();
    get_cache().lock().unwrap().current_bytes = 0;
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
    if dest_w <= 0 || dest_h <= 0 {
        return;
    }
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
        let iy = ((dy as f32 * sy) as u32).min(img.height.saturating_sub(1));

        for dx in 0..dest_w {
            let screen_x = dest_x + dx;
            if screen_x < 0 || screen_x >= screen_w {
                continue;
            }
            let ix = ((dx as f32 * sx) as u32).min(img.width.saturating_sub(1));

            let src_idx = ((iy * img.width + ix) * 4) as usize;
            if src_idx + 3 < img.rgba.len() {
                let r = img.rgba[src_idx] as u32;
                let g = img.rgba[src_idx + 1] as u32;
                let b = img.rgba[src_idx + 2] as u32;
                let a = img.rgba[src_idx + 3] as u32;

                let dst_idx = screen_y as usize * stride + screen_x as usize;
                if dst_idx < buf.len() {
                    if a > 0 {
                        if a >= 255 {
                            buf[dst_idx] = (255 << 24) | (r << 16) | (g << 8) | b;
                        } else {
                            let dst = buf[dst_idx];
                            let dr = (dst >> 16) & 0xFF;
                            let dg = (dst >> 8) & 0xFF;
                            let db = dst & 0xFF;
                            let alpha = a as u32;
                            let inv_alpha = 255 - alpha;
                            let nr = (r * alpha + dr * inv_alpha) / 255;
                            let ng = (g * alpha + dg * inv_alpha) / 255;
                            let nb = (b * alpha + db * inv_alpha) / 255;
                            buf[dst_idx] = (255 << 24) | (nr << 16) | (ng << 8) | nb;
                        }
                    }
                }
            }
        }
    }
}

pub fn draw_placeholder(
    buf: &mut [u32],
    stride: usize,
    dest_x: i32,
    dest_y: i32,
    dest_w: i32,
    dest_h: i32,
    screen_w: i32,
    screen_h: i32,
    is_loading: bool,
) {
    let bg = if is_loading { 0xFF2A2A30 } else { 0xFF1A1A1E };
    for dy in 0..dest_h {
        let screen_y = dest_y + dy;
        if screen_y < 0 || screen_y >= screen_h {
            continue;
        }
        for dx in 0..dest_w {
            let screen_x = dest_x + dx;
            if screen_x < 0 || screen_x >= screen_w {
                continue;
            }
            let checker = ((dx / 8) + (dy / 8)) % 2 == 0;
            let color = if checker { bg } else { 0xFF15151A };
            let dst_idx = screen_y as usize * stride + screen_x as usize;
            if dst_idx < buf.len() {
                buf[dst_idx] = color;
            }
        }
    }
}

pub async fn fetch_image(url: &str) -> Option<Arc<DecodedImage>> {
    if let Some(cached) = get_cached_image(url) {
        return Some(cached);
    }

    get_stats().lock().unwrap().total_loads += 1;

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .user_agent("NoirBrowser/0.2")
        .build()
        .ok()?;

    for attempt in 0..MAX_RETRIES {
        match client.get(url).send().await {
            Ok(response) => {
                if !response.status().is_success() {
                    tracing::warn!("Image fetch failed ({}): HTTP {}", url, response.status());
                    if attempt < MAX_RETRIES - 1 {
                        tokio::time::sleep(std::time::Duration::from_millis(RETRY_DELAY_MS * (attempt + 1) as u64)).await;
                        continue;
                    }
                    get_stats().lock().unwrap().failed_loads += 1;
                    return None;
                }

                match response.bytes().await {
                    Ok(bytes) => {
                        match decode_image_bytes(&bytes) {
                            Some(decoded) => {
                                tracing::info!(
                                    "Image loaded: {} ({}x{}, {} bytes, format: {:?})",
                                    url, decoded.width, decoded.height, bytes.len(), decoded.format
                                );
                                return Some(cache_image(url, decoded));
                            }
                            None => {
                                tracing::warn!("Image decode failed: {}", url);
                                get_stats().lock().unwrap().failed_loads += 1;
                                return None;
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Image read error (attempt {}): {}", attempt + 1, e);
                        if attempt < MAX_RETRIES - 1 {
                            tokio::time::sleep(std::time::Duration::from_millis(RETRY_DELAY_MS * (attempt + 1) as u64)).await;
                        }
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Image fetch error (attempt {}): {}", attempt + 1, e);
                if attempt < MAX_RETRIES - 1 {
                    tokio::time::sleep(std::time::Duration::from_millis(RETRY_DELAY_MS * (attempt + 1) as u64)).await;
                }
            }
        }
    }

    get_stats().lock().unwrap().failed_loads += 1;
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_png_format() {
        let png_magic = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        assert_eq!(detect_format(&png_magic), ImageFormat::Png);
    }

    #[test]
    fn test_detect_jpeg_format() {
        let mut jpeg_magic = vec![0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46];
        jpeg_magic.extend_from_slice(&[0x49, 0x46, 0x00, 0x01]);
        assert_eq!(detect_format(&jpeg_magic), ImageFormat::Jpeg);
    }

    #[test]
    fn test_detect_gif_format() {
        let mut gif87 = b"GIF87a".to_vec();
        gif87.extend_from_slice(&[0x01, 0x00, 0x01, 0x00]);
        assert_eq!(detect_format(&gif87), ImageFormat::Gif);

        let mut gif89 = b"GIF89a".to_vec();
        gif89.extend_from_slice(&[0x01, 0x00, 0x01, 0x00]);
        assert_eq!(detect_format(&gif89), ImageFormat::Gif);
    }

    #[test]
    fn test_detect_webp_format() {
        let mut webp_magic = b"RIFF".to_vec();
        webp_magic.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
        webp_magic.extend_from_slice(b"WEBP");
        webp_magic.extend_from_slice(&[0x56, 0x50, 0x38, 0x4C]);
        assert_eq!(detect_format(&webp_magic), ImageFormat::WebP);
    }

    #[test]
    fn test_detect_bmp_format() {
        let mut bmp_magic = b"BM".to_vec();
        bmp_magic.extend_from_slice(&[0x36, 0x00, 0x00, 0x00]);
        bmp_magic.extend_from_slice(&[0x28, 0x00, 0x00, 0x00]);
        assert_eq!(detect_format(&bmp_magic), ImageFormat::Bmp);
    }

    #[test]
    fn test_detect_unknown_format() {
        let unknown = b"XXXX".to_vec();
        assert_eq!(detect_format(&unknown), ImageFormat::Unknown);
    }

    #[test]
    fn test_detect_empty_bytes() {
        let empty = vec![];
        assert_eq!(detect_format(&empty), ImageFormat::Unknown);
    }

    #[test]
    fn test_image_stats_default() {
        let stats = ImageStats::default();
        assert_eq!(stats.total_loads, 0);
        assert_eq!(stats.cache_hits, 0);
        assert_eq!(stats.cache_misses, 0);
        assert_eq!(stats.failed_loads, 0);
        assert_eq!(stats.total_bytes, 0);
    }

    #[test]
    fn test_image_stats_clone() {
        let stats = ImageStats {
            total_loads: 5,
            cache_hits: 3,
            cache_misses: 2,
            failed_loads: 1,
            total_bytes: 1024,
        };
        let cloned = stats.clone();
        assert_eq!(cloned.total_loads, 5);
        assert_eq!(cloned.cache_hits, 3);
    }

    #[test]
    fn test_cache_creation() {
        clear_cache();
        let result = get_cached_image("http://test.com/img.png");
        assert!(result.is_none());
    }

    #[test]
    fn test_take_image_dirty_initial_false() {
        clear_cache();
        let _ = take_image_dirty();
    }
}
