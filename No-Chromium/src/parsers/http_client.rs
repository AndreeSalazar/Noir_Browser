use std::error::Error;
use std::time::Duration;

pub fn fetch_html(url: &str) -> Result<String, Box<dyn Error>> {
    let response = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(20))
        .user_agent("No-Chromium/0.1 Sovereign Rust Vulkan Browser")
        .build()?
        .get(url)
        .send()?
        .error_for_status()?;
    let text = response.text()?;
    Ok(text)
}
