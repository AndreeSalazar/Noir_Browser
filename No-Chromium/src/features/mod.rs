//! Features Module - Features avanzadas de navegador
//!
//! Submódulos:
//! - `reader_mode/`: Vista limpia sin distracciones
//! - `find_in_page/`: Buscar texto en página
//! - `print_pdf/`: Imprimir/guardar como PDF
//! - `screenshot/`: Captura de pantalla
//! - `network_monitor/`: Ver requests HTTP
//! - `password_manager/`: Guardar/auto-fill passwords
//! - `pwa/`: Progressive Web Apps

#![allow(dead_code)]

pub mod reader_mode;
pub mod find_in_page;
pub mod print_pdf;
pub mod screenshot;
pub mod network_monitor;
pub mod password_manager;
pub mod pwa;

pub use reader_mode::ReaderMode;
pub use find_in_page::{FindInPage, FindMatch, FindOptions};
pub use print_pdf::PrintPdf;
pub use screenshot::Screenshot;
pub use network_monitor::{NetworkMonitor, NetworkRequest, RequestStatus};
pub use password_manager::{PasswordManager, PasswordEntry, SavedPassword};
pub use pwa::{PwaManager, ServiceWorker, WebManifest};
