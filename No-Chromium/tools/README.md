# No-Chromium Tools

Estas herramientas son para automatizar analisis pesado fuera del renderer.

## Flujo recomendado

1. Ejecutar `page_probe.py` sobre una pagina real o un `.body` cacheado.
2. Mirar `unsupported_properties`, scripts grandes, media y formularios.
3. Implementar en Rust solo lo que aporta render/calidad real.
4. Repetir con el mismo cache para comparar antes/despues.

## Comandos utiles

```powershell
python tools/page_probe.py profile/cache/resources/document/page.body --top 30
python tools/page_probe.py --json profile/cache/resources/document/page.body
python tools/page_probe.py --fetch-css https://www.iana.org/domains/example
python tools/app_shell_probe.py --json profile/cache/resources/document/page.body
python tools/app_shell_probe.py --yt-dlp https://www.youtube.com/watch?v=VIDEO_ID
```

`page_probe.py` es general: DOM, CSSOM inicial, scripts, media y controles.
`app_shell_probe.py` es especializado para SPAs y YouTube.
