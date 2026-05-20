use std::error::Error;
use super::JsEngine;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChallengeType {
    Anubis,
    Cloudflare,
}

pub fn detect_challenge(html: &str) -> Option<ChallengeType> {
    if html.contains("anubis-pow") || html.contains("anubis-challenge") || html.contains("window.anubisChallenge") {
        Some(ChallengeType::Anubis)
    } else if html.contains("cf-challenge") || html.contains("challenges.cloudflare.com") || html.contains("cf-turnstile") {
        Some(ChallengeType::Cloudflare)
    } else {
        None
    }
}

pub fn solve_challenge(html: &str, _url: &str) -> Result<String, String> {
    let scripts = extract_scripts(html);
    let mut engine = JsEngine::new();
    
    // Set up standard mock globals so scripts expecting a browser context don't throw ReferenceError
    let init_script = "
        var window = {};
        var document = { cookie: '' };
        var console = { log: function() {} };
    ";
    engine.run_sandboxed(init_script)?;

    for script in scripts {
        // Execute the script sandboxed
        let _ = engine.run_sandboxed(&script);
    }

    // Attempt to extract the resulting cookie/token
    if let Ok(cookie_val) = engine.run_sandboxed("document.cookie") {
        if let Ok(js_str) = cookie_val.to_string(&mut engine.context) {
            let rust_str = js_str.to_std_string_escaped();
            if !rust_str.is_empty() {
                return Ok(rust_str);
            }
        }
    }

    if let Ok(token_val) = engine.run_sandboxed("window.anubisToken || window.cfToken") {
        if let Ok(js_str) = token_val.to_string(&mut engine.context) {
            let rust_str = js_str.to_std_string_escaped();
            if !rust_str.is_empty() && rust_str != "undefined" {
                return Ok(format!("cf_clearance={}", rust_str));
            }
        }
    }

    Err("Could not solve challenge or extract token".to_string())
}

pub fn extract_scripts(html: &str) -> Vec<String> {
    let mut scripts = Vec::new();
    let mut input = html;
    while let Some(start_tag) = input.find("<script") {
        let tag_end_search = &input[start_tag..];
        if let Some(tag_end_relative) = tag_end_search.find('>') {
            let content_start = start_tag + tag_end_relative + 1;
            let search_end = &input[content_start..];
            if let Some(end_tag_relative) = search_end.find("</script>") {
                let content_end = content_start + end_tag_relative;
                let script_content = &input[content_start..content_end];
                scripts.push(script_content.to_string());
                input = &input[content_end + 9..];
            } else {
                break;
            }
        } else {
            break;
        }
    }
    scripts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_anubis_challenge() {
        let html = r#"<html><head><div class="anubis-pow">Solve PoW</div></head></html>"#;
        assert_eq!(detect_challenge(html), Some(ChallengeType::Anubis));
    }

    #[test]
    fn test_detect_cloudflare_challenge() {
        let html = r#"<html><head><script src="https://challenges.cloudflare.com/turnstile/v0/api.js"></script></head></html>"#;
        assert_eq!(detect_challenge(html), Some(ChallengeType::Cloudflare));
    }

    #[test]
    fn test_solve_pow_challenge() {
        let html = r#"
            <html>
            <head>
            <script>
                var salt = "securesalt_";
                // A simulated challenge: find nonce where sum of char codes is multiple of 42
                var nonce = 0;
                while (true) {
                    var cand = salt + nonce;
                    var sum = 0;
                    for (var i = 0; i < cand.length; i++) {
                        sum += cand.charCodeAt(i);
                    }
                    if (sum % 42 === 0) {
                        break;
                    }
                    nonce++;
                }
                document.cookie = "anubis_token=" + nonce;
            </script>
            </head>
            </html>
        "#;

        let result = solve_challenge(html, "http://example.com").expect("should solve challenge");
        assert!(result.starts_with("anubis_token="));
        println!("Solved token: {}", result);
    }
}
