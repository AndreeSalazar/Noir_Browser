use anyhow::Result;
pub struct HttpFetcher {}
impl HttpFetcher {
    pub fn new() -> Self { Self {} }
    pub async fn get(&self, url: &str) -> Result<Vec<u8>> {
        tracing::warn!("Stub: Fetching {}", url);
        Ok(vec![])
    }
}
