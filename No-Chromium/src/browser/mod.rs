//! Módulo Browser - Gestión de navegación y tabs
//!
//! Coordina la gestión de pestañas, navegación, historial de sesión
//! y comunicación con otros procesos vía IPC.

pub mod coordinator;
pub mod tab_manager;
pub mod navigation;

#[cfg(feature = "privacy")]
pub mod privacy;

use tokio::sync::mpsc;
use crate::utils::ipc::{BrowserMessage, RendererMessage};

/// Identificador único para una pestaña
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TabId(pub u64);

impl TabId {
    /// Genera un nuevo TabId único
    pub fn new() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for TabId {
    fn default() -> Self {
        Self::new()
    }
}

/// Estado de una pestaña
#[derive(Debug, Clone)]
pub enum TabState {
    Loading { url: String, progress: f32 },
    Loaded { url: String, title: String },
    Error { url: String, message: String },
    Closed,
}

/// Coordinador del proceso Browser
pub struct BrowserCoordinator {
    tab_manager: tab_manager::TabManager,
    browser_tx: mpsc::Sender<BrowserMessage>,
    renderer_tx: mpsc::Sender<RendererMessage>,
}

impl BrowserCoordinator {
    /// Crea un nuevo coordinador con canales IPC inicializados
    pub fn new(
        browser_tx: mpsc::Sender<BrowserMessage>,
        renderer_tx: mpsc::Sender<RendererMessage>,
    ) -> Self {
        Self {
            tab_manager: tab_manager::TabManager::new(),
            browser_tx,
            renderer_tx,
        }
    }
    
    /// Abre una nueva pestaña con la URL especificada
    pub async fn open_tab(&mut self, url: String) -> anyhow::Result<TabId> {
        let tab_id = self.tab_manager.create_tab(url.clone())?;
        
        // Enviar mensaje al renderer para cargar la URL
        self.renderer_tx
            .send(RendererMessage::Navigate {
                url,
                tab_id,
            })
            .await?;
        
        Ok(tab_id)
    }
    
    /// Cierra una pestaña y libera sus recursos
    pub async fn close_tab(&mut self, tab_id: TabId) -> anyhow::Result<()> {
        // Notificar al renderer para limpiar recursos
        self.renderer_tx
            .send(RendererMessage::CloseTab { tab_id })
            .await?;
        
        // Remover del tab manager
        self.tab_manager.remove_tab(tab_id)?;
        
        Ok(())
    }
    
    /// Obtiene el estado actual de una pestaña
    pub fn get_tab_state(&self, tab_id: TabId) -> Option<&TabState> {
        self.tab_manager.get_state(tab_id)
    }
    
    /// Lista todas las pestañas activas
    pub fn list_tabs(&self) -> Vec<(TabId, &TabState)> {
        self.tab_manager.list_active()
    }
}
