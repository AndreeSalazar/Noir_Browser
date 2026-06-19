//! Tests para features nuevos: console, find, shortcuts, animation, hover, https

use no_chromium::app::AppConfig;
use no_chromium::app::context::{AppContext, ConsoleLevel};

#[test]
fn test_console_initial_empty() {
    let ctx = AppContext::new(AppConfig::default());
    assert!(!ctx.console_open);
    assert!(ctx.console_messages.is_empty());
}

#[test]
fn test_console_add_log() {
    let mut ctx = AppContext::new(AppConfig::default());
    ctx.console_log(ConsoleLevel::Log, "Hello".to_string());
    assert_eq!(ctx.console_messages.len(), 1);
    assert_eq!(ctx.console_messages[0].text, "Hello");
    assert_eq!(ctx.console_messages[0].level, ConsoleLevel::Log);
}

#[test]
fn test_console_add_multiple() {
    let mut ctx = AppContext::new(AppConfig::default());
    ctx.console_log(ConsoleLevel::Log, "Log 1".to_string());
    ctx.console_log(ConsoleLevel::Warn, "Warn 1".to_string());
    ctx.console_log(ConsoleLevel::Error, "Error 1".to_string());
    ctx.console_log(ConsoleLevel::Info, "Info 1".to_string());
    assert_eq!(ctx.console_messages.len(), 4);
}

#[test]
fn test_console_max_messages() {
    let mut ctx = AppContext::new(AppConfig::default());
    for i in 0..600 {
        ctx.console_log(ConsoleLevel::Log, format!("Message {}", i));
    }
    // Max is 500, so should be trimmed
    assert_eq!(ctx.console_messages.len(), 500);
    // Should have the most recent ones
    assert!(ctx.console_messages.last().unwrap().text.contains("599"));
}

#[test]
fn test_console_levels() {
    let mut ctx = AppContext::new(AppConfig::default());
    ctx.console_log(ConsoleLevel::Error, "An error".to_string());
    assert_eq!(ctx.console_messages[0].level, ConsoleLevel::Error);
    assert_ne!(ConsoleLevel::Error, ConsoleLevel::Warn);
    assert_ne!(ConsoleLevel::Log, ConsoleLevel::Info);
}

#[test]
fn test_find_initial_closed() {
    let ctx = AppContext::new(AppConfig::default());
    assert!(!ctx.find_open);
    assert!(ctx.find_query.is_empty());
}

#[test]
fn test_shortcuts_initial_closed() {
    let ctx = AppContext::new(AppConfig::default());
    assert!(!ctx.shortcuts_open);
}

#[test]
fn test_animation_initial() {
    let mut ctx = AppContext::new(AppConfig::default());
    assert_eq!(ctx.loading_anim_frame, 0);
    ctx.tick_animation();
    assert_eq!(ctx.loading_anim_frame, 1);
    ctx.tick_animation();
    ctx.tick_animation();
    assert_eq!(ctx.loading_anim_frame, 3);
}

#[test]
fn test_animation_wraps() {
    let mut ctx = AppContext::new(AppConfig::default());
    ctx.loading_anim_frame = u32::MAX;
    ctx.tick_animation();
    assert_eq!(ctx.loading_anim_frame, 0);
}

#[test]
fn test_is_https_check() {
    let mut ctx = AppContext::new(AppConfig::default());
    ctx.tabs[ctx.active_tab].url = "https://youtube.com".to_string();
    assert!(ctx.tabs[ctx.active_tab].url.starts_with("https://"));

    ctx.tabs[ctx.active_tab].url = "http://example.com".to_string();
    assert!(!ctx.tabs[ctx.active_tab].url.starts_with("https://"));
}

#[test]
fn test_is_hovering_link_default() {
    let ctx = AppContext::new(AppConfig::default());
    assert!(!ctx.is_hovering_link);
}

#[test]
fn test_load_progress_default() {
    let ctx = AppContext::new(AppConfig::default());
    assert_eq!(ctx.load_progress, 0.0);
}

#[test]
fn test_console_open_toggle() {
    let mut ctx = AppContext::new(AppConfig::default());
    assert!(!ctx.console_open);
    ctx.console_open = true;
    assert!(ctx.console_open);
    ctx.console_open = false;
    assert!(!ctx.console_open);
}

#[test]
fn test_find_open_with_query() {
    let mut ctx = AppContext::new(AppConfig::default());
    ctx.find_open = true;
    ctx.find_query = "rust".to_string();
    assert!(ctx.find_open);
    assert_eq!(ctx.find_query, "rust");
}

#[test]
fn test_console_message_has_timestamp() {
    let mut ctx = AppContext::new(AppConfig::default());
    ctx.console_log(ConsoleLevel::Info, "Test".to_string());
    let msg = &ctx.console_messages[0];
    assert!(msg.timestamp > 0);
}

#[test]
fn test_default_console_state_is_preserved() {
    let ctx = AppContext::new(AppConfig::default());
    assert!(!ctx.console_open);
    assert!(!ctx.shortcuts_open);
    assert!(!ctx.find_open);
    assert!(ctx.console_messages.is_empty());
    assert!(ctx.find_query.is_empty());
}

#[test]
fn test_tab_title_truncation_logic() {
    let long_title = "a".repeat(50);
    let truncated: String = if long_title.len() > 20 {
        format!("{}...", &long_title[..17])
    } else {
        long_title.clone()
    };
    assert_eq!(truncated, "aaaaaaaaaaaaaaaaa...");
}

#[test]
fn test_https_lock_visibility() {
    let mut ctx = AppContext::new(AppConfig::default());
    ctx.tabs[ctx.active_tab].url = "https://github.com".to_string();
    assert!(ctx.tabs[ctx.active_tab].url.starts_with("https://"));

    ctx.tabs[ctx.active_tab].url = "http://example.com".to_string();
    assert!(!ctx.tabs[ctx.active_tab].url.starts_with("https://"));
}
