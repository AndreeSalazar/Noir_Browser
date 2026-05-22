//! Lógica de Auto-Scaling de Procesos
//!
//! Determina dinámicamente el modelo de procesos óptimo
//! basado en la memoria RAM disponible del sistema.

/// Modelos de proceso disponibles
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessModel {
    /// Todo en un solo proceso/task - para sistemas con ≤2GB RAM
    /// ⚠️ Menos estable pero mínimo uso de memoria
    SingleProcess,
    
    /// Browser + 1 renderer compartido - para 2-4GB RAM
    /// ⚖️ Balance entre estabilidad y consumo
    Aggregated,
    
    /// Browser + renderer por tab - para 4-8GB RAM
    /// 🎯 Aislamiento moderado por pestaña
    ModerateIsolation,
    
    /// Browser + renderer + GPU + network separados - para ≥8GB RAM
    /// 🚀 Máximo aislamiento y rendimiento paralelo
    FullIsolation,
}

impl ProcessModel {
    /// Determina el modelo óptimo basado en RAM disponible (en MB)
    pub fn from_available_ram(available_ram_mb: u64) -> Self {
        match available_ram_mb {
            0..=2048 => Self::SingleProcess,
            2049..=4096 => Self::Aggregated,
            4097..=8192 => Self::ModerateIsolation,
            _ => Self::FullIsolation,
        }
    }
    
    /// Retorna si este modelo usa aislamiento completo de procesos
    pub fn uses_full_isolation(self) -> bool {
        matches!(self, Self::FullIsolation)
    }
    
    /// Retorna si este modelo permite múltiples renderer processes
    pub fn allows_multiple_renderers(self) -> bool {
        matches!(self, Self::ModerateIsolation | Self::FullIsolation)
    }
    
    /// Número máximo de renderer processes permitidos
    pub fn max_renderer_processes(self) -> usize {
        match self {
            Self::SingleProcess => 1,
            Self::Aggregated => 2,
            Self::ModerateIsolation => 4,
            Self::FullIsolation => usize::MAX, // Limitado por RAM dinámica
        }
    }
    
    /// Estimación de memoria base requerida para este modelo (en MB)
    pub fn estimated_base_memory_mb(self) -> u64 {
        match self {
            Self::SingleProcess => 512,
            Self::Aggregated => 1024,
            Self::ModerateIsolation => 2048,
            Self::FullIsolation => 4096,
        }
    }
    
    /// Memoria estimada por renderer adicional (en MB)
    pub fn memory_per_renderer_mb(self) -> u64 {
        match self {
            Self::SingleProcess | Self::Aggregated => 256,
            Self::ModerateIsolation => 512,
            Self::FullIsolation => 1024,
        }
    }
}

/// Determina el modelo de procesos óptimo para el sistema actual
pub fn determine_process_model() -> ProcessModel {
    let available_ram = detect_available_ram_mb();
    ProcessModel::from_available_ram(available_ram)
}

/// Detecta la memoria RAM disponible del sistema (en MB)
fn detect_available_ram_mb() -> u64 {
    #[cfg(target_os = "windows")]
    {
        detect_windows_ram()
    }
    
    #[cfg(target_os = "linux")]
    {
        detect_linux_ram()
    }
    
    #[cfg(target_os = "macos")]
    {
        detect_macos_ram()
    }
    
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        // Fallback conservador para otras plataformas
        4096
    }
}

#[cfg(target_os = "windows")]
fn detect_windows_ram() -> u64 {
    use windows::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
    
    unsafe {
        let mut status = MEMORYSTATUSEX::default();
        status.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;
        
        if GlobalMemoryStatusEx(&mut status).is_ok() {
            // Convertir bytes a MB
            return status.ullAvailPhys / (1024 * 1024);
        }
    }
    4096 // Fallback
}

#[cfg(target_os = "linux")]
fn detect_linux_ram() -> u64 {
    use std::fs;
    
    // Leer /proc/meminfo para obtener MemAvailable
    if let Ok(content) = fs::read_to_string("/proc/meminfo") {
        for line in content.lines() {
            if line.starts_with("MemAvailable:") {
                // Formato: "MemAvailable:    1234567 kB"
                if let Some(value_str) = line.split_whitespace().nth(1) {
                    if let Ok(kb) = value_str.parse::<u64>() {
                        return kb / 1024; // Convertir KB a MB
                    }
                }
            }
        }
    }
    4096 // Fallback
}

#[cfg(target_os = "macos")]
fn detect_macos_ram() -> u64 {
    // En macOS, usar sysctl para obtener memoria disponible
    // Implementación simplificada - requiere crate `sysctl`
    // Por ahora, fallback conservador
    4096
}

/// Detecta la memoria RAM disponible del sistema (en MB) - función pública
pub fn detect_available_ram() -> u64 {
    detect_available_ram_mb()
}

/// Calcula cuántos renderer processes se pueden crear con la RAM disponible
pub fn calculate_max_renderers(
    available_ram_mb: u64,
    model: ProcessModel,
    base_overhead_mb: u64,
) -> usize {
    let remaining_ram = available_ram_mb.saturating_sub(base_overhead_mb);
    let per_renderer = model.memory_per_renderer_mb();
    
    if per_renderer == 0 {
        return model.max_renderer_processes();
    }
    
    let max_by_ram = (remaining_ram / per_renderer) as usize;
    max_by_ram.min(model.max_renderer_processes())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_model_selection_by_ram() {
        assert_eq!(ProcessModel::from_available_ram(1024), ProcessModel::SingleProcess);
        assert_eq!(ProcessModel::from_available_ram(2048), ProcessModel::SingleProcess);
        assert_eq!(ProcessModel::from_available_ram(2049), ProcessModel::Aggregated);
        assert_eq!(ProcessModel::from_available_ram(4096), ProcessModel::Aggregated);
        assert_eq!(ProcessModel::from_available_ram(4097), ProcessModel::ModerateIsolation);
        assert_eq!(ProcessModel::from_available_ram(8192), ProcessModel::ModerateIsolation);
        assert_eq!(ProcessModel::from_available_ram(8193), ProcessModel::FullIsolation);
        assert_eq!(ProcessModel::from_available_ram(16384), ProcessModel::FullIsolation);
    }
    
    #[test]
    fn test_model_properties() {
        let single = ProcessModel::SingleProcess;
        assert!(!single.uses_full_isolation());
        assert!(!single.allows_multiple_renderers());
        assert_eq!(single.max_renderer_processes(), 1);
        
        let full = ProcessModel::FullIsolation;
        assert!(full.uses_full_isolation());
        assert!(full.allows_multiple_renderers());
        assert_eq!(full.max_renderer_processes(), usize::MAX);
    }
    
    #[test]
    fn test_memory_estimates() {
        assert_eq!(ProcessModel::SingleProcess.estimated_base_memory_mb(), 512);
        assert_eq!(ProcessModel::FullIsolation.estimated_base_memory_mb(), 4096);
        
        assert_eq!(ProcessModel::SingleProcess.memory_per_renderer_mb(), 256);
        assert_eq!(ProcessModel::FullIsolation.memory_per_renderer_mb(), 1024);
    }
    
    #[test]
    fn test_renderer_calculation() {
        // 8GB disponibles, modelo FullIsolation, 4GB overhead base
        let max = calculate_max_renderers(8192, ProcessModel::FullIsolation, 4096);
        // (8192 - 4096) / 1024 = 4 renderers posibles
        assert!(max >= 4);
    }
}
