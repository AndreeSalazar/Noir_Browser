use anyhow::{Context, Result};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

static COOKIE_JAR: OnceLock<Arc<Mutex<HashMap<String, String>>>> = OnceLock::new();

fn get_cookie_jar() -> &'static Arc<Mutex<HashMap<String, String>>> {
    COOKIE_JAR.get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
}

pub struct HttpFetcher {
    client: reqwest::Client,
}

pub struct FetchResult {
    pub url: String,
    pub final_url: String,
    pub status: u16,
    pub content_type: Option<String>,
    pub body: String,
    pub headers: HashMap<String, String>,
}

impl HttpFetcher {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .danger_accept_invalid_certs(false)
            .redirect(reqwest::redirect::Policy::none())
            .user_agent("NoirBrowser/0.2 (Rust; no-chromium)")
            .build()
            .expect("Failed to create HTTP client");

        Self { client }
    }

    pub async fn get(&self, url: &str) -> Result<FetchResult> {
        self.fetch_with_redirects(url, "GET", None).await
    }

    pub async fn post(&self, url: &str, body: &str) -> Result<FetchResult> {
        self.fetch_with_redirects(url, "POST", Some(body.to_string())).await
    }

    pub async fn fetch_with_redirects(
        &self,
        url: &str,
        method: &str,
        body: Option<String>,
    ) -> Result<FetchResult> {
        let mut current_url = url.to_string();
        let mut redirect_count = 0u32;
        let max_redirects = 20;

        loop {
            let cookies_for_url = self.get_cookies_for_url(&current_url);

            let mut req = match method {
                "POST" => {
                    let mut builder = self.client.post(&current_url);
                    if let Some(body_content) = &body {
                        builder = builder
                            .header("Content-Type", "application/x-www-form-urlencoded")
                            .body(body_content.clone());
                    }
                    builder
                }
                "PUT" => {
                    let mut builder = self.client.put(&current_url);
                    if let Some(body_content) = &body {
                        builder = builder
                            .header("Content-Type", "application/json")
                            .body(body_content.clone());
                    }
                    builder
                }
                "DELETE" => self.client.delete(&current_url),
                _ => self.client.get(&current_url),
            };

            if !cookies_for_url.is_empty() {
                req = req.header("Cookie", &cookies_for_url);
            }

            let response = req.send().await.with_context(|| {
                format!("Failed to fetch: {}", current_url)
            })?;

            let status = response.status().as_u16();
            let final_url = response.url().to_string();

            self.store_cookies_from_response(&final_url, &response);

            if status == 301 || status == 302 || status == 307 || status == 308 {
                if redirect_count >= max_redirects {
                    return Err(anyhow::anyhow!(
                        "Too many redirects (max {})",
                        max_redirects
                    ));
                }

                if let Some(location) = response.headers().get("location") {
                    if let Ok(loc) = location.to_str() {
                        current_url = if loc.starts_with("http") {
                            loc.to_string()
                        } else if loc.starts_with("//") {
                            format!("https:{}", loc)
                        } else if loc.starts_with('/') {
                            if let Ok(parsed) = url::Url::parse(&final_url) {
                                format!(
                                    "{}://{}{}",
                                    parsed.scheme(),
                                    parsed.host_str().unwrap_or(""),
                                    loc
                                )
                            } else {
                                loc.to_string()
                            }
                        } else {
                            loc.to_string()
                        };

                        if status == 301 || status == 302 {
                            let _ = method;
                        }
                        redirect_count += 1;
                        continue;
                    }
                }
            }

            let content_type = response
                .headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string());

            let mut headers = HashMap::new();
            for (key, value) in response.headers() {
                if let Ok(v) = value.to_str() {
                    headers.insert(key.to_string(), v.to_string());
                }
            }

            let ct = content_type.clone().unwrap_or_default();
            let body = if ct.contains("text") || ct.contains("json") || ct.contains("xml") || ct.contains("html") || ct.contains("javascript") || ct.is_empty() {
                response.text().await.unwrap_or_default()
            } else {
                format!("[Binary content: {}]", ct)
            };

            return Ok(FetchResult {
                url: url.to_string(),
                final_url,
                status,
                content_type,
                body,
                headers,
            });
        }
    }

    fn get_cookies_for_url(&self, url: &str) -> String {
        let jar = get_cookie_jar().lock().unwrap();
        let domain = url::Url::parse(url)
            .ok()
            .and_then(|u| u.host_str().map(|s| s.to_string()))
            .unwrap_or_default();

        let mut cookies = Vec::new();
        for (key, value) in jar.iter() {
            if key.starts_with(&format!("{}|", domain)) || domain.ends_with(&key.split('|').next().unwrap_or("")) {
                let cookie_str = key.split('|').nth(1).unwrap_or(key);
                cookies.push(format!("{}={}", cookie_str, value));
            }
        }
        cookies.join("; ")
    }

    fn store_cookies_from_response(&self, url: &str, response: &reqwest::Response) {
        let domain = url::Url::parse(url)
            .ok()
            .and_then(|u| u.host_str().map(|s| s.to_string()))
            .unwrap_or_default();

        let mut jar = get_cookie_jar().lock().unwrap();
        for (key, value) in response.headers() {
            if key == "set-cookie" {
                if let Ok(cookie_str) = value.to_str() {
                    for part in cookie_str.split(';') {
                        let part = part.trim();
                        if let Some((name, val)) = part.split_once('=') {
                            let name = name.trim();
                            let val = val.trim();
                            jar.insert(
                                format!("{}|{}", domain, name),
                                val.to_string(),
                            );
                        }
                    }
                }
            }
        }
    }
}
