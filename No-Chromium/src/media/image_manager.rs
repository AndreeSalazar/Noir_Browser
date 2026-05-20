use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::error::Error;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};
use winit::event_loop::EventLoopProxy;

use crate::app::BrowserEvent;

#[derive(Clone, Debug)]
pub struct LoadedImage {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

static IMAGE_CACHE: OnceLock<Mutex<HashMap<String, Arc<LoadedImage>>>> = OnceLock::new();

pub fn get_image_cache() -> &'static Mutex<HashMap<String, Arc<LoadedImage>>> {
    IMAGE_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn cache_key(url: &str) -> String {
    let mut hasher = DefaultHasher::new();
    url.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

fn cache_path(key: &str) -> PathBuf {
    PathBuf::from("profile")
        .join("cache")
        .join("resources")
        .join("image")
        .join(format!("{}.bin", key))
}

pub async fn fetch_image_bytes(url: &str) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
    let key = cache_key(url);
    let path = cache_path(&key);

    if path.exists() {
        if let Ok(bytes) = std::fs::read(&path) {
            println!("[Image Cache] Loaded from disk cache: {}", url);
            return Ok(bytes);
        }
    }

    println!("[Image Cache] Downloading from network: {}", url);
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;
    
    let bytes = client
        .get(url)
        .header(reqwest::header::USER_AGENT, "No-Chromium/0.1 Sovereign Rust Vulkan Browser")
        .send()
        .await?
        .bytes()
        .await?;

    let bytes_vec = bytes.to_vec();

    // Ensure directory exists and write to cache
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(&path, &bytes_vec);

    Ok(bytes_vec)
}

pub fn spawn_image_decode_task(url: String, proxy: EventLoopProxy<BrowserEvent>) {
    // Check memory cache first
    {
        let cache = get_image_cache().lock().unwrap();
        if cache.contains_key(&url) {
            return;
        }
    }

    tokio::spawn(async move {
        match fetch_image_bytes(&url).await {
            Ok(bytes) => {
                // Decode image in a CPU blocking task to not stall the async executor
                let url_clone = url.clone();
                let decode_res = tokio::task::spawn_blocking(move || {
                    image::load_from_memory(&bytes)
                }).await;

                match decode_res {
                    Ok(Ok(img)) => {
                        let rgba_img = img.to_rgba8();
                        let loaded = Arc::new(LoadedImage {
                            width: rgba_img.width(),
                            height: rgba_img.height(),
                            rgba: rgba_img.into_raw(),
                        });

                        {
                            let mut cache = get_image_cache().lock().unwrap();
                            cache.insert(url.clone(), loaded);
                        }

                        println!("[Image Cache] Successfully decoded image: {}", url);
                        let _ = proxy.send_event(BrowserEvent::ImageLoaded { url });
                    }
                    Ok(Err(e)) => {
                        println!("[Image Cache] Failed to decode image from {}: {:?}", url_clone, e);
                    }
                    Err(join_err) => {
                        println!("[Image Cache] Blocking decode join error for {}: {:?}", url_clone, join_err);
                    }
                }
            }
            Err(e) => {
                println!("[Image Cache] Failed to download image from {}: {:?}", url, e);
            }
        }
    });
}

/// Rutina de pre-caché para descargar imágenes comunes al iniciar la aplicación.
pub fn pre_cache_resources(proxy: EventLoopProxy<BrowserEvent>) {
    let common_urls = vec![
        // Google Logo
        "https://www.google.com/images/branding/googlelogo/2x/googlelogo_color_272x92dp.png".to_string(),
        // Rust Artwork Logo (as a beautiful local fallback / test)
        "https://raw.githubusercontent.com/rust-lang/rust-artwork/master/logo/rust-logo-512x512.png".to_string(),
        // DuckDuckGo Icon
        "https://duckduckgo.com/favicon.png".to_string(),
    ];

    println!("[Image Cache] Inicializando pre-caché de arranque para {} recursos comunes...", common_urls.len());
    for url in common_urls {
        spawn_image_decode_task(url, proxy.clone());
    }
}
