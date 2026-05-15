use reqwest::blocking::Client;
use reqwest::header::{
    ACCEPT, CONTENT_TYPE, ETAG, IF_MODIFIED_SINCE, IF_NONE_MATCH, LAST_MODIFIED, USER_AGENT,
};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::error::Error;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

static CLIENT: OnceLock<Client> = OnceLock::new();

#[derive(Debug, Clone)]
pub struct ResourceResponse {
    pub requested_url: String,
    pub final_url: String,
    pub status: u16,
    pub content_type: Option<String>,
    pub body: String,
    pub body_bytes: usize,
    pub cache_status: CacheStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheStatus {
    Network,
    Revalidated,
    Fallback,
}

impl ResourceResponse {
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }

    pub fn is_html_like(&self) -> bool {
        self.content_type
            .as_deref()
            .map(|content_type| {
                let lower = content_type.to_ascii_lowercase();
                lower.contains("text/html")
                    || lower.contains("application/xhtml")
                    || lower.contains("text/plain")
                    || lower.contains("application/xml")
            })
            .unwrap_or(true)
    }
}

pub fn fetch_document(url: &str) -> Result<ResourceResponse, Box<dyn Error>> {
    let cache_key = cache_key(url);
    let cached = CachedResource::load(&cache_key);
    let mut request = client()
        .get(url)
        .header(USER_AGENT, "No-Chromium/0.1 Sovereign Rust Vulkan Browser")
        .header(
            ACCEPT,
            "text/html,application/xhtml+xml,application/xml;q=0.9,text/plain;q=0.8,*/*;q=0.5",
        );

    if let Some(cached) = &cached {
        if let Some(etag) = &cached.etag {
            request = request.header(IF_NONE_MATCH, etag);
        }
        if let Some(last_modified) = &cached.last_modified {
            request = request.header(IF_MODIFIED_SINCE, last_modified);
        }
    }

    let response = match request.send() {
        Ok(response) => response,
        Err(error) => {
            if let Some(cached) = cached {
                println!("[Cache] Network fallback for {}", url);
                return cached.to_response(CacheStatus::Fallback);
            }
            return Err(Box::new(error));
        }
    };

    let status = response.status().as_u16();
    let final_url = response.url().to_string();
    let content_type = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);
    let etag = response
        .headers()
        .get(ETAG)
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);
    let last_modified = response
        .headers()
        .get(LAST_MODIFIED)
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);

    if status == 304 {
        if let Some(cached) = cached {
            println!("[Cache] Revalidated {}", url);
            return cached.to_response(CacheStatus::Revalidated);
        }
    }

    let body = response.text()?;
    let body_bytes = body.len();
    let resource = ResourceResponse {
        requested_url: url.to_string(),
        final_url,
        status,
        content_type,
        body,
        body_bytes,
        cache_status: CacheStatus::Network,
    };

    if resource.is_success() && resource.is_html_like() {
        CachedResource::from_response(&resource, etag, last_modified)
            .save(&cache_key, &resource.body);
    }

    Ok(resource)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedResource {
    requested_url: String,
    final_url: String,
    status: u16,
    content_type: Option<String>,
    etag: Option<String>,
    last_modified: Option<String>,
    body_file: String,
    body_bytes: usize,
    stored_unix: u64,
}

impl CachedResource {
    fn from_response(
        response: &ResourceResponse,
        etag: Option<String>,
        last_modified: Option<String>,
    ) -> Self {
        Self {
            requested_url: response.requested_url.clone(),
            final_url: response.final_url.clone(),
            status: response.status,
            content_type: response.content_type.clone(),
            etag,
            last_modified,
            body_file: format!("{}.body", cache_key(&response.requested_url)),
            body_bytes: response.body_bytes,
            stored_unix: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|duration| duration.as_secs())
                .unwrap_or_default(),
        }
    }

    fn load(cache_key: &str) -> Option<Self> {
        let meta_path = cache_dir().join(format!("{cache_key}.json"));
        let contents = fs::read_to_string(meta_path).ok()?;
        serde_json::from_str(&contents).ok()
    }

    fn save(&self, cache_key: &str, body: &str) {
        let dir = cache_dir();
        let _ = fs::create_dir_all(&dir);

        let body_path = dir.join(&self.body_file);
        let meta_path = dir.join(format!("{cache_key}.json"));
        if fs::write(body_path, body).is_err() {
            return;
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(meta_path, json);
        }
    }

    fn to_response(&self, cache_status: CacheStatus) -> Result<ResourceResponse, Box<dyn Error>> {
        let body = fs::read_to_string(cache_dir().join(&self.body_file))?;
        Ok(ResourceResponse {
            requested_url: self.requested_url.clone(),
            final_url: self.final_url.clone(),
            status: self.status,
            content_type: self.content_type.clone(),
            body_bytes: body.len(),
            body,
            cache_status,
        })
    }
}

fn cache_key(url: &str) -> String {
    let mut hasher = DefaultHasher::new();
    url.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

fn cache_dir() -> PathBuf {
    PathBuf::from("profile").join("cache").join("resources")
}

fn client() -> &'static Client {
    CLIENT.get_or_init(|| {
        Client::builder()
            .timeout(Duration::from_secs(20))
            .redirect(reqwest::redirect::Policy::limited(10))
            .build()
            .expect("No-Chromium HTTP client should initialize")
    })
}
