//! DASH MPD Parser - MPEG-DASH manifest parser
//!
//! Parsea MPD (Media Presentation Description) XML para streaming DASH.

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DashProfile {
    Live,
    OnDemand,
    Main,
    Full,
}

impl DashProfile {
    pub fn from_str(s: &str) -> Self {
        match s {
            "urn:mpeg:dash:profile:isoff-live:2011" => Self::Live,
            "urn:mpeg:dash:profile:isoff-on-demand:2011" => Self::OnDemand,
            _ => Self::Main,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DashRepresentation {
    pub id: String,
    pub bandwidth: u32,
    pub width: u32,
    pub height: u32,
    pub codecs: String,
    pub frame_rate: String,
    pub sar: String,
    pub initialization: Option<String>,
    pub media_template: Option<String>,
    pub segment_template: Option<SegmentTemplate>,
    pub base_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SegmentTemplate {
    pub media: String,
    pub initialization: String,
    pub start_number: u32,
    pub timescale: u32,
    pub duration: u32,
}

#[derive(Debug, Clone)]
pub struct DashAdaptationSet {
    pub id: String,
    pub content_type: String, // "video", "audio", "text"
    pub lang: String,
    pub representations: Vec<DashRepresentation>,
    pub mime_type: String,
}

#[derive(Debug, Clone)]
pub struct DashPeriod {
    pub id: String,
    pub duration_ms: u64,
    pub adaptation_sets: Vec<DashAdaptationSet>,
}

pub struct DashMpd {
    pub profiles: DashProfile,
    pub min_buffer_time_ms: u32,
    pub media_presentation_duration_ms: u64,
    pub type_static: bool,
    pub periods: Vec<DashPeriod>,
    pub base_urls: Vec<String>,
    pub locations: Vec<String>,
    pub raw_xml: String,
}

impl DashMpd {
    pub fn new() -> Self {
        Self {
            profiles: DashProfile::Main,
            min_buffer_time_ms: 0,
            media_presentation_duration_ms: 0,
            type_static: false,
            periods: Vec::new(),
            base_urls: Vec::new(),
            locations: Vec::new(),
            raw_xml: String::new(),
        }
    }

    /// Parsea un MPD XML
    pub fn parse(&mut self, content: &str) -> Result<(), String> {
        self.raw_xml = content.to_string();
        // Atributos MPD
        self.profiles = Self::extract_attr(content, "profiles")
            .map(|s| DashProfile::from_str(s.trim()))
            .unwrap_or(DashProfile::Main);
        self.min_buffer_time_ms = Self::extract_attr(content, "minBufferTime")
            .map(|s| Self::parse_duration_ms(s.trim()).unwrap_or(0) as u32)
            .unwrap_or(0);
        self.media_presentation_duration_ms = Self::extract_attr(content, "mediaPresentationDuration")
            .map(|s| Self::parse_duration_ms(s.trim()).unwrap_or(0))
            .unwrap_or(0);
        if let Some(t) = Self::extract_attr(content, "type") {
            self.type_static = t.trim() == "static";
        }
        // BaseURL
        self.base_urls = Self::extract_tag_values(content, "BaseURL");
        // Location
        self.locations = Self::extract_tag_values(content, "Location");
        // Periods
        self.periods = self.parse_periods(content)?;
        Ok(())
    }

    fn parse_periods(&self, content: &str) -> Result<Vec<DashPeriod>, String> {
        let mut periods = Vec::new();
        let period_starts: Vec<usize> = content.match_indices("<Period").map(|(i, _)| i).collect();
        for i in 0..period_starts.len() {
            let start = period_starts[i];
            let end = if i + 1 < period_starts.len() {
                period_starts[i + 1]
            } else {
                content.len()
            };
            let period_xml = &content[start..end];
            let id = Self::extract_attr(period_xml, "id")
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("period-{}", i));
            let duration_ms = Self::extract_attr(period_xml, "duration")
                .map(|s| Self::parse_duration_ms(s.trim()).unwrap_or(0))
                .unwrap_or(0);
            let adaptation_sets = self.parse_adaptation_sets(period_xml);
            periods.push(DashPeriod {
                id,
                duration_ms,
                adaptation_sets,
            });
        }
        Ok(periods)
    }

    fn parse_adaptation_sets(&self, period_xml: &str) -> Vec<DashAdaptationSet> {
        let mut sets = Vec::new();
        let set_starts: Vec<usize> = period_xml.match_indices("<AdaptationSet").map(|(i, _)| i).collect();
        for i in 0..set_starts.len() {
            let start = set_starts[i];
            let end = if i + 1 < set_starts.len() {
                set_starts[i + 1]
            } else {
                period_xml.len()
            };
            let set_xml = &period_xml[start..end];
            let id = Self::extract_attr(set_xml, "id").unwrap_or("").to_string();
            let content_type = Self::extract_attr(set_xml, "contentType")
                .or_else(|| Self::extract_attr(set_xml, "mimeType"))
                .unwrap_or("").to_string();
            let lang = Self::extract_attr(set_xml, "lang").unwrap_or("").to_string();
            let mime_type = Self::extract_attr(set_xml, "mimeType").unwrap_or("").to_string();
            let representations = self.parse_representations(set_xml);
            sets.push(DashAdaptationSet {
                id,
                content_type,
                lang,
                representations,
                mime_type,
            });
        }
        sets
    }

    fn parse_representations(&self, set_xml: &str) -> Vec<DashRepresentation> {
        let mut reps = Vec::new();
        let rep_starts: Vec<usize> = set_xml.match_indices("<Representation").map(|(i, _)| i).collect();
        for i in 0..rep_starts.len() {
            let start = rep_starts[i];
            let end = if i + 1 < rep_starts.len() {
                rep_starts[i + 1]
            } else {
                set_xml.len()
            };
            let rep_xml = &set_xml[start..end];
            let id = Self::extract_attr(rep_xml, "id").unwrap_or("").to_string();
            let bandwidth = Self::extract_attr(rep_xml, "bandwidth")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            let width = Self::extract_attr(rep_xml, "width")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            let height = Self::extract_attr(rep_xml, "height")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            let codecs = Self::extract_attr(rep_xml, "codecs").unwrap_or("").to_string();
            let frame_rate = Self::extract_attr(rep_xml, "frameRate").unwrap_or("").to_string();
            let sar = Self::extract_attr(rep_xml, "sar").unwrap_or("").to_string();
            let initialization = Self::extract_tag_value(rep_xml, "Initialization");
            let base_url = Self::extract_tag_value(rep_xml, "BaseURL");
            // SegmentTemplate
            let segment_template = if let Some(seg_xml) = Self::extract_tag_inner(rep_xml, "SegmentTemplate") {
                Some(SegmentTemplate {
                    media: Self::extract_attr(&seg_xml, "media").unwrap_or("").to_string(),
                    initialization: Self::extract_attr(&seg_xml, "initialization").unwrap_or("").to_string(),
                    start_number: Self::extract_attr(&seg_xml, "startNumber")
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(1),
                    timescale: Self::extract_attr(&seg_xml, "timescale")
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(1),
                    duration: Self::extract_attr(&seg_xml, "duration")
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0),
                })
            } else {
                None
            };
            let media_template = if let Some(seg_xml) = Self::extract_tag_inner(rep_xml, "SegmentTemplate") {
                Self::extract_attr(&seg_xml, "media").map(|s| s.to_string())
            } else {
                None
            };
            reps.push(DashRepresentation {
                id,
                bandwidth,
                width,
                height,
                codecs,
                frame_rate,
                sar,
                initialization,
                media_template,
                segment_template,
                base_url,
            });
        }
        reps
    }

    /// Extrae un atributo XML
    fn extract_attr<'a>(content: &'a str, name: &str) -> Option<&'a str> {
        // Buscar con whitespace o '<' antes para evitar matches en sub-strings
        // como "bandwidth=" matching "width="
        let pat1 = format!(" {}=\"", name);
        if let Some(start) = content.find(pat1.as_str()) {
            let value_start = start + pat1.len();
            if let Some(end_offset) = content[value_start..].find('"') {
                return Some(&content[value_start..value_start + end_offset]);
            }
        }
        let pat2 = format!("{}='", name);
        if let Some(start) = content.find(pat2.as_str()) {
            let value_start = start + pat2.len();
            if let Some(end_offset) = content[value_start..].find('\'') {
                return Some(&content[value_start..value_start + end_offset]);
            }
        }
        // También buscar al inicio del string (caso "id=...")
        let pat3 = format!("{}=\"", name);
        if content.starts_with(pat3.as_str()) {
            let value_start = pat3.len();
            if let Some(end_offset) = content[value_start..].find('"') {
                return Some(&content[value_start..value_start + end_offset]);
            }
        }
        None
    }

    fn extract_tag_value(content: &str, tag: &str) -> Option<String> {
        Self::extract_tag_inner(content, tag).map(|s| s.to_string())
    }

    fn extract_tag_inner<'a>(content: &'a str, tag: &str) -> Option<&'a str> {
        let open = format!("<{}", tag);
        let close = format!("</{}>", tag);
        if let Some(start) = content.find(open.as_str()) {
            let after_open = start + open.len();
            // Find end of open tag
            if let Some(rel_end) = content[after_open..].find('>') {
                let end_of_open = after_open + rel_end;
                // Check si es self-closing
                if end_of_open > 0 && content.as_bytes()[end_of_open - 1] == b'/' {
                    return Some(&content[after_open..end_of_open - 1]);
                }
                let inner_start = end_of_open + 1;
                if let Some(end_offset) = content[inner_start..].find(close.as_str()) {
                    return Some(&content[inner_start..inner_start + end_offset]);
                }
            }
        }
        None
    }

    fn extract_tag_values(content: &str, tag: &str) -> Vec<String> {
        let mut result = Vec::new();
        let open = format!("<{}>", tag);
        let close = format!("</{}>", tag);
        let mut pos = 0;
        while let Some(start) = content[pos..].find(open.as_str()) {
            let abs_start = pos + start + open.len();
            if let Some(end_offset) = content[abs_start..].find(close.as_str()) {
                result.push(content[abs_start..abs_start + end_offset].to_string());
                pos = abs_start + end_offset + close.len();
            } else {
                break;
            }
        }
        result
    }

    /// Parsea duración ISO 8601 (PT1H30M15S) o segundos
    fn parse_duration_ms(s: &str) -> Option<u64> {
        if let Some(rest) = s.strip_prefix("PT") {
            let mut total_ms: u64 = 0;
            let mut current_num = String::new();
            for c in rest.chars() {
                if c.is_ascii_digit() || c == '.' {
                    current_num.push(c);
                } else {
                    let val: f64 = current_num.parse().unwrap_or(0.0);
                    match c {
                        'H' => total_ms += (val * 3600.0 * 1000.0) as u64,
                        'M' => total_ms += (val * 60.0 * 1000.0) as u64,
                        'S' => total_ms += (val * 1000.0) as u64,
                        _ => {}
                    }
                    current_num.clear();
                }
            }
            return Some(total_ms);
        }
        // Try as plain number (seconds)
        if let Ok(secs) = s.parse::<f64>() {
            return Some((secs * 1000.0) as u64);
        }
        None
    }

    pub fn video_representations(&self) -> Vec<&DashRepresentation> {
        self.periods.iter()
            .flat_map(|p| p.adaptation_sets.iter())
            .filter(|s| s.content_type == "video" || s.mime_type.starts_with("video/"))
            .flat_map(|s| s.representations.iter())
            .collect()
    }

    pub fn audio_representations(&self) -> Vec<&DashRepresentation> {
        self.periods.iter()
            .flat_map(|p| p.adaptation_sets.iter())
            .filter(|s| s.content_type == "audio" || s.mime_type.starts_with("audio/"))
            .flat_map(|s| s.representations.iter())
            .collect()
    }

    pub fn best_video(&self) -> Option<&DashRepresentation> {
        let mut best: Option<&DashRepresentation> = None;
        for r in self.video_representations() {
            if best.is_none() || r.bandwidth > best.unwrap().bandwidth {
                best = Some(r);
            }
        }
        best
    }

    pub fn period_count(&self) -> usize {
        self.periods.len()
    }
}

impl Default for DashMpd {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dash_profile() {
        assert_eq!(DashProfile::from_str("urn:mpeg:dash:profile:isoff-live:2011"), DashProfile::Live);
        assert_eq!(DashProfile::from_str("urn:mpeg:dash:profile:isoff-on-demand:2011"), DashProfile::OnDemand);
    }

    #[test]
    fn test_mpd_new() {
        let m = DashMpd::new();
        assert_eq!(m.period_count(), 0);
    }

    #[test]
    fn test_parse_duration_pt() {
        assert_eq!(DashMpd::parse_duration_ms("PT1H"), Some(3600 * 1000));
        assert_eq!(DashMpd::parse_duration_ms("PT30M"), Some(30 * 60 * 1000));
        assert_eq!(DashMpd::parse_duration_ms("PT15S"), Some(15 * 1000));
        assert_eq!(DashMpd::parse_duration_ms("PT1H30M15S"), Some(3600*1000 + 30*60*1000 + 15*1000));
    }

    #[test]
    fn test_parse_duration_seconds() {
        assert_eq!(DashMpd::parse_duration_ms("60"), Some(60_000));
    }

    #[test]
    fn test_extract_attr() {
        let xml = r#"<MPD profiles="urn:mpeg:dash:profile:isoff-on-demand:2011">"#;
        assert_eq!(DashMpd::extract_attr(xml, "profiles"), Some("urn:mpeg:dash:profile:isoff-on-demand:2011"));
    }

    #[test]
    fn test_extract_attr_width() {
        let xml = r#"<Representation id="1" bandwidth="500000" width="640" height="360"/>"#;
        let result = DashMpd::extract_attr(xml, "width");
        eprintln!("DEBUG extract width from '{}' = {:?}", xml, result);
        assert_eq!(result, Some("640"));
    }

    #[test]
    fn test_extract_attr_single_quote() {
        let xml = r#"<MPD profiles='live'>"#;
        assert_eq!(DashMpd::extract_attr(xml, "profiles"), Some("live"));
    }

    #[test]
    fn test_extract_tag_inner() {
        let xml = r#"<BaseURL>http://example.com/</BaseURL>"#;
        assert_eq!(DashMpd::extract_tag_inner(xml, "BaseURL"), Some("http://example.com/"));
    }

    #[test]
    fn test_extract_tag_values() {
        let xml = r#"<Location>http://a.com/</Location><Location>http://b.com/</Location>"#;
        let vals = DashMpd::extract_tag_values(xml, "Location");
        assert_eq!(vals.len(), 2);
    }

    #[test]
    fn test_parse_simple_mpd() {
        let mut m = DashMpd::new();
        let xml = r#"<?xml version="1.0"?>
<MPD xmlns="urn:mpeg:dash:schema:mpd:2011" type="static" mediaPresentationDuration="PT60S" minBufferTime="PT2S" profiles="urn:mpeg:dash:profile:isoff-on-demand:2011">
<Period id="0" duration="PT60S">
<AdaptationSet contentType="video" mimeType="video/mp4">
<Representation id="1" bandwidth="500000" width="640" height="360" codecs="avc1.42c01e" frameRate="30">
<BaseURL>video_360.mp4</BaseURL>
</Representation>
<Representation id="2" bandwidth="2000000" width="1280" height="720" codecs="avc1.42c01e" frameRate="30">
<BaseURL>video_720.mp4</BaseURL>
</Representation>
</AdaptationSet>
</Period>
</MPD>"#;
        m.parse(xml).unwrap();
        assert!(m.type_static);
        assert_eq!(m.media_presentation_duration_ms, 60_000);
        assert_eq!(m.min_buffer_time_ms, 2_000);
        assert_eq!(m.period_count(), 1);
        // Verify both reps parsed correctly
        let reps = &m.periods[0].adaptation_sets[0].representations;
        assert_eq!(reps.len(), 2);
        assert_eq!(reps[0].width, 640);
        assert_eq!(reps[0].height, 360);
        assert_eq!(reps[1].width, 1280);
        assert_eq!(reps[1].height, 720);
    }

    #[test]
    fn test_parse_video_representations() {
        let mut m = DashMpd::new();
        let xml = r#"<MPD>
<Period>
<AdaptationSet contentType="video">
<Representation id="v1" bandwidth="1000000" width="640" height="360"/>
<Representation id="v2" bandwidth="2000000" width="1280" height="720"/>
</AdaptationSet>
</Period>
</MPD>"#;
        m.parse(xml).unwrap();
        let vids = m.video_representations();
        assert_eq!(vids.len(), 2);
    }

    #[test]
    fn test_parse_audio_representations() {
        let mut m = DashMpd::new();
        let xml = r#"<MPD>
<Period>
<AdaptationSet contentType="audio">
<Representation id="a1" bandwidth="128000" codecs="mp4a.40.2"/>
</AdaptationSet>
</Period>
</MPD>"#;
        m.parse(xml).unwrap();
        let auds = m.audio_representations();
        assert_eq!(auds.len(), 1);
    }

    #[test]
    fn test_best_video() {
        let mut m = DashMpd::new();
        let xml = r#"<MPD>
<Period>
<AdaptationSet contentType="video">
<Representation id="v1" bandwidth="1000000" width="640" height="360"/>
<Representation id="v2" bandwidth="5000000" width="1920" height="1080"/>
<Representation id="v3" bandwidth="2000000" width="1280" height="720"/>
</AdaptationSet>
</Period>
</MPD>"#;
        m.parse(xml).unwrap();
        let best = m.best_video().unwrap();
        assert_eq!(best.bandwidth, 5000000);
        assert_eq!(best.width, 1920);
        assert_eq!(best.height, 1080);
    }

    #[test]
    fn test_parse_segment_template() {
        let mut m = DashMpd::new();
        let xml = r#"<MPD>
<Period>
<AdaptationSet contentType="video">
<Representation id="v1" bandwidth="1000000">
<SegmentTemplate media="$RepresentationID$/seg-$Number$.m4s" initialization="$RepresentationID$/init.mp4" startNumber="1" timescale="1000" duration="5000"/>
</Representation>
</AdaptationSet>
</Period>
</MPD>"#;
        m.parse(xml).unwrap();
        let rep = &m.periods[0].adaptation_sets[0].representations[0];
        assert!(rep.segment_template.is_some());
        let st = rep.segment_template.as_ref().unwrap();
        assert_eq!(st.duration, 5000);
        assert_eq!(st.timescale, 1000);
    }

    #[test]
    fn test_parse_multiple_periods() {
        let mut m = DashMpd::new();
        let xml = r#"<MPD>
<Period id="0" duration="PT30S">
<AdaptationSet contentType="video">
<Representation id="v1" bandwidth="1000000"/>
</AdaptationSet>
</Period>
<Period id="1" duration="PT30S">
<AdaptationSet contentType="video">
<Representation id="v2" bandwidth="2000000"/>
</AdaptationSet>
</Period>
</MPD>"#;
        m.parse(xml).unwrap();
        assert_eq!(m.period_count(), 2);
    }

    #[test]
    fn test_parse_with_base_url() {
        let mut m = DashMpd::new();
        let xml = r#"<MPD>
<BaseURL>http://cdn.example.com/</BaseURL>
<Period>
<AdaptationSet contentType="video">
<Representation id="v1" bandwidth="1000000"/>
</AdaptationSet>
</Period>
</MPD>"#;
        m.parse(xml).unwrap();
        assert_eq!(m.base_urls.len(), 1);
    }

    #[test]
    fn test_representation_codecs() {
        let mut m = DashMpd::new();
        let xml = r#"<MPD>
<Period>
<AdaptationSet contentType="video">
<Representation id="v1" bandwidth="1000000" codecs="avc1.640028"/>
</AdaptationSet>
</Period>
</MPD>"#;
        m.parse(xml).unwrap();
        let rep = &m.periods[0].adaptation_sets[0].representations[0];
        assert_eq!(rep.codecs, "avc1.640028");
    }

    #[test]
    fn test_duration_ms_complex() {
        assert_eq!(DashMpd::parse_duration_ms("PT1H30M"), Some(3600*1000 + 30*60*1000));
        assert_eq!(DashMpd::parse_duration_ms("PT0.5S"), Some(500));
    }
}
