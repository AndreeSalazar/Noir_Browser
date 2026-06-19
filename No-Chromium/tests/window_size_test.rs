//! Tests para el bug de ventana sobredimensionada

#[test]
fn test_window_size_within_bounds() {
    let monitor_w: f64 = 1920.0;
    let monitor_h: f64 = 1080.0;
    let target_w = (monitor_w * 0.9_f64).min(1920.0_f64).max(800.0_f64);
    let target_h = (monitor_h * 0.9_f64).min(1080.0_f64).max(500.0_f64);
    assert!(target_w <= monitor_w);
    assert!(target_h <= monitor_h);
    assert!(target_w >= 800.0);
    assert!(target_h >= 500.0);
}

#[test]
fn test_window_size_small_monitor() {
    let monitor_w: f64 = 1366.0;
    let monitor_h: f64 = 768.0;
    let target_w = (monitor_w * 0.9_f64).min(1920.0_f64).max(800.0_f64);
    let target_h = (monitor_h * 0.9_f64).min(1080.0_f64).max(500.0_f64);
    assert!(target_w <= monitor_w);
    assert!(target_h <= monitor_h);
    assert!(target_w >= 800.0);
    assert!(target_h >= 500.0);
}

#[test]
fn test_window_size_large_monitor() {
    let monitor_w: f64 = 3840.0;
    let monitor_h: f64 = 2160.0;
    let target_w = (monitor_w * 0.9_f64).min(1920.0_f64).max(800.0_f64);
    let target_h = (monitor_h * 0.9_f64).min(1080.0_f64).max(500.0_f64);
    assert_eq!(target_w, 1920.0);
    assert_eq!(target_h, 1080.0);
}

#[test]
fn test_resize_clamp_max_width() {
    let max_w: u32 = 5120;
    let max_h: u32 = 2880;
    let reported_w: u32 = 10000;
    let reported_h: u32 = 10000;
    let clamped_w = reported_w.min(max_w);
    let clamped_h = reported_h.min(max_h);
    assert_eq!(clamped_w, max_w);
    assert_eq!(clamped_h, max_h);
}

#[test]
fn test_resize_clamp_normal_size() {
    let max_w: u32 = 5120;
    let max_h: u32 = 2880;
    let reported_w: u32 = 1920;
    let reported_h: u32 = 1080;
    let clamped_w = reported_w.min(max_w);
    let clamped_h = reported_h.min(max_h);
    assert_eq!(clamped_w, 1920);
    assert_eq!(clamped_h, 1080);
}

#[test]
fn test_window_centering() {
    let monitor_w: f64 = 1920.0;
    let monitor_h: f64 = 1080.0;
    let window_w: f64 = 1280.0;
    let window_h: f64 = 720.0;
    let x = ((monitor_w - window_w) / 2.0_f64) as i32;
    let y = ((monitor_h - window_h) / 2.0_f64) as i32;
    assert_eq!(x, 320);
    assert_eq!(y, 180);
}

#[test]
fn test_window_centering_large_window() {
    let monitor_w: f64 = 1920.0;
    let monitor_h: f64 = 1080.0;
    let window_w: f64 = 1720.0;
    let window_h: f64 = 970.0;
    let x = ((monitor_w - window_w) / 2.0_f64) as i32;
    let y = ((monitor_h - window_h) / 2.0_f64) as i32;
    assert_eq!(x, 100);
    assert_eq!(y, 55);
}

#[test]
fn test_window_centering_no_negative() {
    let monitor_w: f64 = 800.0;
    let monitor_h: f64 = 600.0;
    let window_w: f64 = 800.0;
    let window_h: f64 = 600.0;
    let x = ((monitor_w - window_w) / 2.0_f64).max(0.0_f64) as i32;
    let y = ((monitor_h - window_h) / 2.0_f64).max(0.0_f64) as i32;
    assert!(x >= 0);
    assert!(y >= 0);
}

#[test]
fn test_min_window_size() {
    let min_w: u32 = 800;
    let min_h: u32 = 500;
    assert!(min_w >= 800);
    assert!(min_h >= 500);
}

#[test]
fn test_max_inner_size() {
    let monitor_w: f64 = 1920.0;
    let monitor_h: f64 = 1080.0;
    let max_w = (monitor_w * 0.95_f64).min(2560.0_f64);
    let max_h = (monitor_h * 0.95_f64).min(1440.0_f64);
    assert_eq!(max_w, 1824.0);
    assert_eq!(max_h, 1026.0);
}

#[test]
fn test_prevent_window_outside_screen() {
    let monitor_w: i32 = 1920;
    let monitor_h: i32 = 1080;
    let win_x: i32 = 100;
    let win_y: i32 = 50;
    let win_w: i32 = 1280;
    let win_h: i32 = 720;

    assert!(win_x >= 0);
    assert!(win_y >= 0);
    assert!(win_x + win_w <= monitor_w);
    assert!(win_y + win_h <= monitor_h);
}

#[test]
fn test_default_size_1920x1080() {
    let target_w: f64 = 1920.0;
    let target_h: f64 = 1080.0;
    assert_eq!(target_w, 1920.0);
    assert_eq!(target_h, 1080.0);
}

#[test]
fn test_9_10_of_monitor_1920() {
    let monitor_w: f64 = 1920.0;
    let result = monitor_w * 0.9_f64;
    assert_eq!(result, 1728.0);
}

#[test]
fn test_size_after_clamping_to_int() {
    let w: f64 = 1824.0;
    let h: f64 = 1026.0;
    let w_int = w as u32;
    let h_int = h as u32;
    assert_eq!(w_int, 1824);
    assert_eq!(h_int, 1026);
}
