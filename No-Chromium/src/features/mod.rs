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
//! - `tab_groups/`: Agrupar tabs con colores
//! - `favorites/`: Bookmarks con tags y folders
//! - `service_worker/`: Service workers con cache API y push

#![allow(dead_code)]

pub mod reader_mode;
pub mod find_in_page;
pub mod print_pdf;
pub mod screenshot;
pub mod network_monitor;
pub mod password_manager;
pub mod pwa;
pub mod tab_groups;
pub mod favorites;
pub mod service_worker;

pub use reader_mode::ReaderMode;
pub use find_in_page::{FindInPage, FindMatch, FindOptions};
pub use print_pdf::PrintPdf;
pub use screenshot::Screenshot;
pub use network_monitor::{NetworkMonitor, NetworkRequest, RequestStatus};
pub use password_manager::{PasswordManager, PasswordEntry, SavedPassword};
pub use pwa::{PwaManager, ServiceWorker, WebManifest};
pub use tab_groups::{TabGroup, TabGroupManager, GroupColor};
pub use favorites::{Favorite, FavoritesManager};
pub use service_worker::{ServiceWorkerManager, WorkerRegistration, WorkerState, CacheEntry, PushMessage};
