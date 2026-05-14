mod generated_rust;
use generated_rust::array_logic::JsArray;
use std::time::Instant;

fn main() {
    let size = 10_000_000; // 10 Millones de elementos
    println!("[*] Inicializando array de {} elementos...", size);
    
    let mut elements = Vec::with_capacity(size);
    for i in 0..size {
        elements.push(Some(i));
    }
    
    let arr = JsArray { elements };
    
    println!("[*] Iniciando Benchmark: Array.map (ADN V8 en Rust)...");
    
    let start = Instant::now();
    
    // Ejecutamos el mapeo nativo (x => x * 2)
    let result = arr.map_native(|val, _idx| val * 2);
    
    let duration = start.elapsed();
    
    println!("========================================");
    println!("       BENCHMARK NO-CHROMIUM           ");
    println!("========================================");
    println!("Tiempo total: {:?}", duration);
    println!("Elementos procesados: {}", result.elements.len());
    println!("Velocidad: ~{:.2} millones de op/seg", (size as f64 / duration.as_secs_f64()) / 1_000_000.0);
    println!("========================================");
}
