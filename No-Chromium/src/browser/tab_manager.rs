// TabManager: Gestiona la creación, destrucción y comunicación con procesos renderer.
// Stub implementado para resolver error E0583.

use crate::utils::ipc::{BrowserMessage, RenderMessage};
use tokio::sync::mpsc;

pub type TabId = u64;

pub struct TabManager {
    next_id: TabId,
    // TODO: Agregar mapa de tabs activos y canales IPC
}

impl TabManager {
    pub fn new() -> Self {
        Self { next_id: 0 }
    }

    pub fn create_tab(&mut self) -> TabId {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn send_to_renderer(&self, tab_id: TabId, msg: RenderMessage) {
        // TODO: Implementar envío IPC al proceso renderer
        let _ = (tab_id, msg);
    }
}
