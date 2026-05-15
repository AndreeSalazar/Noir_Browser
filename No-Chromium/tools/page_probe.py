#!/usr/bin/env python3
"""Probe a real page/cache body and summarize DOM, CSSOM and JS bootstrap signals.

This is intentionally a heavy offline/diagnostic tool. Rust stays as the browser
runtime; Python helps inspect messy real-world pages and produce JSON we can use
to decide the next native features.

Examples:
  python tools/page_probe.py profile/cache/resources/document/page.body
  python tools/page_probe.py --json https://www.youtube.com/
  python tools/page_probe.py --fetch-css --top 30 https://www.iana.org/domains/example
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from collections import Counter, defaultdict
from concurrent.futures import ThreadPoolExecutor, as_completed
from html.parser import HTMLParser
from pathlib import Path
from typing import Any
from urllib.parse import urljoin
from urllib.request import Request, urlopen


SUPPORTED_CSS_PROPERTIES = {
    "background",
    "background-color",
    "color",
    "display",
    "font",
    "font-family",
    "font-size",
    "font-style",
    "font-weight",
    "height",
    "line-height",
    "margin",
    "margin-bottom",
    "margin-left",
    "margin-right",
    "margin-top",
    "max-width",
    "min-height",
    "opacity",
    "padding",
    "padding-bottom",
    "padding-left",
    "padding-right",
    "padding-top",
    "text-align",
    "text-transform",
    "visibility",
    "width",
}

APP_JSON_MARKERS = (
    "ytInitialData",
    "ytInitialPlayerResponse",
    "__NEXT_DATA__",
    "__NUXT__",
    "__APOLLO_STATE__",
    "__INITIAL_STATE__",
    "__PRELOADED_STATE__",
    "window.__data",
)

MEDIA_TAGS = {"img", "picture", "source", "video", "audio", "iframe", "embed", "object"}
FORM_TAGS = {"form", "input", "button", "select", "textarea", "option", "label"}


def read_source(source: str) -> tuple[str, str | None]:
    if source.startswith(("http://", "https://")):
        request = Request(
            source,
            headers={
                "User-Agent": "No-Chromium page probe",
                "Accept": "text/html,application/xhtml+xml,text/plain,*/*;q=0.5",
            },
        )
        with urlopen(request, timeout=25) as response:
            charset = response.headers.get_content_charset() or "utf-8"
            body = response.read().decode(charset, "replace")
            return body, response.geturl()
    return Path(source).read_text(encoding="utf-8", errors="replace"), None


def fetch_text(url: str) -> str:
    request = Request(url, headers={"User-Agent": "No-Chromium page probe"})
    with urlopen(request, timeout=20) as response:
        charset = response.headers.get_content_charset() or "utf-8"
        return response.read().decode(charset, "replace")


class PageSignalParser(HTMLParser):
    def __init__(self, base_url: str | None) -> None:
        super().__init__(convert_charrefs=True)
        self.base_url = base_url
        self.tag_counts: Counter[str] = Counter()
        self.class_counts: Counter[str] = Counter()
        self.id_counts: Counter[str] = Counter()
        self.css_properties: Counter[str] = Counter()
        self.inline_style_count = 0
        self.stylesheets: list[str] = []
        self.style_blocks: list[str] = []
        self.script_blocks: list[str] = []
        self.script_sources: list[str] = []
        self.script_types: Counter[str] = Counter()
        self.media: list[dict[str, str]] = []
        self.forms: list[dict[str, str]] = []
        self.meta: dict[str, str] = {}
        self.text_samples: list[str] = []
        self._capture: str | None = None
        self._buffer: list[str] = []
        self._title: str | None = None

    @property
    def title(self) -> str | None:
        return self._title

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        tag = tag.lower()
        attributes = {name.lower(): value or "" for name, value in attrs}
        self.tag_counts[tag] += 1

        for class_name in attributes.get("class", "").split():
            self.class_counts[class_name] += 1
        if element_id := attributes.get("id"):
            self.id_counts[element_id] += 1

        if style := attributes.get("style"):
            self.inline_style_count += 1
            collect_css_properties(style, self.css_properties)

        if tag == "title":
            self._start_capture("title")
        elif tag == "style":
            self._start_capture("style")
        elif tag == "script":
            src = attributes.get("src")
            script_type = attributes.get("type", "classic") or "classic"
            self.script_types[script_type.lower()] += 1
            if src:
                self.script_sources.append(resolve_url(self.base_url, src))
            else:
                self._start_capture("script")

        if tag == "link" and "stylesheet" in attributes.get("rel", "").lower():
            href = attributes.get("href")
            if href:
                self.stylesheets.append(resolve_url(self.base_url, href))

        if tag == "meta":
            key = attributes.get("property") or attributes.get("name")
            content = attributes.get("content")
            if key and content:
                self.meta[key] = compact(content, 180)

        if tag in MEDIA_TAGS:
            self.media.append(compact_attributes(tag, attributes, self.base_url))

        if tag in FORM_TAGS:
            self.forms.append(compact_attributes(tag, attributes, self.base_url))

    def handle_endtag(self, tag: str) -> None:
        tag = tag.lower()
        if self._capture == tag:
            text = "".join(self._buffer)
            if tag == "title":
                self._title = compact(text, 180)
            elif tag == "style":
                self.style_blocks.append(text)
                collect_css_properties(text, self.css_properties)
            elif tag == "script":
                self.script_blocks.append(text)
            self._capture = None
            self._buffer = []

    def handle_data(self, data: str) -> None:
        if self._capture:
            self._buffer.append(data)
            return
        sample = compact(data, 140)
        if len(sample) > 24 and len(self.text_samples) < 24:
            self.text_samples.append(sample)

    def _start_capture(self, tag: str) -> None:
        self._capture = tag
        self._buffer = []


def compact_attributes(tag: str, attributes: dict[str, str], base_url: str | None) -> dict[str, str]:
    kept: dict[str, str] = {"tag": tag}
    for name in ("type", "name", "id", "class", "alt", "title", "placeholder", "aria-label"):
        if value := attributes.get(name):
            kept[name] = compact(value, 120)
    for name in ("href", "src", "data", "poster"):
        if value := attributes.get(name):
            kept[name] = compact(resolve_url(base_url, value), 220)
    return kept


def collect_css_properties(css: str, out: Counter[str]) -> None:
    css = re.sub(r"/\*.*?\*/", "", css, flags=re.S)
    for name in re.findall(r"(?<![-\w])([a-zA-Z-]+)\s*:", css):
        out[name.lower()] += 1


def collect_external_css(urls: list[str], limit: int) -> tuple[Counter[str], list[dict[str, Any]]]:
    properties: Counter[str] = Counter()
    reports: list[dict[str, Any]] = []
    with ThreadPoolExecutor(max_workers=8) as pool:
        futures = {pool.submit(fetch_text, url): url for url in urls[:limit]}
        for future in as_completed(futures):
            url = futures[future]
            try:
                css = future.result()
            except Exception as error:  # noqa: BLE001 - diagnostic tool should keep going.
                reports.append({"url": url, "ok": False, "error": str(error)})
                continue
            local: Counter[str] = Counter()
            collect_css_properties(css, local)
            properties.update(local)
            reports.append({"url": url, "ok": True, "bytes": len(css), "properties": sum(local.values())})
    return properties, reports


def js_signals(html: str, scripts: list[str]) -> dict[str, Any]:
    joined_inline = "\n".join(scripts)
    haystack = html if len(html) < 4_000_000 else joined_inline
    return {
        "markers": {marker: haystack.count(marker) for marker in APP_JSON_MARKERS},
        "inline_bytes": sum(len(script) for script in scripts),
        "large_inline_scripts": sum(1 for script in scripts if len(script) > 64_000),
        "json_assignments": find_json_assignments(haystack),
    }


def find_json_assignments(text: str) -> list[dict[str, Any]]:
    found: list[dict[str, Any]] = []
    for marker in APP_JSON_MARKERS:
        index = text.find(marker)
        if index < 0:
            continue
        brace = text.find("{", index)
        bracket = text.find("[", index)
        starts = [pos for pos in (brace, bracket) if pos >= 0]
        if not starts:
            found.append({"name": marker, "found": True, "json_like": False})
            continue
        start = min(starts)
        end = scan_json_end(text, start)
        found.append(
            {
                "name": marker,
                "found": True,
                "json_like": end is not None,
                "approx_bytes": (end - start + 1) if end else None,
            }
        )
    return found


def scan_json_end(text: str, start: int) -> int | None:
    opener = text[start]
    closer = "}" if opener == "{" else "]"
    depth = 0
    in_string = False
    escaped = False
    for index, char in enumerate(text[start:], start):
        if in_string:
            if escaped:
                escaped = False
            elif char == "\\":
                escaped = True
            elif char == '"':
                in_string = False
            continue
        if char == '"':
            in_string = True
        elif char == opener:
            depth += 1
        elif char == closer:
            depth -= 1
            if depth == 0:
                return index
    return None


def build_report(
    source: str,
    base_url: str | None,
    fetch_css: bool,
    css_limit: int,
    top: int,
) -> dict[str, Any]:
    html, final_url = read_source(source)
    effective_base = base_url or final_url or (source if source.startswith(("http://", "https://")) else None)
    parser = PageSignalParser(effective_base)
    parser.feed(html)

    css_properties = Counter(parser.css_properties)
    external_css: list[dict[str, Any]] = []
    if fetch_css and parser.stylesheets:
        fetched_properties, external_css = collect_external_css(parser.stylesheets, css_limit)
        css_properties.update(fetched_properties)

    unsupported = Counter(
        {name: count for name, count in css_properties.items() if name not in SUPPORTED_CSS_PROPERTIES}
    )

    return {
        "source": source,
        "final_url": final_url,
        "bytes": len(html),
        "title": parser.title,
        "meta": dict(sorted(parser.meta.items())[:top]),
        "dom": {
            "top_tags": parser.tag_counts.most_common(top),
            "top_classes": parser.class_counts.most_common(top),
            "ids": parser.id_counts.most_common(top),
            "text_samples": parser.text_samples[:top],
        },
        "cssom": {
            "inline_style_attributes": parser.inline_style_count,
            "style_blocks": len(parser.style_blocks),
            "linked_stylesheets": parser.stylesheets[:top],
            "external_css": external_css[:top],
            "top_properties": css_properties.most_common(top),
            "unsupported_properties": unsupported.most_common(top),
        },
        "js": {
            "external_scripts": parser.script_sources[:top],
            "script_types": parser.script_types.most_common(top),
            **js_signals(html, parser.script_blocks),
        },
        "media": parser.media[:top],
        "forms": parser.forms[:top],
    }


def print_human(report: dict[str, Any]) -> None:
    print(f"bytes: {report['bytes']}")
    if report.get("final_url"):
        print(f"final_url: {report['final_url']}")
    if report.get("title"):
        print(f"title: {report['title']}")

    print("\nDOM:")
    for tag, count in report["dom"]["top_tags"][:12]:
        print(f"  {tag}: {count}")

    print("\nCSSOM:")
    print(f"  inline style attrs: {report['cssom']['inline_style_attributes']}")
    print(f"  style blocks: {report['cssom']['style_blocks']}")
    print(f"  linked stylesheets: {len(report['cssom']['linked_stylesheets'])}")
    print("  top properties:")
    for name, count in report["cssom"]["top_properties"][:12]:
        support = "ok" if name in SUPPORTED_CSS_PROPERTIES else "missing"
        print(f"    {name}: {count} ({support})")

    print("\nJS:")
    print(f"  inline bytes: {report['js']['inline_bytes']}")
    print(f"  large inline scripts: {report['js']['large_inline_scripts']}")
    print(f"  external scripts listed: {len(report['js']['external_scripts'])}")
    for name, count in report["js"]["markers"].items():
        if count:
            print(f"  marker {name}: {count}")

    print("\nNative surfaces:")
    print(f"  media nodes: {len(report['media'])}")
    print(f"  form/control nodes: {len(report['forms'])}")


def resolve_url(base_url: str | None, value: str) -> str:
    if not base_url:
        return value
    return urljoin(base_url, value)


def compact(text: str, limit: int) -> str:
    normalized = " ".join(text.split())
    if len(normalized) <= limit:
        return normalized
    return normalized[: limit - 3] + "..."


def main() -> int:
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8", errors="replace")

    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", help="HTML file/cache body or URL")
    parser.add_argument("--base-url", help="base URL to resolve relative links for local files")
    parser.add_argument("--fetch-css", action="store_true", help="download linked stylesheets")
    parser.add_argument("--css-limit", type=int, default=16, help="max external CSS files to fetch")
    parser.add_argument("--json", action="store_true", help="emit machine-readable JSON")
    parser.add_argument("--top", type=int, default=20, help="max items per section")
    args = parser.parse_args()

    report = build_report(
        source=args.source,
        base_url=args.base_url,
        fetch_css=args.fetch_css,
        css_limit=args.css_limit,
        top=args.top,
    )

    if args.json:
        print(json.dumps(report, ensure_ascii=False, indent=2))
    else:
        print_human(report)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
