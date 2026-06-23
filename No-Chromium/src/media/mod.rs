pub mod image_support;
pub mod image_manager;
pub mod video;
pub mod audio;
pub mod mse;
pub mod video_codecs;
pub mod frame;
pub mod video_texture;
pub mod audio_playback;
pub mod pipeline;
pub mod mp4;
pub mod hls;
pub mod dash;
pub mod webvtt;
pub mod player_ui;
pub mod http_range;
pub mod yuv_gpu;
pub mod url_extractor;
pub mod image_loader;       // FASE A2: Image subresource loader
pub mod pre_cached_assets;

pub use image_support::{
    get_cached_image, draw_image_to_buffer, fetch_image, take_image_dirty,
    get_image_stats, clear_cache, draw_placeholder, ImageFormat, LoadStatus,
};
pub use video::{VideoElement, VideoSource, VideoState, VideoManager, VideoEvent, VideoControls};
pub use audio::{AudioElement, AudioSource, AudioState, AudioManager};
pub use mse::{MediaSource, MediaSourceManager, SourceBuffer, SourceState, AppendBufferState, AppendMode, TimeRange, QualityLevel};
pub use video_codecs::{H264Decoder, H264StreamParser, NalUnit, NalUnitType, SpsInfo, PpsInfo, FrameType};
pub use frame::{YuvFrame, RgbConverter, FrameDecimator, ColorSpace};
pub use video_texture::{VideoTexture, VideoTextureManager, VideoTextureFormat, VideoTextureDescriptor, TextureUsage};
pub use audio_playback::{Waveform, SampleFormat, AudioConfig, ToneGenerator, AudioBuffer, AudioMixer};
pub use pipeline::{MediaPipeline, PipelineState, PipelineStats, DecodedFrame};
pub use mp4::{Mp4Parser, Mp4Info, Mp4Box, Mp4Builder, BoxHeader};
pub use hls::{HlsPlaylist, StreamVariant, MediaSegment, PlaylistType, EncryptionKey};
pub use dash::{DashMpd, DashProfile, DashPeriod, DashAdaptationSet, DashRepresentation, SegmentTemplate};
pub use webvtt::{WebVtt, VttCue, VttAlign, VttStyle};
pub use player_ui::{PlayerControl, PlayerUiConfig, PlayerLayout, PlayerControls, format_player_time, format_seconds};
pub use http_range::{ByteRange, RangeRequest, RangeDownloader};
pub use yuv_gpu::{YuvGpuConverter, YuvGpuConfig, ShaderColorSpace, YUV_TO_RGB_SHADER};
pub use url_extractor::{VideoUrlExtractor, ExtractedVideoSource, VideoSourceType, ExtractionResult};
