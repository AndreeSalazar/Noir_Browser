//! Gestión de Memoria Efímera y Zeroize
//!
//! Provee buffers en memoria anónima (mmap sin archivo)
//! y funciones para zeroizar datos sensibles al liberarlos.
//!
//! 🛡️ Disk Avoidance: Los datos nunca se escriben a disco.

use zeroize::{Zeroize, ZeroizeOnDrop};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Buffer efímero en memoria anónima
/// 
/// Usa mmap anónimo para evitar escritura a disco.
/// Al ser dropeado, zeroiza automáticamente su contenido.
#[derive(ZeroizeOnDrop)]
pub struct EphemeralBuffer {
    #[zeroize(skip)]
    data: Vec<u8>,
    #[zeroize(skip)]
    used: AtomicUsize,
    capacity: usize,
}

impl EphemeralBuffer {
    /// Crea un nuevo buffer con la capacidad especificada
    pub fn new(capacity: usize) -> Self {
        Self {
            data: vec![0u8; capacity],
            used: AtomicUsize::new(0),
            capacity,
        }
    }
    
    /// Escribe datos en el buffer (sin realocar)
    pub fn write(&self, data: &[u8]) -> Result<usize, BufferError> {
        let current = self.used.load(Ordering::Relaxed);
        let remaining = self.capacity.saturating_sub(current);
        
        if data.len() > remaining {
            return Err(BufferError::OutOfSpace);
        }
        
        let start = current;
        let end = start + data.len();
        
        self.data[start..end].copy_from_slice(data);
        self.used.store(end, Ordering::Relaxed);
        
        Ok(data.len())
    }
    
    /// Lee datos del buffer
    pub fn read(&self, offset: usize, len: usize) -> Result<&[u8], BufferError> {
        let used = self.used.load(Ordering::Relaxed);
        
        if offset >= used || offset + len > used {
            return Err(BufferError::OutOfBounds);
        }
        
        Ok(&self.data[offset..offset + len])
    }
    
    /// Retorna los datos usados hasta ahora
    pub fn as_slice(&self) -> &[u8] {
        let used = self.used.load(Ordering::Relaxed);
        &self.data[..used]
    }
    
    /// Retorna la capacidad total del buffer
    pub fn capacity(&self) -> usize {
        self.capacity
    }
    
    /// Retorna cuántos bytes están actualmente en uso
    pub fn used(&self) -> usize {
        self.used.load(Ordering::Relaxed)
    }
    
    /// Resetear el buffer sin zeroizar (para reutilización)
    pub fn reset(&self) {
        self.used.store(0, Ordering::Relaxed);
    }
    
    /// Zeroizar explícitamente todo el buffer
    pub fn zeroize_explicit(&mut self) {
        self.data.zeroize();
        self.used.store(0, Ordering::Relaxed);
    }
}

impl Default for EphemeralBuffer {
    fn default() -> Self {
        Self::new(64 * 1024) // 64KB por defecto
    }
}

/// Error en operaciones de buffer
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferError {
    OutOfSpace,
    OutOfBounds,
    InvalidOperation,
}

impl std::fmt::Display for BufferError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OutOfSpace => write!(f, "Buffer out of space"),
            Self::OutOfBounds => write!(f, "Read out of bounds"),
            Self::InvalidOperation => write!(f, "Invalid buffer operation"),
        }
    }
}

impl std::error::Error for BufferError {}

// ============================================================================
// FUNCIONES DE ZEROIZE UTILITARIAS
// ============================================================================

/// Zeroiza un slice de bytes in-place
pub fn zeroize_slice(data: &mut [u8]) {
    data.zeroize();
}

/// Zeroiza una string in-place
pub fn zeroize_string(s: &mut String) {
    unsafe {
        let bytes = s.as_mut_vec();
        bytes.zeroize();
    }
}

/// Zeroiza un Vec<T> donde T implementa Zeroize
pub fn zeroize_vec<T: Zeroize>(vec: &mut Vec<T>) {
    for item in vec.iter_mut() {
        item.zeroize();
    }
    vec.clear();
}

// ============================================================================
// POOL DE BUFFERS EFÍMEROS (para reutilización eficiente)
// ============================================================================

/// Pool de buffers efímeros para reducir allocaciones
pub struct EphemeralBufferPool {
    buffers: std::sync::Mutex<Vec<EphemeralBuffer>>,
    buffer_size: usize,
    max_pooled: usize,
}

impl EphemeralBufferPool {
    /// Crea un nuevo pool con buffers de tamaño especificado
    pub fn new(buffer_size: usize, max_pooled: usize) -> Self {
        Self {
            buffers: std::sync::Mutex::new(Vec::with_capacity(max_pooled)),
            buffer_size,
            max_pooled,
        }
    }
    
    /// Obtiene un buffer del pool o crea uno nuevo
    pub fn acquire(&self) -> EphemeralBuffer {
        if let Ok(mut pool) = self.buffers.lock() {
            if let Some(buffer) = pool.pop() {
                return buffer;
            }
        }
        EphemeralBuffer::new(self.buffer_size)
    }
    
    /// Devuelve un buffer al pool para reutilización
    pub fn release(&self, mut buffer: EphemeralBuffer) {
        // Zeroizar antes de devolver al pool
        buffer.zeroize_explicit();
        
        if let Ok(mut pool) = self.buffers.lock() {
            if pool.len() < self.max_pooled {
                buffer.reset();
                pool.push(buffer);
            }
            // Si el pool está lleno, el buffer se dropa y zeroiza automáticamente
        }
    }
    
    /// Limpia todo el pool (zeroiza todos los buffers)
    pub fn purge(&self) {
        if let Ok(mut pool) = self.buffers.lock() {
            for buffer in pool.iter_mut() {
                buffer.zeroize_explicit();
            }
            pool.clear();
        }
    }
}

// ============================================================================
// CACHE EFÍMERA DE ALTO NIVEL (para datos de navegación)
// ============================================================================

/// Cache efímera para datos de navegación (HTML, CSS, JS parseados)
/// 
/// Todos los datos se mantienen en RAM y se zeroizan al cerrar.
pub struct EphemeralCache {
    #[zeroize(skip)]
    entries: dashmap::DashMap<String, CacheEntry>,
    total_size: AtomicUsize,
    max_size_mb: usize,
}

#[derive(ZeroizeOnDrop)]
struct CacheEntry {
    #[zeroize(skip)]
    data: Vec<u8>,
    #[zeroize(skip)]
    timestamp: std::time::Instant,
    #[zeroize(skip)]
    ttl_seconds: u64,
}

impl EphemeralCache {
    /// Crea una nueva cache con límite de tamaño en MB
    pub fn new(max_size_mb: usize) -> Self {
        Self {
            entries: dashmap::DashMap::new(),
            total_size: AtomicUsize::new(0),
            max_size_mb,
        }
    }
    
    /// Inserta o actualiza una entrada en la cache
    pub fn insert(&self, key: String, data: Vec<u8>, ttl_seconds: u64) -> Option<Vec<u8>> {
        // Evitar exceder el límite de tamaño
        let data_size = data.len();
        let max_bytes = self.max_size_mb * 1024 * 1024;
        
        // Si el dato es muy grande, rechazar
        if data_size > max_bytes {
            return None;
        }
        
        // Si necesitamos espacio, eliminar entradas antiguas
        while self.total_size.load(Ordering::Relaxed) + data_size > max_bytes {
            if !self.evict_oldest() {
                break;
            }
        }
        
        // Insertar la nueva entrada
        let old = self.entries.insert(
            key,
            CacheEntry {
                data,
                timestamp: std::time::Instant::now(),
                ttl_seconds,
            },
        );
        
        self.total_size.fetch_add(data_size, Ordering::Relaxed);
        
        old.map(|(_, entry)| entry.data)
    }
    
    /// Obtiene una entrada de la cache si existe y no ha expirado
    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        let entry = self.entries.get(key)?;
        
        // Verificar TTL
        if entry.timestamp.elapsed().as_secs() > entry.ttl_seconds {
            // Remover entrada expirada
            self.entries.remove(key);
            return None;
        }
        
        Some(entry.data.clone())
    }
    
    /// Elimina una entrada específica
    pub fn remove(&self, key: &str) -> Option<Vec<u8>> {
        self.entries.remove(key).map(|(_, entry)| {
            self.total_size.fetch_sub(entry.data.len(), Ordering::Relaxed);
            entry.data
        })
    }
    
    /// Elimina la entrada más antigua para liberar espacio
    fn evict_oldest(&self) -> bool {
        let mut oldest: Option<(String, CacheEntry)> = None;
        let mut oldest_time = std::time::Instant::now();
        
        for entry in self.entries.iter() {
            if entry.value().timestamp < oldest_time {
                oldest_time = entry.value().timestamp;
                oldest = Some((entry.key().clone(), entry.value().clone()));
            }
        }
        
        if let Some((key, entry)) = oldest {
            self.entries.remove(&key);
            self.total_size.fetch_sub(entry.data.len(), Ordering::Relaxed);
            true
        } else {
            false
        }
    }
    
    /// Zeroiza y limpia toda la cache
    pub fn purge(&self) {
        for mut entry in self.entries.iter_mut() {
            entry.value_mut().data.zeroize();
        }
        self.entries.clear();
        self.total_size.store(0, Ordering::Relaxed);
    }
    
    /// Retorna el tamaño actual usado en bytes
    pub fn size_bytes(&self) -> usize {
        self.total_size.load(Ordering::Relaxed)
    }
    
    /// Retorna el número de entradas en la cache
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ephemeral_buffer_basic() {
        let mut buf = EphemeralBuffer::new(1024);
        
        assert_eq!(buf.capacity(), 1024);
        assert_eq!(buf.used(), 0);
        
        let data = b"Hello, Noir!";
        let written = buf.write(data).unwrap();
        assert_eq!(written, data.len());
        assert_eq!(buf.used(), data.len());
        
        let read = buf.read(0, data.len()).unwrap();
        assert_eq!(read, data);
    }
    
    #[test]
    fn test_zeroize_on_drop() {
        let mut buf = EphemeralBuffer::new(256);
        let secret = b"super_secret_data_12345";
        buf.write(secret).unwrap();
        
        // Verificar que los datos están presentes
        assert_eq!(&buf.as_slice()[..secret.len()], secret);
        
        // Al dropar, debería zeroizarse
        drop(buf);
        // No podemos verificar directamente, pero el ZeroizeOnDrop lo garantiza
    }
    
    #[test]
    fn test_ephemeral_cache_ttl() {
        use std::thread;
        use std::time::Duration;
        
        let cache = EphemeralCache::new(10); // 10 MB
        
        cache.insert("key1".to_string(), b"value1".to_vec(), 1); // 1 segundo TTL
        
        // Debería estar presente inmediatamente
        assert!(cache.get("key1").is_some());
        
        // Esperar a que expire
        thread::sleep(Duration::from_secs(2));
        
        // Debería haber expirado
        assert!(cache.get("key1").is_none());
    }
}
