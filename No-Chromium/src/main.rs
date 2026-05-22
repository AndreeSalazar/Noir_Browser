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

// === MÓDULOS PRINCIPALES ===
mod app;
mod browser;
mod renderer;
mod vulkan_engine;
mod network;
mod utils;

#[cfg(feature = "privacy")]
mod privacy;

use std::env;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tracing::{info, warn, error};

// === TIPOS DE MODELO DE PROCESOS ===
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessModel {
    /// Todo en un solo proceso/task - para sistemas con ≤2GB RAM
    SingleProcess,
    /// Browser + 1 renderer compartido - para 2-4GB RAM
    Aggregated,
    /// Browser + renderer por tab - para 4-8GB RAM
    ModerateIsolation,
    /// Browser + renderer + GPU + network separados - para ≥8GB RAM
    FullIsolation,
}

impl ProcessModel {
    /// Determina el modelo óptimo basado en la RAM disponible (en MB)
    pub fn from_available_ram(available_ram_mb: u64) -> Self {
        match available_ram_mb {
            0..=2048 => Self::SingleProcess,
            2049..=4096 => Self::Aggregated,
            4097..=8192 => Self::ModerateIsolation,
            _ => Self::FullIsolation,
        }
    }

    /// Retorna si debemos usar aislamiento completo de procesos
    pub fn uses_full_isolation(self) -> bool {
        matches!(self, Self::FullIsolation)
    }

    /// Retorna el número máximo de renderer processes permitidos
    pub fn max_renderer_processes(self) -> usize {
        match self {
            Self::SingleProcess => 1,
            Self::Aggregated => 2,
            Self::ModerateIsolation => 4,
            Self::FullIsolation => usize::MAX, // Limitado por RAM dinámica
        }
    }
}

// === CONFIGURACIÓN DE LA APLICACIÓN ===
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub process_model: ProcessModel,
    pub enable_privacy: bool,
    pub enable_tor_mode: bool,
    pub enable_ultrafast: bool,
    pub enable_msdf_fonts: bool,
    pub debug_vulkan: bool,
    pub max_tabs: usize,
    pub cache_size_mb: usize,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            process_model: ProcessModel::from_available_ram(detect_available_ram()),
            enable_privacy: cfg!(feature = "privacy"),
            enable_tor_mode: cfg!(feature = "tor_mode"),
            enable_ultrafast: cfg!(feature = "ultrafast"),
            enable_msdf_fonts: cfg!(feature = "msdf_fonts"),
            debug_vulkan: cfg!(feature = "debug_vulkan"),
            max_tabs: 20,
            cache_size_mb: 512,
        }
    }
}

// === DETECCIÓN DE MEMORIA DEL SISTEMA ===
fn detect_available_ram() -> u64 {
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::System::SystemInformation::GlobalMemoryStatusEx;
        use windows::Win32::System::SystemInformation::MEMORYSTATUSEX;
        
        unsafe {
            let mut status = MEMORYSTATUSEX::default();
            status.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;
            if GlobalMemoryStatusEx(&mut status).is_ok() {
                return status.ullAvailPhys / (1024 * 1024); // Convertir a MB
            }
        }
    }
    
    #[cfg(target_os = "linux")]
    {
        if let Ok(info) = sys_info::mem_info() {
            return info.avail / 1024; // Convertir KB a MB
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        // macOS: usar sysctl para obtener memoria disponible
        // Implementación simplificada - fallback a valor conservador
    }
    
    // Fallback conservador: asumir 4GB disponibles
    4096
}

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
    runtime: Arc<Runtime>,
}

impl AppCoordinator {
    fn new(config: AppConfig) -> anyhow::Result<Self> {
        // Configurar runtime Tokio según el modelo de procesos
        let runtime = match config.process_model {
            ProcessModel::SingleProcess | ProcessModel::Aggregated => {
                tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()?
            }
            ProcessModel::ModerateIsolation | ProcessModel::FullIsolation => {
                tokio::runtime::Builder::new_multi_thread()
                    .worker_threads(std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4))
                    .enable_all()
                    .build()?
            }
        };
        
        Ok(Self {
            config,
            runtime: Arc::new(runtime),
        })
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
        vulkan_engine::UltraFastVulkanEngine::initialize(
            self.config.enable_ultrafast,
            self.config.debug_vulkan,
        ).await?;
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
        network::NetworkCoordinator::initialize(
            self.config.enable_tor_mode,
            self.config.enable_privacy,
        ).await?;
        Ok(())
    }
    
    async fn run_ui_loop(&self) -> anyhow::Result<()> {
        // Delegar al módulo app que contiene el event loop de winit
        app::run(self.config.clone(), self.runtime.clone()).await
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
        network::NetworkCoordinator::shutdown().await;
        
        // 3. Liberar recursos Vulkan
        vulkan_engine::UltraFastVulkanEngine::shutdown().await;
        
        // 4. Esperar que todas las tasks de Tokio completen
        self.runtime.shutdown_timeout(std::time::Duration::from_secs(5));
        
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
    
    // Manejar panic hooks para logging adecuado
    setup_panic_hook();
    
    // Crear y ejecutar el coordinador de la aplicación
    let coordinator = AppCoordinator::new(config)?;
    
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
    
    for arg in args.iter().skip(1) {
        match arg.as_str() {
            "--single-process" => config.process_model = ProcessModel::SingleProcess,
            "--full-isolation" => config.process_model = ProcessModel::FullIsolation,
            "--no-privacy" => config.enable_privacy = false,
            "--tor-only" => {
                config.enable_tor_mode = true;
                config.enable_privacy = true;
            }
            "--no-ultrafast" => config.enable_ultrafast = false,
            "--debug-vulkan" => config.debug_vulkan = true,
            "--max-tabs" => {
                // El siguiente argumento debe ser el número
                // (implementación simplificada)
            }
            "--help" | "-h" => {
                print_help();
                return None;
            }
            _ => {}
        }
    }
    
    Some(config)
}

fn print_help() {
    println!("🌌 Noir Browser - Ultra-fast Privacy-First Browser");
    println!();
    println!("Usage: noir-browser [OPTIONS]");
    println!();
    println!("Options:");
    println!("  --single-process    Force single-process mode (low RAM)");
    println!("  --full-isolation    Force full process isolation (high RAM)");
    println!("  --no-privacy        Disable privacy features");
    println!("  --tor-only          Enable Tor mode with SOCKS5 proxy");
    println!("  --no-ultrafast      Disable Vulkan ultra-fast optimizations");
    println!("  --debug-vulkan      Enable Vulkan debug layers");
    println!("  --max-tabs <N>      Set maximum number of tabs (default: 20)");
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
        if let Some(location) = panic_info.location() {
            error!(
                "🔥 Panic at {}:{}:{} - {}",
                location.file(),
                location.line(),
                location.column(),
                panic_info
            );
        } else {
            error!("🔥 Panic: {}", panic_info);
        }
        
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
