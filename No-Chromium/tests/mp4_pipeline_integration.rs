//! Integration test: MP4 → MediaPipeline → decoded frames
//!
//! Valida el pipeline completo end-to-end con un MP4 sintético.

use no_chromium::media::mp4::Mp4Parser;
use no_chromium::media::pipeline::MediaPipeline;

fn make_mp4_with_h264() -> Vec<u8> {
    let mut mp4 = Vec::new();
    // ftyp box
    let ftyp_data = b"isom\x00\x00\x02\x00isomiso2avc1mp41";
    mp4.extend_from_slice(&((8 + ftyp_data.len()) as u32).to_be_bytes());
    mp4.extend_from_slice(b"ftyp");
    mp4.extend_from_slice(ftyp_data);
    // mdat con H.264: SPS + PPS + IDR slice
    let mut mdat = Vec::new();
    mdat.extend_from_slice(&[0x00, 0x00, 0x00, 0x01, 0x67, 0x42, 0x00, 0x1E, 0xAB, 0xCD]);
    mdat.extend_from_slice(&[0x00, 0x00, 0x00, 0x01, 0x68, 0xCE, 0x38, 0x80]);
    mdat.extend_from_slice(&[0x00, 0x00, 0x00, 0x01, 0x65, 0x88, 0x80, 0x40]);
    mp4.extend_from_slice(&((8 + mdat.len()) as u32).to_be_bytes());
    mp4.extend_from_slice(b"mdat");
    mp4.extend_from_slice(&mdat);
    mp4
}

#[test]
fn test_mp4_to_pipeline_integration() {
    let mp4_data = make_mp4_with_h264();
    let mut parser = Mp4Parser::new();
    parser.parse(&mp4_data).unwrap();
    assert_eq!(parser.info.major_brand, "isom");

    let mut pipeline = MediaPipeline::new(1280, 720);
    pipeline.with_source(60_000);
    let frames = pipeline.feed_encoded(&parser.mdat_data);
    assert!(frames > 0, "expected at least 1 frame, got {}", frames);

    let frame = pipeline.next_frame();
    assert!(frame.is_some());
    let f = frame.unwrap();
    assert!(f.is_keyframe);
    assert!(!f.rgb_data.is_empty());
}

#[test]
fn test_mp4_pipeline_with_size_16x16() {
    let mp4_data = make_mp4_with_h264();
    let mut parser = Mp4Parser::new();
    parser.parse(&mp4_data).unwrap();

    let mut pipeline = MediaPipeline::new(16, 16);
    let _ = pipeline.feed_encoded(&parser.mdat_data);
    let frame = pipeline.next_frame();
    assert!(frame.is_some());
    let f = frame.unwrap();
    // El SPS sintético no parsea width/height real, usa defaults
    // El rgb_data debe coincidir con width*height*4 del frame
    assert_eq!(f.rgb_data.len(), (f.width * f.height * 4) as usize);
}

#[test]
fn test_mp4_pipeline_play_pause() {
    let mp4_data = make_mp4_with_h264();
    let mut parser = Mp4Parser::new();
    parser.parse(&mp4_data).unwrap();

    let mut pipeline = MediaPipeline::new(64, 64);
    let _ = pipeline.feed_encoded(&parser.mdat_data);
    assert!(pipeline.pending_count() > 0);

    pipeline.play();
    assert_eq!(pipeline.stats.state, no_chromium::media::pipeline::PipelineState::Playing);
    pipeline.pause();
    assert_eq!(pipeline.stats.state, no_chromium::media::pipeline::PipelineState::Paused);
}

#[test]
fn test_mp4_pipeline_seek() {
    let mp4_data = make_mp4_with_h264();
    let mut parser = Mp4Parser::new();
    parser.parse(&mp4_data).unwrap();

    let mut pipeline = MediaPipeline::new(64, 64);
    for _ in 0..3 {
        pipeline.feed_test_chunk(0);
    }
    pipeline.seek(100);
    assert_eq!(pipeline.current_pts_ms, 100);
}

#[test]
fn test_mp4_parser_extracts_mdat() {
    let mp4_data = make_mp4_with_h264();
    let mut parser = Mp4Parser::new();
    parser.parse(&mp4_data).unwrap();
    assert!(!parser.mdat_data.is_empty());
    // First 4 bytes should be Annex B start code
    assert_eq!(parser.mdat_data[0], 0x00);
    assert_eq!(parser.mdat_data[1], 0x00);
    assert_eq!(parser.mdat_data[2], 0x00);
    assert_eq!(parser.mdat_data[3], 0x01);
}

#[test]
fn test_mp4_parser_brands() {
    let mp4_data = make_mp4_with_h264();
    let mut parser = Mp4Parser::new();
    parser.parse(&mp4_data).unwrap();
    assert!(parser.info.compatible_brands.contains(&"isom".to_string()));
    assert!(parser.info.compatible_brands.contains(&"iso2".to_string()));
    assert!(parser.info.compatible_brands.contains(&"avc1".to_string()));
    assert!(parser.info.compatible_brands.contains(&"mp41".to_string()));
}

#[test]
fn test_pipeline_multiple_frames() {
    let mut pipeline = MediaPipeline::new(32, 32);
    for i in 0..30 {
        pipeline.feed_test_chunk(i);
    }
    assert_eq!(pipeline.pending_count(), 30);
    // Pop all frames
    let mut count = 0;
    while pipeline.next_frame().is_some() {
        count += 1;
    }
    assert_eq!(count, 30);
}

#[test]
fn test_pipeline_texture_upload() {
    let mut pipeline = MediaPipeline::new(32, 32);
    pipeline.init_textures(2);
    pipeline.feed_test_chunk(0);
    let frame = pipeline.next_frame().unwrap();
    assert!(frame.texture_id > 0);
    assert!(pipeline.textures.count() == 2);
}

#[test]
fn test_pipeline_drop_frames() {
    let mut pipeline = MediaPipeline::new(32, 32);
    for i in 0..10 {
        pipeline.feed_test_chunk(i);
    }
    let initial_dropped = pipeline.dropped_frames;
    pipeline.skip_to_pts(200);
    assert!(pipeline.dropped_frames > initial_dropped);
}

#[test]
fn test_mp4_data_round_trip() {
    let mp4_data = make_mp4_with_h264();
    let mut parser = Mp4Parser::new();
    parser.parse(&mp4_data).unwrap();
    // Re-parse should give same info
    let mut parser2 = Mp4Parser::new();
    parser2.parse(&mp4_data).unwrap();
    assert_eq!(parser.info.major_brand, parser2.info.major_brand);
    assert_eq!(parser.mdat_data, parser2.mdat_data);
}

#[test]
fn test_mp4_pipeline_with_fps_calculation() {
    let mut pipeline = MediaPipeline::new(64, 64);
    for i in 0..60 {
        pipeline.feed_test_chunk(i);
    }
    pipeline.calculate_fps(2000); // 60 frames in 2 seconds
    assert!((pipeline.stats.current_fps - 30.0).abs() < 1.0);
}

#[test]
fn test_mp4_pipeline_reset() {
    let mut pipeline = MediaPipeline::new(64, 64);
    for i in 0..5 {
        pipeline.feed_test_chunk(i);
    }
    assert!(pipeline.pending_count() > 0);
    pipeline.reset();
    assert_eq!(pipeline.pending_count(), 0);
    assert_eq!(pipeline.stats.frames_decoded, 0);
}
