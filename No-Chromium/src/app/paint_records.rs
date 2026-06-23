//! Paint Records (Chrome Blink-inspired)
//!
//! Basado en el algoritmo de Chrome Blink:
//! - En vez de pintar directamente, generar un log de operaciones ("paint records")
//! - Cada record es una instruccion: draw_rect, draw_text, draw_image
//! - Permite replay, caching, y optimizaciones de dirty regions
//! - En restyle, solo se re-pintan los rectangulos sucios (dirty regions)

use std::collections::HashSet;

/// Tipo de operacion de paint
#[derive(Debug, Clone, PartialEq)]
pub enum PaintOp {
    /// Dibujar rectangulo solido
    FillRect { x: i32, y: i32, w: i32, h: i32, color: u32 },
    /// Dibujar texto
    DrawText { x: i32, y: i32, text: String, color: u32, scale: f32 },
    /// Dibujar borde
    DrawBorder { x: i32, y: i32, w: i32, h: i32, color: u32, thickness: u32 },
    /// Dibujar imagen (placeholder)
    DrawImage { x: i32, y: i32, w: i32, h: i32, src: String },
    /// Dibujar icono (procedural)
    DrawIcon { x: i32, y: i32, kind: IconKind, color: u32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IconKind {
    Back,
    Forward,
    Reload,
    Home,
    Lock,
    Play,
    Pause,
    Volume,
    Fullscreen,
}

/// Un record de paint individual
#[derive(Debug, Clone)]
pub struct PaintRecord {
    pub op: PaintOp,
    /// Region que ocupa (para dirty tracking)
    pub dirty_rect: (i32, i32, i32, i32),
}

/// Lista de paint records (Chrome paint pipeline)
#[derive(Debug, Default)]
pub struct PaintRecords {
    records: Vec<PaintRecord>,
    /// Rectangulos sucios (no re-pintados)
    dirty_regions: HashSet<(i32, i32, i32, i32)>,
    /// Estadisticas
    pub total_ops: u64,
    pub culled_ops: u64,
}

impl PaintRecords {
    pub fn new() -> Self {
        Self::default()
    }

    /// Agrega un paint op con su region
    pub fn push(&mut self, op: PaintOp, dirty_rect: (i32, i32, i32, i32)) {
        self.records.push(PaintRecord { op, dirty_rect });
        self.dirty_regions.insert(dirty_rect);
        self.total_ops += 1;
    }

    /// Helpers de alto nivel
    pub fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: u32) {
        self.push(PaintOp::FillRect { x, y, w, h, color }, (x, y, w, h));
    }

    pub fn draw_text(&mut self, x: i32, y: i32, text: &str, color: u32, scale: f32) {
        let w = (text.chars().count() as i32) * 7 + 4;
        let h = 14;
        self.push(
            PaintOp::DrawText { x, y, text: text.to_string(), color, scale },
            (x, y, w, h),
        );
    }

    pub fn draw_border(&mut self, x: i32, y: i32, w: i32, h: i32, color: u32, thickness: u32) {
        self.push(
            PaintOp::DrawBorder { x, y, w, h, color, thickness },
            (x, y, w, h),
        );
    }

    pub fn draw_image(&mut self, x: i32, y: i32, w: i32, h: i32, src: &str) {
        self.push(
            PaintOp::DrawImage { x, y, w, h, src: src.to_string() },
            (x, y, w, h),
        );
    }

    pub fn draw_icon(&mut self, x: i32, y: i32, kind: IconKind, color: u32) {
        self.push(
            PaintOp::DrawIcon { x, y, kind, color },
            (x - 8, y - 8, 16, 16),
        );
    }

    /// Cull records que no intersectan con el dirty rect
    pub fn cull(&mut self, dirty: (i32, i32, i32, i32)) -> Vec<PaintRecord> {
        let mut out = Vec::new();
        for rec in self.records.drain(..) {
            if rects_intersect(rec.dirty_rect, dirty) {
                out.push(rec);
            } else {
                self.culled_ops += 1;
            }
        }
        out
    }

    /// Replay sobre un buffer
    pub fn replay<F: FnMut(&PaintOp)>(&self, mut f: F) {
        for rec in &self.records {
            f(&rec.op);
        }
    }

    /// Cuantos records hay
    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// Limpia todo
    pub fn clear(&mut self) {
        self.records.clear();
        self.dirty_regions.clear();
    }

    /// Estadisticas: hit rate de culling
    pub fn stats(&self) -> (u64, u64, f32) {
        let cull_rate = if self.total_ops > 0 {
            self.culled_ops as f32 / self.total_ops as f32
        } else {
            0.0
        };
        (self.total_ops, self.culled_ops, cull_rate)
    }
}

/// Verifica si dos rectangulos se intersectan
fn rects_intersect(a: (i32, i32, i32, i32), b: (i32, i32, i32, i32)) -> bool {
    let (ax, ay, aw, ah) = a;
    let (bx, by, bw, bh) = b;
    ax < bx + bw && ax + aw > bx && ay < by + bh && ay + ah > by
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_creation() {
        let records = PaintRecords::new();
        assert_eq!(records.len(), 0);
    }

    #[test]
    fn test_push_fill_rect() {
        let mut records = PaintRecords::new();
        records.fill_rect(10, 20, 100, 50, 0xFF000000);
        assert_eq!(records.len(), 1);
    }

    #[test]
    fn test_push_multiple() {
        let mut records = PaintRecords::new();
        records.fill_rect(0, 0, 10, 10, 0xFF);
        records.draw_text(20, 20, "Hello", 0xFF, 1.0);
        records.draw_border(30, 30, 50, 50, 0xFF, 1);
        assert_eq!(records.len(), 3);
    }

    #[test]
    fn test_cull_intersecting() {
        let mut records = PaintRecords::new();
        records.fill_rect(0, 0, 10, 10, 0xFF);  // in viewport
        records.fill_rect(1000, 1000, 10, 10, 0xFF);  // far away
        let visible = records.cull((0, 0, 500, 500));
        assert_eq!(visible.len(), 1);
    }

    #[test]
    fn test_cull_culled_count() {
        let mut records = PaintRecords::new();
        for i in 0..10 {
            records.fill_rect(i * 1000, 0, 10, 10, 0xFF);
        }
        let visible = records.cull((0, 0, 500, 500));
        assert_eq!(visible.len(), 1);
        let (total, culled, _) = records.stats();
        assert_eq!(total, 10);
        assert_eq!(culled, 9);
    }

    #[test]
    fn test_replay() {
        let mut records = PaintRecords::new();
        records.fill_rect(0, 0, 10, 10, 0xFF);
        records.draw_text(20, 20, "test", 0xFF, 1.0);
        let mut count = 0;
        records.replay(|_| count += 1);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_clear() {
        let mut records = PaintRecords::new();
        records.fill_rect(0, 0, 10, 10, 0xFF);
        records.clear();
        assert!(records.is_empty());
    }

    #[test]
    fn test_icon_kinds() {
        let mut records = PaintRecords::new();
        records.draw_icon(0, 0, IconKind::Back, 0xFF);
        records.draw_icon(0, 0, IconKind::Home, 0xFF);
        assert_eq!(records.len(), 2);
    }

    #[test]
    fn test_rects_intersect() {
        assert!(rects_intersect((0, 0, 10, 10), (5, 5, 10, 10)));
        assert!(!rects_intersect((0, 0, 10, 10), (20, 20, 10, 10)));
        assert!(rects_intersect((0, 0, 100, 100), (50, 50, 10, 10)));
    }
}
