# SVG Icon Library

Iconos base para No-Chromium. Estan en SVG puro, con trazos simples y colores heredables mediante `currentColor` para que puedan tintarse desde Rust, CSS o un futuro atlas de iconos.

Primer set:
- `browser-close.svg`
- `browser-minimize.svg`
- `browser-maximize.svg`
- `nav-back.svg`
- `nav-forward.svg`
- `nav-reload.svg`
- `nav-home.svg`
- `internet-globe.svg`
- `security-lock.svg`
- `security-shield.svg`
- `search.svg`
- `bookmark-star.svg`
- `brand-youtube.svg`
- `brand-github.svg`
- `brand-x.svg`
- `brand-openai.svg`
- `framework-threejs.svg`
- `framework-spline.svg`
- `framework-rive.svg`
- `icon-manifest.json`

La UI actual ya dibuja una version vectorial inmediata de los iconos principales desde `src/ui/ui_gen.rs`. Estos archivos quedan como ADN visual para evolucionar hacia atlas SVG, favicons y guias de navegacion.
