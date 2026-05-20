mod app;
mod browser;
mod js_engine;
mod layout;
mod media;
mod parsers;
mod render;
mod runtime;
mod ui;
mod vulkan_engine;
#[allow(dead_code)]
#[path = "generated_rust/web_platform_data.rs"]
pub(crate) mod web_platform_data;

fn main() {
    app::run();
}
