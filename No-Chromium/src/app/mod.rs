//! Noir App - Módulo principal limpio y modular
//!
//! Punto de entrada de la aplicación que coordina todos los subsistemas.

use std::error::Error;
use std::result::Result as StdResult;

pub mod config;
pub mod context;
pub mod event_loop;
pub mod renderer;
pub mod input;
pub mod navigation;
pub mod state;
pub mod draw;
pub mod glyphs;
pub mod paint_records;       // Chrome Blink paint records
pub mod layer_tree;          // Chrome compositor layers
pub mod navigation_pipeline; // Chrome navigation flow state machine

pub use config::AppConfig;
pub use context::AppContext;

/// Tipo Result para la aplicación
pub type AppResult<T> = StdResult<T, Box<dyn Error>>;

/// Ejecuta la aplicación con la configuración dada
pub fn execute(config: AppConfig) -> AppResult<()> {
    // Crear contexto
    let mut context = AppContext::new(config);

    // Inicializar subsistemas
    context.initialize()?;

    // Ejecutar event loop
    event_loop::run(context)?;

    Ok(())
}
