//! Downloads - Manager de descargas

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use super::path::StoragePaths;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DownloadStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone)]
pub struct Download {
    pub id: u64,
    pub url: String,
    pub filename: String,
    pub path: PathBuf,
    pub status: DownloadStatus,
    pub progress: f32,
    pub total_bytes: u64,
    pub received_bytes: u64,
    pub started: u64,
    pub finished: Option<u64>,
    pub error: Option<String>,
}

impl Download {
    pub fn new(url: String, downloads_dir: &PathBuf) -> Self {
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0);
        let filename = url.split('/').last().unwrap_or("download").to_string();
        let path = downloads_dir.join(&filename);

        Self {
            id,
            url,
            filename,
            path,
            status: DownloadStatus::Pending,
            progress: 0.0,
            total_bytes: 0,
            received_bytes: 0,
            started: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            finished: None,
            error: None,
        }
    }
}

pub struct DownloadManager {
    downloads: Arc<Mutex<Vec<Download>>>,
    #[allow(dead_code)]
    downloads_dir: PathBuf,
}

impl DownloadManager {
    pub fn new(paths: &StoragePaths) -> Self {
        let dir = paths.downloads_dir();
        let _ = std::fs::create_dir_all(&dir);
        Self {
            downloads: Arc::new(Mutex::new(Vec::new())),
            downloads_dir: dir,
        }
    }

    pub fn add(&self, url: String) -> u64 {
        let download = Download::new(url, &self.downloads_dir);
        let id = download.id;
        self.downloads.lock().unwrap().push(download);
        id
    }

    pub fn all(&self) -> Vec<Download> {
        self.downloads.lock().unwrap().clone()
    }

    pub fn count(&self) -> usize {
        self.downloads.lock().unwrap().len()
    }

    pub fn get(&self, id: u64) -> Option<Download> {
        self.downloads.lock().unwrap()
            .iter()
            .find(|d| d.id == id)
            .cloned()
    }

    pub fn update_status(&self, id: u64, status: DownloadStatus) {
        let mut downloads = self.downloads.lock().unwrap();
        if let Some(d) = downloads.iter_mut().find(|d| d.id == id) {
            d.status = status;
        }
    }

    pub fn update_progress(&self, id: u64, received: u64, total: u64) {
        let mut downloads = self.downloads.lock().unwrap();
        if let Some(d) = downloads.iter_mut().find(|d| d.id == id) {
            d.received_bytes = received;
            d.total_bytes = total;
            d.progress = if total > 0 { received as f32 / total as f32 } else { 0.0 };
            d.status = DownloadStatus::InProgress;
        }
    }

    pub fn complete(&self, id: u64) {
        let mut downloads = self.downloads.lock().unwrap();
        if let Some(d) = downloads.iter_mut().find(|d| d.id == id) {
            d.status = DownloadStatus::Completed;
            d.progress = 1.0;
            d.finished = Some(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0)
            );
        }
    }

    pub fn fail(&self, id: u64, error: String) {
        let mut downloads = self.downloads.lock().unwrap();
        if let Some(d) = downloads.iter_mut().find(|d| d.id == id) {
            d.status = DownloadStatus::Failed;
            d.error = Some(error);
        }
    }

    pub fn clear_completed(&self) {
        self.downloads.lock().unwrap()
            .retain(|d| d.status != DownloadStatus::Completed);
    }

    pub fn active_count(&self) -> usize {
        self.downloads.lock().unwrap()
            .iter()
            .filter(|d| matches!(d.status, DownloadStatus::Pending | DownloadStatus::InProgress))
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_paths() -> StoragePaths {
        use std::time::{SystemTime, UNIX_EPOCH};
        let mut paths = StoragePaths::new();
        let t = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        paths.data_dir = std::env::temp_dir().join(format!("noir-downloads-test-{}", t));
        paths.ensure_dir().unwrap();
        paths
    }

    #[test]
    fn test_download_creation() {
        let paths = temp_paths();
        let d = Download::new("https://example.com/file.zip".to_string(), &paths.downloads_dir());
        assert_eq!(d.filename, "file.zip");
        assert_eq!(d.status, DownloadStatus::Pending);
    }

    #[test]
    fn test_download_manager_add() {
        let paths = temp_paths();
        let m = DownloadManager::new(&paths);
        let id = m.add("https://test.com/file.pdf".to_string());
        assert_eq!(m.count(), 1);
        assert!(id > 0);
    }

    #[test]
    fn test_download_progress() {
        let paths = temp_paths();
        let m = DownloadManager::new(&paths);
        let id = m.add("https://test.com/big.zip".to_string());
        m.update_progress(id, 500, 1000);
        let d = m.get(id).unwrap();
        assert_eq!(d.progress, 0.5);
        assert_eq!(d.status, DownloadStatus::InProgress);
    }

    #[test]
    fn test_download_complete() {
        let paths = temp_paths();
        let m = DownloadManager::new(&paths);
        let id = m.add("https://test.com/file".to_string());
        m.complete(id);
        let d = m.get(id).unwrap();
        assert_eq!(d.status, DownloadStatus::Completed);
        assert_eq!(d.progress, 1.0);
    }

    #[test]
    fn test_download_fail() {
        let paths = temp_paths();
        let m = DownloadManager::new(&paths);
        let id = m.add("https://test.com/file".to_string());
        m.fail(id, "Network error".to_string());
        let d = m.get(id).unwrap();
        assert_eq!(d.status, DownloadStatus::Failed);
        assert_eq!(d.error, Some("Network error".to_string()));
    }

    #[test]
    fn test_download_active_count() {
        let paths = temp_paths();
        let m = DownloadManager::new(&paths);
        let id1 = m.add("https://test.com/a".to_string());
        let id2 = m.add("https://test.com/b".to_string());
        m.complete(id2);
        assert_eq!(m.active_count(), 1);
        let _ = id1;
    }

    #[test]
    fn test_download_clear_completed() {
        let paths = temp_paths();
        let m = DownloadManager::new(&paths);
        let id1 = m.add("https://test.com/a".to_string());
        let _id2 = m.add("https://test.com/b".to_string());
        m.complete(id1);
        m.clear_completed();
        assert_eq!(m.count(), 1);
    }
}
