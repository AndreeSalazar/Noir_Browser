pub static GOOGLE_LOGO: &[u8] = include_bytes!("../../assets/pre_cache/googlelogo_color_272x92dp.png");
pub static RUST_LOGO: &[u8] = include_bytes!("../../assets/pre_cache/rust_logo.png");
pub static DUCKDUCKGO_FAVICON: &[u8] = include_bytes!("../../assets/pre_cache/duckduckgo_favicon.png");
pub static YOUTUBE_LOGO: &[u8] = include_bytes!("../../assets/pre_cache/youtube_logo.png");
pub static GITHUB_LOGO: &[u8] = include_bytes!("../../assets/pre_cache/github_logo.png");

pub fn get_pre_cached_assets() -> Vec<(&'static str, &'static [u8])> {
    vec![
        (
            "https://www.google.com/images/branding/googlelogo/2x/googlelogo_color_272x92dp.png",
            GOOGLE_LOGO,
        ),
        (
            "https://www.rust-lang.org/static/images/rust-logo-blk.png",
            RUST_LOGO,
        ),
        (
            "https://duckduckgo.com/favicon.png",
            DUCKDUCKGO_FAVICON,
        ),
        (
            "https://upload.wikimedia.org/wikipedia/commons/3/34/YouTube_logo_%282017%29.png",
            YOUTUBE_LOGO,
        ),
        (
            "https://github.githubassets.com/images/modules/logos_page/GitHub-Mark.png",
            GITHUB_LOGO,
        ),
    ]
}
