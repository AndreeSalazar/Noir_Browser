use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex, OnceLock};

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

pub async fn fetch_image_bytes(url: &str) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
    let client = reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(5))
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap_or_default();

    let bytes = client
        .get(url)
        .header(reqwest::header::USER_AGENT, "NoirBrowser/0.1")
        .send()
        .await?
        .bytes()
        .await?;

    Ok(bytes.to_vec())
}
