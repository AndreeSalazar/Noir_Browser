// Network: Módulo base para el proceso de red (fetch, proxy, DNS).
// Stub implementado para resolver error E0583.

pub mod fetch;
pub mod socks_proxy;
pub mod doh_resolver;
pub mod circuit;

// Re-exports (stubs disponibles para uso futuro)
#[allow(unused_imports)]
pub use fetch::HttpFetcher;
#[allow(unused_imports)]
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
