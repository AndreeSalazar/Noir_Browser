# Chromium ADN Notes

Estas notas son para inspiracion arquitectonica, no para copiar codigo.

Fuentes primarias revisadas:
- Chromium Life of a Navigation: `https://chromium.googlesource.com/chromium/src/+/master/docs/navigation.md`
- Chromium Network Stack: `https://www.chromium.org/developers/design-documents/network-stack/`
- Chromium Audio / Video Playback: `https://www.chromium.org/developers/design-documents/video/`

## Reglas que adoptamos

1. Una navegacion no es solo un string HTML.
   Debe producir metadatos: URL solicitada, URL final, estado HTTP, content-type, cuerpo y tamano.

2. El browser state decide cuando una carga se acepta.
   Si llega una respuesta vieja despues de navegar a otra URL, se descarta.

3. Media se descubre antes de reproducirse.
   Primero detectamos audio/video/source/embed/MSE; despues vendra resolver streams y decodificar.

4. Historial y documento son capas separadas.
   El historial registra visitas confirmadas; el documento conserva contenido/render/media.

5. El loader debe poder revalidar.
   Si hay `ETag` o `Last-Modified`, se usan `If-None-Match` y `If-Modified-Since`; si llega `304`, el cuerpo sale del cache local.

6. Los recursos tienen tipo desde el borde de red.
   `Document`, `Style`, `Script`, `Media`, `Image` y `Other` usan accept headers y buckets de cache separados.

7. CSS empieza como cascade pequeno, no como motor gigante.
   Primero respetamos selectores simples y propiedades visibles: ocultamiento, color, font-size, font-weight, line-height, margin-bottom y text-transform.

## Siguiente fase

- Cache con expiracion y control de tamano.
- Ampliar selectores CSS: descendientes completos, atributos y pseudo-clases simples.
- Scripts externos usando `ResourceType::Script`.
- Audio backend Rust con `cpal` o `rodio`.
- Resolver MSE/HLS/DASH para streams modernos.
