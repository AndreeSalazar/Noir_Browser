//! GPU Textures and Glyph Atlas (FASE D2)
//!
//! Maneja texturas GPU para el renderer:
//! - Atlas de glyphs: bitmap font consolidado en una sola textura
//! - Image textures: texturas para imagenes decodificadas
//! - Pool de texturas con caching
//!
//! Inspirado en:
//! - Chrome Skia: texture atlas
//! - Firefox WebRender: glyph cache
//! - Safari WebKit: image bitmap caching

use std::collections::HashMap;
use std::sync::Arc;

/// Tipo de textura GPU
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureKind {
    Glyph,     // Atlas de glyphs
    Image,     // Imagen decodificada
    Video,     // Frame de video
    Canvas,    // Canvas 2D
    Composite, // Layer de compositing
}

impl TextureKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            TextureKind::Glyph => "glyph",
            TextureKind::Image => "image",
            TextureKind::Video => "video",
            TextureKind::Canvas => "canvas",
            TextureKind::Composite => "composite",
        }
    }
}

/// Formato de pixel
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PixelFormat {
    Rgba8,
    Bgra8,
    R8,         // grayscale
    Rgba16f,    // HDR
}

impl PixelFormat {
    pub fn bytes_per_pixel(&self) -> u32 {
        match self {
            PixelFormat::Rgba8 | PixelFormat::Bgra8 => 4,
            PixelFormat::R8 => 1,
            PixelFormat::Rgba16f => 8,
        }
    }
}

/// Descriptor de textura
#[derive(Debug, Clone, Copy)]
pub struct TextureDescriptor {
    pub width: u32,
    pub height: u32,
    pub format: PixelFormat,
    pub has_alpha: bool,
    pub mipmapped: bool,
    pub srgb: bool,
}

impl TextureDescriptor {
    pub fn rgba(width: u32, height: u32) -> Self {
        Self {
            width, height,
            format: PixelFormat::Rgba8,
            has_alpha: true,
            mipmapped: false,
            srgb: true,
        }
    }
    pub fn data_size(&self) -> u32 {
        self.width * self.height * self.format.bytes_per_pixel()
    }
}

/// Una textura GPU (placeholder - en la realidad seria wgpu::Texture)
#[derive(Debug, Clone)]
pub struct GpuTexture {
    pub id: u64,
    pub kind: TextureKind,
    pub descriptor: TextureDescriptor,
    pub data: Vec<u8>,
    pub dirty: bool,
    pub ref_count: u32,
}

impl GpuTexture {
    pub fn new(id: u64, kind: TextureKind, descriptor: TextureDescriptor, data: Vec<u8>) -> Self {
        let ref_count = 1;
        Self {
            id, kind, descriptor, data, dirty: true, ref_count,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.data.len() == self.descriptor.data_size() as usize
    }

    pub fn width(&self) -> u32 { self.descriptor.width }
    pub fn height(&self) -> u32 { self.descriptor.height }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }
}

/// Atlas de glyphs
#[derive(Debug)]
pub struct GlyphAtlas {
    texture: Option<GpuTexture>,
    /// Tamano del atlas
    pub width: u32,
    pub height: u32,
    /// Layout de glyphs (posiciones)
    glyphs: HashMap<char, GlyphSlot>,
    /// Cursor para colocar nuevos glyphs
    next_x: u32,
    next_y: u32,
    pub row_height: u32,
    pub padding: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct GlyphSlot {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

impl GlyphAtlas {
    pub fn new(width: u32, height: u32) -> Self {
        let descriptor = TextureDescriptor::rgba(width, height);
        let data = vec![0u8; descriptor.data_size() as usize];
        let texture = GpuTexture::new(0, TextureKind::Glyph, descriptor, data);
        Self {
            texture: Some(texture),
            width, height,
            glyphs: HashMap::new(),
            next_x: 0,
            next_y: 0,
            row_height: 0,
            padding: 1,
        }
    }

    /// Carga un glyph en el atlas
    pub fn upload_glyph(&mut self, ch: char, bitmap: &[u8], gw: u32, gh: u32) -> Option<GlyphSlot> {
        if gw > self.width || gh > self.height {
            return None;
        }
        // Wrap a nueva fila si no cabe (solo si ya estamos en una fila)
        if self.next_x > 0 && self.next_x + gw > self.width {
            self.next_x = 0;
            self.next_y += self.row_height + self.padding;
            self.row_height = 0;
        }
        if self.next_y + gh > self.height {
            return None;  // atlas lleno
        }

        let slot = GlyphSlot {
            x: self.next_x,
            y: self.next_y,
            w: gw,
            h: gh,
        };
        // Copiar bitmap al atlas
        if let Some(tex) = &mut self.texture {
            for row in 0..gh {
                let src_start = (row * gw) as usize;
                let src_end = src_start + gw as usize;
                let dst_start = ((self.next_y + row) * self.width + self.next_x) as usize;
                let dst_end = dst_start + gw as usize;
                if src_end <= bitmap.len() && dst_end <= tex.data.len() {
                    tex.data[dst_start..dst_end].copy_from_slice(&bitmap[src_start..src_end]);
                }
            }
            tex.mark_dirty();
        }
        self.glyphs.insert(ch, slot);
        self.next_x += gw + self.padding;
        if gh > self.row_height {
            self.row_height = gh;
        }
        Some(slot)
    }

    pub fn get_glyph(&self, ch: char) -> Option<GlyphSlot> {
        self.glyphs.get(&ch).copied()
    }

    pub fn has_glyph(&self, ch: char) -> bool {
        self.glyphs.contains_key(&ch)
    }

    pub fn glyph_count(&self) -> usize {
        self.glyphs.len()
    }

    pub fn texture(&self) -> Option<&GpuTexture> {
        self.texture.as_ref()
    }

    pub fn clear(&mut self) {
        self.glyphs.clear();
        self.next_x = 0;
        self.next_y = 0;
        self.row_height = 0;
        if let Some(tex) = &mut self.texture {
            tex.data.fill(0);
            tex.mark_dirty();
        }
    }
}

/// Pool de texturas para caching
#[derive(Debug, Default)]
pub struct TexturePool {
    textures: HashMap<u64, GpuTexture>,
    by_kind: HashMap<TextureKind, Vec<u64>>,
    next_id: u64,
    pub max_size: usize,
    pub total_memory: u64,
}

impl TexturePool {
    pub fn new() -> Self {
        Self {
            max_size: 100,
            ..Default::default()
        }
    }

    pub fn with_max_size(max_size: usize) -> Self {
        Self {
            max_size,
            ..Default::default()
        }
    }

    /// Allocate una textura nueva
    pub fn allocate(&mut self, kind: TextureKind, descriptor: TextureDescriptor) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let data = vec![0u8; descriptor.data_size() as usize];
        let texture = GpuTexture::new(id, kind, descriptor, data);
        self.total_memory += texture.data.len() as u64;
        self.by_kind.entry(kind).or_default().push(id);
        self.textures.insert(id, texture);
        id
    }

    /// Obtener referencia a una textura
    pub fn get(&self, id: u64) -> Option<&GpuTexture> {
        self.textures.get(&id)
    }

    /// Obtener referencia mutable
    pub fn get_mut(&mut self, id: u64) -> Option<&mut GpuTexture> {
        self.textures.get_mut(&id)
    }

    /// Eliminar una textura
    pub fn deallocate(&mut self, id: u64) -> bool {
        if let Some(tex) = self.textures.remove(&id) {
            self.total_memory -= tex.data.len() as u64;
            if let Some(v) = self.by_kind.get_mut(&tex.kind) {
                v.retain(|&x| x != id);
            }
            true
        } else {
            false
        }
    }

    /// Cargar data en una textura existente
    pub fn upload(&mut self, id: u64, data: Vec<u8>) -> Result<(), String> {
        if let Some(tex) = self.textures.get_mut(&id) {
            if data.len() != tex.data.len() {
                return Err(format!("Data size mismatch: expected {}, got {}", tex.data.len(), data.len()));
            }
            self.total_memory -= tex.data.len() as u64;
            tex.data = data;
            self.total_memory += tex.data.len() as u64;
            tex.mark_dirty();
            Ok(())
        } else {
            Err(format!("Texture {} not found", id))
        }
    }

    pub fn count(&self) -> usize {
        self.textures.len()
    }

    pub fn count_of_kind(&self, kind: TextureKind) -> usize {
        self.by_kind.get(&kind).map(|v| v.len()).unwrap_or(0)
    }

    /// LRU eviction - eliminar texturas menos usadas
    pub fn evict_unused(&mut self, keep: usize) {
        if self.textures.len() <= keep {
            return;
        }
        // Recolectar texturas no-dirty
        let mut candidates: Vec<u64> = self.textures.iter()
            .filter(|(_, t)| !t.dirty && t.ref_count <= 1)
            .map(|(id, _)| *id)
            .collect();
        // Sort by id (oldest first)
        candidates.sort();
        while self.textures.len() > keep && !candidates.is_empty() {
            let id = candidates.remove(0);
            self.deallocate(id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_texture_kind_str() {
        assert_eq!(TextureKind::Glyph.as_str(), "glyph");
        assert_eq!(TextureKind::Image.as_str(), "image");
    }

    #[test]
    fn test_pixel_format_bpp() {
        assert_eq!(PixelFormat::Rgba8.bytes_per_pixel(), 4);
        assert_eq!(PixelFormat::R8.bytes_per_pixel(), 1);
        assert_eq!(PixelFormat::Rgba16f.bytes_per_pixel(), 8);
    }

    #[test]
    fn test_texture_descriptor() {
        let d = TextureDescriptor::rgba(100, 100);
        assert_eq!(d.data_size(), 40000);
    }

    #[test]
    fn test_gpu_texture_creation() {
        let d = TextureDescriptor::rgba(2, 2);
        let data = vec![0u8; 16];
        let t = GpuTexture::new(1, TextureKind::Image, d, data);
        assert_eq!(t.id, 1);
        assert!(t.is_valid());
        assert!(t.dirty);
    }

    #[test]
    fn test_gpu_texture_dirty() {
        let d = TextureDescriptor::rgba(2, 2);
        let data = vec![0u8; 16];
        let mut t = GpuTexture::new(1, TextureKind::Image, d, data);
        t.mark_clean();
        assert!(!t.dirty);
        t.mark_dirty();
        assert!(t.dirty);
    }

    #[test]
    fn test_glyph_atlas_new() {
        let atlas = GlyphAtlas::new(256, 256);
        assert_eq!(atlas.width, 256);
        assert_eq!(atlas.glyph_count(), 0);
    }

    #[test]
    fn test_glyph_atlas_upload() {
        let mut atlas = GlyphAtlas::new(256, 256);
        let bitmap = vec![0xFF; 8 * 8];
        let slot = atlas.upload_glyph('A', &bitmap, 8, 8).unwrap();
        assert_eq!(slot.w, 8);
        assert_eq!(slot.h, 8);
        assert!(atlas.has_glyph('A'));
    }

    #[test]
    fn test_glyph_atlas_row_wrap() {
        let mut atlas = GlyphAtlas::new(50, 50);
        // Cada glyph es 20x20
        for i in 0..3 {
            let ch = char::from_u32(65 + i).unwrap();
            let bitmap = vec![0xFF; 20 * 20];
            atlas.upload_glyph(ch, &bitmap, 20, 20).unwrap();
        }
        // El tercero debe estar en la segunda fila
        let slot = atlas.get_glyph('C').unwrap();
        assert!(slot.y > 0, "Third glyph should wrap to next row");
    }

    #[test]
    fn test_glyph_atlas_full() {
        let mut atlas = GlyphAtlas::new(10, 10);
        let bitmap = vec![0xFF; 10 * 10];
        assert!(atlas.upload_glyph('A', &bitmap, 10, 10).is_some());
        // Atlas lleno - no mas espacio
        let result = atlas.upload_glyph('B', &bitmap, 10, 10);
        assert!(result.is_none());
    }

    #[test]
    fn test_glyph_atlas_clear() {
        let mut atlas = GlyphAtlas::new(256, 256);
        let bitmap = vec![0xFF; 8 * 8];
        atlas.upload_glyph('A', &bitmap, 8, 8).unwrap();
        atlas.clear();
        assert_eq!(atlas.glyph_count(), 0);
    }

    #[test]
    fn test_texture_pool_creation() {
        let pool = TexturePool::new();
        assert_eq!(pool.count(), 0);
        assert_eq!(pool.total_memory, 0);
    }

    #[test]
    fn test_texture_pool_allocate() {
        let mut pool = TexturePool::new();
        let id = pool.allocate(TextureKind::Image, TextureDescriptor::rgba(10, 10));
        assert_eq!(pool.count(), 1);
        assert_eq!(pool.total_memory, 400);
    }

    #[test]
    fn test_texture_pool_deallocate() {
        let mut pool = TexturePool::new();
        let id = pool.allocate(TextureKind::Image, TextureDescriptor::rgba(10, 10));
        assert!(pool.deallocate(id));
        assert_eq!(pool.count(), 0);
    }

    #[test]
    fn test_texture_pool_upload() {
        let mut pool = TexturePool::new();
        let id = pool.allocate(TextureKind::Image, TextureDescriptor::rgba(2, 2));
        let data = vec![0xFFu8; 16];
        assert!(pool.upload(id, data).is_ok());
    }

    #[test]
    fn test_texture_pool_upload_wrong_size() {
        let mut pool = TexturePool::new();
        let id = pool.allocate(TextureKind::Image, TextureDescriptor::rgba(2, 2));
        let data = vec![0xFFu8; 8];  // wrong size
        assert!(pool.upload(id, data).is_err());
    }

    #[test]
    fn test_texture_pool_count_of_kind() {
        let mut pool = TexturePool::new();
        pool.allocate(TextureKind::Image, TextureDescriptor::rgba(2, 2));
        pool.allocate(TextureKind::Image, TextureDescriptor::rgba(2, 2));
        pool.allocate(TextureKind::Glyph, TextureDescriptor::rgba(2, 2));
        assert_eq!(pool.count_of_kind(TextureKind::Image), 2);
        assert_eq!(pool.count_of_kind(TextureKind::Glyph), 1);
    }

    #[test]
    fn test_texture_pool_evict() {
        let mut pool = TexturePool::with_max_size(10);
        for _ in 0..5 {
            let id = pool.allocate(TextureKind::Image, TextureDescriptor::rgba(2, 2));
            if let Some(t) = pool.get_mut(id) {
                t.mark_clean();  // mark not dirty
            }
        }
        pool.evict_unused(3);
        assert!(pool.count() <= 3);
    }
}
