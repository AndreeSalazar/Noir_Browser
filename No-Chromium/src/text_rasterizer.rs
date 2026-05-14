use fontdue::{Font, FontSettings};
use std::fs;

pub struct RasterizedText {
    pub width: u32,
    pub height: u32,
    pub rgba_data: Vec<u8>,
}

impl RasterizedText {
    pub fn new(text: &str, px_size: f32) -> Self {
        println!("[*] Rasterizando texto en CPU: '{}'", text);
        
        // Cargar fuente nativa de Windows (Arial)
        let font_bytes = fs::read("C:\\Windows\\Fonts\\arial.ttf")
            .expect("Fallo al leer la fuente Arial de Windows");
            
        let font = Font::from_bytes(font_bytes, FontSettings::default())
            .expect("Fallo al parsear la fuente Arial");

        // Calcular el tamaño total del canvas necesario
        // Para simplificar esta prueba de concepto, rasterizaremos letra por letra y las juntaremos
        // horizontalmente en un solo bitmap continuo.
        
        let mut total_width = 0;
        let mut max_height = 0;
        
        // Primera pasada: medir
        let mut glyph_layouts = Vec::new();
        for c in text.chars() {
            let (metrics, bitmap) = font.rasterize(c, px_size);
            total_width += metrics.width + 2; // +2 de espaciado
            if metrics.height > max_height {
                max_height = metrics.height;
            }
            glyph_layouts.push((metrics, bitmap));
        }

        // Crear buffer RGBA vacío
        let mut rgba_data = vec![0u8; (total_width * max_height * 4) as usize];

        // Segunda pasada: dibujar en el buffer continuo
        let mut cursor_x = 0;
        for (metrics, bitmap) in glyph_layouts {
            for y in 0..metrics.height {
                for x in 0..metrics.width {
                    // fontdue retorna alpha puro (escala de grises)
                    let alpha = bitmap[y * metrics.width + x];
                    if alpha > 0 {
                        // Calcular índice en la textura gigante
                        let global_x = cursor_x + x;
                        let global_y = y; // Simple align to top
                        
                        let idx = (global_y * total_width + global_x) * 4;
                        
                        rgba_data[idx] = 255;       // R (Blanco puro)
                        rgba_data[idx + 1] = 255;   // G
                        rgba_data[idx + 2] = 255;   // B
                        rgba_data[idx + 3] = alpha; // A
                    }
                }
            }
            cursor_x += metrics.width + 2;
        }

        println!("[+] Texto rasterizado exitosamente. Dimensión textura: {}x{} píxeles.", total_width, max_height);

        Self {
            width: total_width as u32,
            height: max_height as u32,
            rgba_data,
        }
    }
}
