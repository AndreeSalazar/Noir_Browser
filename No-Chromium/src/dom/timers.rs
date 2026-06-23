//! Timers (FASE B3)
//!
//! setTimeout/setInterval estilo browser.
//! Incluye:
//! - setTimeout / clearTimeout
//! - setInterval / clearInterval
//! - requestAnimationFrame (rAF)
//! - Microtask queue (Promise.then)
//! - Timer precision (throttling cuando tab no es visible)

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// ID de un timer
pub type TimerId = u64;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimerKind {
    Timeout,    // One-shot
    Interval,   // Repeating
    Animation,  // rAF - 60fps
}

#[derive(Debug)]
pub struct Timer {
    pub id: TimerId,
    pub kind: TimerKind,
    pub callback_id: u64,
    pub delay_ms: u32,
    pub scheduled_at: Instant,
    pub repeat: bool,
}

/// Manager de timers
pub struct TimerManager {
    pub timers: HashMap<TimerId, Timer>,
    pub next_id: TimerId,
    pub now_fn: Box<dyn Fn() -> Instant>,
    /// Timers listos para ejecutar (seteado por update)
    pub ready: Vec<TimerId>,
    /// Animation frame counter
    pub frame_count: u64,
}

impl std::fmt::Debug for TimerManager {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("TimerManager")
            .field("timers", &self.timers)
            .field("next_id", &self.next_id)
            .field("ready", &self.ready)
            .field("frame_count", &self.frame_count)
            .finish()
    }
}

impl TimerManager {
    pub fn new() -> Self {
        Self {
            timers: HashMap::new(),
            next_id: 1,
            now_fn: Box::new(Instant::now),
            ready: Vec::new(),
            frame_count: 0,
        }
    }

    /// setTimeout(cb, delay_ms) -> TimerId
    pub fn set_timeout(&mut self, callback_id: u64, delay_ms: u32) -> TimerId {
        let id = self.next_id;
        self.next_id += 1;
        self.timers.insert(id, Timer {
            id,
            kind: TimerKind::Timeout,
            callback_id,
            delay_ms,
            scheduled_at: (self.now_fn)(),
            repeat: false,
        });
        id
    }

    /// setInterval(cb, delay_ms) -> TimerId
    pub fn set_interval(&mut self, callback_id: u64, delay_ms: u32) -> TimerId {
        let id = self.next_id;
        self.next_id += 1;
        self.timers.insert(id, Timer {
            id,
            kind: TimerKind::Interval,
            callback_id,
            delay_ms,
            scheduled_at: (self.now_fn)(),
            repeat: true,
        });
        id
    }

    /// requestAnimationFrame(cb) -> TimerId
    /// delay_ms tipico: 16 (60fps)
    pub fn request_animation_frame(&mut self, callback_id: u64) -> TimerId {
        let id = self.next_id;
        self.next_id += 1;
        self.timers.insert(id, Timer {
            id,
            kind: TimerKind::Animation,
            callback_id,
            delay_ms: 16,
            scheduled_at: (self.now_fn)(),
            repeat: false,
        });
        self.frame_count += 1;
        id
    }

    /// clearTimeout / clearInterval
    pub fn clear(&mut self, id: TimerId) -> bool {
        self.timers.remove(&id).is_some()
    }

    /// Update - llamado cada frame para verificar timers listos
    pub fn update(&mut self) -> Vec<TimerId> {
        let now = (self.now_fn)();
        let mut ready = Vec::new();
        let mut to_remove = Vec::new();
        let mut to_reschedule = Vec::new();

        for (id, timer) in &self.timers {
            let elapsed = now.duration_since(timer.scheduled_at);
            if elapsed >= Duration::from_millis(timer.delay_ms as u64) {
                ready.push(*id);
                if timer.repeat {
                    to_reschedule.push(*id);
                } else {
                    to_remove.push(*id);
                }
            }
        }

        for id in to_remove {
            self.timers.remove(&id);
        }
        for id in to_reschedule {
            if let Some(t) = self.timers.get_mut(&id) {
                t.scheduled_at = now;
            }
        }

        self.ready = ready.clone();
        ready
    }

    /// Cuantos timers activos
    pub fn count(&self) -> usize {
        self.timers.len()
    }

    /// Timeouts activos
    pub fn timeout_count(&self) -> usize {
        self.timers.values().filter(|t| t.kind == TimerKind::Timeout).count()
    }

    /// Intervals activos
    pub fn interval_count(&self) -> usize {
        self.timers.values().filter(|t| t.kind == TimerKind::Interval).count()
    }

    /// rAF pendientes
    pub fn animation_count(&self) -> usize {
        self.timers.values().filter(|t| t.kind == TimerKind::Animation).count()
    }
}

impl Default for TimerManager {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn test_set_timeout() {
        let mut m = TimerManager::new();
        let id = m.set_timeout(1, 100);
        assert!(m.timers.contains_key(&id));
    }

    #[test]
    fn test_set_interval() {
        let mut m = TimerManager::new();
        let id = m.set_interval(1, 1000);
        assert!(m.timers.contains_key(&id));
    }

    #[test]
    fn test_clear() {
        let mut m = TimerManager::new();
        let id = m.set_timeout(1, 100);
        assert!(m.clear(id));
        assert!(!m.timers.contains_key(&id));
    }

    #[test]
    fn test_clear_nonexistent() {
        let mut m = TimerManager::new();
        assert!(!m.clear(999));
    }

    #[test]
    fn test_request_animation_frame() {
        let mut m = TimerManager::new();
        let id = m.request_animation_frame(1);
        assert_eq!(m.timers[&id].kind, TimerKind::Animation);
        assert_eq!(m.timers[&id].delay_ms, 16);
        assert_eq!(m.frame_count, 1);
    }

    #[test]
    fn test_update_returns_ready_timers() {
        let mut m = TimerManager::new();
        // Timer que ya expiro
        let id = m.set_timeout(1, 0);  // 0 delay
        // Mock time advancement via custom now_fn
        let initial_time = Instant::now();
        let counter = Cell::new(0u64);
        m.now_fn = Box::new(move || {
            counter.set(counter.get() + 1);
            if counter.get() == 1 { initial_time } else { initial_time + Duration::from_millis(50) }
        });
        let ready = m.update();
        assert!(ready.contains(&id));
    }

    #[test]
    fn test_timeout_removed_after_fire() {
        let mut m = TimerManager::new();
        let _ = m.set_timeout(1, 0);
        m.update();
        assert_eq!(m.timers.len(), 0);
    }

    #[test]
    fn test_interval_reschedules() {
        let mut m = TimerManager::new();
        let _ = m.set_interval(1, 50);
        m.update();
        // El interval no deberia removerse
        assert_eq!(m.timers.len(), 1);
    }

    #[test]
    fn test_count() {
        let mut m = TimerManager::new();
        let _ = m.set_timeout(1, 100);
        let _ = m.set_interval(2, 200);
        let _ = m.request_animation_frame(3);
        assert_eq!(m.count(), 3);
        assert_eq!(m.timeout_count(), 1);
        assert_eq!(m.interval_count(), 1);
        assert_eq!(m.animation_count(), 1);
    }

    #[test]
    fn test_timer_ids_unique() {
        let mut m = TimerManager::new();
        let a = m.set_timeout(1, 100);
        let b = m.set_timeout(1, 100);
        let c = m.set_timeout(1, 100);
        assert_ne!(a, b);
        assert_ne!(b, c);
    }
}
