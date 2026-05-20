use std::sync::{Arc, OnceLock, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
use std::io::Cursor;
use symphonia::core::codecs::{Decoder, DecoderOptions};
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::{FormatOptions, FormatReader};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::core::audio::{AudioBufferRef, Signal};

use crate::app::{BrowserEvent, get_event_proxy};
use crate::media::audio::AudioBackend;
use crate::media::image_manager::{LoadedImage, get_image_cache};

pub struct VideoPlayer {
    playing: Arc<AtomicBool>,
    audio_backend: Option<Arc<AudioBackend>>,
}

fn decode_audio(
    reader: &mut dyn FormatReader,
    decoder: &mut dyn Decoder,
    audio_backend: &AudioBackend,
    playing: &AtomicBool,
) -> Result<(), SymphoniaError> {
    loop {
        if !playing.load(Ordering::Relaxed) {
            break;
        }

        let packet = match reader.next_packet() {
            Ok(packet) => packet,
            Err(e) => return Err(e),
        };

        let decoded = match decoder.decode(&packet) {
            Ok(decoded) => decoded,
            Err(SymphoniaError::IoError(ref e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                break;
            }
            Err(e) => return Err(e),
        };

        let sample_rate = decoded.spec().rate;
        let channels = decoded.spec().channels.count() as u16;
        let mut samples = Vec::new();
        
        match decoded {
            AudioBufferRef::F32(buf) => {
                let num_planes = buf.planes().planes().len();
                let num_frames = buf.frames();
                if num_planes > 0 {
                    for frame_idx in 0..num_frames {
                        for plane_idx in 0..num_planes {
                            samples.push(buf.planes().planes()[plane_idx][frame_idx]);
                        }
                    }
                }
            }
            AudioBufferRef::S16(buf) => {
                let num_planes = buf.planes().planes().len();
                let num_frames = buf.frames();
                if num_planes > 0 {
                    for frame_idx in 0..num_frames {
                        for plane_idx in 0..num_planes {
                            let sample = buf.planes().planes()[plane_idx][frame_idx] as f32 / 32768.0;
                            samples.push(sample);
                        }
                    }
                }
            }
            _ => {}
        }
        
        if !samples.is_empty() {
            audio_backend.play_samples(samples, sample_rate, channels);
            while !audio_backend.is_empty() && playing.load(Ordering::Relaxed) {
                thread::sleep(Duration::from_millis(10));
            }
        }
    }
    Ok(())
}

fn generate_visual_frame(
    width: u32,
    height: u32,
    frame_index: u64,
    _title: &str,
    _video_id: &str,
) -> LoadedImage {
    let mut rgba = vec![0; (width * height * 4) as usize];
    
    // Draw background gradient
    for y in 0..height {
        let idx = ((y * width) * 4) as usize;
        let r = ((0.08 + 0.12 * (frame_index as f32 * 0.04).sin()) * 255.0) as u8;
        let g = (0.04 * 255.0) as u8;
        let b = ((0.16 + 0.12 * (frame_index as f32 * 0.04 + 2.0).cos()) * 255.0) as u8;
        
        for x in 0..width {
            let pixel_idx = idx + (x * 4) as usize;
            rgba[pixel_idx] = r;
            rgba[pixel_idx + 1] = g;
            rgba[pixel_idx + 2] = b;
            rgba[pixel_idx + 3] = 255;
        }
    }
    
    // Draw visual spectrum analyzer bars
    let num_bars = 48;
    let bar_width = width / num_bars;
    let base_y = height - 40;
    
    for bar in 0..num_bars {
        let phase = frame_index as f32 * 0.12 + bar as f32 * 0.35;
        let val = (phase.sin().abs() * 0.65 + (phase * 2.1).cos().abs() * 0.35) * 0.7;
        let bar_height = ((height as f32 - 100.0) * val) as u32;
        
        let start_x = bar * bar_width;
        let end_x = (start_x + bar_width - 2).min(width);
        let start_y = base_y.saturating_sub(bar_height);
        
        for y in start_y..base_y {
            let row_idx = ((y * width) * 4) as usize;
            let t_bar = bar as f32 / num_bars as f32;
            let r = (t_bar * 255.0) as u8;
            let g = (210.0 * (1.0 - t_bar * 0.5)) as u8;
            let b = ((1.0 - t_bar) * 255.0 + t_bar * 128.0) as u8;
            
            for x in start_x..end_x {
                let pixel_idx = row_idx + (x * 4) as usize;
                rgba[pixel_idx] = r;
                rgba[pixel_idx + 1] = g;
                rgba[pixel_idx + 2] = b;
                rgba[pixel_idx + 3] = 255;
            }
        }
    }
    
    LoadedImage {
        width,
        height,
        rgba,
    }
}

impl VideoPlayer {
    pub fn new() -> Self {
        VideoPlayer {
            playing: Arc::new(AtomicBool::new(false)),
            audio_backend: AudioBackend::new().map(Arc::new),
        }
    }

    pub fn start(&self, url: String, title: String, video_id: String) {
        if self.playing.swap(true, Ordering::Relaxed) {
            return;
        }

        let playing = self.playing.clone();
        let audio_backend = self.audio_backend.clone();
        
        thread::spawn(move || {
            println!("[Player Thread] Starting video stream: {}", url);
            
            // 1. Launch visualizer frame generation loop (~30 FPS)
            let playing_visual = playing.clone();
            let vid = video_id.clone();
            let t = title.clone();
            thread::spawn(move || {
                let mut frame_index = 0;
                while playing_visual.load(Ordering::Relaxed) {
                    let frame = generate_visual_frame(640, 360, frame_index, &t, &vid);
                    {
                        let mut cache = get_image_cache().lock().unwrap();
                        cache.insert("video://stream".to_string(), Arc::new(frame));
                    }
                    
                    if let Some(proxy) = get_event_proxy() {
                        let _ = proxy.send_event(BrowserEvent::ImageLoaded {
                            url: "video://stream".to_string(),
                        });
                    }
                    
                    frame_index += 1;
                    thread::sleep(Duration::from_millis(33)); // ~30 FPS
                }
            });

            // 2. Play audio stream if device backend is available
            if let Some(backend) = audio_backend {
                let client = reqwest::blocking::Client::builder()
                    .timeout(Duration::from_secs(30))
                    .build()
                    .unwrap_or_default();
                
                if let Ok(resp) = client.get(&url).send() {
                    if resp.status().is_success() {
                        if let Ok(bytes) = resp.bytes() {
                            let cursor = Cursor::new(bytes.to_vec());
                            let mss = MediaSourceStream::new(Box::new(cursor), Default::default());
                            let mut hint = Hint::new();
                            hint.with_extension("mp4");
                            
                            let format_opts = FormatOptions::default();
                            let metadata_opts = MetadataOptions::default();
                            
                            if let Ok(probed) = symphonia::default::get_probe().format(&hint, mss, &format_opts, &metadata_opts) {
                                let track = probed.format.tracks().iter().find(|t| t.codec_params.sample_rate.is_some());
                                if let Some(track) = track {
                                    let decoder_opts = DecoderOptions::default();
                                    if let Ok(mut decoder) = symphonia::default::get_codecs().make(&track.codec_params, &decoder_opts) {
                                        let mut reader = probed.format;
                                        let _ = decode_audio(&mut *reader, &mut *decoder, &backend, &playing);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });
    }

    pub fn stop(&self) {
        self.playing.store(false, Ordering::Relaxed);
        if let Some(ref backend) = self.audio_backend {
            backend.stop();
        }
    }

    pub fn is_playing(&self) -> bool {
        self.playing.load(Ordering::Relaxed)
    }
}

static ACTIVE_PLAYER: OnceLock<Mutex<Option<VideoPlayer>>> = OnceLock::new();

pub fn get_active_player() -> &'static Mutex<Option<VideoPlayer>> {
    ACTIVE_PLAYER.get_or_init(|| Mutex::new(None))
}

pub fn play_stream(url: String, title: String, video_id: String) {
    let mut player_opt = get_active_player().lock().unwrap();
    if let Some(ref old_player) = *player_opt {
        old_player.stop();
    }
    
    let player = VideoPlayer::new();
    player.start(url, title, video_id);
    *player_opt = Some(player);
}

pub fn stop_active_playback() {
    let mut player_opt = get_active_player().lock().unwrap();
    if let Some(ref old_player) = *player_opt {
        old_player.stop();
    }
    *player_opt = None;
}

pub fn is_any_video_playing() -> bool {
    let player_opt = get_active_player().lock().unwrap();
    if let Some(ref p) = *player_opt {
        p.is_playing()
    } else {
        false
    }
}

