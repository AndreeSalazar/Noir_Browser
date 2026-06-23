//! A/V Sync (FASE E2)
//!
//! Sincronizacion audio/video para playback.
//! Mantiene:
//! - Clock master (audio clock)
//! - Drift tolerance
//! - Sync strategies (drop, repeat, resample)
//!
//! Inspirado en:
//! - Chrome media pipeline
//! - Firefox MSE
//! - GStreamer

use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SyncMode {
    /// Resample audio si drift > threshold
    AudioResample,
    /// Drop/repeat video frames para追上
    VideoDropRepeat,
    /// Ajuste de clock master
    ClockAdjust,
    /// Freeze: ambos se pausan hasta alinear
    Freeze,
}

impl SyncMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            SyncMode::AudioResample => "audio-resample",
            SyncMode::VideoDropRepeat => "video-drop-repeat",
            SyncMode::ClockAdjust => "clock-adjust",
            SyncMode::Freeze => "freeze",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SyncState {
    InSync,
    SlightlyDrifted,
    HeavilyDrifted,
    OutOfSync,
}

#[derive(Debug, Clone)]
pub struct AvClock {
    pub audio_pts_ms: f64,
    pub video_pts_ms: f64,
    pub master_clock: MasterClock,
    pub drift_tolerance_ms: f64,
    pub max_drift_ms: f64,
    pub mode: SyncMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MasterClock {
    Audio,
    Video,
    System,
    External,
}

impl MasterClock {
    pub fn as_str(&self) -> &'static str {
        match self {
            MasterClock::Audio => "audio",
            MasterClock::Video => "video",
            MasterClock::System => "system",
            MasterClock::External => "external",
        }
    }
}

impl AvClock {
    pub fn new(master: MasterClock) -> Self {
        Self {
            audio_pts_ms: 0.0,
            video_pts_ms: 0.0,
            master_clock: master,
            drift_tolerance_ms: 20.0,   // 20ms es OK
            max_drift_ms: 100.0,        // 100ms es max antes de resync
            mode: SyncMode::AudioResample,
        }
    }

    /// Update audio PTS
    pub fn update_audio(&mut self, pts_ms: f64) {
        self.audio_pts_ms = pts_ms;
    }

    /// Update video PTS
    pub fn update_video(&mut self, pts_ms: f64) {
        self.video_pts_ms = pts_ms;
    }

    /// Calcular drift actual (positiva = video ahead, negativa = audio ahead)
    pub fn drift(&self) -> f64 {
        self.video_pts_ms - self.audio_pts_ms
    }

    /// Estado de sync basado en drift
    pub fn sync_state(&self) -> SyncState {
        let d = self.drift().abs();
        if d < 5.0 {
            SyncState::InSync
        } else if d < self.drift_tolerance_ms {
            SyncState::SlightlyDrifted
        } else if d < self.max_drift_ms {
            SyncState::HeavilyDrifted
        } else {
            SyncState::OutOfSync
        }
    }

    /// Corregir sync
    /// Retorna: accion recomendada
    pub fn correct(&mut self) -> SyncAction {
        let drift = self.drift();
        let abs = drift.abs();

        if abs < 5.0 {
            return SyncAction::None;
        }

        match self.mode {
            SyncMode::AudioResample => {
                if abs > self.max_drift_ms {
                    // Demasiado drift, resample audio para追上
                    SyncAction::ResampleAudio { factor: 1.01 }
                } else {
                    SyncAction::ResampleAudio { factor: 1.005 }
                }
            }
            SyncMode::VideoDropRepeat => {
                if drift > 0.0 {
                    // Video ahead, drop video
                    SyncAction::DropVideoFrames { count: 1 }
                } else {
                    // Audio ahead, repeat video
                    SyncAction::RepeatVideoFrames { count: 1 }
                }
            }
            SyncMode::ClockAdjust => {
                // Ajustar el master clock
                SyncAction::AdjustClock { delta_ms: drift * 0.5 }
            }
            SyncMode::Freeze => {
                SyncAction::Freeze
            }
        }
    }

    /// Reset sync
    pub fn reset(&mut self) {
        self.audio_pts_ms = 0.0;
        self.video_pts_ms = 0.0;
    }
}

/// Accion de correccion
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SyncAction {
    None,
    ResampleAudio { factor: f64 },
    DropVideoFrames { count: u32 },
    RepeatVideoFrames { count: u32 },
    AdjustClock { delta_ms: f64 },
    Freeze,
}

impl SyncAction {
    pub fn is_action(&self) -> bool {
        !matches!(self, SyncAction::None)
    }
}

/// Frame scheduler
#[derive(Debug, Default)]
pub struct FrameScheduler {
    pub next_pts: f64,
    pub frame_interval_ms: f64,
    pub dropped_frames: u64,
    pub repeated_frames: u64,
    pub last_action: Option<SyncAction>,
}

impl FrameScheduler {
    pub fn new(fps: f64) -> Self {
        Self {
            next_pts: 0.0,
            frame_interval_ms: 1000.0 / fps,
            dropped_frames: 0,
            repeated_frames: 0,
            last_action: None,
        }
    }

    /// Obtener el PTS del proximo frame
    pub fn next_frame_pts(&self) -> f64 {
        self.next_pts
    }

    /// Avanzar al siguiente frame
    pub fn advance(&mut self) {
        self.next_pts += self.frame_interval_ms;
    }

    /// Handle sync action
    pub fn apply_action(&mut self, action: SyncAction) {
        self.last_action = Some(action);
        match action {
            SyncAction::DropVideoFrames { count } => {
                self.dropped_frames += count as u64;
                for _ in 0..count {
                    self.advance();
                }
            }
            SyncAction::RepeatVideoFrames { count } => {
                self.repeated_frames += count as u64;
                // No advance
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_mode_str() {
        assert_eq!(SyncMode::AudioResample.as_str(), "audio-resample");
        assert_eq!(SyncMode::VideoDropRepeat.as_str(), "video-drop-repeat");
    }

    #[test]
    fn test_master_clock_str() {
        assert_eq!(MasterClock::Audio.as_str(), "audio");
        assert_eq!(MasterClock::Video.as_str(), "video");
    }

    #[test]
    fn test_av_clock_creation() {
        let clock = AvClock::new(MasterClock::Audio);
        assert_eq!(clock.master_clock, MasterClock::Audio);
        assert_eq!(clock.audio_pts_ms, 0.0);
    }

    #[test]
    fn test_drift() {
        let mut clock = AvClock::new(MasterClock::Audio);
        clock.update_audio(100.0);
        clock.update_video(120.0);
        // Video ahead by 20ms
        assert_eq!(clock.drift(), 20.0);
    }

    #[test]
    fn test_drift_negative() {
        let mut clock = AvClock::new(MasterClock::Audio);
        clock.update_audio(120.0);
        clock.update_video(100.0);
        // Audio ahead by 20ms
        assert_eq!(clock.drift(), -20.0);
    }

    #[test]
    fn test_sync_state_in_sync() {
        let mut clock = AvClock::new(MasterClock::Audio);
        clock.update_audio(100.0);
        clock.update_video(102.0);
        assert_eq!(clock.sync_state(), SyncState::InSync);
    }

    #[test]
    fn test_sync_state_slightly_drifted() {
        let mut clock = AvClock::new(MasterClock::Audio);
        clock.update_audio(100.0);
        clock.update_video(115.0);
        assert_eq!(clock.sync_state(), SyncState::SlightlyDrifted);
    }

    #[test]
    fn test_sync_state_heavily_drifted() {
        let mut clock = AvClock::new(MasterClock::Audio);
        clock.update_audio(100.0);
        clock.update_video(150.0);
        assert_eq!(clock.sync_state(), SyncState::HeavilyDrifted);
    }

    #[test]
    fn test_sync_state_out_of_sync() {
        let mut clock = AvClock::new(MasterClock::Audio);
        clock.update_audio(100.0);
        clock.update_video(300.0);
        assert_eq!(clock.sync_state(), SyncState::OutOfSync);
    }

    #[test]
    fn test_correct_resample() {
        let mut clock = AvClock::new(MasterClock::Audio);
        clock.mode = SyncMode::AudioResample;
        clock.update_audio(100.0);
        clock.update_video(150.0);
        let action = clock.correct();
        assert!(matches!(action, SyncAction::ResampleAudio { .. }));
    }

    #[test]
    fn test_correct_drop_video() {
        let mut clock = AvClock::new(MasterClock::Audio);
        clock.mode = SyncMode::VideoDropRepeat;
        clock.update_audio(100.0);
        clock.update_video(150.0);
        let action = clock.correct();
        assert!(matches!(action, SyncAction::DropVideoFrames { .. }));
    }

    #[test]
    fn test_correct_repeat_video() {
        let mut clock = AvClock::new(MasterClock::Audio);
        clock.mode = SyncMode::VideoDropRepeat;
        clock.update_audio(150.0);
        clock.update_video(100.0);
        let action = clock.correct();
        assert!(matches!(action, SyncAction::RepeatVideoFrames { .. }));
    }

    #[test]
    fn test_correct_adjust_clock() {
        let mut clock = AvClock::new(MasterClock::Audio);
        clock.mode = SyncMode::ClockAdjust;
        clock.update_audio(100.0);
        clock.update_video(150.0);
        let action = clock.correct();
        assert!(matches!(action, SyncAction::AdjustClock { .. }));
    }

    #[test]
    fn test_correct_freeze() {
        let mut clock = AvClock::new(MasterClock::Audio);
        clock.mode = SyncMode::Freeze;
        clock.update_audio(100.0);
        clock.update_video(150.0);
        let action = clock.correct();
        assert!(matches!(action, SyncAction::Freeze));
    }

    #[test]
    fn test_correct_no_action() {
        let mut clock = AvClock::new(MasterClock::Audio);
        clock.update_audio(100.0);
        clock.update_video(101.0);
        let action = clock.correct();
        assert!(!action.is_action());
    }

    #[test]
    fn test_reset() {
        let mut clock = AvClock::new(MasterClock::Audio);
        clock.update_audio(100.0);
        clock.update_video(200.0);
        clock.reset();
        assert_eq!(clock.audio_pts_ms, 0.0);
        assert_eq!(clock.video_pts_ms, 0.0);
    }

    #[test]
    fn test_frame_scheduler_60fps() {
        let mut s = FrameScheduler::new(60.0);
        assert!((s.frame_interval_ms - 16.666).abs() < 0.01);
        assert_eq!(s.next_frame_pts(), 0.0);
        s.advance();
        assert!((s.next_frame_pts() - 16.666).abs() < 0.01);
    }

    #[test]
    fn test_frame_scheduler_drop() {
        let mut s = FrameScheduler::new(30.0);
        s.apply_action(SyncAction::DropVideoFrames { count: 2 });
        assert_eq!(s.dropped_frames, 2);
        // PTS advanced 2 frames
        assert!((s.next_frame_pts() - 66.66).abs() < 0.01);
    }

    #[test]
    fn test_frame_scheduler_repeat() {
        let mut s = FrameScheduler::new(30.0);
        s.apply_action(SyncAction::RepeatVideoFrames { count: 1 });
        assert_eq!(s.repeated_frames, 1);
        // PTS no avanza
        assert_eq!(s.next_frame_pts(), 0.0);
    }

    #[test]
    fn test_sync_action_is_action() {
        assert!(!SyncAction::None.is_action());
        assert!(SyncAction::Freeze.is_action());
    }
}
