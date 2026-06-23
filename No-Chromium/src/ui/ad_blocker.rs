//! Ad Blocker - Tracker blocking, ad filtering (Brave-style)
//!
//! Blocklist de dominios conocidos de ads/trackers
//! Network filtering, cosmetic filtering, script blocking

use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlockCategory {
    Ad,
    Tracker,
    Analytics,
    Social,
    Cryptominer,
    Fingerprint,
    Malware,
}

impl BlockCategory {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "ad" | "ads" => Self::Ad,
            "tracker" | "tracking" => Self::Tracker,
            "analytics" => Self::Analytics,
            "social" => Self::Social,
            "cryptominer" | "miner" => Self::Cryptominer,
            "fingerprint" => Self::Fingerprint,
            "malware" => Self::Malware,
            _ => Self::Ad,
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            Self::Ad => "ad",
            Self::Tracker => "tracker",
            Self::Analytics => "analytics",
            Self::Social => "social",
            Self::Cryptominer => "cryptominer",
            Self::Fingerprint => "fingerprint",
            Self::Malware => "malware",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BlockAction {
    Block,           // No cargar el recurso
    BlockCookie,     // Bloquear cookies
    BlockScript,     // Bloquear scripts
    BlockCSS,        // Bloquear CSS
    CosmeticOnly,    // Solo ocultar visualmente
    NoOp,            // Permitir
}

impl BlockAction {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "block" => Self::Block,
            "block-cookie" => Self::BlockCookie,
            "block-script" => Self::BlockScript,
            "block-css" => Self::BlockCSS,
            "cosmetic" => Self::CosmeticOnly,
            _ => Self::NoOp,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BlockRule {
    pub pattern: String,
    pub category: BlockCategory,
    pub action: BlockAction,
    pub domains: Vec<String>,  // empty = all domains
}

impl BlockRule {
    pub fn new(pattern: &str, category: BlockCategory, action: BlockAction) -> Self {
        Self {
            pattern: pattern.to_string(),
            category,
            action,
            domains: Vec::new(),
        }
    }

    pub fn matches(&self, url: &str) -> bool {
        if self.domains.is_empty() {
            return url.contains(&self.pattern);
        }
        for domain in &self.domains {
            if url.contains(domain) && url.contains(&self.pattern) {
                return true;
            }
        }
        false
    }
}

pub struct AdBlocker {
    pub enabled: bool,
    pub rules: Vec<BlockRule>,
    pub blocked_count: u32,
    pub cosmetic_hide_count: u32,
    pub blocklist: HashSet<String>,
    pub whitelist: HashSet<String>,
    pub stats: BlockStats,
    pub strict_mode: bool,
}

#[derive(Debug, Clone, Default)]
pub struct BlockStats {
    pub total_requests: u64,
    pub blocked_requests: u64,
    pub ads_blocked: u32,
    pub trackers_blocked: u32,
    pub analytics_blocked: u32,
    pub scripts_blocked: u32,
}

impl AdBlocker {
    pub fn new() -> Self {
        let mut ab = Self {
            enabled: true,
            rules: Vec::new(),
            blocked_count: 0,
            cosmetic_hide_count: 0,
            blocklist: HashSet::new(),
            whitelist: HashSet::new(),
            stats: BlockStats::default(),
            strict_mode: false,
        };
        ab.add_default_rules();
        ab
    }

    fn add_default_rules(&mut self) {
        // Anuncios
        self.add_domain("doubleclick.net", BlockCategory::Ad, BlockAction::Block);
        self.add_domain("googleadservices.com", BlockCategory::Ad, BlockAction::Block);
        self.add_domain("googlesyndication.com", BlockCategory::Ad, BlockAction::Block);
        self.add_domain("adservice.google.com", BlockCategory::Ad, BlockAction::Block);
        self.add_domain("amazon-adsystem.com", BlockCategory::Ad, BlockAction::Block);
        self.add_domain("adsystem.amazon.com", BlockCategory::Ad, BlockAction::Block);
        self.add_domain("adnxs.com", BlockCategory::Ad, BlockAction::Block);
        self.add_domain("adsrvr.org", BlockCategory::Ad, BlockAction::Block);
        self.add_domain("adform.net", BlockCategory::Ad, BlockAction::Block);
        self.add_domain("criteo.com", BlockCategory::Ad, BlockAction::Block);
        self.add_domain("outbrain.com", BlockCategory::Ad, BlockAction::Block);
        self.add_domain("taboola.com", BlockCategory::Ad, BlockAction::Block);
        self.add_pattern("/ads/", BlockCategory::Ad, BlockAction::Block);
        self.add_pattern("/adsbygoogle", BlockCategory::Ad, BlockAction::Block);
        self.add_pattern("/advertising", BlockCategory::Ad, BlockAction::Block);
        // Trackers
        self.add_domain("google-analytics.com", BlockCategory::Tracker, BlockAction::Block);
        self.add_domain("googletagmanager.com", BlockCategory::Tracker, BlockAction::Block);
        self.add_domain("facebook.net", BlockCategory::Tracker, BlockAction::Block);
        self.add_domain("connect.facebook.net", BlockCategory::Tracker, BlockAction::Block);
        self.add_domain("twitter.com/i/adsct", BlockCategory::Tracker, BlockAction::Block);
        self.add_domain("scorecardresearch.com", BlockCategory::Tracker, BlockAction::Block);
        self.add_domain("hotjar.com", BlockCategory::Tracker, BlockAction::Block);
        self.add_domain("mixpanel.com", BlockCategory::Tracker, BlockAction::Block);
        self.add_domain("segment.io", BlockCategory::Tracker, BlockAction::Block);
        self.add_domain("amplitude.com", BlockCategory::Tracker, BlockAction::Block);
        self.add_domain("fullstory.com", BlockCategory::Tracker, BlockAction::Block);
        self.add_domain("mouseflow.com", BlockCategory::Tracker, BlockAction::Block);
        // Analytics
        self.add_domain("newrelic.com", BlockCategory::Analytics, BlockAction::Block);
        self.add_domain("nr-data.net", BlockCategory::Analytics, BlockAction::Block);
        self.add_domain("datadoghq.com", BlockCategory::Analytics, BlockAction::Block);
        // Cryptominers
        self.add_domain("coinhive.com", BlockCategory::Cryptominer, BlockAction::Block);
        self.add_domain("crypto-loot.com", BlockCategory::Cryptominer, BlockAction::Block);
        self.add_domain("minero.cc", BlockCategory::Cryptominer, BlockAction::Block);
        // Fingerprinting
        self.add_domain("fpjs.io", BlockCategory::Fingerprint, BlockAction::Block);
        // Malicious
        self.add_domain("malware-site.com", BlockCategory::Malware, BlockAction::Block);
    }

    pub fn add_domain(&mut self, domain: &str, category: BlockCategory, action: BlockAction) {
        self.blocklist.insert(domain.to_string());
        self.rules.push(BlockRule::new(domain, category, action));
    }

    pub fn add_pattern(&mut self, pattern: &str, category: BlockCategory, action: BlockAction) {
        self.rules.push(BlockRule::new(pattern, category, action));
    }

    pub fn add_whitelist(&mut self, domain: &str) {
        self.whitelist.insert(domain.to_string());
    }

    pub fn should_block(&mut self, url: &str) -> BlockAction {
        self.stats.total_requests += 1;
        if !self.enabled {
            return BlockAction::NoOp;
        }
        // Check whitelist first
        for w in &self.whitelist {
            if url.contains(w) {
                return BlockAction::NoOp;
            }
        }
        // Check rules
        for rule in &self.rules {
            if rule.matches(url) {
                self.stats.blocked_requests += 1;
                match rule.category {
                    BlockCategory::Ad => self.stats.ads_blocked += 1,
                    BlockCategory::Tracker => self.stats.trackers_blocked += 1,
                    BlockCategory::Analytics => self.stats.analytics_blocked += 1,
                    _ => {}
                }
                if matches!(rule.action, BlockAction::BlockScript) {
                    self.stats.scripts_blocked += 1;
                }
                if rule.action == BlockAction::CosmeticOnly {
                    self.cosmetic_hide_count += 1;
                } else {
                    self.blocked_count += 1;
                }
                return rule.action;
            }
        }
        BlockAction::NoOp
    }

    pub fn is_blocked(&self, url: &str) -> bool {
        for w in &self.whitelist {
            if url.contains(w) {
                return false;
            }
        }
        for rule in &self.rules {
            if rule.matches(url) {
                return rule.action != BlockAction::CosmeticOnly;
            }
        }
        false
    }

    pub fn reset_stats(&mut self) {
        self.stats = BlockStats::default();
        self.blocked_count = 0;
        self.cosmetic_hide_count = 0;
    }

    pub fn total_rules(&self) -> usize {
        self.rules.len()
    }

    pub fn enabled_categories(&self) -> HashMap<BlockCategory, u32> {
        let mut map = HashMap::new();
        for rule in &self.rules {
            *map.entry(rule.category).or_insert(0) += 1;
        }
        map
    }
}

impl Default for AdBlocker {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_from_str() {
        assert_eq!(BlockCategory::from_str("ad"), BlockCategory::Ad);
        assert_eq!(BlockCategory::from_str("tracker"), BlockCategory::Tracker);
        assert_eq!(BlockCategory::from_str("ads"), BlockCategory::Ad);
    }

    #[test]
    fn test_category_to_str() {
        assert_eq!(BlockCategory::Ad.to_str(), "ad");
        assert_eq!(BlockCategory::Tracker.to_str(), "tracker");
    }

    #[test]
    fn test_action_from_str() {
        assert_eq!(BlockAction::from_str("block"), BlockAction::Block);
        assert_eq!(BlockAction::from_str("cosmetic"), BlockAction::CosmeticOnly);
    }

    #[test]
    fn test_rule_new() {
        let r = BlockRule::new("example.com", BlockCategory::Ad, BlockAction::Block);
        assert_eq!(r.pattern, "example.com");
    }

    #[test]
    fn test_rule_matches() {
        let r = BlockRule::new("doubleclick.net", BlockCategory::Ad, BlockAction::Block);
        assert!(r.matches("https://doubleclick.net/ad.js"));
        assert!(!r.matches("https://example.com"));
    }

    #[test]
    fn test_rule_with_domains() {
        let mut r = BlockRule::new("/ads/", BlockCategory::Ad, BlockAction::Block);
        r.domains.push("youtube.com".to_string());
        assert!(r.matches("https://youtube.com/ads/script.js"));
        assert!(!r.matches("https://youtube.com/main.js"));
    }

    #[test]
    fn test_blocker_new() {
        let ab = AdBlocker::new();
        assert!(ab.enabled);
        assert!(ab.total_rules() > 0);
    }

    #[test]
    fn test_blocker_blocks_doubleclick() {
        let mut ab = AdBlocker::new();
        let action = ab.should_block("https://doubleclick.net/ad.js");
        assert_eq!(action, BlockAction::Block);
    }

    #[test]
    fn test_blocker_blocks_tracker() {
        let mut ab = AdBlocker::new();
        let action = ab.should_block("https://www.google-analytics.com/ga.js");
        assert_eq!(action, BlockAction::Block);
    }

    #[test]
    fn test_blocker_allows_normal() {
        let mut ab = AdBlocker::new();
        let action = ab.should_block("https://example.com/page.html");
        assert_eq!(action, BlockAction::NoOp);
    }

    #[test]
    fn test_blocker_whitelist() {
        let mut ab = AdBlocker::new();
        ab.add_whitelist("example.com");
        let action = ab.should_block("https://example.com/ads/script.js");
        assert_eq!(action, BlockAction::NoOp);
    }

    #[test]
    fn test_blocker_disabled() {
        let mut ab = AdBlocker::new();
        ab.enabled = false;
        let action = ab.should_block("https://doubleclick.net/ad.js");
        assert_eq!(action, BlockAction::NoOp);
    }

    #[test]
    fn test_blocker_stats() {
        let mut ab = AdBlocker::new();
        ab.should_block("https://doubleclick.net/ad.js");
        ab.should_block("https://www.google-analytics.com/ga.js");
        ab.should_block("https://example.com/page.html");
        assert_eq!(ab.stats.blocked_requests, 2);
        assert_eq!(ab.stats.ads_blocked, 1);
        assert_eq!(ab.stats.trackers_blocked, 1);
    }

    #[test]
    fn test_blocker_pattern_block() {
        let mut ab = AdBlocker::new();
        let action = ab.should_block("https://example.com/ads/banner.js");
        assert_eq!(action, BlockAction::Block);
    }

    #[test]
    fn test_blocker_is_blocked() {
        let ab = AdBlocker::new();
        assert!(ab.is_blocked("https://doubleclick.net/ad.js"));
        assert!(!ab.is_blocked("https://example.com/page.html"));
    }

    #[test]
    fn test_blocker_add_pattern() {
        let mut ab = AdBlocker::new();
        ab.add_pattern("my-ad-network", BlockCategory::Ad, BlockAction::Block);
        let action = ab.should_block("https://my-ad-network.com/script.js");
        assert_eq!(action, BlockAction::Block);
    }

    #[test]
    fn test_blocker_reset_stats() {
        let mut ab = AdBlocker::new();
        ab.should_block("https://doubleclick.net/ad.js");
        ab.reset_stats();
        assert_eq!(ab.stats.blocked_requests, 0);
    }

    #[test]
    fn test_blocker_categories() {
        let ab = AdBlocker::new();
        let cats = ab.enabled_categories();
        assert!(cats.contains_key(&BlockCategory::Ad));
        assert!(cats.contains_key(&BlockCategory::Tracker));
    }

    #[test]
    fn test_blocker_cosmetic() {
        let mut ab = AdBlocker::new();
        ab.add_pattern("cosmetic-only.com", BlockCategory::Ad, BlockAction::CosmeticOnly);
        let action = ab.should_block("https://cosmetic-only.com/thing.js");
        assert_eq!(action, BlockAction::CosmeticOnly);
    }

    #[test]
    fn test_blocker_count() {
        let ab = AdBlocker::new();
        assert!(ab.total_rules() >= 30);
    }
}
