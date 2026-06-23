//! HLS Parser - HTTP Live Streaming (Apple)
//!
//! Parsea archivos M3U8 master y media playlists.

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlaylistType {
    Master,
    Media,
    Event,
    Vod,
}

impl PlaylistType {
    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "MASTER" => Self::Master,
            "MEDIA" => Self::Media,
            "EVENT" => Self::Event,
            "VOD" => Self::Vod,
            _ => Self::Media,
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            Self::Master => "MASTER",
            Self::Media => "MEDIA",
            Self::Event => "EVENT",
            Self::Vod => "VOD",
        }
    }
}

#[derive(Debug, Clone)]
pub struct StreamVariant {
    pub bandwidth: u32,
    pub avg_bandwidth: u32,
    pub codecs: String,
    pub resolution: String,
    pub width: u32,
    pub height: u32,
    pub frame_rate: f32,
    pub uri: String,
    pub audio: String,
    pub subtitles: String,
}

#[derive(Debug, Clone)]
pub struct MediaSegment {
    pub sequence: u32,
    pub uri: String,
    pub duration: f32,
    pub discontinuity: bool,
    pub byte_range: Option<String>,
    pub program_date_time: Option<String>,
}

pub struct HlsPlaylist {
    pub version: u32,
    pub target_duration: u32,
    pub media_sequence: u32,
    pub discontinuity_sequence: u32,
    pub endlist: bool,
    pub playlist_type: PlaylistType,
    pub segments: Vec<MediaSegment>,
    pub variants: Vec<StreamVariant>,
    pub keys: Vec<EncryptionKey>,
    pub raw_attributes: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct EncryptionKey {
    pub method: String,
    pub uri: String,
    pub iv: Option<String>,
    pub key_format: Option<String>,
}

impl HlsPlaylist {
    pub fn new() -> Self {
        Self {
            version: 0,
            target_duration: 0,
            media_sequence: 0,
            discontinuity_sequence: 0,
            endlist: false,
            playlist_type: PlaylistType::Media,
            segments: Vec::new(),
            variants: Vec::new(),
            keys: Vec::new(),
            raw_attributes: HashMap::new(),
        }
    }

    /// Parsea contenido M3U8
    pub fn parse(&mut self, content: &str) -> Result<(), String> {
        let mut lines = content.lines().map(|s| s.trim()).collect::<Vec<_>>();
        if lines.is_empty() || !lines[0].starts_with("#EXTM3U") {
            return Err("Invalid M3U8: missing #EXTM3U".to_string());
        }
        // Detectar si es master
        let has_stream_inf = lines.iter().any(|l| l.contains("#EXT-X-STREAM-INF"));
        if has_stream_inf {
            self.playlist_type = PlaylistType::Master;
            return self.parse_master(&lines);
        }
        self.parse_media(&lines)
    }

    fn parse_master(&mut self, lines: &[&str]) -> Result<(), String> {
        let mut current: Option<StreamVariant> = None;
        for line in lines {
            if line.starts_with("#EXT-X-VERSION:") {
                self.version = line[15..].trim().parse().unwrap_or(0);
            } else if line.starts_with("#EXT-X-STREAM-INF:") {
                let attrs = self.parse_attributes(&line[18..]);
                let mut v = StreamVariant {
                    bandwidth: 0,
                    avg_bandwidth: 0,
                    codecs: String::new(),
                    resolution: String::new(),
                    width: 0,
                    height: 0,
                    frame_rate: 0.0,
                    uri: String::new(),
                    audio: String::new(),
                    subtitles: String::new(),
                };
                if let Some(bw) = attrs.get("BANDWIDTH") {
                    v.bandwidth = bw.parse().unwrap_or(0);
                }
                if let Some(bw) = attrs.get("AVERAGE-BANDWIDTH") {
                    v.avg_bandwidth = bw.parse().unwrap_or(0);
                }
                if let Some(c) = attrs.get("CODECS") {
                    v.codecs = c.clone();
                }
                if let Some(r) = attrs.get("RESOLUTION") {
                    v.resolution = r.clone();
                    let parts: Vec<&str> = r.split('x').collect();
                    if parts.len() == 2 {
                        v.width = parts[0].parse().unwrap_or(0);
                        v.height = parts[1].parse().unwrap_or(0);
                    }
                }
                if let Some(fr) = attrs.get("FRAME-RATE") {
                    v.frame_rate = fr.parse().unwrap_or(0.0);
                }
                if let Some(a) = attrs.get("AUDIO") {
                    v.audio = a.clone();
                }
                if let Some(s) = attrs.get("SUBTITLES") {
                    v.subtitles = s.clone();
                }
                current = Some(v);
            } else if !line.starts_with('#') && !line.is_empty() {
                if let Some(mut v) = current.take() {
                    v.uri = line.to_string();
                    self.variants.push(v);
                }
            }
        }
        Ok(())
    }

    fn parse_media(&mut self, lines: &[&str]) -> Result<(), String> {
        let mut current: Option<MediaSegment> = None;
        let mut segment_count = 0u32;
        let mut pending_discontinuity = false;
        let mut pending_pdt: Option<String> = None;
        for line in lines {
            if line.starts_with("#EXT-X-VERSION:") {
                self.version = line[15..].trim().parse().unwrap_or(0);
            } else if line.starts_with("#EXT-X-TARGETDURATION:") {
                self.target_duration = line[22..].trim().parse().unwrap_or(0);
            } else if line.starts_with("#EXT-X-MEDIA-SEQUENCE:") {
                self.media_sequence = line[22..].trim().parse().unwrap_or(0);
            } else if line.starts_with("#EXT-X-DISCONTINUITY-SEQUENCE:") {
                self.discontinuity_sequence = line[31..].trim().parse().unwrap_or(0);
            } else if line.starts_with("#EXT-X-PLAYLIST-TYPE:") {
                self.playlist_type = PlaylistType::from_str(line[21..].trim());
            } else if line.starts_with("#EXT-X-ENDLIST") {
                self.endlist = true;
            } else if line.starts_with("#EXT-X-KEY:") {
                let attrs = self.parse_attributes(&line[11..]);
                let key = EncryptionKey {
                    method: attrs.get("METHOD").cloned().unwrap_or_default(),
                    uri: attrs.get("URI").cloned().unwrap_or_default(),
                    iv: attrs.get("IV").cloned(),
                    key_format: attrs.get("KEYFORMAT").cloned(),
                };
                self.keys.push(key);
            } else if line.starts_with("#EXTINF:") {
                let info = &line[8..];
                let parts: Vec<&str> = info.split(',').collect();
                let duration = parts[0].trim().parse().unwrap_or(0.0);
                segment_count += 1;
                current = Some(MediaSegment {
                    sequence: self.media_sequence + segment_count - 1,
                    uri: String::new(),
                    duration,
                    discontinuity: pending_discontinuity,
                    byte_range: None,
                    program_date_time: pending_pdt.take(),
                });
                pending_discontinuity = false;
            } else if line.starts_with("#EXT-X-DISCONTINUITY") {
                pending_discontinuity = true;
            } else if line.starts_with("#EXT-X-BYTERANGE:") {
                if let Some(s) = current.as_mut() {
                    s.byte_range = Some(line[18..].to_string());
                }
            } else if line.starts_with("#EXT-X-PROGRAM-DATE-TIME:") {
                pending_pdt = Some(line[25..].to_string());
            } else if !line.starts_with('#') && !line.is_empty() {
                if let Some(mut s) = current.take() {
                    s.uri = line.to_string();
                    self.segments.push(s);
                }
            } else if line.starts_with('#') {
                if let Some(rest) = line.split(':').next() {
                    if !line.contains("=") && rest.starts_with("#EXT-X-") {
                        continue;
                    }
                }
                let key = line.split(':').next().unwrap_or(line).to_string();
                let val = line.split(':').nth(1).unwrap_or("").to_string();
                self.raw_attributes.insert(key, val);
            }
        }
        Ok(())
    }

    fn parse_attributes(&self, s: &str) -> HashMap<String, String> {
        let mut attrs = HashMap::new();
        let mut current_key = String::new();
        let mut in_quotes = false;
        let mut chars = s.chars().peekable();
        let mut current = String::new();
        while let Some(c) = chars.next() {
            match c {
                '"' => {
                    in_quotes = !in_quotes;
                    current.push(c);
                }
                ',' if !in_quotes => {
                    if let Some(eq) = current.find('=') {
                        let k = current[..eq].trim().to_string();
                        let v = current[eq+1..].trim().trim_matches('"').to_string();
                        attrs.insert(k, v);
                    }
                    current.clear();
                }
                _ => current.push(c),
            }
        }
        if !current.is_empty() {
            if let Some(eq) = current.find('=') {
                let k = current[..eq].trim().to_string();
                let v = current[eq+1..].trim().trim_matches('"').to_string();
                attrs.insert(k, v);
            }
        }
        let _ = current_key; // suppress warning
        attrs
    }

    pub fn total_duration(&self) -> f32 {
        self.segments.iter().map(|s| s.duration).sum()
    }

    pub fn best_variant(&self) -> Option<&StreamVariant> {
        self.variants.iter().max_by_key(|v| v.bandwidth)
    }

    pub fn lowest_variant(&self) -> Option<&StreamVariant> {
        self.variants.iter().min_by_key(|v| v.bandwidth)
    }
}

impl Default for HlsPlaylist {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_playlist_type() {
        assert_eq!(PlaylistType::from_str("master"), PlaylistType::Master);
        assert_eq!(PlaylistType::from_str("VOD"), PlaylistType::Vod);
        assert_eq!(PlaylistType::Master.to_str(), "MASTER");
    }

    #[test]
    fn test_playlist_new() {
        let p = HlsPlaylist::new();
        assert_eq!(p.version, 0);
        assert!(!p.endlist);
    }

    #[test]
    fn test_parse_invalid() {
        let mut p = HlsPlaylist::new();
        let r = p.parse("not a playlist");
        assert!(r.is_err());
    }

    #[test]
    fn test_parse_media_basic() {
        let mut p = HlsPlaylist::new();
        let content = r#"#EXTM3U
#EXT-X-VERSION:3
#EXT-X-TARGETDURATION:10
#EXT-X-MEDIA-SEQUENCE:0
#EXTINF:10.0,
segment0.ts
#EXTINF:10.0,
segment1.ts
#EXT-X-ENDLIST
"#;
        p.parse(content).unwrap();
        assert_eq!(p.version, 3);
        assert_eq!(p.target_duration, 10);
        assert_eq!(p.media_sequence, 0);
        assert_eq!(p.segments.len(), 2);
        assert_eq!(p.segments[0].duration, 10.0);
        assert!(p.endlist);
    }

    #[test]
    fn test_parse_master() {
        let mut p = HlsPlaylist::new();
        let content = r#"#EXTM3U
#EXT-X-VERSION:3
#EXT-X-STREAM-INF:BANDWIDTH=1280000,AVERAGE-BANDWIDTH=1000000,CODECS="avc1.42c01e,mp4a.40.2",RESOLUTION=640x360
video_360p.m3u8
#EXT-X-STREAM-INF:BANDWIDTH=2560000,AVERAGE-BANDWIDTH=2000000,CODECS="avc1.42c01e,mp4a.40.2",RESOLUTION=1280x720
video_720p.m3u8
"#;
        p.parse(content).unwrap();
        assert_eq!(p.playlist_type, PlaylistType::Master);
        assert_eq!(p.variants.len(), 2);
        assert_eq!(p.variants[0].bandwidth, 1280000);
        assert_eq!(p.variants[0].width, 640);
        assert_eq!(p.variants[0].height, 360);
        assert_eq!(p.variants[1].bandwidth, 2560000);
        assert_eq!(p.variants[1].width, 1280);
    }

    #[test]
    fn test_parse_vod() {
        let mut p = HlsPlaylist::new();
        let content = r#"#EXTM3U
#EXT-X-VERSION:3
#EXT-X-PLAYLIST-TYPE:VOD
#EXT-X-TARGETDURATION:10
#EXTINF:10.0,
s0.ts
#EXT-X-ENDLIST
"#;
        p.parse(content).unwrap();
        assert_eq!(p.playlist_type, PlaylistType::Vod);
    }

    #[test]
    fn test_parse_encryption() {
        let mut p = HlsPlaylist::new();
        let content = r#"#EXTM3U
#EXT-X-VERSION:3
#EXT-X-KEY:METHOD=AES-128,URI="key.php",IV=0x1234
#EXT-X-TARGETDURATION:10
#EXTINF:10.0,
enc.ts
#EXT-X-ENDLIST
"#;
        p.parse(content).unwrap();
        assert_eq!(p.keys.len(), 1);
        assert_eq!(p.keys[0].method, "AES-128");
    }

    #[test]
    fn test_parse_discontinuity() {
        let mut p = HlsPlaylist::new();
        let content = r#"#EXTM3U
#EXT-X-VERSION:3
#EXT-X-TARGETDURATION:10
#EXTINF:5.0,
s0.ts
#EXT-X-DISCONTINUITY
#EXTINF:5.0,
s1.ts
#EXT-X-ENDLIST
"#;
        p.parse(content).unwrap();
        assert_eq!(p.segments.len(), 2);
        assert!(!p.segments[0].discontinuity);
        assert!(p.segments[1].discontinuity);
    }

    #[test]
    fn test_total_duration() {
        let mut p = HlsPlaylist::new();
        let content = r#"#EXTM3U
#EXT-X-VERSION:3
#EXT-X-TARGETDURATION:10
#EXTINF:5.5,
s0.ts
#EXTINF:4.5,
s1.ts
#EXT-X-ENDLIST
"#;
        p.parse(content).unwrap();
        assert_eq!(p.total_duration(), 10.0);
    }

    #[test]
    fn test_best_variant() {
        let mut p = HlsPlaylist::new();
        let content = r#"#EXTM3U
#EXT-X-VERSION:3
#EXT-X-STREAM-INF:BANDWIDTH=500000
lo.m3u8
#EXT-X-STREAM-INF:BANDWIDTH=5000000
hi.m3u8
"#;
        p.parse(content).unwrap();
        let best = p.best_variant().unwrap();
        assert_eq!(best.bandwidth, 5000000);
    }

    #[test]
    fn test_lowest_variant() {
        let mut p = HlsPlaylist::new();
        let content = r#"#EXTM3U
#EXT-X-VERSION:3
#EXT-X-STREAM-INF:BANDWIDTH=5000000
hi.m3u8
#EXT-X-STREAM-INF:BANDWIDTH=500000
lo.m3u8
"#;
        p.parse(content).unwrap();
        let lo = p.lowest_variant().unwrap();
        assert_eq!(lo.bandwidth, 500000);
    }

    #[test]
    fn test_parse_attributes_simple() {
        let p = HlsPlaylist::new();
        let attrs = p.parse_attributes("BANDWIDTH=1280000,RESOLUTION=640x360");
        assert_eq!(attrs.get("BANDWIDTH").unwrap(), "1280000");
        assert_eq!(attrs.get("RESOLUTION").unwrap(), "640x360");
    }

    #[test]
    fn test_parse_attributes_quoted() {
        let p = HlsPlaylist::new();
        let attrs = p.parse_attributes(r#"CODECS="avc1.42c01e,mp4a.40.2",BANDWIDTH=1000"#);
        assert_eq!(attrs.get("CODECS").unwrap(), "avc1.42c01e,mp4a.40.2");
        assert_eq!(attrs.get("BANDWIDTH").unwrap(), "1000");
    }

    #[test]
    fn test_segment_sequence() {
        let mut p = HlsPlaylist::new();
        let content = r#"#EXTM3U
#EXT-X-VERSION:3
#EXT-X-MEDIA-SEQUENCE:100
#EXT-X-TARGETDURATION:10
#EXTINF:5.0,
s100.ts
#EXTINF:5.0,
s101.ts
#EXT-X-ENDLIST
"#;
        p.parse(content).unwrap();
        assert_eq!(p.segments[0].sequence, 100);
        assert_eq!(p.segments[1].sequence, 101);
    }

    #[test]
    fn test_program_date_time() {
        let mut p = HlsPlaylist::new();
        let content = r#"#EXTM3U
#EXT-X-VERSION:3
#EXT-X-TARGETDURATION:10
#EXT-X-PROGRAM-DATE-TIME:2024-01-01T00:00:00Z
#EXTINF:5.0,
s0.ts
#EXT-X-ENDLIST
"#;
        p.parse(content).unwrap();
        assert!(p.segments[0].program_date_time.is_some());
    }
}
