//! Audio Playback - Abstracción sobre cpal para reproducción de audio
//!
//! Genera waveforms (sine, square, noise) y maneja buffer de salida.

use std::f32::consts::PI;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Waveform {
    Sine,
    Square,
    Sawtooth,
    Triangle,
    Noise,
    Silence,
}

impl Waveform {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "sine" | "sin" => Self::Sine,
            "square" | "sqr" => Self::Square,
            "sawtooth" | "saw" => Self::Sawtooth,
            "triangle" | "tri" => Self::Triangle,
            "noise" | "rand" => Self::Noise,
            _ => Self::Silence,
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            Self::Sine => "sine",
            Self::Square => "square",
            Self::Sawtooth => "sawtooth",
            Self::Triangle => "triangle",
            Self::Noise => "noise",
            Self::Silence => "silence",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SampleFormat {
    F32,
    I16,
    U16,
}

impl SampleFormat {
    pub fn bytes_per_sample(&self) -> usize {
        match self {
            Self::F32 => 4,
            Self::I16 => 2,
            Self::U16 => 2,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AudioConfig {
    pub sample_rate: u32,
    pub channels: u16,
    pub format: SampleFormat,
    pub buffer_size: u32,
    pub volume: f32,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            sample_rate: 44100,
            channels: 2,
            format: SampleFormat::F32,
            buffer_size: 512,
            volume: 1.0,
        }
    }
}

pub struct ToneGenerator {
    pub waveform: Waveform,
    pub frequency: f32,
    pub amplitude: f32,
    pub phase: f32,
    pub config: AudioConfig,
    pub playing: bool,
    pub samples_generated: u64,
}

impl ToneGenerator {
    pub fn new(waveform: Waveform, frequency: f32) -> Self {
        Self {
            waveform,
            frequency,
            amplitude: 0.5,
            phase: 0.0,
            config: AudioConfig::default(),
            playing: false,
            samples_generated: 0,
        }
    }

    pub fn with_config(mut self, config: AudioConfig) -> Self {
        self.config = config;
        self
    }

    pub fn set_frequency(&mut self, freq: f32) {
        self.frequency = freq.max(0.0).min(22000.0);
    }

    pub fn set_amplitude(&mut self, amp: f32) {
        self.amplitude = amp.clamp(0.0, 1.0);
    }

    pub fn start(&mut self) {
        self.playing = true;
    }

    pub fn stop(&mut self) {
        self.playing = false;
    }

    /// Genera N samples
    pub fn generate(&mut self, count: usize) -> Vec<f32> {
        let mut out = vec![0.0f32; count];
        if !self.playing {
            return out;
        }
        let dt = 1.0 / self.config.sample_rate as f32;
        for i in 0..count {
            let t = self.phase;
            let sample = match self.waveform {
                Waveform::Sine => (2.0 * PI * self.frequency * t).sin(),
                Waveform::Square => {
                    let v = (2.0 * PI * self.frequency * t).sin();
                    if v >= 0.0 { 1.0 } else { -1.0 }
                }
                Waveform::Sawtooth => {
                    let p = t * self.frequency;
                    p - p.floor() - 0.5
                }
                Waveform::Triangle => {
                    let p = (t * self.frequency) % 1.0;
                    if p < 0.5 { 4.0 * p - 1.0 } else { 3.0 - 4.0 * p }
                }
                Waveform::Noise => {
                    use std::collections::hash_map::DefaultHasher;
                    use std::hash::{Hash, Hasher};
                    let mut h = DefaultHasher::new();
                    (self.samples_generated + i as u64).hash(&mut h);
                    (h.finish() as f32 / u64::MAX as f32) * 2.0 - 1.0
                }
                Waveform::Silence => 0.0,
            };
            out[i] = sample * self.amplitude * self.config.volume;
            self.phase += dt;
            if self.phase > 1.0 {
                self.phase -= 1.0;
            }
            self.samples_generated += 1;
        }
        out
    }

    /// Genera samples interleaved (L R L R L R ...)
    pub fn generate_interleaved(&mut self, frames: usize) -> Vec<f32> {
        let samples = self.generate(frames);
        let mut out = Vec::with_capacity(frames * self.config.channels as usize);
        for s in samples {
            for _ in 0..self.config.channels {
                out.push(s);
            }
        }
        out
    }
}

pub struct AudioBuffer {
    pub data: Vec<f32>,
    pub sample_rate: u32,
    pub channels: u16,
    pub cursor: usize,
    pub playing: bool,
    pub loop_playback: bool,
}

impl AudioBuffer {
    pub fn new(sample_rate: u32, channels: u16) -> Self {
        Self {
            data: Vec::new(),
            sample_rate,
            channels,
            cursor: 0,
            playing: false,
            loop_playback: false,
        }
    }

    pub fn load(&mut self, data: Vec<f32>) {
        self.data = data;
        self.cursor = 0;
    }

    pub fn play(&mut self) {
        self.playing = true;
    }

    pub fn stop(&mut self) {
        self.playing = false;
    }

    pub fn reset(&mut self) {
        self.cursor = 0;
    }

    /// Genera N frames de audio (interleaved)
    pub fn fill(&mut self, frames: usize) -> Vec<f32> {
        let total_samples = self.data.len();
        let mut out = vec![0.0f32; frames * self.channels as usize];
        if !self.playing || total_samples == 0 {
            return out;
        }
        for i in 0..frames {
            if self.cursor >= total_samples {
                if self.loop_playback {
                    self.cursor = 0;
                } else {
                    self.playing = false;
                    break;
                }
            }
            for c in 0..self.channels as usize {
                out[i * self.channels as usize + c] = self.data[self.cursor + c.min(self.data.len() - self.cursor - 1)];
            }
            self.cursor += self.channels as usize;
        }
        out
    }

    pub fn progress(&self) -> f32 {
        if self.data.is_empty() { return 0.0; }
        (self.cursor as f32 / self.data.len() as f32).clamp(0.0, 1.0)
    }

    pub fn duration_seconds(&self) -> f32 {
        if self.sample_rate == 0 { return 0.0; }
        self.data.len() as f32 / (self.sample_rate as f32 * self.channels as f32)
    }
}

pub struct AudioMixer {
    pub config: AudioConfig,
    pub generators: Vec<ToneGenerator>,
    pub buffers: Vec<AudioBuffer>,
    pub master_volume: f32,
    pub muted: bool,
}

impl AudioMixer {
    pub fn new(config: AudioConfig) -> Self {
        Self {
            config,
            generators: Vec::new(),
            buffers: Vec::new(),
            master_volume: 1.0,
            muted: false,
        }
    }

    pub fn add_generator(&mut self, gen: ToneGenerator) {
        self.generators.push(gen);
    }

    pub fn add_buffer(&mut self, buf: AudioBuffer) {
        self.buffers.push(buf);
    }

    pub fn set_master_volume(&mut self, v: f32) {
        self.master_volume = v.clamp(0.0, 1.0);
    }

    pub fn toggle_mute(&mut self) {
        self.muted = !self.muted;
    }

    /// Render N frames mezclando todas las fuentes
    pub fn render(&mut self, frames: usize) -> Vec<f32> {
        let mut mix = vec![0.0f32; frames * self.config.channels as usize];
        for gen in &mut self.generators {
            let s = gen.generate_interleaved(frames);
            for i in 0..mix.len().min(s.len()) {
                mix[i] += s[i];
            }
        }
        for buf in &mut self.buffers {
            let s = buf.fill(frames);
            for i in 0..mix.len().min(s.len()) {
                mix[i] += s[i];
            }
        }
        let vol = if self.muted { 0.0 } else { self.master_volume };
        for s in &mut mix {
            *s = (*s * vol).clamp(-1.0, 1.0);
        }
        mix
    }
}

impl Default for AudioMixer {
    fn default() -> Self {
        Self::new(AudioConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_waveform_from_str() {
        assert_eq!(Waveform::from_str("sine"), Waveform::Sine);
        assert_eq!(Waveform::from_str("SQUARE"), Waveform::Square);
        assert_eq!(Waveform::from_str("noise"), Waveform::Noise);
    }

    #[test]
    fn test_waveform_to_str() {
        assert_eq!(Waveform::Sine.to_str(), "sine");
        assert_eq!(Waveform::Noise.to_str(), "noise");
    }

    #[test]
    fn test_sample_format_bps() {
        assert_eq!(SampleFormat::F32.bytes_per_sample(), 4);
        assert_eq!(SampleFormat::I16.bytes_per_sample(), 2);
    }

    #[test]
    fn test_audio_config_default() {
        let c = AudioConfig::default();
        assert_eq!(c.sample_rate, 44100);
        assert_eq!(c.channels, 2);
    }

    #[test]
    fn test_tone_generator_new() {
        let g = ToneGenerator::new(Waveform::Sine, 440.0);
        assert_eq!(g.frequency, 440.0);
        assert!(!g.playing);
    }

    #[test]
    fn test_tone_generator_with_config() {
        let cfg = AudioConfig { sample_rate: 48000, channels: 1, format: SampleFormat::F32, buffer_size: 256, volume: 0.5 };
        let g = ToneGenerator::new(Waveform::Sine, 440.0).with_config(cfg);
        assert_eq!(g.config.sample_rate, 48000);
    }

    #[test]
    fn test_tone_set_frequency() {
        let mut g = ToneGenerator::new(Waveform::Sine, 440.0);
        g.set_frequency(880.0);
        assert_eq!(g.frequency, 880.0);
        g.set_frequency(30000.0);
        assert_eq!(g.frequency, 22000.0);
    }

    #[test]
    fn test_tone_set_amplitude() {
        let mut g = ToneGenerator::new(Waveform::Sine, 440.0);
        g.set_amplitude(0.5);
        assert_eq!(g.amplitude, 0.5);
        g.set_amplitude(2.0);
        assert_eq!(g.amplitude, 1.0);
    }

    #[test]
    fn test_tone_generate_sine() {
        let mut g = ToneGenerator::new(Waveform::Sine, 440.0);
        g.start();
        let s = g.generate(100);
        assert_eq!(s.len(), 100);
        assert!(s.iter().any(|&v| v != 0.0));
    }

    #[test]
    fn test_tone_generate_silence() {
        let mut g = ToneGenerator::new(Waveform::Sine, 440.0);
        g.start();
        g.waveform = Waveform::Silence;
        let s = g.generate(100);
        assert!(s.iter().all(|&v| v == 0.0));
    }

    #[test]
    fn test_tone_generate_not_playing() {
        let mut g = ToneGenerator::new(Waveform::Sine, 440.0);
        let s = g.generate(100);
        assert!(s.iter().all(|&v| v == 0.0));
    }

    #[test]
    fn test_tone_generate_interleaved() {
        let mut g = ToneGenerator::new(Waveform::Sine, 440.0);
        g.start();
        let s = g.generate_interleaved(50);
        // 50 frames * 2 channels = 100 samples
        assert_eq!(s.len(), 100);
    }

    #[test]
    fn test_tone_generate_noise() {
        let mut g = ToneGenerator::new(Waveform::Noise, 0.0);
        g.start();
        let s = g.generate(100);
        assert!(s.iter().any(|&v| v != 0.0));
    }

    #[test]
    fn test_audio_buffer_new() {
        let b = AudioBuffer::new(44100, 2);
        assert_eq!(b.sample_rate, 44100);
        assert!(!b.playing);
    }

    #[test]
    fn test_audio_buffer_load_play() {
        let mut b = AudioBuffer::new(44100, 2);
        b.load(vec![0.5; 1000]);
        b.play();
        assert!(b.playing);
    }

    #[test]
    fn test_audio_buffer_fill() {
        let mut b = AudioBuffer::new(44100, 2);
        b.load(vec![0.5; 1000]);
        b.play();
        let frames = b.fill(50);
        assert_eq!(frames.len(), 50 * 2);
    }

    #[test]
    fn test_audio_buffer_loop() {
        let mut b = AudioBuffer::new(44100, 2);
        b.load(vec![0.5; 100]);
        b.loop_playback = true;
        b.play();
        let _ = b.fill(60); // consume 60 frames = 120 samples
        let _ = b.fill(60); // needs to loop
        assert!(b.playing);
    }

    #[test]
    fn test_audio_buffer_stop_no_loop() {
        let mut b = AudioBuffer::new(44100, 2);
        b.load(vec![0.5; 100]);
        b.play();
        let _ = b.fill(60); // consume 60 frames = 120 samples
        assert!(!b.playing);
    }

    #[test]
    fn test_audio_buffer_duration() {
        let mut b = AudioBuffer::new(44100, 2);
        b.load(vec![0.5; 44100 * 2]); // 1 second stereo
        assert_eq!(b.duration_seconds(), 1.0);
    }

    #[test]
    fn test_audio_buffer_progress() {
        let mut b = AudioBuffer::new(44100, 2);
        b.load(vec![0.5; 1000]);
        b.play();
        b.fill(100); // 200 samples consumed (1 channel)
        assert!(b.progress() > 0.0);
    }

    #[test]
    fn test_audio_mixer_new() {
        let m = AudioMixer::new(AudioConfig::default());
        assert_eq!(m.master_volume, 1.0);
        assert!(!m.muted);
    }

    #[test]
    fn test_audio_mixer_add_sources() {
        let mut m = AudioMixer::new(AudioConfig::default());
        m.add_generator(ToneGenerator::new(Waveform::Sine, 440.0));
        m.add_buffer(AudioBuffer::new(44100, 2));
        assert_eq!(m.generators.len(), 1);
        assert_eq!(m.buffers.len(), 1);
    }

    #[test]
    fn test_audio_mixer_render() {
        let mut m = AudioMixer::new(AudioConfig::default());
        let mut g = ToneGenerator::new(Waveform::Sine, 440.0);
        g.start();
        m.add_generator(g);
        let s = m.render(50);
        assert_eq!(s.len(), 50 * 2);
    }

    #[test]
    fn test_audio_mixer_mute() {
        let mut m = AudioMixer::new(AudioConfig::default());
        let mut g = ToneGenerator::new(Waveform::Sine, 440.0);
        g.start();
        m.add_generator(g);
        m.toggle_mute();
        let s = m.render(50);
        assert!(s.iter().all(|&v| v == 0.0));
    }

    #[test]
    fn test_audio_mixer_master_volume() {
        let mut m = AudioMixer::new(AudioConfig::default());
        m.set_master_volume(0.0);
        let mut g = ToneGenerator::new(Waveform::Sine, 440.0);
        g.start();
        m.add_generator(g);
        let s = m.render(50);
        assert!(s.iter().all(|&v| v == 0.0));
    }

    #[test]
    fn test_tone_square() {
        let mut g = ToneGenerator::new(Waveform::Square, 440.0);
        g.start();
        let s = g.generate(1000);
        // Square wave should have values close to ±amplitude
        assert!(s.iter().any(|&v| v > 0.4));
    }
}
