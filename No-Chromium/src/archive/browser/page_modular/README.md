# Page Modular

Esta carpeta divide la antigua `browser/page.rs` por responsabilidades.

- `mod.rs`: fachada interna de pagina, carga del documento, estilos base, extraccion DOM, layout de texto y render.
- `app_shell.rs`: vistas ligeras para SPAs/JS-heavy apps, incluyendo YouTube `ytInitialData` y `ytInitialPlayerResponse`.

Siguientes extracciones naturales:

- `style.rs`: derivacion de fondo/color/contraste desde CSS.
- `layout.rs`: caja de texto, porcentajes, `calc(...)`, wrapping y hitboxes.
- `dom_text.rs`: recorrido DOM, defaults por etiqueta y aplicacion de CSS.
- `summaries.rs`: banners de HTTP, CSS y media.
