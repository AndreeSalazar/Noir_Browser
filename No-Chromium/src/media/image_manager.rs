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
        .connect_timeout(std::time::Duration::from_secs(5))
        .timeout(std::time::Duration::from_secs(30))
        .pool_idle_timeout(std::time::Duration::from_secs(60))
        .tcp_nodelay(true)
        .build()
        .unwrap_or_default();
    
    let user_agent = "Noir/1.0 (Vulkan; Rust) compatible; Googlebot/2.1; +https://github.com/AndreeSalazar/Noir_Browser";
    let bytes = client
        .get(url)
        .header(reqwest::header::USER_AGENT, user_agent)
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

    if let Some(handle) = crate::app::get_runtime_handle() {
        handle.spawn(async move {
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
    } else {
        println!("[Image Cache] Warning: No Tokio runtime handle set, could not spawn decode task for {}", url);
    }
}

/// Pre-popula la caché de imágenes en disco y en memoria de forma instantánea usando recursos empaquetados.
pub fn pre_populate_offline_cache(proxy: EventLoopProxy<BrowserEvent>) {
    let assets = super::pre_cached_assets::get_pre_cached_assets();
    println!("[Image Cache] Pre-populando {} recursos offline estáticos...", assets.len());

    for (url, bytes) in assets {
        let key = cache_key(url);
        let path = cache_path(&key);

        // Asegurar que el archivo de caché en disco exista
        if !path.exists() {
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            if let Err(e) = std::fs::write(&path, bytes) {
                println!("[Image Cache] Error escribiendo asset offline estático a disco: {:?}", e);
            } else {
                println!("[Image Cache] Escrito a caché de disco: {}", url);
            }
        }

        // Decodificar e insertar en la caché en memoria inmediatamente
        match image::load_from_memory(bytes) {
            Ok(img) => {
                let rgba_img = img.to_rgba8();
                let loaded = Arc::new(LoadedImage {
                    width: rgba_img.width(),
                    height: rgba_img.height(),
                    rgba: rgba_img.into_raw(),
                });

                {
                    let mut cache = get_image_cache().lock().unwrap();
                    cache.insert(url.to_string(), loaded);
                }
                println!("[Image Cache] Listo en caché en memoria: {}", url);
                
                // Notificar al EventLoop
                let _ = proxy.send_event(BrowserEvent::ImageLoaded { url: url.to_string() });
            }
            Err(e) => {
                println!("[Image Cache] Error al decodificar asset offline estático {}: {:?}", url, e);
            }
        }
    }
}

/// Rutina de pre-caché para descargar imágenes comunes al iniciar la aplicación.
pub fn pre_cache_resources(proxy: EventLoopProxy<BrowserEvent>) {
    // Primero, pre-cargar y activar todos los assets locales/estáticos de manera instantánea
    pre_populate_offline_cache(proxy);
}
