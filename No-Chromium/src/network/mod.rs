// Network: Módulo base para el proceso de red (fetch, proxy, DNS).
// Stub implementado para resolver error E0583.

pub mod fetch;
pub mod socks_proxy;
pub mod doh_resolver;
pub mod circuit;

// Re-exports
pub use fetch::HttpFetcher as Fetcher;
pub use socks_proxy::SocksProxy;

/// Stub para el coordinador de red
pub struct NetworkCoordinator;

impl NetworkCoordinator {
    pub async fn initialize() -> anyhow::Result<()> {
        tracing::info!("[Network] Coordinator initialized (stub)");
        Ok(())
    }

    pub async fn shutdown() -> anyhow::Result<()> {
        tracing::info!("[Network] Coordinator shutting down (stub)");
        Ok(())
    }
}
