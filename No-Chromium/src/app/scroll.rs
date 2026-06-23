//! Smooth Scrolling (FASE A4)
//!
//! Implementa scroll suave estilo Chrome/Brave.
//! Incluye:
//! - Scroll inercial (decelera despues de soltar)
//! - Scrollbar visual
//! - Bound checking (no scroll fuera del contenido)
//! - Scroll to element
//! - Page down/up, Home/End

/// Estado del scroll
#[derive(Debug, Clone)]
pub struct ScrollState {
    pub offset_y: f32,
    pub velocity: f32,           // px/frame
    pub target_offset: f32,      // para scroll suave
    pub content_height: f32,    // altura total del contenido
    pub viewport_height: f32,    // altura visible
    pub inertia: bool,
    pub last_update_ms: u64,
}

impl ScrollState {
    pub fn new(viewport_h: f32) -> Self {
        Self {
            offset_y: 0.0,
            velocity: 0.0,
            target_offset: 0.0,
            content_height: 0.0,
            viewport_height: viewport_h,
            inertia: true,
            last_update_ms: 0,
        }
    }

    /// Set viewport height (cuando cambia el tamano de la ventana)
    pub fn set_viewport_height(&mut self, h: f32) {
        self.viewport_height = h;
        self.clamp();
    }

    /// Actualizar contenido (cuando cambia el layout)
    pub fn set_content_height(&mut self, h: f32) {
        self.content_height = h;
        self.clamp();
    }

    /// Scroll por delta (wheel, touchpad)
    pub fn scroll_by(&mut self, delta: f32) {
        self.target_offset += delta;
        self.velocity = delta;  // para inercia
        self.clamp();
    }

    /// Scroll a posicion absoluta
    pub fn scroll_to(&mut self, y: f32) {
        self.target_offset = y;
        self.velocity = 0.0;
        self.clamp();
    }

    /// Scroll to top (inmediato, sin lerp)
    pub fn scroll_to_top(&mut self) {
        self.target_offset = 0.0;
        self.offset_y = 0.0;
        self.velocity = 0.0;
    }

    /// Scroll to bottom (inmediato, sin lerp)
    pub fn scroll_to_bottom(&mut self) {
        self.target_offset = self.max_offset();
        self.offset_y = self.target_offset;
        self.velocity = 0.0;
    }

    /// Page down
    pub fn page_down(&mut self) {
        self.scroll_by(self.viewport_height * 0.9);
    }

    /// Page up
    pub fn page_up(&mut self) {
        self.scroll_by(-self.viewport_height * 0.9);
    }

    /// Actualiza la simulacion (llamar cada frame)
    pub fn update(&mut self, dt_ms: u32) {
        if self.inertia && self.velocity.abs() > 0.5 {
            // Aplicar velocidad al target
            self.target_offset += self.velocity;
            // Deceleracion
            self.velocity *= 0.92;
            // Parar si es muy baja
            if self.velocity.abs() < 0.5 {
                self.velocity = 0.0;
            }
        }
        // Smooth interpolation hacia target
        let diff = self.target_offset - self.offset_y;
        if diff.abs() > 0.5 {
            // Lerp: 20% del camino por frame
            self.offset_y += diff * 0.20;
        } else {
            self.offset_y = self.target_offset;
        }
        self.clamp();
        self.last_update_ms += dt_ms as u64;
    }

    fn clamp(&mut self) {
        let max = self.max_offset();
        if self.target_offset < 0.0 {
            self.target_offset = 0.0;
            self.velocity = 0.0;
        }
        if self.target_offset > max {
            self.target_offset = max;
            self.velocity = 0.0;
        }
        if self.offset_y < 0.0 {
            self.offset_y = 0.0;
        }
        if self.offset_y > max {
            self.offset_y = max;
        }
    }

    pub fn max_offset(&self) -> f32 {
        (self.content_height - self.viewport_height).max(0.0)
    }

    /// Porcentaje de scroll (0.0 = top, 1.0 = bottom)
    pub fn progress(&self) -> f32 {
        if self.max_offset() == 0.0 {
            0.0
        } else {
            self.offset_y / self.max_offset()
        }
    }

    /// Tamano del thumb del scrollbar (0.0 - 1.0)
    pub fn scrollbar_thumb_size(&self) -> f32 {
        if self.content_height == 0.0 {
            0.0
        } else {
            (self.viewport_height / self.content_height).min(1.0)
        }
    }

    /// Posicion del thumb del scrollbar (0.0 - 1.0)
    pub fn scrollbar_thumb_pos(&self) -> f32 {
        self.progress()
    }

    pub fn at_top(&self) -> bool {
        self.offset_y <= 0.0
    }

    pub fn at_bottom(&self) -> bool {
        self.offset_y >= self.max_offset()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scroll_creation() {
        let s = ScrollState::new(800.0);
        assert_eq!(s.offset_y, 0.0);
    }

    #[test]
    fn test_scroll_by() {
        let mut s = ScrollState::new(800.0);
        s.set_content_height(2000.0);
        s.scroll_by(100.0);
        assert!(s.target_offset > 0.0);
    }

    #[test]
    fn test_scroll_clamp_top() {
        let mut s = ScrollState::new(800.0);
        s.set_content_height(2000.0);
        s.scroll_by(-100.0);  // scroll up past top
        assert_eq!(s.target_offset, 0.0);
    }

    #[test]
    fn test_scroll_clamp_bottom() {
        let mut s = ScrollState::new(800.0);
        s.set_content_height(2000.0);
        s.max_offset();  // = 1200
        s.scroll_by(5000.0);  // way past bottom
        assert_eq!(s.target_offset, s.max_offset());
    }

    #[test]
    fn test_page_down() {
        let mut s = ScrollState::new(800.0);
        s.set_content_height(5000.0);
        s.page_down();
        // page down deberia scrollear 90% viewport
        assert!(s.target_offset > 0.0);
    }

    #[test]
    fn test_scroll_to_top() {
        let mut s = ScrollState::new(800.0);
        s.set_content_height(2000.0);
        s.scroll_by(500.0);
        s.scroll_to_top();
        assert_eq!(s.target_offset, 0.0);
    }

    #[test]
    fn test_scroll_to_bottom() {
        let mut s = ScrollState::new(800.0);
        s.set_content_height(2000.0);
        s.scroll_to_bottom();
        assert_eq!(s.target_offset, s.max_offset());
    }

    #[test]
    fn test_progress() {
        let mut s = ScrollState::new(800.0);
        s.set_content_height(2000.0);
        s.scroll_to_bottom();
        let p = s.progress();
        assert!((p - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_inertia_decel() {
        let mut s = ScrollState::new(800.0);
        s.set_content_height(5000.0);
        s.scroll_by(100.0);
        s.velocity = 50.0;  // initial velocity
        s.update(16);
        assert!(s.velocity < 50.0);  // decelero
    }

    #[test]
    fn test_smooth_interp() {
        let mut s = ScrollState::new(800.0);
        s.set_content_height(5000.0);
        s.scroll_by(100.0);
        s.velocity = 0.0;  // no inertia
        let initial = s.offset_y;
        s.update(16);
        // El offset_y debe acercarse al target
        assert!(s.offset_y > initial);
    }

    #[test]
    fn test_scrollbar_thumb_size() {
        let mut s = ScrollState::new(800.0);
        s.set_content_height(2000.0);
        let size = s.scrollbar_thumb_size();
        assert!((size - 0.4).abs() < 0.01);  // 800/2000 = 0.4
    }

    #[test]
    fn test_at_top_bottom() {
        let mut s = ScrollState::new(800.0);
        s.set_content_height(2000.0);
        assert!(s.at_top());
        s.scroll_to_bottom();
        assert!(s.at_bottom());
    }

    #[test]
    fn test_small_content_no_scroll() {
        let mut s = ScrollState::new(800.0);
        s.set_content_height(500.0);  // menor que viewport
        s.scroll_by(1000.0);
        assert_eq!(s.target_offset, 0.0);  // no se puede scroll
    }
}
