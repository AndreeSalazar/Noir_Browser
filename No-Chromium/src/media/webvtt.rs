//! WebVTT Parser - Subtítulos para video
//!
//! Parsea archivos .vtt con cues y configuración.

use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VttAlign {
    Start,
    Center,
    End,
    Left,
    Right,
}

impl VttAlign {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "center" => Self::Center,
            "end" | "right" => Self::End,
            "left" => Self::Left,
            _ => Self::Start,
        }
    }
}

#[derive(Debug, Clone)]
pub struct VttCue {
    pub start: Duration,
    pub end: Duration,
    pub text: String,
    pub identifier: Option<String>,
    pub align: VttAlign,
    pub line: i32,
    pub position: i32,
    pub size: u32,
    pub vertical: bool,
}

impl VttCue {
    pub fn new(start: Duration, end: Duration, text: &str) -> Self {
        Self {
            start, end,
            text: text.to_string(),
            identifier: None,
            align: VttAlign::Center,
            line: -1,
            position: -1,
            size: 100,
            vertical: false,
        }
    }

    pub fn duration(&self) -> Duration {
        self.end.saturating_sub(self.start)
    }

    pub fn contains(&self, time: Duration) -> bool {
        time >= self.start && time <= self.end
    }
}

#[derive(Debug, Clone)]
pub struct VttStyle {
    pub selector: String,
    pub color: Option<String>,
    pub background: Option<String>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

pub struct WebVtt {
    pub cues: Vec<VttCue>,
    pub styles: Vec<VttStyle>,
    pub raw_text: String,
}

impl WebVtt {
    pub fn new() -> Self {
        Self {
            cues: Vec::new(),
            styles: Vec::new(),
            raw_text: String::new(),
        }
    }

    pub fn parse(&mut self, content: &str) -> Result<(), String> {
        self.raw_text = content.to_string();
        let mut lines = content.lines().map(|s| s.to_string()).collect::<Vec<_>>();
        if lines.is_empty() || !lines[0].trim_start().starts_with("WEBVTT") {
            return Err("Invalid WebVTT: missing WEBVTT header".to_string());
        }
        // Skip header: parse STYLE/NOTE blocks (they can contain blank lines)
        // Continue until we hit something that's not a STYLE/NOTE block
        let mut i = 1;
        loop {
            // Skip blank lines
            while i < lines.len() && lines[i].trim().is_empty() {
                i += 1;
            }
            if i >= lines.len() { break; }
            if lines[i].trim_start().starts_with("STYLE") {
                let style_block = self.parse_style_block(&lines[i..]);
                if let Some((styles, consumed)) = style_block {
                    self.styles.extend(styles);
                    i += consumed;
                    continue;
                }
                break;
            }
            if lines[i].trim_start().starts_with("NOTE") {
                while i < lines.len() && !lines[i].trim().is_empty() {
                    i += 1;
                }
                continue;
            }
            // Not a header block, stop
            break;
        }
        // Parse cues
        while i < lines.len() {
            // Skip blank lines
            while i < lines.len() && lines[i].trim().is_empty() {
                i += 1;
            }
            if i >= lines.len() { break; }
            // Possible cue identifier or timing
            let mut cue_lines: Vec<String> = Vec::new();
            while i < lines.len() && !lines[i].trim().is_empty() {
                cue_lines.push(lines[i].clone());
                i += 1;
            }
            if cue_lines.is_empty() { continue; }
            // Try to parse as cue
            if let Some(cue) = self.parse_cue(&cue_lines) {
                self.cues.push(cue);
            }
        }
        Ok(())
    }

    fn parse_cue(&self, lines: &[String]) -> Option<VttCue> {
        if lines.is_empty() { return None; }
        // Find timing line (contains "-->")
        let mut timing_idx = None;
        for (idx, l) in lines.iter().enumerate() {
            if l.contains("-->") {
                timing_idx = Some(idx);
                break;
            }
        }
        let timing_idx = timing_idx?;
        let (start, end) = Self::parse_timestamp_line(&lines[timing_idx])?;
        let mut cue = VttCue::new(start, end, "");
        // Optional settings on same line after timing
        let timing_line = &lines[timing_idx];
        if let Some(colon_pos) = timing_line.find(':') {
            let settings_part = &timing_line[colon_pos+1..];
            for setting in settings_part.split_whitespace() {
                if let Some(val) = setting.strip_prefix("align:") {
                    cue.align = VttAlign::from_str(val);
                } else if let Some(val) = setting.strip_prefix("line:") {
                    cue.line = val.trim_end_matches('%').parse().unwrap_or(-1);
                } else if let Some(val) = setting.strip_prefix("position:") {
                    cue.position = val.trim_end_matches('%').parse().unwrap_or(-1);
                } else if let Some(val) = setting.strip_prefix("size:") {
                    cue.size = val.trim_end_matches('%').parse().unwrap_or(100);
                } else if setting == "vertical:lr" || setting == "vertical:rl" {
                    cue.vertical = true;
                }
            }
        }
        // Identifier line (before timing)
        if timing_idx > 0 {
            cue.identifier = Some(lines[0].clone());
        }
        // Text lines (after timing)
        let text_lines: Vec<&str> = if timing_idx + 1 < lines.len() {
            lines[timing_idx + 1..].iter().map(|s| s.as_str()).collect()
        } else {
            Vec::new()
        };
        cue.text = text_lines.join("\n");
        Some(cue)
    }

    fn parse_timestamp_line(line: &str) -> Option<(Duration, Duration)> {
        let parts: Vec<&str> = line.split("-->").collect();
        if parts.len() < 2 { return None; }
        let start = Self::parse_timestamp(parts[0].trim())?;
        let mut end_str = parts[1].trim();
        // Remove any settings after first whitespace
        if let Some(space) = end_str.find(char::is_whitespace) {
            end_str = &end_str[..space];
        }
        let end = Self::parse_timestamp(end_str)?;
        Some((start, end))
    }

    fn parse_timestamp(s: &str) -> Option<Duration> {
        // Format: HH:MM:SS.mmm or MM:SS.mmm
        let parts: Vec<&str> = s.split(':').collect();
        let mut total_ms: u64 = 0;
        match parts.len() {
            3 => {
                let h: u64 = parts[0].parse().ok()?;
                let m: u64 = parts[1].parse().ok()?;
                let (sec, ms) = Self::parse_seconds_ms(parts[2])?;
                total_ms = h * 3600_000 + m * 60_000 + sec * 1000 + ms;
            }
            2 => {
                let m: u64 = parts[0].parse().ok()?;
                let (sec, ms) = Self::parse_seconds_ms(parts[1])?;
                total_ms = m * 60_000 + sec * 1000 + ms;
            }
            _ => return None,
        }
        Some(Duration::from_millis(total_ms))
    }

    fn parse_seconds_ms(s: &str) -> Option<(u64, u64)> {
        if let Some(dot_pos) = s.find('.') {
            let sec: u64 = s[..dot_pos].parse().ok()?;
            let ms_str = &s[dot_pos+1..];
            let ms_padded = format!("{:0<3}", ms_str);
            let ms: u64 = ms_padded[..3].parse().ok()?;
            Some((sec, ms))
        } else {
            let sec: u64 = s.parse().ok()?;
            Some((sec, 0))
        }
    }

    fn parse_style_block(&self, lines: &[String]) -> Option<(Vec<VttStyle>, usize)> {
        // STYLE\n...CSS...\n(blank line)
        let mut i = 0;
        let mut styles = Vec::new();
        if !lines[0].trim_start().starts_with("STYLE") {
            return None;
        }
        i = 1;
        // Collect until blank line
        let mut css = String::new();
        while i < lines.len() && !lines[i].trim().is_empty() {
            css.push_str(&lines[i]);
            css.push('\n');
            i += 1;
        }

        // Simple parser: each "selector { ... }"
        let mut chars = css.chars().peekable();
        let mut current_selector = String::new();
        let mut in_block = false;
        let mut current_props = String::new();
        while let Some(c) = chars.next() {
            if c == '{' {
                in_block = true;
                current_selector = current_selector.trim().to_string();
                current_props.clear();
            } else if c == '}' {
                let mut style = VttStyle {
                    selector: current_selector.clone(),
                    color: None,
                    background: None,
                    bold: false,
                    italic: false,
                    underline: false,
                };
                for prop in current_props.split(';') {
                    let p = prop.trim();
                    if let Some(v) = p.strip_prefix("color:") {
                        style.color = Some(v.trim().to_string());
                    } else if let Some(v) = p.strip_prefix("background:") {
                        style.background = Some(v.trim().to_string());
                    } else if p == "font-weight: bold" {
                        style.bold = true;
                    } else if p == "font-style: italic" {
                        style.italic = true;
                    } else if p.contains("underline") {
                        style.underline = true;
                    }
                }
                styles.push(style);
                current_selector.clear();
                in_block = false;
            } else if in_block {
                current_props.push(c);
            } else {
                current_selector.push(c);
            }
        }
        Some((styles, i))
    }

    pub fn cue_at(&self, time: Duration) -> Option<&VttCue> {
        self.cues.iter().find(|c| c.contains(time))
    }

    pub fn total_duration(&self) -> Duration {
        self.cues.iter().map(|c| c.end).max().unwrap_or(Duration::ZERO)
    }
}

impl Default for WebVtt {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vtt_align() {
        assert_eq!(VttAlign::from_str("center"), VttAlign::Center);
        assert_eq!(VttAlign::from_str("start"), VttAlign::Start);
    }

    #[test]
    fn test_vtt_cue_new() {
        let cue = VttCue::new(Duration::from_secs(1), Duration::from_secs(2), "hello");
        assert_eq!(cue.start, Duration::from_secs(1));
        assert_eq!(cue.duration(), Duration::from_secs(1));
    }

    #[test]
    fn test_vtt_cue_contains() {
        let cue = VttCue::new(Duration::from_secs(1), Duration::from_secs(2), "x");
        assert!(cue.contains(Duration::from_secs(1)));
        assert!(cue.contains(Duration::from_millis(1500)));
        assert!(!cue.contains(Duration::from_secs(3)));
    }

    #[test]
    fn test_parse_timestamp_2part() {
        let d = WebVtt::parse_timestamp("01:30.500").unwrap();
        assert_eq!(d, Duration::from_millis(90_500));
    }

    #[test]
    fn test_parse_timestamp_3part() {
        let d = WebVtt::parse_timestamp("01:30:15.250").unwrap();
        assert_eq!(d, Duration::from_millis(5415_250));
    }

    #[test]
    fn test_parse_timestamp_no_ms() {
        let d = WebVtt::parse_timestamp("00:30").unwrap();
        assert_eq!(d, Duration::from_secs(30));
    }

    #[test]
    fn test_parse_timestamp_line() {
        let (s, e) = WebVtt::parse_timestamp_line("00:00:01.000 --> 00:00:05.000").unwrap();
        assert_eq!(s, Duration::from_secs(1));
        assert_eq!(e, Duration::from_secs(5));
    }

    #[test]
    fn test_parse_timestamp_line_with_settings() {
        let (s, e) = WebVtt::parse_timestamp_line("00:00:01.000 --> 00:00:05.000 align:center line:50%").unwrap();
        assert_eq!(s, Duration::from_secs(1));
        assert_eq!(e, Duration::from_secs(5));
    }

    #[test]
    fn test_webvtt_new() {
        let v = WebVtt::new();
        assert!(v.cues.is_empty());
    }

    #[test]
    fn test_webvtt_invalid() {
        let mut v = WebVtt::new();
        let r = v.parse("not a vtt file");
        assert!(r.is_err());
    }

    #[test]
    fn test_parse_simple() {
        let mut v = WebVtt::new();
        let content = r#"WEBVTT

00:00:01.000 --> 00:00:05.000
Hello world

00:00:06.000 --> 00:00:10.000
Second cue
with multiple lines
"#;
        v.parse(content).unwrap();
        assert_eq!(v.cues.len(), 2);
        assert_eq!(v.cues[0].text, "Hello world");
        assert_eq!(v.cues[1].text, "Second cue\nwith multiple lines");
    }

    #[test]
    fn test_parse_with_identifier() {
        let mut v = WebVtt::new();
        let content = r#"WEBVTT

cue-1
00:00:01.000 --> 00:00:05.000
First cue
"#;
        v.parse(content).unwrap();
        assert_eq!(v.cues.len(), 1);
        assert_eq!(v.cues[0].identifier, Some("cue-1".to_string()));
    }

    #[test]
    fn test_parse_with_settings() {
        let mut v = WebVtt::new();
        let content = r#"WEBVTT

00:00:01.000 --> 00:00:05.000 align:end line:80% position:25% size:50%
Aligned cue
"#;
        v.parse(content).unwrap();
        let cue = &v.cues[0];
        assert_eq!(cue.align, VttAlign::End);
        assert_eq!(cue.line, 80);
        assert_eq!(cue.position, 25);
        assert_eq!(cue.size, 50);
    }

    #[test]
    fn test_cue_at() {
        let mut v = WebVtt::new();
        let content = r#"WEBVTT

00:00:01.000 --> 00:00:05.000
First

00:00:06.000 --> 00:00:10.000
Second
"#;
        v.parse(content).unwrap();
        let cue = v.cue_at(Duration::from_secs(7));
        assert!(cue.is_some());
        assert_eq!(cue.unwrap().text, "Second");
    }

    #[test]
    fn test_cue_at_none() {
        let mut v = WebVtt::new();
        let content = r#"WEBVTT

00:00:01.000 --> 00:00:05.000
First
"#;
        v.parse(content).unwrap();
        let cue = v.cue_at(Duration::from_secs(10));
        assert!(cue.is_none());
    }

    #[test]
    fn test_total_duration() {
        let mut v = WebVtt::new();
        let content = r#"WEBVTT

00:00:01.000 --> 00:00:05.000
First

00:00:08.000 --> 00:00:15.000
Last
"#;
        v.parse(content).unwrap();
        assert_eq!(v.total_duration(), Duration::from_secs(15));
    }

    #[test]
    fn test_parse_style_block() {
        let mut v = WebVtt::new();
        let content = r#"WEBVTT

STYLE
::cue {
  color: yellow;
  background: black;
  font-weight: bold;
  font-style: italic;
}

00:00:01.000 --> 00:00:05.000
Styled cue
"#;
        v.parse(content).unwrap();
        assert!(!v.styles.is_empty());
        let style = &v.styles[0];
        assert_eq!(style.color, Some("yellow".to_string()));
        assert_eq!(style.background, Some("black".to_string()));
        assert!(style.bold);
        assert!(style.italic);
    }

    #[test]
    fn test_parse_vertical() {
        let mut v = WebVtt::new();
        let content = r#"WEBVTT

00:00:01.000 --> 00:00:05.000 vertical:rl
Vertical text
"#;
        v.parse(content).unwrap();
        assert!(v.cues[0].vertical);
    }

    #[test]
    fn test_parse_note_block() {
        let mut v = WebVtt::new();
        let content = r#"WEBVTT

NOTE This is a comment

00:00:01.000 --> 00:00:05.000
Hello
"#;
        v.parse(content).unwrap();
        assert_eq!(v.cues.len(), 1);
    }
}
