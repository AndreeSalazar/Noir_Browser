//! H.264 Codec - Decoder básico para video
//!
//! Implementa NAL unit parsing y frame extraction para H.264 (AVC).

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NalUnitType {
    Unspecified,
    SliceNonIdr,
    SliceIdr,
    Sei,
    Sps,
    Pps,
    AccessUnitDelimiter,
    EndOfSequence,
    EndOfStream,
    Filler,
}

impl NalUnitType {
    pub fn from_u8(v: u8) -> Self {
        match v & 0x1F {
            0 => Self::Unspecified,
            1 => Self::SliceNonIdr,
            5 => Self::SliceIdr,
            6 => Self::Sei,
            7 => Self::Sps,
            8 => Self::Pps,
            9 => Self::AccessUnitDelimiter,
            10 => Self::EndOfSequence,
            11 => Self::EndOfStream,
            12 => Self::Filler,
            _ => Self::Unspecified,
        }
    }
}

#[derive(Debug, Clone)]
pub struct NalUnit {
    pub nal_type: NalUnitType,
    pub ref_idc: u8,
    pub data: Vec<u8>,
    pub size: usize,
}

#[derive(Debug, Clone, Default)]
pub struct SpsInfo {
    pub profile_idc: u8,
    pub constraint_flags: u8,
    pub level_idc: u8,
    pub width: u32,
    pub height: u32,
    pub num_ref_frames: u32,
    pub poc_type: u32,
    pub max_num_ref_frames: u32,
    pub frame_mbs_only: bool,
}

#[derive(Debug, Clone, Default)]
pub struct PpsInfo {
    pub pps_id: u32,
    pub sps_id: u32,
    pub entropy_coding_mode: u8,
    pub weighted_prediction: bool,
    pub deblocking_filter: bool,
    pub pic_init_qp: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrameType {
    pub is_idr: bool,
    pub is_keyframe: bool,
    pub is_reference: bool,
    pub width: u32,
    pub height: u32,
    pub pts: i64, // Presentation timestamp
    pub dts: i64, // Decode timestamp
}

pub struct H264Decoder {
    pub sps: Option<SpsInfo>,
    pub pps: Option<PpsInfo>,
    pub frames_decoded: u32,
    pub keyframes: u32,
    pub bytes_processed: u64,
    pub pts: i64,
    pub dts: i64,
    pub width: u32,
    pub height: u32,
}

impl H264Decoder {
    pub fn new() -> Self {
        Self {
            sps: None,
            pps: None,
            frames_decoded: 0,
            keyframes: 0,
            bytes_processed: 0,
            pts: 0,
            dts: 0,
            width: 0,
            height: 0,
        }
    }

    /// Encuentra NAL units en Annex B stream (con start codes 0x00 0x00 0x01 o 0x00 0x00 0x00 0x01)
    pub fn find_nal_units(data: &[u8]) -> Vec<NalUnit> {
        let mut nals = Vec::new();
        let mut i = 0;
        let mut start = None;
        let mut last_start = 0;
        let mut prefix_len = 0;
        while i < data.len() {
            if i + 2 < data.len() && data[i] == 0 && data[i+1] == 0 && data[i+2] == 1 {
                if let Some(s) = start {
                    let size = i - s;
                    if size > 0 {
                        let nal_data = &data[s+prefix_len..i];
                        let nal_type = NalUnitType::from_u8(nal_data[0]);
                        let ref_idc = (nal_data[0] >> 5) & 0x03;
                        nals.push(NalUnit {
                            nal_type,
                            ref_idc,
                            data: nal_data.to_vec(),
                            size,
                        });
                    }
                }
                start = Some(i);
                prefix_len = 3;
                i += 3;
            } else if i + 3 < data.len() && data[i] == 0 && data[i+1] == 0 && data[i+2] == 0 && data[i+3] == 1 {
                if let Some(s) = start {
                    let size = i - s;
                    if size > 0 {
                        let nal_data = &data[s+prefix_len..i];
                        let nal_type = NalUnitType::from_u8(nal_data[0]);
                        let ref_idc = (nal_data[0] >> 5) & 0x03;
                        nals.push(NalUnit {
                            nal_type,
                            ref_idc,
                            data: nal_data.to_vec(),
                            size,
                        });
                    }
                }
                start = Some(i);
                prefix_len = 4;
                i += 4;
            } else {
                i += 1;
            }
            last_start = i;
        }
        if let Some(s) = start {
            let size = data.len() - s;
            if size > prefix_len {
                let nal_data = &data[s+prefix_len..];
                let nal_type = NalUnitType::from_u8(nal_data[0]);
                let ref_idc = (nal_data[0] >> 5) & 0x03;
                nals.push(NalUnit {
                    nal_type,
                    ref_idc,
                    data: nal_data.to_vec(),
                    size,
                });
            }
        }
        let _ = last_start; // suppress warning
        nals
    }

    /// Procesa un NAL unit
    pub fn process_nal(&mut self, nal: &NalUnit) -> Option<FrameType> {
        self.bytes_processed += nal.size as u64;
        match nal.nal_type {
            NalUnitType::Sps => {
                self.sps = Some(Self::parse_sps(&nal.data));
                if let Some(sps) = &self.sps {
                    self.width = sps.width;
                    self.height = sps.height;
                }
                None
            }
            NalUnitType::Pps => {
                self.pps = Some(Self::parse_pps(&nal.data));
                None
            }
            NalUnitType::SliceIdr => {
                self.frames_decoded += 1;
                self.keyframes += 1;
                self.dts += 1;
                self.pts = self.dts;
                Some(FrameType {
                    is_idr: true,
                    is_keyframe: true,
                    is_reference: true,
                    width: self.width,
                    height: self.height,
                    pts: self.pts,
                    dts: self.dts,
                })
            }
            NalUnitType::SliceNonIdr => {
                self.frames_decoded += 1;
                self.dts += 1;
                self.pts = self.dts;
                Some(FrameType {
                    is_idr: false,
                    is_keyframe: false,
                    is_reference: nal.ref_idc > 0,
                    width: self.width,
                    height: self.height,
                    pts: self.pts,
                    dts: self.dts,
                })
            }
            _ => None,
        }
    }

    /// Parse básico de SPS (sequence parameter set)
    pub fn parse_sps(data: &[u8]) -> SpsInfo {
        let mut sps = SpsInfo::default();
        if data.is_empty() { return sps; }
        sps.profile_idc = data.get(1).copied().unwrap_or(0);
        sps.constraint_flags = data.get(2).copied().unwrap_or(0);
        sps.level_idc = data.get(3).copied().unwrap_or(0);
        // Width y height están más adelante en el bitstream
        // Por ahora usamos defaults desde profile
        sps.width = 1280;
        sps.height = 720;
        sps
    }

    pub fn parse_pps(data: &[u8]) -> PpsInfo {
        let mut pps = PpsInfo::default();
        if data.is_empty() { return pps; }
        pps.pps_id = (data.get(1).copied().unwrap_or(0) & 0xFF) as u32;
        pps.sps_id = 0;
        pps
    }

    pub fn has_config(&self) -> bool {
        self.sps.is_some() && self.pps.is_some()
    }

    pub fn aspect_ratio(&self) -> f32 {
        if self.height == 0 { return 16.0 / 9.0; }
        self.width as f32 / self.height as f32
    }

    /// Determina si necesita keyframe
    pub fn needs_keyframe(&self) -> bool {
        self.sps.is_none() || self.keyframes == 0
    }
}

impl Default for H264Decoder {
    fn default() -> Self { Self::new() }
}

pub struct H264StreamParser {
    pub decoder: H264Decoder,
    pub keyframe_interval: u32,
    pub last_keyframe_at: u32,
}

impl H264StreamParser {
    pub fn new() -> Self {
        Self {
            decoder: H264Decoder::new(),
            keyframe_interval: 60,
            last_keyframe_at: 0,
        }
    }

    pub fn with_keyframe_interval(mut self, interval: u32) -> Self {
        self.keyframe_interval = interval;
        self
    }

    pub fn feed(&mut self, data: &[u8]) -> Vec<FrameType> {
        let mut frames = Vec::new();
        let nals = H264Decoder::find_nal_units(data);
        for nal in &nals {
            if let Some(frame) = self.decoder.process_nal(nal) {
                if frame.is_keyframe {
                    self.last_keyframe_at = self.decoder.frames_decoded;
                }
                frames.push(frame);
            }
        }
        frames
    }

    pub fn decoder(&self) -> &H264Decoder {
        &self.decoder
    }

    pub fn frames_decoded(&self) -> u32 {
        self.decoder.frames_decoded
    }

    pub fn keyframes(&self) -> u32 {
        self.decoder.keyframes
    }
}

impl Default for H264StreamParser {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_nal(nal_type: u8) -> Vec<u8> {
        // Header byte: forbidden_zero_bit(1) + nal_ref_idc(2) + nal_unit_type(5)
        // Use ref_idc=3, so header = 0b0110_NNNNN where N is nal_type
        let header = 0x60 | (nal_type & 0x1F);
        vec![0x00, 0x00, 0x00, 0x01, header, 0xFF, 0xFF, 0xFF, 0xFF]
    }

    #[test]
    fn test_nal_type_from_u8() {
        assert_eq!(NalUnitType::from_u8(5), NalUnitType::SliceIdr);
        assert_eq!(NalUnitType::from_u8(7), NalUnitType::Sps);
        assert_eq!(NalUnitType::from_u8(8), NalUnitType::Pps);
        assert_eq!(NalUnitType::from_u8(1), NalUnitType::SliceNonIdr);
    }

    #[test]
    fn test_find_nal_units_single() {
        let data = make_nal(5);
        let nals = H264Decoder::find_nal_units(&data);
        assert_eq!(nals.len(), 1);
        assert_eq!(nals[0].nal_type, NalUnitType::SliceIdr);
    }

    #[test]
    fn test_find_nal_units_multiple() {
        let mut data = Vec::new();
        data.extend(make_nal(7));  // SPS
        data.extend(make_nal(8));  // PPS
        data.extend(make_nal(5));  // IDR
        let nals = H264Decoder::find_nal_units(&data);
        assert_eq!(nals.len(), 3);
        assert_eq!(nals[0].nal_type, NalUnitType::Sps);
        assert_eq!(nals[1].nal_type, NalUnitType::Pps);
        assert_eq!(nals[2].nal_type, NalUnitType::SliceIdr);
    }

    #[test]
    fn test_find_nal_units_3byte_prefix() {
        let data = vec![0x00, 0x00, 0x01, 0x67, 0xFF, 0xFF, 0x00, 0x00, 0x01, 0x68, 0xFF];
        let nals = H264Decoder::find_nal_units(&data);
        assert_eq!(nals.len(), 2);
        assert_eq!(nals[0].nal_type, NalUnitType::Sps);
        assert_eq!(nals[1].nal_type, NalUnitType::Pps);
    }

    #[test]
    fn test_decoder_new() {
        let d = H264Decoder::new();
        assert_eq!(d.frames_decoded, 0);
        assert!(!d.has_config());
    }

    #[test]
    fn test_decoder_process_sps() {
        let mut d = H264Decoder::new();
        let nals = H264Decoder::find_nal_units(&make_nal(7));
        d.process_nal(&nals[0]);
        assert!(d.sps.is_some());
        assert_eq!(d.width, 1280);
        assert_eq!(d.height, 720);
    }

    #[test]
    fn test_decoder_process_idr() {
        let mut d = H264Decoder::new();
        let nals = H264Decoder::find_nal_units(&make_nal(5));
        let frame = d.process_nal(&nals[0]).unwrap();
        assert!(frame.is_keyframe);
        assert!(frame.is_idr);
        assert_eq!(d.frames_decoded, 1);
        assert_eq!(d.keyframes, 1);
    }

    #[test]
    fn test_decoder_process_non_idr() {
        let mut d = H264Decoder::new();
        let nals = H264Decoder::find_nal_units(&make_nal(1));
        let frame = d.process_nal(&nals[0]).unwrap();
        assert!(!frame.is_keyframe);
        assert!(!frame.is_idr);
    }

    #[test]
    fn test_decoder_has_config() {
        let mut d = H264Decoder::new();
        let sps = H264Decoder::find_nal_units(&make_nal(7));
        let pps = H264Decoder::find_nal_units(&make_nal(8));
        d.process_nal(&sps[0]);
        assert!(!d.has_config());
        d.process_nal(&pps[0]);
        assert!(d.has_config());
    }

    #[test]
    fn test_decoder_aspect_ratio() {
        let mut d = H264Decoder::new();
        d.width = 1920;
        d.height = 1080;
        assert_eq!(d.aspect_ratio(), 16.0/9.0);
    }

    #[test]
    fn test_decoder_needs_keyframe() {
        let d = H264Decoder::new();
        assert!(d.needs_keyframe());
    }

    #[test]
    fn test_parser_new() {
        let p = H264StreamParser::new();
        assert_eq!(p.frames_decoded(), 0);
    }

    #[test]
    fn test_parser_feed() {
        let mut p = H264StreamParser::new();
        let mut data = Vec::new();
        data.extend(make_nal(7)); // SPS
        data.extend(make_nal(8)); // PPS
        data.extend(make_nal(5)); // IDR
        data.extend(make_nal(1)); // non-IDR
        let frames = p.feed(&data);
        assert_eq!(frames.len(), 2);
        assert!(frames[0].is_keyframe);
        assert!(!frames[1].is_keyframe);
    }

    #[test]
    fn test_parser_keyframe_interval() {
        let p = H264StreamParser::new().with_keyframe_interval(30);
        assert_eq!(p.keyframe_interval, 30);
    }

    #[test]
    fn test_ref_idc() {
        let nals = H264Decoder::find_nal_units(&make_nal(5));
        // Header byte 0x65 = 0110 0101 = ref_idc=3, nal_type=5
        assert!(nals[0].ref_idc > 0);
    }

    #[test]
    fn test_bytes_processed() {
        let mut d = H264Decoder::new();
        let nals = H264Decoder::find_nal_units(&make_nal(5));
        d.process_nal(&nals[0]);
        assert!(d.bytes_processed > 0);
    }

    #[test]
    fn test_parse_sps_minimal() {
        let sps = H264Decoder::parse_sps(&[0x67, 0x42, 0x00, 0x1E, 0xAB, 0xCD]);
        assert_eq!(sps.profile_idc, 0x42);
        assert_eq!(sps.level_idc, 0x1E);
    }
}
