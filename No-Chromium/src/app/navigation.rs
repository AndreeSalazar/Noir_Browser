use super::state::NoirApp;

impl NoirApp {
    pub fn resolve_url(&self) -> String {
        let input = self.url_bar.trim();
        if input.starts_with("http://") || input.starts_with("https://") {
            return input.to_string();
        }

        if input.contains('.') && !input.contains(' ') {
            return format!("https://{}", input);
        }

        Self::resolve_search_query(input)
    }

    fn resolve_search_query(input: &str) -> String {
        let lower = input.to_lowercase();
        let parts: Vec<&str> = lower.splitn(2, ' ').collect();
        let query = if parts.len() > 1 { parts[1] } else { "" };

        match parts[0] {
            "yt" | "youtube" => format!("https://www.youtube.com/results?search_query={}", query.replace(' ', "+")),
            "gg" | "google" => format!("https://www.google.com/search?q={}", query.replace(' ', "+")),
            "gh" | "github" => format!("https://github.com/search?q={}", query.replace(' ', "+")),
            "ddg" | "duckduckgo" | "duck" => format!("https://duckduckgo.com/?q={}", query.replace(' ', "+")),
            "wiki" | "wikipedia" => format!("https://en.wikipedia.org/wiki/Special:Search?search={}", query.replace(' ', "+")),
            "reddit" => format!("https://www.reddit.com/search/?q={}", query.replace(' ', "+")),
            "so" | "stackoverflow" => format!("https://stackoverflow.com/search?q={}", query.replace(' ', "+")),
            "mdn" => format!("https://developer.mozilla.org/en-US/search?q={}", query.replace(' ', "+")),
            "crates" => format!("https://crates.io/search?q={}", query.replace(' ', "+")),
            "docs" | "docsrs" => format!("https://docs.rs/releases/search?query={}", query.replace(' ', "+")),
            "npm" => format!("https://www.npmjs.com/search?q={}", query.replace(' ', "+")),
            _ => format!("https://duckduckgo.com/?q={}", input.replace(' ', "+")),
        }
    }
}
