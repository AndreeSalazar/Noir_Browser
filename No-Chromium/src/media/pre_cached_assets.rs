//! Pre-cached assets for fast startup

// Placeholder PNG (1x1 transparent pixel)
const PLACEHOLDER_PNG: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A,
    0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
    0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4,
    0x89, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x44, 0x41,
    0x54, 0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00,
    0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00,
    0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE,
    0x42, 0x60, 0x82,
];

pub static GOOGLE_LOGO: &[u8] = PLACEHOLDER_PNG;
pub static RUST_LOGO: &[u8] = PLACEHOLDER_PNG;
pub static DUCKDUCKGO_FAVICON: &[u8] = PLACEHOLDER_PNG;
pub static YOUTUBE_LOGO: &[u8] = PLACEHOLDER_PNG;
pub static GITHUB_LOGO: &[u8] = PLACEHOLDER_PNG;

pub fn get_pre_cached_assets() -> Vec<(&'static str, &'static [u8])> {
    vec![
        ("google_logo", GOOGLE_LOGO),
        ("rust_logo", RUST_LOGO),
        ("duckduckgo_favicon", DUCKDUCKGO_FAVICON),
        ("youtube_logo", YOUTUBE_LOGO),
        ("github_logo", GITHUB_LOGO),
    ]
}

pub fn get_logo_by_name(name: &str) -> Option<&'static [u8]> {
    match name {
        "google" | "google_logo" => Some(GOOGLE_LOGO),
        "rust" | "rust_logo" => Some(RUST_LOGO),
        "duckduckgo" | "ddg" => Some(DUCKDUCKGO_FAVICON),
        "youtube" | "yt" => Some(YOUTUBE_LOGO),
        "github" => Some(GITHUB_LOGO),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assets_exist() {
        assert!(!GOOGLE_LOGO.is_empty());
        assert!(!YOUTUBE_LOGO.is_empty());
    }

    #[test]
    fn test_get_assets() {
        let assets = get_pre_cached_assets();
        assert_eq!(assets.len(), 5);
    }

    #[test]
    fn test_get_by_name() {
        assert!(get_logo_by_name("google").is_some());
        assert!(get_logo_by_name("yt").is_some());
        assert!(get_logo_by_name("unknown").is_none());
    }
}
