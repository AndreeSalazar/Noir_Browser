//! MSE Append Queue (FASE E4)
//!
//! Cola de appends para MediaSource Extensions.
//! Maneja el orden de appends, el tracking de bytes, y la limpieza de segmentos viejos.
//!
//! Inspirado en:
//! - W3C Media Source Extensions spec
//! - Chrome MSE implementation
//! - shaka-player (Google)

use std::collections::VecDeque;

/// Estado de un append pendiente
#[derive(Debug, Clone)]
pub struct PendingAppend {
    pub data: Vec<u8>,
    pub timestamp_offset_ms: f64,
    pub append_window_start_ms: f64,
    pub append_window_end_ms: f64,
    pub received_at_ms: u64,
}

/// Estado de la cola
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppendQueueState {
    /// Cola vacia, listo para appends
    Idle,
    /// Appends en cola esperando procesamiento
    Queued,
    /// Procesando un append
    Processing,
    /// Error - cola en mal estado
    Error,
}

#[derive(Debug)]
pub struct MseAppendQueue {
    queue: VecDeque<PendingAppend>,
    state: AppendQueueState,
    /// Tamano maximo de la cola en bytes
    pub max_queue_bytes: u64,
    /// Tamano maximo en segmentos
    pub max_queue_segments: u32,
    /// Total de bytes appended
    pub total_appended_bytes: u64,
    /// Total de segmentos appendeds
    pub total_appended_segments: u32,
    /// Appends rechazados por overflow
    pub rejected_appends: u64,
}

impl MseAppendQueue {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            state: AppendQueueState::Idle,
            max_queue_bytes: 32 * 1024 * 1024,  // 32 MB
            max_queue_segments: 1000,
            total_appended_bytes: 0,
            total_appended_segments: 0,
            rejected_appends: 0,
        }
    }

    /// Append data
    /// Retorna: true si se enqueuo, false si se rechazo
    pub fn append(&mut self, data: Vec<u8>, timestamp_offset_ms: f64, append_window_start_ms: f64, append_window_end_ms: f64) -> bool {
        // Check overflow
        let current_bytes: u64 = self.queue.iter().map(|a| a.data.len() as u64).sum();
        if current_bytes + data.len() as u64 > self.max_queue_bytes {
            self.rejected_appends += 1;
            return false;
        }
        if self.queue.len() as u32 >= self.max_queue_segments {
            self.rejected_appends += 1;
            return false;
        }

        let append = PendingAppend {
            data,
            timestamp_offset_ms,
            append_window_start_ms,
            append_window_end_ms,
            received_at_ms: 0,
        };
        self.queue.push_back(append);
        if matches!(self.state, AppendQueueState::Idle) {
            self.state = AppendQueueState::Queued;
        }
        true
    }

    /// Take el siguiente append para procesar
    pub fn take_next(&mut self) -> Option<PendingAppend> {
        let next = self.queue.pop_front();
        if next.is_some() {
            if !self.queue.is_empty() {
                self.state = AppendQueueState::Queued;
            } else {
                self.state = AppendQueueState::Idle;
            }
        }
        next
    }

    /// Mark un append como procesado
    pub fn mark_processed(&mut self, append: &PendingAppend) {
        self.total_appended_bytes += append.data.len() as u64;
        self.total_appended_segments += 1;
        if self.queue.is_empty() {
            self.state = AppendQueueState::Idle;
        }
    }

    /// Quitar segmentos antes de un timestamp (cleanup)
    pub fn remove_before(&mut self, timestamp_ms: f64) -> u64 {
        let initial = self.queue.len();
        self.queue.retain(|a| a.timestamp_offset_ms >= timestamp_ms);
        (initial - self.queue.len()) as u64
    }

    /// Cancelar todo
    pub fn abort(&mut self) {
        self.queue.clear();
        self.state = AppendQueueState::Idle;
    }

    /// Marcar como error
    pub fn mark_error(&mut self) {
        self.state = AppendQueueState::Error;
    }

    /// Marcar como processing
    pub fn mark_processing(&mut self) {
        self.state = AppendQueueState::Processing;
    }

    /// Tamano actual de la cola
    pub fn queue_size_bytes(&self) -> u64 {
        self.queue.iter().map(|a| a.data.len() as u64).sum()
    }

    /// Segmentos en cola
    pub fn queue_len(&self) -> usize {
        self.queue.len()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    pub fn state(&self) -> AppendQueueState {
        self.state
    }
}

impl Default for MseAppendQueue {
    fn default() -> Self { Self::new() }
}

/// Bitrate estimator para MSE
#[derive(Debug, Default)]
pub struct MseBandwidthEstimator {
    pub samples: Vec<(u64, u64)>,  // (timestamp_ms, bytes)
    pub window_ms: u64,
    pub current_kbps: u32,
}

impl MseBandwidthEstimator {
    pub fn new() -> Self { Self::default() }

    pub fn add_sample(&mut self, timestamp_ms: u64, bytes: u64) {
        self.samples.push((timestamp_ms, bytes));
        // Limpiar fuera de ventana
        if self.window_ms > 0 {
            self.samples.retain(|(ts, _)| timestamp_ms - *ts <= self.window_ms);
        }
        self.recalculate();
    }

    fn recalculate(&mut self) {
        if self.samples.len() < 2 {
            return;
        }
        let first = self.samples.first().unwrap().0;
        let last = self.samples.last().unwrap().0;
        let total_bytes: u64 = self.samples.iter().map(|(_, b)| *b).sum();
        let duration_ms = last - first;
        if duration_ms == 0 {
            return;
        }
        let bps = (total_bytes * 8 * 1000) / duration_ms;
        self.current_kbps = (bps / 1000) as u32;
    }

    pub fn set_window(&mut self, ms: u64) {
        self.window_ms = ms;
    }
}

/// Buffer level tracker
#[derive(Debug, Clone, Copy)]
pub struct BufferLevel {
    pub buffered_ms: f64,
    pub current_playback_ms: f64,
    pub target_buffer_ms: f64,
    pub is_starving: bool,
}

impl BufferLevel {
    /// Calcular si el buffer esta en starvation
    pub fn check(buffered_ms: f64, current_ms: f64, target_ms: f64) -> Self {
        let behind = buffered_ms - (current_ms - current_ms);
        let is_starving = buffered_ms < target_ms;
        Self {
            buffered_ms: behind,
            current_playback_ms: current_ms,
            target_buffer_ms: target_ms,
            is_starving,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_queue_creation() {
        let q = MseAppendQueue::new();
        assert!(q.is_empty());
        assert_eq!(q.state(), AppendQueueState::Idle);
    }

    #[test]
    fn test_queue_append() {
        let mut q = MseAppendQueue::new();
        let data = vec![0u8; 1000];
        assert!(q.append(data, 0.0, 0.0, 0.0));
        assert!(!q.is_empty());
        assert_eq!(q.queue_len(), 1);
        assert_eq!(q.queue_size_bytes(), 1000);
    }

    #[test]
    fn test_queue_take_next() {
        let mut q = MseAppendQueue::new();
        q.append(vec![0u8; 100], 0.0, 0.0, 0.0);
        q.append(vec![0u8; 200], 100.0, 0.0, 0.0);
        let first = q.take_next().unwrap();
        assert_eq!(first.data.len(), 100);
        assert_eq!(first.timestamp_offset_ms, 0.0);
        let second = q.take_next().unwrap();
        assert_eq!(second.data.len(), 200);
        assert!(q.is_empty());
    }

    #[test]
    fn test_queue_mark_processed() {
        let mut q = MseAppendQueue::new();
        let append = PendingAppend {
            data: vec![0u8; 500],
            timestamp_offset_ms: 0.0,
            append_window_start_ms: 0.0,
            append_window_end_ms: 0.0,
            received_at_ms: 0,
        };
        q.mark_processed(&append);
        assert_eq!(q.total_appended_bytes, 500);
        assert_eq!(q.total_appended_segments, 1);
    }

    #[test]
    fn test_queue_overflow_bytes() {
        let mut q = MseAppendQueue::new();
        q.max_queue_bytes = 1000;
        assert!(q.append(vec![0u8; 600], 0.0, 0.0, 0.0));
        assert!(!q.append(vec![0u8; 600], 0.0, 0.0, 0.0));  // overflow
        assert_eq!(q.rejected_appends, 1);
    }

    #[test]
    fn test_queue_overflow_segments() {
        let mut q = MseAppendQueue::new();
        q.max_queue_segments = 2;
        assert!(q.append(vec![0u8; 1], 0.0, 0.0, 0.0));
        assert!(q.append(vec![0u8; 1], 100.0, 0.0, 0.0));
        assert!(!q.append(vec![0u8; 1], 200.0, 0.0, 0.0));
    }

    #[test]
    fn test_queue_remove_before() {
        let mut q = MseAppendQueue::new();
        q.append(vec![0u8; 1], 0.0, 0.0, 0.0);
        q.append(vec![0u8; 1], 100.0, 0.0, 0.0);
        q.append(vec![0u8; 1], 200.0, 0.0, 0.0);
        let removed = q.remove_before(150.0);
        assert_eq!(removed, 2);
        assert_eq!(q.queue_len(), 1);
    }

    #[test]
    fn test_queue_abort() {
        let mut q = MseAppendQueue::new();
        q.append(vec![0u8; 100], 0.0, 0.0, 0.0);
        q.abort();
        assert!(q.is_empty());
    }

    #[test]
    fn test_queue_mark_error() {
        let mut q = MseAppendQueue::new();
        q.mark_error();
        assert_eq!(q.state(), AppendQueueState::Error);
    }

    #[test]
    fn test_bandwidth_estimator_creation() {
        let e = MseBandwidthEstimator::new();
        assert_eq!(e.current_kbps, 0);
    }

    #[test]
    fn test_bandwidth_estimator_samples() {
        let mut e = MseBandwidthEstimator::new();
        e.set_window(1000);
        e.add_sample(0, 100_000);
        e.add_sample(1000, 100_000);
        assert!(e.current_kbps > 0);
    }

    #[test]
    fn test_buffer_level() {
        let bl = BufferLevel::check(500.0, 200.0, 1000.0);
        assert!(bl.is_starving);
        let bl = BufferLevel::check(2000.0, 200.0, 1000.0);
        assert!(!bl.is_starving);
    }

    #[test]
    fn test_state_transitions() {
        let mut q = MseAppendQueue::new();
        assert_eq!(q.state(), AppendQueueState::Idle);
        q.append(vec![0u8; 1], 0.0, 0.0, 0.0);
        assert_eq!(q.state(), AppendQueueState::Queued);
        q.take_next();
        assert_eq!(q.state(), AppendQueueState::Idle);
    }
}
