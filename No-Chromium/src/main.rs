//! Noir Browser - Entry Point Principal
//! 
//! Arquitectura Fusionada: Chrome × Tor × Vulkan
//! - Multi-proceso nativo con Tokio
//! - Privacidad por defecto (FPI, anti-fingerprint, disk avoidance)
//! - Renderizado GPU puro con Vulkan 1.3 + Ash (zero-copy, bindless)
//! - Auto-scaling de procesos según memoria disponible
//!
//! 🧬 Filosofía: Velocidad de Chrome + Privacidad de Tor + Vulkan Ultra-Fast

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(dead_code)] // Stubs en desarrollo - se habilitará al integrar módulos

// === MÓDULOS PRINCIPALES ===
mod app;
mod browser;
mod renderer;
mod vulkan_engine;
mod network;
mod parsers;
mod media;
mod utils;
mod js_engine;

#[cfg(feature = "privacy")]
mod privacy;

use std::env;
use tracing::{info, error};

// Reutilizar ProcessModel desde utils (única definición)
use crate::utils::process_model::ProcessModel;

// === CONFIGURACIÓN DE LA APLICACIÓN ===
use crate::app::AppConfig;

// === INICIALIZACIÓN DEL TRACING/LOGGING ===
fn init_tracing(config: &AppConfig) {
    use tracing_subscriber::{fmt, EnvFilter, prelude::*};
    
    let filter = if config.debug_vulkan {
        EnvFilter::new("noir=debug,ash=info,vulkan=info")
    } else {
        EnvFilter::new("noir=info")
    };
    
    let subscriber = fmt::layer()
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true);
    
    tracing_subscriber::registry()
        .with(filter)
        .with(subscriber)
        .init();
    
    info!("🌌 Noir Browser initializing with config: {:?}", config);
}

// === COORDINADOR PRINCIPAL DE LA APLICACIÓN ===
struct AppCoordinator {
    config: AppConfig,
}


impl AppCoordinator {
    fn new(config: AppConfig) -> Self {
        Self { config }
    }
    
    /// Ejecuta el ciclo principal de la aplicación
    async fn run(self) -> anyhow::Result<()> {
        info!("🚀 Starting Noir Browser with {:?} model", self.config.process_model);
        
        // 1. Inicializar motor Vulkan (Fase 0 - base ultra-fast)
        #[cfg(feature = "ultrafast")]
        {
            info!("⚡ Initializing Vulkan 1.3 Ultra-Fast Engine...");
            self.init_vulkan_engine().await?;
        }
        
        // 2. Inicializar módulos de privacidad si están habilitados
        #[cfg(feature = "privacy")]
        {
            if self.config.enable_privacy {
                info!("🔒 Initializing Privacy Module (FPI + Anti-Fingerprint)...");
                self.init_privacy_module().await?;
            }
        }
        
        // 3. Inicializar red (SOCKS5/Tor mode si está habilitado)
        if self.config.enable_tor_mode {
            info!("🧅 Initializing Tor Mode (SOCKS5 + Circuit Rotation)...");
            self.init_network_module().await?;
        }
        
        // 4. Inicializar UI loop con winit
        info!("🎨 Starting UI Event Loop...");
        self.run_ui_loop().await?;
        
        // 5. Shutdown limpio con zeroize de memoria sensible
        self.cleanup().await?;
        
        info!("✅ Noir Browser shutdown complete");
        Ok(())
    }
    
    async fn init_vulkan_engine(&self) -> anyhow::Result<()> {
        // Delegar al módulo vulkan_engine
        vulkan_engine::UltraFastVulkanEngine::initialize().await?;
        Ok(())
    }
    
    #[cfg(feature = "privacy")]
    async fn init_privacy_module(&self) -> anyhow::Result<()> {
        // Inicializar First-Party Isolation
        privacy::FirstPartyIsolation::initialize()?;
        
        // Configurar anti-fingerprint jitter
        privacy::FingerprintProtector::enable_canvas_jitter(true);
        privacy::FingerprintProtector::enable_webgl_jitter(true);
        
        // Configurar cache efímera (disk avoidance)
        privacy::EphemeralCache::initialize(self.config.cache_size_mb * 1024 * 1024)?;
        
        Ok(())
    }
    
    async fn init_network_module(&self) -> anyhow::Result<()> {
        network::NetworkCoordinator::initialize().await?;
        Ok(())
    }
    
    async fn run_ui_loop(&self) -> anyhow::Result<()> {
        // Delegar al módulo app que contiene el event loop de winit
        app::run(self.config.clone()).await
    }
    
    async fn cleanup(&self) -> anyhow::Result<()> {
        // 1. Zeroize de memoria sensible (cookies, historial, cache)
        #[cfg(feature = "privacy")]
        {
            if let Some(cache) = privacy::EphemeralCache::global() {
                cache.purge();
                info!("🧹 Ephemeral cache zeroized");
            }
        }
        
        // 2. Cerrar conexiones de red limpiamente
        let _ = network::NetworkCoordinator::shutdown().await;
        
        // 3. Liberar recursos Vulkan
        let _ = vulkan_engine::UltraFastVulkanEngine::shutdown().await;
        
        // 4. El Runtime se limpia automáticamente al salir del scope
        // self.runtime.shutdown_timeout(std::time::Duration::from_secs(5));
        
        Ok(())
    }
}

// === PUNTO DE ENTRADA PRINCIPAL ===
#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> anyhow::Result<()> {
    // Parsear argumentos de línea de comandos
    let args: Vec<String> = env::args().collect();
    let config = parse_config_from_args(&args).unwrap_or_default();
    
    // Inicializar sistema de logging
    init_tracing(&config);
    
    info!("Process model: {:?} (RAM detectada: {} MB)", 
        config.process_model, 
        utils::process_model::detect_available_ram()
    );
    
    // Manejar panic hooks para logging adecuado
    setup_panic_hook();
    
    // Crear y ejecutar el coordinador de la aplicación
    let coordinator = AppCoordinator::new(config);
    
    match coordinator.run().await {
        Ok(()) => Ok(()),
        Err(e) => {
            error!("❌ Application error: {}", e);
            std::process::exit(1);
        }
    }
}

// === PARSING DE ARGUMENTOS DE LÍNEA DE COMANDOS ===
fn parse_config_from_args(args: &[String]) -> Option<AppConfig> {
    let mut config = AppConfig::default();
    let mut explicit_model = false;
    
    let mut i = 1; // Skip argv[0]
    while i < args.len() {
        match args[i].as_str() {
            "--single-process" => {
                config.process_model = ProcessModel::SingleProcess;
                explicit_model = true;
            }
            "--aggregated" => {
                config.process_model = ProcessModel::Aggregated;
                explicit_model = true;
            }
            "--moderate-isolation" => {
                config.process_model = ProcessModel::ModerateIsolation;
                explicit_model = true;
            }
            "--full-isolation" => {
                config.process_model = ProcessModel::FullIsolation;
                explicit_model = true;
            }
            "--no-privacy" => config.enable_privacy = false,
            "--tor-only" => {
                config.enable_tor_mode = true;
                config.enable_privacy = true;
            }
            "--no-ultrafast" => config.enable_ultrafast = false,
            "--debug-vulkan" => config.debug_vulkan = true,
            "--msdf-fonts" => config.enable_msdf_fonts = true,
            "--max-tabs" => {
                i += 1;
                if let Some(val) = args.get(i) {
                    if let Ok(n) = val.parse::<u32>() {
                        config.max_tabs = n;
                    }
                }
            }
            "--help" | "-h" => {
                print_help();
                return None;
            }
            _ => {}
        }
        i += 1;
    }
    
    // Auto-detectar process model basado en RAM si no se especificó uno
    if !explicit_model {
        let ram_mb = utils::process_model::detect_available_ram();
        config.process_model = ProcessModel::from_available_ram(ram_mb);
    }
    
    Some(config)
}

fn print_help() {
    println!("🌌 Noir Browser - Ultra-fast Privacy-First Browser");
    println!();
    println!("Usage: noir-browser [OPTIONS]");
    println!();
    println!("Process Model (auto-detected by RAM if not specified):");
    println!("  --single-process    All in one process (≤2GB RAM)");
    println!("  --aggregated        Browser + 1 shared renderer (2-4GB RAM)");
    println!("  --moderate-isolation Browser + renderer per tab (4-8GB RAM)");
    println!("  --full-isolation    Full process isolation (≥8GB RAM)");
    println!();
    println!("Features:");
    println!("  --no-privacy        Disable privacy features (FPI, anti-fingerprint)");
    println!("  --tor-only          Enable Tor mode with SOCKS5 proxy");
    println!("  --no-ultrafast      Disable Vulkan ultra-fast optimizations");
    println!("  --debug-vulkan      Enable Vulkan debug layers");
    println!("  --msdf-fonts        Enable MSDF font rendering");
    println!();
    println!("Limits:");
    println!("  --max-tabs <N>      Set maximum number of tabs (default: 20)");
    println!();
    println!("  -h, --help          Show this help message");
    println!();
    println!("Features (compile-time):");
    println!("  ultrafast    Vulkan 1.3 zero-copy rendering (default)");
    println!("  privacy      First-Party Isolation + anti-fingerprint");
    println!("  tor_mode     SOCKS5 proxy + circuit rotation");
    println!("  msdf_fonts   Multi-channel Signed Distance Field fonts");
    println!("  debug_vulkan Vulkan validation layers");
}

fn setup_panic_hook() {
    std::panic::set_hook(Box::new(|panic_info| {
        let msg = if let Some(location) = panic_info.location() {
            format!(
                "🔥 Panic at {}:{}:{} - {}",
                location.file(),
                location.line(),
                location.column(),
                panic_info
            )
        } else {
            format!("🔥 Panic: {}", panic_info)
        };
        // Salida directa a stderr + tracing
        eprintln!("{}", msg);
        error!("{}", msg);
        
        // Intentar cleanup antes de salir
        #[cfg(feature = "privacy")]
        {
            if let Some(cache) = privacy::EphemeralCache::global() {
                cache.purge();
            }
        }
    }));
}

// === TESTS ===
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_process_model_selection() {
        assert_eq!(ProcessModel::from_available_ram(1024), ProcessModel::SingleProcess);
        assert_eq!(ProcessModel::from_available_ram(3072), ProcessModel::Aggregated);
        assert_eq!(ProcessModel::from_available_ram(6144), ProcessModel::ModerateIsolation);
        assert_eq!(ProcessModel::from_available_ram(16384), ProcessModel::FullIsolation);
    }
    
    #[test]
    fn test_max_renderer_processes() {
        assert_eq!(ProcessModel::SingleProcess.max_renderer_processes(), 1);
        assert_eq!(ProcessModel::Aggregated.max_renderer_processes(), 2);
        assert_eq!(ProcessModel::ModerateIsolation.max_renderer_processes(), 4);
        // FullIsolation tiene límite dinámico
    }
    
    #[test]
    fn test_config_default() {
        let config = AppConfig::default();
        assert!(config.enable_ultrafast == cfg!(feature = "ultrafast"));
        assert!(config.enable_privacy == cfg!(feature = "privacy"));
    }
}
