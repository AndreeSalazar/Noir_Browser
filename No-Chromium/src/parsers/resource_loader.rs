use reqwest::Client;
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
    pub resource_type: ResourceType,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceType {
    Document,
    Style,
    Script,
    Media,
    Image,
    Other,
}

impl ResourceType {
    fn accept_header(self) -> &'static str {
        match self {
            ResourceType::Document => {
                "text/html,application/xhtml+xml,application/xml;q=0.9,text/plain;q=0.8,*/*;q=0.5"
            }
            ResourceType::Style => "text/css,*/*;q=0.5",
            ResourceType::Script => {
                "application/javascript,text/javascript,application/ecmascript,*/*;q=0.5"
            }
            ResourceType::Media => {
                "video/*,audio/*,application/dash+xml,application/vnd.apple.mpegurl,*/*;q=0.5"
            }
            ResourceType::Image => {
                "image/avif,image/webp,image/png,image/jpeg,image/svg+xml,*/*;q=0.5"
            }
            ResourceType::Other => "*/*",
        }
    }

    fn cache_bucket(self) -> &'static str {
        match self {
            ResourceType::Document => "document",
            ResourceType::Style => "style",
            ResourceType::Script => "script",
            ResourceType::Media => "media",
            ResourceType::Image => "image",
            ResourceType::Other => "other",
        }
    }

    fn is_textual(self) -> bool {
        matches!(
            self,
            ResourceType::Document
                | ResourceType::Style
                | ResourceType::Script
                | ResourceType::Other
        )
    }
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

pub async fn fetch_document(url: &str) -> Result<ResourceResponse, Box<dyn Error>> {
    fetch_resource(url, ResourceType::Document).await
}

#[allow(dead_code)]
pub async fn fetch_style(url: &str) -> Result<ResourceResponse, Box<dyn Error>> {
    fetch_resource(url, ResourceType::Style).await
}

#[allow(dead_code)]
pub async fn fetch_script(url: &str) -> Result<ResourceResponse, Box<dyn Error>> {
    fetch_resource(url, ResourceType::Script).await
}

fn get_newtab_html() -> String {
    r#"<!DOCTYPE html>
<html>
<head>
<style>
  body {
    background-color: #0b0c10;
    color: #c5c6c7;
    margin: 0;
    padding: 0;
    width: 100%;
    height: 100%;
  }
  .container {
    max-width: 800px;
    margin-top: 80px;
    margin-left: auto;
    margin-right: auto;
    text-align: center;
  }
  .logo {
    color: #66fcf1;
    font-size: 42px;
    font-weight: bold;
    margin-bottom: 5px;
    text-align: center;
  }
  .subtitle {
    color: #45a29e;
    font-size: 15px;
    margin-bottom: 45px;
    text-align: center;
  }
  .search-container {
    margin-bottom: 50px;
    text-align: center;
  }
  .search-bar {
    background-color: #1f2833;
    padding: 12px 20px;
    border-radius: 24px;
    color: #c5c6c7;
    font-size: 15px;
    max-width: 500px;
    margin-left: auto;
    margin-right: auto;
    text-align: left;
  }
  .search-placeholder {
    color: #45a29e;
  }
  .grid-container {
    margin-top: 30px;
    text-align: center;
  }
  .shortcut-card {
    display: inline-block;
    background-color: #1f2833;
    padding: 18px 12px;
    border-radius: 12px;
    width: 130px;
    margin: 10px;
    text-align: center;
    text-decoration: none;
    color: #66fcf1;
  }
  .shortcut-icon {
    display: block;
    margin-left: auto;
    margin-right: auto;
    margin-bottom: 12px;
    border-radius: 4px;
  }
  .shortcut-title {
    color: #ffffff;
    font-weight: bold;
    font-size: 14px;
    margin-top: 4px;
  }
  .shortcut-desc {
    color: #888888;
    font-size: 10px;
    margin-top: 2px;
  }
  .footer-text {
    margin-top: 100px;
    color: #45a29e;
    font-size: 11px;
    text-align: center;
  }
</style>
</head>
<body>
  <div class="container">
    <h1 class="logo">NOIR BROWSER</h1>
    <p class="subtitle">Sovereign High-Performance Vulkan Engine</p>
    
    <div class="search-container">
      <div class="search-bar">
        <span class="search-placeholder">Search the web or enter address...</span>
      </div>
    </div>
    
    <div class="grid-container">
      <a class="shortcut-card" href="https://www.google.com">
        <img class="shortcut-icon" src="https://www.google.com/images/branding/googlelogo/2x/googlelogo_color_272x92dp.png" width="80" height="27" alt="Google" />
        <div class="shortcut-title">Google</div>
        <div class="shortcut-desc">Search Engine</div>
      </a>
      
      <a class="shortcut-card" href="https://www.youtube.com">
        <img class="shortcut-icon" src="https://upload.wikimedia.org/wikipedia/commons/3/34/YouTube_logo_%282017%29.png" width="64" height="15" alt="YouTube" />
        <div class="shortcut-title">YouTube</div>
        <div class="shortcut-desc">Videos</div>
      </a>
      
      <a class="shortcut-card" href="https://github.com">
        <img class="shortcut-icon" src="https://github.githubassets.com/images/modules/logos_page/GitHub-Mark.png" width="32" height="32" alt="GitHub" />
        <div class="shortcut-title">GitHub</div>
        <div class="shortcut-desc">Repository</div>
      </a>
      
      <a class="shortcut-card" href="https://www.rust-lang.org">
        <img class="shortcut-icon" src="https://www.rust-lang.org/static/images/rust-logo-blk.png" width="32" height="32" alt="Rust" />
        <div class="shortcut-title">Rust Lang</div>
        <div class="shortcut-desc">Programming</div>
      </a>
    </div>
    
    <p class="footer-text">Powered by pure Vulkan, gpu-allocator, & Rust</p>
  </div>
</body>
</html>
"#.to_string()
}

pub async fn fetch_resource(
    url: &str,
    resource_type: ResourceType,
) -> Result<ResourceResponse, Box<dyn Error>> {
    if url == "noir://newtab" || url.starts_with("noir:") {
        let body = get_newtab_html();
        let body_bytes = body.len();
        return Ok(ResourceResponse {
            requested_url: url.to_string(),
            final_url: url.to_string(),
            resource_type,
            status: 200,
            content_type: Some("text/html".to_string()),
            body,
            body_bytes,
            cache_status: CacheStatus::Network,
        });
    }

    let cache_key = cache_key(url, resource_type);
    let cached = CachedResource::load(&cache_key, resource_type);
    let mut request = client()
        .get(url)
        .header(USER_AGENT, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .header(ACCEPT, resource_type.accept_header());

    if let Some(cached) = &cached {
        if let Some(etag) = &cached.etag {
            request = request.header(IF_NONE_MATCH, etag);
        }
        if let Some(last_modified) = &cached.last_modified {
            request = request.header(IF_MODIFIED_SINCE, last_modified);
        }
    }

    let response = match request.send().await {
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

    let body = response.text().await?;
    let final_status = status;
    let final_url = final_url;
    let final_content_type = content_type;
    let final_etag = etag;
    let final_last_modified = last_modified;

    // Challenge detection removed - js_engine::challenge module deleted

    let body_bytes = body.len();
    let resource = ResourceResponse {
        requested_url: url.to_string(),
        final_url,
        resource_type,
        status: final_status,
        content_type: final_content_type,
        body,
        body_bytes,
        cache_status: CacheStatus::Network,
    };

    if resource.is_success() && resource.resource_type.is_textual() {
        CachedResource::from_response(&resource, final_etag, final_last_modified)
            .save(&cache_key, &resource.body);
    }

    Ok(resource)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedResource {
    requested_url: String,
    final_url: String,
    resource_type: ResourceType,
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
            resource_type: response.resource_type,
            status: response.status,
            content_type: response.content_type.clone(),
            etag,
            last_modified,
            body_file: format!(
                "{}.body",
                cache_key(&response.requested_url, response.resource_type)
            ),
            body_bytes: response.body_bytes,
            stored_unix: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|duration| duration.as_secs())
                .unwrap_or_default(),
        }
    }

    fn load(cache_key: &str, resource_type: ResourceType) -> Option<Self> {
        let meta_path = cache_dir(resource_type).join(format!("{cache_key}.json"));
        let contents = fs::read_to_string(meta_path).ok()?;
        serde_json::from_str(&contents).ok()
    }

    fn save(&self, cache_key: &str, body: &str) {
        let dir = cache_dir(self.resource_type);
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
        let body = fs::read_to_string(cache_dir(self.resource_type).join(&self.body_file))?;
        Ok(ResourceResponse {
            requested_url: self.requested_url.clone(),
            final_url: self.final_url.clone(),
            resource_type: self.resource_type,
            status: self.status,
            content_type: self.content_type.clone(),
            body_bytes: body.len(),
            body,
            cache_status,
        })
    }
}

fn cache_key(url: &str, resource_type: ResourceType) -> String {
    let mut hasher = DefaultHasher::new();
    resource_type.hash(&mut hasher);
    url.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

fn cache_dir(resource_type: ResourceType) -> PathBuf {
    PathBuf::from("profile")
        .join("cache")
        .join("resources")
        .join(resource_type.cache_bucket())
}

fn client() -> &'static Client {
    CLIENT.get_or_init(|| {
        Client::builder()
            .connect_timeout(Duration::from_secs(6))
            .timeout(Duration::from_secs(30))
            .pool_idle_timeout(Duration::from_secs(90))
            .pool_max_idle_per_host(64)
            .tcp_nodelay(true)
            .redirect(reqwest::redirect::Policy::limited(10))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .build()
            .expect("No-Chromium HTTP client should initialize")
    })
}
