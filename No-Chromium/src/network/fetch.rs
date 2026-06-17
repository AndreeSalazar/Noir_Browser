use anyhow::{Context, Result};
use std::time::Duration;

pub struct HttpFetcher {
    client: reqwest::Client,
}

pub struct FetchResult {
    pub url: String,
    pub final_url: String,
    pub status: u16,
    pub content_type: Option<String>,
    pub body: String,
}

impl HttpFetcher {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .danger_accept_invalid_certs(false)
            .user_agent("NoirBrowser/0.1 (Rust; no-chromium)")
            .build()
            .expect("Failed to create HTTP client");

        Self { client }
    }

    pub async fn get(&self, url: &str) -> Result<FetchResult> {
        let response = self
            .client
            .get(url)
            .send()
            .await
            .with_context(|| format!("Failed to fetch: {}", url))?;

        let status = response.status().as_u16();
        let final_url = response.url().to_string();
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let body = response
            .text()
            .await
            .with_context(|| format!("Failed to read body from: {}", url))?;

        Ok(FetchResult {
            url: url.to_string(),
            final_url,
            status,
            content_type,
            body,
        })
    }
}
