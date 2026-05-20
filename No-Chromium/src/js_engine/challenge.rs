use super::JsEngine;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChallengeType {
    Anubis,
    Cloudflare,
}

pub fn detect_challenge(html: &str) -> Option<ChallengeType> {
    if html.contains("anubis-pow") 
        || html.contains("anubis-challenge") 
        || html.contains("window.anubisChallenge") 
        || html.contains("anubis_challenge")
        || html.contains("TecharoHQ/anubis") 
    {
        Some(ChallengeType::Anubis)
    } else if html.contains("cf-challenge") 
        || html.contains("challenges.cloudflare.com") 
        || html.contains("cf-turnstile") 
    {
        Some(ChallengeType::Cloudflare)
    } else {
        None
    }
}

pub async fn solve_challenge(html: &str, url: &str) -> Result<String, String> {
    if let Some(ChallengeType::Anubis) = detect_challenge(html) {
        if let Some((id, response_hash, nonce)) = solve_anubis_pow(html) {
            let base_prefix = get_anubis_base_prefix(html);
            let mut parsed_url = url::Url::parse(url).map_err(|e| e.to_string())?;
            let verify_path = format!("{}/.within.website/x/cmd/anubis/api/pass-challenge", base_prefix);
            parsed_url.set_path(&verify_path);
            parsed_url.set_query(None);
            
            parsed_url.query_pairs_mut()
                .append_pair("id", &id)
                .append_pair("response", &response_hash)
                .append_pair("nonce", &nonce.to_string())
                .append_pair("redir", url)
                .append_pair("elapsedTime", "120");
            
            let verify_url = parsed_url.to_string();
            println!("[JS Engine] Submitting Anubis solution to {}", verify_url);
            
            let client = reqwest::Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .map_err(|e| e.to_string())?;
            
            let verify_resp = client.get(&verify_url)
                .header(reqwest::header::USER_AGENT, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
                .send()
                .await
                .map_err(|e| e.to_string())?;
            
            let mut cookies = Vec::new();
            for cookie_val in verify_resp.headers().get_all(reqwest::header::SET_COOKIE) {
                if let Ok(cookie_str) = cookie_val.to_str() {
                    let cookie_part = if let Some(idx) = cookie_str.find(';') {
                        &cookie_str[..idx]
                    } else {
                        cookie_str
                    };
                    cookies.push(cookie_part.to_string());
                }
            }
            
            if cookies.is_empty() {
                return Err("No Set-Cookie header returned in pass-challenge response".to_string());
            }
            
            let cookie_header = cookies.join("; ");
            return Ok(cookie_header);
        }
    }

    // Default to the JavaScript execution engine for inline challenge scripts
    let scripts = extract_scripts(html);
    let mut engine = JsEngine::new();
    
    let init_script = "
        var window = {};
        var document = { cookie: '' };
        var console = { log: function() {} };
    ";
    engine.run_sandboxed(init_script)?;

    for script in scripts {
        let _ = engine.run_sandboxed(&script);
    }

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

pub fn solve_anubis_pow(html: &str) -> Option<(String, String, u64)> {
    let challenge_tag = "<script id=\"anubis_challenge\" type=\"application/json\">";
    let start_idx = html.find(challenge_tag)?;
    let content_start = start_idx + challenge_tag.len();
    let end_idx = html[content_start..].find("</script>")?;
    let json_str = &html[content_start..content_start + end_idx];
    
    let json_val: serde_json::Value = serde_json::from_str(json_str).ok()?;
    
    let challenge_obj = json_val.get("challenge")?;
    let id = challenge_obj.get("id")?.as_str()?;
    let random_data = challenge_obj.get("randomData")?.as_str()?;
    let difficulty = challenge_obj.get("difficulty")?.as_u64()? as u32;

    println!("[JS Engine] Solving Anubis PoW: id={}, difficulty={}", id, difficulty);
    
    use sha2::{Sha256, Digest};
    
    let p = (difficulty / 2) as usize;
    let odd = difficulty % 2 != 0;
    
    let mut nonce = 0u64;
    loop {
        let input_str = format!("{}{}", random_data, nonce);
        let mut hasher = Sha256::new();
        hasher.update(input_str.as_bytes());
        let hash_result = hasher.finalize();
        
        let mut valid = true;
        for s in 0..p {
            if hash_result[s] != 0 {
                valid = false;
                break;
            }
        }
        if valid && odd && (hash_result[p] >> 4) != 0 {
            valid = false;
        }
        
        if valid {
            let hash_hex = format!("{:x}", hash_result);
            println!("[JS Engine] Found Anubis solution! nonce={}, hash={}", nonce, hash_hex);
            return Some((id.to_string(), hash_hex, nonce));
        }
        nonce += 1;
        if nonce > 10_000_000 {
            return None;
        }
    }
}

pub fn get_anubis_base_prefix(html: &str) -> String {
    let prefix_tag = "<script id=\"anubis_base_prefix\" type=\"application/json\">";
    if let Some(start_idx) = html.find(prefix_tag) {
        let content_start = start_idx + prefix_tag.len();
        if let Some(end_idx) = html[content_start..].find("</script>") {
            let json_str = &html[content_start..content_start + end_idx];
            if let Ok(serde_json::Value::String(s)) = serde_json::from_str(json_str) {
                return s;
            }
        }
    }
    "".to_string()
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

    #[tokio::test]
    async fn test_solve_pow_challenge() {
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

        let result = solve_challenge(html, "http://example.com").await.expect("should solve challenge");
        assert!(result.starts_with("anubis_token="));
        println!("Solved token: {}", result);
    }
}
