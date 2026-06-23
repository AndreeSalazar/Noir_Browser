//! Image Loading - Lazy loading, placeholders, dimensionado automático
//!
//! Maneja el ciclo de vida visual de imágenes en la página.

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ImageState {
    /// Aún no se ha solicitado
    Pending,
    /// Esperando respuesta
    Loading,
    /// Cargada y lista
    Loaded,
    /// Error al cargar
    Failed,
}

#[derive(Debug, Clone)]
pub struct ImageInfo {
    pub src: String,
    pub alt: String,
    pub natural_w: u32,
    pub natural_h: u32,
    pub display_w: u32,
    pub display_h: u32,
    pub state: ImageState,
    pub lazy: bool,
    pub retry_count: u32,
}

impl ImageInfo {
    pub fn new(src: &str, alt: &str, w: u32, h: u32) -> Self {
        Self {
            src: src.to_string(),
            alt: alt.to_string(),
            natural_w: w,
            natural_h: h,
            display_w: w,
            display_h: h,
            state: ImageState::Pending,
            lazy: false,
            retry_count: 0,
        }
    }

    /// Calcula dimensiones manteniendo aspect ratio
    pub fn fit_into(&mut self, max_w: u32, max_h: u32) {
        if self.natural_w == 0 || self.natural_h == 0 {
            self.display_w = max_w.min(300);
            self.display_h = max_h.min(200);
            return;
        }
        let ratio = self.natural_w as f64 / self.natural_h as f64;
        if max_w as f64 / max_h as f64 > ratio {
            self.display_h = max_h;
            self.display_w = (max_h as f64 * ratio) as u32;
        } else {
            self.display_w = max_w;
            self.display_h = (max_w as f64 / ratio) as u32;
        }
    }

    /// Genera texto placeholder mientras se carga
    pub fn placeholder_text(&self) -> String {
        format!("[image: {}x{}]", self.display_w, self.display_h)
    }

    /// Aspect ratio como string "16:9"
    pub fn aspect_ratio(&self) -> String {
        if self.natural_h == 0 {
            return "1:1".to_string();
        }
        let r = self.natural_w as f64 / self.natural_h as f64;
        if (r - 16.0/9.0).abs() < 0.05 { return "16:9".to_string(); }
        if (r - 4.0/3.0).abs() < 0.05 { return "4:3".to_string(); }
        if (r - 1.0).abs() < 0.05 { return "1:1".to_string(); }
        if (r - 21.0/9.0).abs() < 0.05 { return "21:9".to_string(); }
        if (r - 3.0/2.0).abs() < 0.05 { return "3:2".to_string(); }
        format!("{:.2}:1", r)
    }
}

pub struct ImageLoader {
    images: Vec<ImageInfo>,
    max_concurrent: u32,
    viewport_h: u32,
}

impl ImageLoader {
    pub fn new() -> Self {
        Self {
            images: Vec::new(),
            max_concurrent: 6,
            viewport_h: 1080,
        }
    }

    pub fn with_viewport(mut self, h: u32) -> Self {
        self.viewport_h = h;
        self
    }

    pub fn add(&mut self, info: ImageInfo) {
        self.images.push(info);
    }

    /// Marca imágenes visibles para carga
    pub fn trigger_visible(&mut self, scroll_y: i32) {
        let top = scroll_y;
        let bottom = scroll_y + self.viewport_h as i32;
        let mut triggered = 0;
        let n = self.images.len();
        for idx in 0..n {
            let img_top = idx as i32 * 300;
            if img_top >= top && img_top <= bottom {
                if let Some(img) = self.images.get_mut(idx) {
                    if img.lazy && img.state == ImageState::Pending {
                        img.state = ImageState::Loading;
                        triggered += 1;
                        if triggered >= self.max_concurrent { break; }
                    }
                }
            }
        }
    }

    /// Marca todas las imágenes no-lazy como loading
    pub fn trigger_eager(&mut self) {
        for img in &mut self.images {
            if !img.lazy && img.state == ImageState::Pending {
                img.state = ImageState::Loading;
            }
        }
    }

    pub fn count_by_state(&self, state: ImageState) -> usize {
        self.images.iter().filter(|i| i.state == state).count()
    }

    pub fn total(&self) -> usize {
        self.images.len()
    }

    pub fn loaded(&self) -> usize {
        self.count_by_state(ImageState::Loaded)
    }

    pub fn failed(&self) -> usize {
        self.count_by_state(ImageState::Failed)
    }

    /// Progreso 0.0-1.0
    pub fn progress(&self) -> f32 {
        if self.images.is_empty() { return 1.0; }
        let done = self.loaded() + self.failed();
        done as f32 / self.images.len() as f32
    }

    /// Parsea dimensiones de atributos width/height
    pub fn parse_dimensions(width_attr: Option<&str>, height_attr: Option<&str>) -> (u32, u32) {
        let w = width_attr.and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
        let h = height_attr.and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
        (w, h)
    }
}

impl Default for ImageLoader {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_info_new() {
        let info = ImageInfo::new("https://x.com/a.png", "alt", 800, 600);
        assert_eq!(info.state, ImageState::Pending);
        assert_eq!(info.natural_w, 800);
    }

    #[test]
    fn test_fit_into_wider() {
        let mut info = ImageInfo::new("a.png", "", 800, 600);
        info.fit_into(400, 300);
        assert_eq!(info.display_w, 400);
        assert_eq!(info.display_h, 300);
    }

    #[test]
    fn test_fit_into_taller_constraint() {
        let mut info = ImageInfo::new("a.png", "", 800, 600);
        info.fit_into(800, 100);
        assert_eq!(info.display_h, 100);
        assert!(info.display_w < 800);
    }

    #[test]
    fn test_fit_into_zero_dim() {
        let mut info = ImageInfo::new("a.png", "", 0, 0);
        info.fit_into(500, 500);
        assert_eq!(info.display_w, 300);
        assert_eq!(info.display_h, 200);
    }

    #[test]
    fn test_placeholder_text() {
        let info = ImageInfo::new("a.png", "", 320, 180);
        assert_eq!(info.placeholder_text(), "[image: 320x180]");
    }

    #[test]
    fn test_aspect_ratio_16_9() {
        let info = ImageInfo::new("a.png", "", 1920, 1080);
        assert_eq!(info.aspect_ratio(), "16:9");
    }

    #[test]
    fn test_aspect_ratio_4_3() {
        let info = ImageInfo::new("a.png", "", 800, 600);
        assert_eq!(info.aspect_ratio(), "4:3");
    }

    #[test]
    fn test_aspect_ratio_1_1() {
        let info = ImageInfo::new("a.png", "", 500, 500);
        assert_eq!(info.aspect_ratio(), "1:1");
    }

    #[test]
    fn test_aspect_ratio_zero_h() {
        let info = ImageInfo::new("a.png", "", 500, 0);
        assert_eq!(info.aspect_ratio(), "1:1");
    }

    #[test]
    fn test_loader_new() {
        let l = ImageLoader::new();
        assert_eq!(l.total(), 0);
        assert_eq!(l.progress(), 1.0);
    }

    #[test]
    fn test_loader_add() {
        let mut l = ImageLoader::new();
        l.add(ImageInfo::new("a.png", "", 100, 100));
        l.add(ImageInfo::new("b.png", "", 200, 200));
        assert_eq!(l.total(), 2);
    }

    #[test]
    fn test_trigger_eager() {
        let mut l = ImageLoader::new();
        l.add(ImageInfo::new("a.png", "", 100, 100));
        l.add(ImageInfo::new("b.png", "", 200, 200));
        l.trigger_eager();
        assert_eq!(l.count_by_state(ImageState::Loading), 2);
    }

    #[test]
    fn test_lazy_not_triggered_by_eager() {
        let mut l = ImageLoader::new();
        let mut img = ImageInfo::new("a.png", "", 100, 100);
        img.lazy = true;
        l.add(img);
        l.trigger_eager();
        assert_eq!(l.count_by_state(ImageState::Pending), 1);
    }

    #[test]
    fn test_progress_partial() {
        let mut l = ImageLoader::new();
        let mut a = ImageInfo::new("a.png", "", 100, 100);
        a.state = ImageState::Loaded;
        let mut b = ImageInfo::new("b.png", "", 100, 100);
        b.state = ImageState::Failed;
        l.add(a);
        l.add(b);
        assert_eq!(l.progress(), 1.0);
    }

    #[test]
    fn test_parse_dimensions() {
        assert_eq!(ImageLoader::parse_dimensions(Some("800"), Some("600")), (800, 600));
        assert_eq!(ImageLoader::parse_dimensions(None, Some("600")), (0, 600));
        assert_eq!(ImageLoader::parse_dimensions(Some("abc"), None), (0, 0));
    }

    #[test]
    fn test_count_by_state() {
        let mut l = ImageLoader::new();
        let mut a = ImageInfo::new("a.png", "", 100, 100);
        a.state = ImageState::Loaded;
        l.add(a);
        l.add(ImageInfo::new("b.png", "", 100, 100));
        assert_eq!(l.loaded(), 1);
        assert_eq!(l.count_by_state(ImageState::Pending), 1);
    }
}
