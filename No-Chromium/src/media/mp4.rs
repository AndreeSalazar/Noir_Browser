//! MP4 Box Parser - Parsing de containers MP4
//!
//! Implementa parsing de boxes ISO BMFF (ftyp, moov, mdat, mvhd, trak, mdia, etc).

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct BoxHeader {
    pub box_type: String,
    pub size: u64,
    pub offset: u64,
}

#[derive(Debug, Clone)]
pub struct Mp4Box {
    pub header: BoxHeader,
    pub data: Vec<u8>,
    pub children: Vec<Mp4Box>,
}

impl Mp4Box {
    pub fn new(header: BoxHeader) -> Self {
        Self {
            header,
            data: Vec::new(),
            children: Vec::new(),
        }
    }

    pub fn find_child(&self, box_type: &str) -> Option<&Mp4Box> {
        self.children.iter().find(|c| c.header.box_type == box_type)
    }

    pub fn find_children(&self, box_type: &str) -> Vec<&Mp4Box> {
        self.children.iter().filter(|c| c.header.box_type == box_type).collect()
    }
}

#[derive(Debug, Clone, Default)]
pub struct Mp4Info {
    pub major_brand: String,
    pub minor_version: u32,
    pub compatible_brands: Vec<String>,
    pub duration_ms: u64,
    pub timescale: u32,
    pub width: u32,
    pub height: u32,
    pub track_count: u32,
    pub has_video: bool,
    pub has_audio: bool,
    pub video_codec: String,
    pub audio_codec: String,
    pub creation_time: u64,
    pub modification_time: u64,
}

pub struct Mp4Parser {
    pub info: Mp4Info,
    pub mdat_data: Vec<u8>,
    pub mdat_offset: u64,
    pub avc_config: Option<Vec<u8>>,
}

impl Mp4Parser {
    pub fn new() -> Self {
        Self {
            info: Mp4Info::default(),
            mdat_data: Vec::new(),
            mdat_offset: 0,
            avc_config: None,
        }
    }

    /// Parsea un MP4 completo desde bytes
    pub fn parse(&mut self, data: &[u8]) -> Result<(), String> {
        let root = Self::parse_boxes(data, 0, data.len() as u64)?;
        for child in &root.children {
            match child.header.box_type.as_str() {
                "ftyp" => self.parse_ftyp(&child.data)?,
                "moov" => self.parse_moov(child)?,
                "mdat" => {
                    self.mdat_data = child.data.clone();
                    self.mdat_offset = child.header.offset + 8; // skip box header
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Parsea boxes recursivamente
    pub fn parse_boxes(data: &[u8], start: usize, end: u64) -> Result<Mp4Box, String> {
        let mut root = Mp4Box::new(BoxHeader {
            box_type: "root".to_string(),
            size: (end - start as u64),
            offset: start as u64,
        });
        let mut pos = start;
        while pos < end as usize {
            if pos + 8 > data.len() {
                break;
            }
            let size = u32::from_be_bytes([data[pos], data[pos+1], data[pos+2], data[pos+3]]) as u64;
            let box_type_bytes = &data[pos+4..pos+8];
            let box_type = String::from_utf8_lossy(box_type_bytes).to_string();
            let header_size: u64 = if size == 1 {
                if pos + 16 > data.len() { return Err("Truncated box".to_string()); }
                u64::from_be_bytes([
                    data[pos+8], data[pos+9], data[pos+10], data[pos+11],
                    data[pos+12], data[pos+13], data[pos+14], data[pos+15]
                ])
            } else {
                size
            };
            if header_size < 8 || pos as u64 + header_size > end {
                break;
            }
            let header = BoxHeader {
                box_type: box_type.clone(),
                size: header_size,
                offset: pos as u64,
            };
            let mut box_obj = Mp4Box::new(header);
            let data_start = pos + 8;
            let data_end = pos + header_size as usize;
            if box_type == "uuid" {
                box_obj.data = data[data_start..data_start + 16.min(data.len() - data_start)].to_vec();
            } else if matches!(box_type.as_str(), "moov" | "trak" | "mdia" | "minf" | "stbl" | "edts" | "udta" | "meta" | "dinf") {
                if data_start < data.len() && data_end <= data.len() {
                    box_obj = Self::parse_boxes(data, data_start, data_end as u64)?;
                    box_obj.header = BoxHeader {
                        box_type,
                        size: header_size,
                        offset: pos as u64,
                    };
                }
            } else {
                if data_start < data.len() && data_end <= data.len() {
                    box_obj.data = data[data_start..data_end].to_vec();
                }
            }
            root.children.push(box_obj);
            pos += header_size as usize;
        }
        Ok(root)
    }

    fn parse_ftyp(&mut self, data: &[u8]) -> Result<(), String> {
        if data.len() < 8 { return Err("ftyp too short".to_string()); }
        self.info.major_brand = String::from_utf8_lossy(&data[0..4]).to_string();
        self.info.minor_version = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
        let mut pos = 8;
        while pos + 4 <= data.len() {
            let brand = String::from_utf8_lossy(&data[pos..pos+4]).to_string();
            self.info.compatible_brands.push(brand);
            pos += 4;
        }
        Ok(())
    }

    fn parse_moov(&mut self, moov: &Mp4Box) -> Result<(), String> {
        if let Some(mvhd) = moov.find_child("mvhd") {
            self.parse_mvhd(&mvhd.data)?;
        }
        for trak in moov.find_children("trak") {
            self.parse_trak(trak)?;
        }
        Ok(())
    }

    fn parse_mvhd(&mut self, data: &[u8]) -> Result<(), String> {
        if data.len() < 4 { return Err("mvhd too short".to_string()); }
        let version = data[0];
        let (timescale_offset, duration_offset) = if version == 1 {
            (20, 24)
        } else {
            (12, 16)
        };
        if data.len() < duration_offset + 8 { return Ok(()); }
        self.info.timescale = u32::from_be_bytes([
            data[timescale_offset], data[timescale_offset+1],
            data[timescale_offset+2], data[timescale_offset+3]
        ]);
        let duration = if version == 1 {
            u64::from_be_bytes([
                data[duration_offset], data[duration_offset+1], data[duration_offset+2],
                data[duration_offset+3], data[duration_offset+4], data[duration_offset+5],
                data[duration_offset+6], data[duration_offset+7]
            ])
        } else {
            u32::from_be_bytes([
                data[duration_offset], data[duration_offset+1],
                data[duration_offset+2], data[duration_offset+3]
            ]) as u64
        };
        self.info.duration_ms = if self.info.timescale > 0 {
            (duration * 1000) / self.info.timescale as u64
        } else {
            0
        };
        Ok(())
    }

    fn parse_trak(&mut self, trak: &Mp4Box) -> Result<(), String> {
        self.info.track_count += 1;
        let mut is_video = false;
        let mut is_audio = false;
        // Buscar hdlr en cualquier nivel (porque trak > mdia > hdlr está anidado)
        let hdlr_data = Self::find_box_data(trak, "hdlr");
        if let Some(data) = hdlr_data {
            if data.len() >= 12 {
                let handler = String::from_utf8_lossy(&data[8..12]).to_string();
                if handler == "vide" { is_video = true; }
                if handler == "soun" { is_audio = true; }
            }
        }
        // Buscar stsd recursivamente
        if let Some(stsd_data) = Self::find_box_data(trak, "stsd") {
            eprintln!("DEBUG stsd_data len={}", stsd_data.len());
            // stsd: version(1) + flags(3) + entry_count(4) + entries
            // entries son sub-boxes: 4 bytes size + 4 bytes type (codec)
            if stsd_data.len() >= 16 {
                let codec = String::from_utf8_lossy(&stsd_data[12..16]).to_string();
                eprintln!("DEBUG codec: '{}'", codec);
                if is_video {
                    self.info.video_codec = codec.to_string();
                    self.info.has_video = true;
                } else if is_audio {
                    self.info.audio_codec = codec.to_string();
                    self.info.has_audio = true;
                }
            }
        }
        Ok(())
    }

    /// Busca un box por tipo en el árbol recursivamente
    fn find_box_data<'a>(box_obj: &'a Mp4Box, box_type: &str) -> Option<&'a Vec<u8>> {
        if box_obj.header.box_type == box_type {
            return Some(&box_obj.data);
        }
        for child in &box_obj.children {
            if let Some(data) = Self::find_box_data(child, box_type) {
                return Some(data);
            }
        }
        None
    }

    pub fn is_valid_mp4(&self) -> bool {
        !self.info.major_brand.is_empty() && self.info.track_count > 0
    }
}

impl Default for Mp4Parser {
    fn default() -> Self { Self::new() }
}

pub struct Mp4Builder {
    pub boxes: HashMap<String, Mp4Box>,
}

impl Mp4Builder {
    pub fn new() -> Self {
        Self { boxes: HashMap::new() }
    }

    pub fn add_box(&mut self, box_type: &str, data: Vec<u8>) {
        let header = BoxHeader {
            box_type: box_type.to_string(),
            size: 8 + data.len() as u64,
            offset: 0,
        };
        let mut b = Mp4Box::new(header);
        b.data = data;
        self.boxes.insert(box_type.to_string(), b);
    }

    /// Construye un MP4 mínimo con ftyp + mdat
    pub fn build_minimal() -> Vec<u8> {
        let mut out = Vec::new();
        // ftyp
        out.extend_from_slice(&20u32.to_be_bytes());
        out.extend_from_slice(b"ftyp");
        out.extend_from_slice(b"isom");
        out.extend_from_slice(&0u32.to_be_bytes());
        out.extend_from_slice(b"isomiso2avc1mp41");
        // mdat
        out.extend_from_slice(&8u32.to_be_bytes());
        out.extend_from_slice(b"mdat");
        out
    }
}

impl Default for Mp4Builder {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_box(box_type: &str, data: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&((8 + data.len()) as u32).to_be_bytes());
        out.extend_from_slice(box_type.as_bytes());
        out.extend_from_slice(data);
        out
    }

    #[test]
    fn test_parser_new() {
        let p = Mp4Parser::new();
        assert_eq!(p.info.track_count, 0);
    }

    #[test]
    fn test_box_header() {
        let h = BoxHeader { box_type: "ftyp".to_string(), size: 20, offset: 0 };
        assert_eq!(h.box_type, "ftyp");
    }

    #[test]
    fn test_mp4_box_new() {
        let b = Mp4Box::new(BoxHeader { box_type: "test".to_string(), size: 8, offset: 0 });
        assert_eq!(b.header.box_type, "test");
    }

    #[test]
    fn test_parse_ftyp() {
        let mut p = Mp4Parser::new();
        let ftyp_data = b"isom\x00\x00\x02\x00isomiso2";
        let mut mp4 = make_box("ftyp", ftyp_data);
        mp4.extend_from_slice(&make_box("moov", b""));
        p.parse(&mp4).unwrap();
        assert_eq!(p.info.major_brand, "isom");
        assert_eq!(p.info.minor_version, 512);
    }

    #[test]
    fn test_parse_ftyp_brands() {
        let mut p = Mp4Parser::new();
        let ftyp = make_box("ftyp", b"mp42\x00\x00\x00\x00mp42isom");
        p.parse(&ftyp).unwrap();
        assert!(p.info.compatible_brands.len() >= 2);
    }

    #[test]
    fn test_parse_mdat() {
        let mut p = Mp4Parser::new();
        let ftyp = make_box("ftyp", b"isom\x00\x00\x02\x00isom");
        let mdat = make_box("mdat", &[0x00, 0x00, 0x00, 0x01, 0x67]); // H.264 SPS start
        let mut mp4 = ftyp.clone();
        mp4.extend_from_slice(&mdat);
        p.parse(&mp4).unwrap();
        assert!(!p.mdat_data.is_empty());
    }

    #[test]
    fn test_parse_mvhd_v0() {
        let mut p = Mp4Parser::new();
        let mut mvhd = vec![0u8; 24]; // version 0
        mvhd[0] = 0; // version
        mvhd[12] = 0; mvhd[13] = 0; mvhd[14] = 0x03; mvhd[15] = 0xE8; // timescale = 1000
        mvhd[16] = 0; mvhd[17] = 0; mvhd[18] = 0xEA; mvhd[19] = 0x60; // duration = 60000
        let mut moov = make_box("mvhd", &mvhd);
        // mvhd needs flags
        mvhd[1] = 0; mvhd[2] = 0; mvhd[3] = 0;
        moov = make_box("mvhd", &mvhd);
        let moov_box = make_box("moov", &moov);
        let ftyp = make_box("ftyp", b"isom\x00\x00\x02\x00isom");
        let mut mp4 = ftyp;
        mp4.extend_from_slice(&moov_box);
        p.parse(&mp4).unwrap();
        assert_eq!(p.info.timescale, 1000);
        assert_eq!(p.info.duration_ms, 60000);
    }

    #[test]
    fn test_parse_trak_video() {
        let mut p = Mp4Parser::new();
        // hdlr
        let hdlr_data = b"\x00\x00\x00\x00\x00\x00\x00\x00vide\x00";
        let hdlr_box = make_box("hdlr", hdlr_data);
        // stsd con avc1
        let mut stsd = vec![0u8; 16];
        stsd[12] = b'a'; stsd[13] = b'v'; stsd[14] = b'c'; stsd[15] = b'1';
        let stsd_box = make_box("stsd", &stsd);
        let mut stbl = stsd_box.clone();
        stbl.extend_from_slice(&make_box("stts", b""));
        stbl.extend_from_slice(&make_box("stsc", b""));
        stbl.extend_from_slice(&make_box("stsz", b""));
        stbl.extend_from_slice(&make_box("stco", b""));
        let stbl_box = make_box("stbl", &stbl);
        let minf = make_box("minf", &stbl_box);
        let mut mdia_content = hdlr_box.clone();
        mdia_content.extend_from_slice(&minf);
        let mdia_box = make_box("mdia", &mdia_content);
        let trak = make_box("trak", &mdia_box);
        let moov = make_box("moov", &trak);
        let ftyp = make_box("ftyp", b"isom\x00\x00\x02\x00isom");
        let mut mp4 = ftyp;
        mp4.extend_from_slice(&moov);
        p.parse(&mp4).unwrap();
        assert!(p.info.has_video);
        assert_eq!(p.info.video_codec, "avc1");
    }

    #[test]
    fn test_parse_trak_audio() {
        let mut p = Mp4Parser::new();
        let hdlr_data = b"\x00\x00\x00\x00\x00\x00\x00\x00soun\x00";
        let hdlr_box = make_box("hdlr", hdlr_data);
        let mut stsd = vec![0u8; 16];
        stsd[12] = b'm'; stsd[13] = b'p'; stsd[14] = b'4'; stsd[15] = b'a';
        let stsd_box = make_box("stsd", &stsd);
        let stbl_box = make_box("stbl", &stsd_box);
        let minf = make_box("minf", &stbl_box);
        let mut mdia_content = hdlr_box;
        mdia_content.extend_from_slice(&minf);
        let mdia_box = make_box("mdia", &mdia_content);
        let trak = make_box("trak", &mdia_box);
        let moov = make_box("moov", &trak);
        let ftyp = make_box("ftyp", b"isom\x00\x00\x02\x00isom");
        let mut mp4 = ftyp;
        mp4.extend_from_slice(&moov);
        p.parse(&mp4).unwrap();
        assert!(p.info.has_audio);
        assert_eq!(p.info.audio_codec, "mp4a");
    }

    #[test]
    fn test_is_valid_mp4() {
        let p = Mp4Parser::new();
        assert!(!p.is_valid_mp4());
    }

    #[test]
    fn test_find_child() {
        let mut b = Mp4Box::new(BoxHeader { box_type: "root".to_string(), size: 0, offset: 0 });
        let child = Mp4Box::new(BoxHeader { box_type: "test".to_string(), size: 0, offset: 0 });
        b.children.push(child);
        assert!(b.find_child("test").is_some());
        assert!(b.find_child("missing").is_none());
    }

    #[test]
    fn test_find_children() {
        let mut b = Mp4Box::new(BoxHeader { box_type: "root".to_string(), size: 0, offset: 0 });
        b.children.push(Mp4Box::new(BoxHeader { box_type: "a".to_string(), size: 0, offset: 0 }));
        b.children.push(Mp4Box::new(BoxHeader { box_type: "a".to_string(), size: 0, offset: 0 }));
        b.children.push(Mp4Box::new(BoxHeader { box_type: "b".to_string(), size: 0, offset: 0 }));
        assert_eq!(b.find_children("a").len(), 2);
    }

    #[test]
    fn test_mp4_builder() {
        let mut b = Mp4Builder::new();
        b.add_box("ftyp", vec![0u8; 12]);
        assert!(b.boxes.contains_key("ftyp"));
    }

    #[test]
    fn test_mp4_build_minimal() {
        let bytes = Mp4Builder::build_minimal();
        assert!(bytes.len() > 0);
        // First box should be ftyp
        assert_eq!(&bytes[4..8], b"ftyp");
    }

    #[test]
    fn test_parse_full_mp4() {
        let mut p = Mp4Parser::new();
        let ftyp = make_box("ftyp", b"isom\x00\x00\x02\x00isomiso2");
        let mdat = make_box("mdat", &[0x67, 0x42, 0x00, 0x1E]);
        let mut mp4 = ftyp;
        mp4.extend_from_slice(&mdat);
        p.parse(&mp4).unwrap();
        assert!(p.is_valid_mp4() == false); // No moov, no track
    }

    #[test]
    fn test_track_count_increments() {
        let mut p = Mp4Parser::new();
        let hdlr_data = b"\x00\x00\x00\x00\x00\x00\x00\x00vide\x00";
        let mut trak1 = make_box("hdlr", hdlr_data);
        trak1.extend_from_slice(&make_box("mdia", &make_box("minf", &make_box("stbl", &[]))));
        let hdlr2 = b"\x00\x00\x00\x00\x00\x00\x00\x00soun\x00";
        let mut trak2 = make_box("hdlr", hdlr2);
        trak2.extend_from_slice(&make_box("mdia", &make_box("minf", &make_box("stbl", &[]))));
        let mut moov = make_box("trak", &trak1);
        moov.extend_from_slice(&make_box("trak", &trak2));
        let moov_box = make_box("moov", &moov);
        let ftyp = make_box("ftyp", b"isom\x00\x00\x02\x00isom");
        let mut mp4 = ftyp;
        mp4.extend_from_slice(&moov_box);
        p.parse(&mp4).unwrap();
        assert_eq!(p.info.track_count, 2);
    }

    #[test]
    fn test_stsd_simple() {
        let mut p = Mp4Parser::new();
        // Solo stsd con "avc1" en bytes 12-15
        let mut stsd = vec![0u8; 16];
        stsd[12] = b'a'; stsd[13] = b'v'; stsd[14] = b'c'; stsd[15] = b'1';
        let stsd_box = make_box("stsd", &stsd);
        // Wrap en stbl (container)
        let stbl_box = make_box("stbl", &stsd_box);
        // minf
        let minf_box = make_box("minf", &stbl_box);
        // mdia
        let hdlr_box = make_box("hdlr", b"\x00\x00\x00\x00\x00\x00\x00\x00vide\x00");
        let mut mdia_content = hdlr_box;
        mdia_content.extend_from_slice(&minf_box);
        let mdia_box = make_box("mdia", &mdia_content);
        // trak
        let trak_box = make_box("trak", &mdia_box);
        // moov
        let moov_box = make_box("moov", &trak_box);
        // ftyp
        let ftyp = make_box("ftyp", b"isom\x00\x00\x02\x00isom");
        let mut mp4 = ftyp;
        mp4.extend_from_slice(&moov_box);
        p.parse(&mp4).unwrap();
        assert!(p.info.has_video, "expected has_video, codec={}, video_codec={}", p.info.has_video, p.info.video_codec);
        assert_eq!(p.info.video_codec, "avc1");
    }

    #[test]
    fn test_stsd_with_siblings() {
        let mut p = Mp4Parser::new();
        let mut stsd = vec![0u8; 16];
        stsd[12] = b'a'; stsd[13] = b'v'; stsd[14] = b'c'; stsd[15] = b'1';
        let stsd_box = make_box("stsd", &stsd);
        let mut stbl = stsd_box.clone();
        stbl.extend_from_slice(&make_box("stts", b""));
        stbl.extend_from_slice(&make_box("stsc", b""));
        stbl.extend_from_slice(&make_box("stsz", b""));
        stbl.extend_from_slice(&make_box("stco", b""));
        let stbl_box = make_box("stbl", &stbl);
        let minf_box = make_box("minf", &stbl_box);
        let hdlr_box = make_box("hdlr", b"\x00\x00\x00\x00\x00\x00\x00\x00vide\x00");
        let mut mdia_content = hdlr_box;
        mdia_content.extend_from_slice(&minf_box);
        let mdia_box = make_box("mdia", &mdia_content);
        let trak_box = make_box("trak", &mdia_box);
        let moov_box = make_box("moov", &trak_box);
        let ftyp = make_box("ftyp", b"isom\x00\x00\x02\x00isom");
        let mut mp4 = ftyp;
        mp4.extend_from_slice(&moov_box);
        p.parse(&mp4).unwrap();
        assert!(p.info.has_video, "has_video={} codec={}", p.info.has_video, p.info.video_codec);
        assert_eq!(p.info.video_codec, "avc1");
    }
}
