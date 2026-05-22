// Coordinator: Gestiona el ciclo de vida del navegador y la comunicación entre procesos.
// Stub implementado para resolver error E0583.

use crate::utils::ipc::BrowserMessage;

pub struct BrowserCoordinator {
    // TODO: Agregar manejadores de tabs y red
}

impl BrowserCoordinator {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn handle_message(&mut self, msg: BrowserMessage) {
        // TODO: Implementar lógica de manejo de mensajes
        match msg {
            BrowserMessage::Navigate { .. } => {}
            BrowserMessage::StopLoading { .. } => {}
            BrowserMessage::GetTitle { .. } => {}
            BrowserMessage::CloseTab { .. } => {}
        }
    }
}
