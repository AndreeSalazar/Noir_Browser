//! Storage Module - Persistencia con JSON
//!
//! Submódulos:
//! - `bookmarks/`: Bookmarks persistentes
//! - `history/`: Historial de navegación
//! - `settings/`: Configuración del usuario
//! - `tabs/`: Estado de tabs (sesión)
//! - `local_storage/`: localStorage API (Web)
//! - `session_storage/`: sessionStorage API (Web)

#![allow(dead_code)]

pub mod bookmarks;
pub mod history;
pub mod settings;
pub mod tabs;
pub mod local_storage;
pub mod session_storage;
pub mod download;
pub mod path;

pub use bookmarks::{Bookmark, BookmarkManager, BookmarkError};
pub use history::{HistoryEntry, HistoryManager, HistoryError};
pub use settings::{Settings, SettingsManager, Theme};
pub use tabs::{TabSnapshot, TabPersistence};
pub use local_storage::LocalStorage;
pub use session_storage::SessionStorage;
pub use download::{Download, DownloadManager, DownloadStatus};
pub use path::StoragePaths;
