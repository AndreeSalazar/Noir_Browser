use std::sync::OnceLock;
use rodio::{OutputStream, OutputStreamHandle, Sink};
use rodio::buffer::SamplesBuffer;

thread_local! {
    static GLOBAL_OUTPUT_STREAM: std::cell::RefCell<Option<OutputStream>> = std::cell::RefCell::new(None);
}

static AUDIO_HANDLE: OnceLock<OutputStreamHandle> = OnceLock::new();

pub fn init_audio_device() -> Option<OutputStreamHandle> {
    let mut handle_out = None;
    GLOBAL_OUTPUT_STREAM.with(|stream| {
        let mut opt = stream.borrow_mut();
        if opt.is_none() {
            if let Ok((s, h)) = OutputStream::try_default() {
                *opt = Some(s);
                handle_out = Some(h);
            }
        }
    });
    handle_out
}

pub fn set_audio_handle(handle: OutputStreamHandle) {
    let _ = AUDIO_HANDLE.set(handle);
}

pub fn get_audio_handle() -> Option<&'static OutputStreamHandle> {
    AUDIO_HANDLE.get()
}

pub struct AudioBackend {
    sink: Sink,
}

impl AudioBackend {
    pub fn new() -> Option<Self> {
        let handle = get_audio_handle()?;
        let sink = Sink::try_new(handle).ok()?;
        Some(AudioBackend { sink })
    }

    pub fn play_samples(&self, samples: Vec<f32>, sample_rate: u32, channels: u16) {
        let buffer = SamplesBuffer::new(channels, sample_rate, samples);
        self.sink.append(buffer);
    }

    pub fn stop(&self) {
        self.sink.stop();
    }

    pub fn is_empty(&self) -> bool {
        self.sink.empty()
    }
}
