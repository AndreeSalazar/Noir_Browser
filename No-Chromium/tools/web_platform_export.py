#!/usr/bin/env python3
"""Download Web Platform datasets and export compact Rust lookup tables.

Python does the heavy internet/data parsing work. Rust receives static arrays
that are cheap to compile into the browser and useful for CSS/HTML/DOM/JS
feature decisions.

Sources:
  - MDN browser-compat-data package data.json
  - MDN data CSS properties database

Examples:
  python tools/web_platform_export.py
  python tools/web_platform_export.py --offline
  python tools/web_platform_export.py --json-report tools/.cache/web_platform/report.json
"""

from __future__ import annotations

import argparse
import json
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Iterable
from urllib.request import Request, urlopen


BCD_URL = "https://unpkg.com/@mdn/browser-compat-data/data.json"
CSS_DATA_URL = "https://raw.githubusercontent.com/mdn/data/main/css/properties.json"
BROWSERS = ("chrome", "firefox", "safari", "edge")


@dataclass(frozen=True)
class Compat:
    chrome: str | None
    firefox: str | None
    safari: str | None
    edge: str | None


@dataclass(frozen=True)
class Feature:
    path: str
    compat: Compat


@dataclass(frozen=True)
class CssProperty:
    name: str
    syntax: str | None
    initial: str | None
    inherited: bool
    compat: Compat


def fetch_json(url: str, cache_path: Path, offline: bool) -> Any:
    if offline:
        return json.loads(cache_path.read_text(encoding="utf-8"))

    cache_path.parent.mkdir(parents=True, exist_ok=True)
    request = Request(url, headers={"User-Agent": "No-Chromium web platform exporter"})
    with urlopen(request, timeout=60) as response:
        body = response.read().decode("utf-8", "replace")
    cache_path.write_text(body, encoding="utf-8")
    return json.loads(body)


def compat_from_node(node: dict[str, Any]) -> Compat:
    compat = node.get("__compat") if isinstance(node.get("__compat"), dict) else {}
    support = compat.get("support") if isinstance(compat.get("support"), dict) else {}
    values = {}
    for browser in BROWSERS:
        values[browser] = normalize_version_added(support.get(browser))
    return Compat(
        chrome=values["chrome"],
        firefox=values["firefox"],
        safari=values["safari"],
        edge=values["edge"],
    )


def normalize_version_added(raw: Any) -> str | None:
    if isinstance(raw, list):
        for item in raw:
            version = normalize_version_added(item)
            if version is not None:
                return version
        return None
    if isinstance(raw, dict):
        value = raw.get("version_added")
    else:
        value = raw

    if value is True:
        return "yes"
    if value is False or value is None:
        return None
    return str(value)


def walk_compat_features(node: Any, prefix: tuple[str, ...]) -> Iterable[Feature]:
    if not isinstance(node, dict):
        return

    if "__compat" in node and prefix:
        yield Feature(".".join(prefix), compat_from_node(node))

    for key, child in node.items():
        if key == "__compat":
            continue
        yield from walk_compat_features(child, (*prefix, key))


def collect_css_properties(bcd: dict[str, Any], css_data: dict[str, Any]) -> list[CssProperty]:
    compat_by_name = {
        feature.path: feature.compat
        for feature in walk_compat_features(bcd.get("css", {}).get("properties", {}), ())
        if "." not in feature.path
    }

    properties: list[CssProperty] = []
    for name, details in sorted(css_data.items()):
        if not isinstance(details, dict):
            continue
        properties.append(
            CssProperty(
                name=name,
                syntax=string_or_none(details.get("syntax")),
                initial=string_or_none(details.get("initial")),
                inherited=bool(details.get("inherited")),
                compat=compat_by_name.get(name, Compat(None, None, None, None)),
            )
        )
    return properties


def collect_html_elements(bcd: dict[str, Any]) -> list[Feature]:
    elements = bcd.get("html", {}).get("elements", {})
    return sorted(walk_compat_features(elements, ()), key=lambda feature: feature.path)


def collect_js_features(bcd: dict[str, Any]) -> list[Feature]:
    javascript = bcd.get("javascript", {})
    return sorted(walk_compat_features(javascript, ()), key=lambda feature: feature.path)


def collect_api_features(bcd: dict[str, Any]) -> list[Feature]:
    api = bcd.get("api", {})
    return sorted(walk_compat_features(api, ()), key=lambda feature: feature.path)


def string_or_none(value: Any) -> str | None:
    if isinstance(value, str) and value.strip():
        return " ".join(value.split())
    return None


def rust_string(value: str) -> str:
    out = ['"']
    for char in value:
        codepoint = ord(char)
        if char == "\\":
            out.append("\\\\")
        elif char == '"':
            out.append('\\"')
        elif char == "\n":
            out.append("\\n")
        elif char == "\r":
            out.append("\\r")
        elif char == "\t":
            out.append("\\t")
        elif codepoint < 32:
            out.append(f"\\u{{{codepoint:x}}}")
        elif codepoint > 126:
            out.append(f"\\u{{{codepoint:x}}}")
        else:
            out.append(char)
    out.append('"')
    return "".join(out)


def rust_opt(value: str | None) -> str:
    return f"Some({rust_string(value)})" if value is not None else "None"


def rust_bool(value: bool) -> str:
    return "true" if value else "false"


def rust_compat(compat: Compat) -> str:
    return (
        "BrowserCompat { "
        f"chrome: {rust_opt(compat.chrome)}, "
        f"firefox: {rust_opt(compat.firefox)}, "
        f"safari: {rust_opt(compat.safari)}, "
        f"edge: {rust_opt(compat.edge)} "
        "}"
    )


def write_rust(
    output: Path,
    bcd_meta: dict[str, Any],
    css_properties: list[CssProperty],
    html_elements: list[Feature],
    js_features: list[Feature],
    api_features: list[Feature],
) -> None:
    output.parent.mkdir(parents=True, exist_ok=True)
    lines: list[str] = [
        "// @generated by tools/web_platform_export.py",
        "// Source data: MDN browser-compat-data and MDN data.",
        "",
        "#[derive(Debug, Clone, Copy, PartialEq, Eq)]",
        "pub struct BrowserCompat {",
        "    pub chrome: Option<&'static str>,",
        "    pub firefox: Option<&'static str>,",
        "    pub safari: Option<&'static str>,",
        "    pub edge: Option<&'static str>,",
        "}",
        "",
        "#[derive(Debug, Clone, Copy, PartialEq, Eq)]",
        "pub struct CompatFeature {",
        "    pub path: &'static str,",
        "    pub compat: BrowserCompat,",
        "}",
        "",
        "#[derive(Debug, Clone, Copy, PartialEq, Eq)]",
        "pub struct CssPropertyData {",
        "    pub name: &'static str,",
        "    pub syntax: Option<&'static str>,",
        "    pub initial: Option<&'static str>,",
        "    pub inherited: bool,",
        "    pub compat: BrowserCompat,",
        "}",
        "",
        f"pub const MDN_BCD_VERSION: &str = {rust_string(str(bcd_meta.get('version', 'unknown')))};",
        f"pub const MDN_BCD_TIMESTAMP: &str = {rust_string(str(bcd_meta.get('timestamp', 'unknown')))};",
        "",
    ]

    lines.extend(write_css_array(css_properties))
    lines.extend(write_feature_array("HTML_ELEMENTS", html_elements))
    lines.extend(write_feature_array("JS_FEATURES", js_features))
    lines.extend(write_feature_array("WEB_API_FEATURES", api_features))
    lines.extend(
        [
            "pub fn css_property(name: &str) -> Option<&'static CssPropertyData> {",
            "    CSS_PROPERTIES.iter().find(|property| property.name == name)",
            "}",
            "",
            "pub fn html_element(name: &str) -> Option<&'static CompatFeature> {",
            "    HTML_ELEMENTS.iter().find(|feature| feature.path == name)",
            "}",
            "",
            "pub fn js_feature(path: &str) -> Option<&'static CompatFeature> {",
            "    JS_FEATURES.iter().find(|feature| feature.path == path)",
            "}",
            "",
            "pub fn web_api_feature(path: &str) -> Option<&'static CompatFeature> {",
            "    WEB_API_FEATURES.iter().find(|feature| feature.path == path)",
            "}",
            "",
            "#[cfg(test)]",
            "mod tests {",
            "    use super::*;",
            "",
            "    #[test]",
            "    fn generated_tables_expose_core_web_features() {",
            "        assert!(css_property(\"background-color\").is_some());",
            "        assert!(html_element(\"div\").is_some());",
            "        assert!(js_feature(\"builtins.Array.map\").is_some());",
            "        assert!(web_api_feature(\"Document.querySelector\").is_some());",
            "    }",
            "}",
            "",
        ]
    )
    output.write_text("\n".join(lines), encoding="utf-8")


def write_css_array(properties: list[CssProperty]) -> list[str]:
    lines = ["pub const CSS_PROPERTIES: &[CssPropertyData] = &["]
    for property_data in properties:
        lines.append(
            "    CssPropertyData { "
            f"name: {rust_string(property_data.name)}, "
            f"syntax: {rust_opt(property_data.syntax)}, "
            f"initial: {rust_opt(property_data.initial)}, "
            f"inherited: {rust_bool(property_data.inherited)}, "
            f"compat: {rust_compat(property_data.compat)} "
            "},"
        )
    lines.extend(["];", ""])
    return lines


def write_feature_array(name: str, features: list[Feature]) -> list[str]:
    lines = [f"pub const {name}: &[CompatFeature] = &["]
    for feature in features:
        lines.append(
            "    CompatFeature { "
            f"path: {rust_string(feature.path)}, "
            f"compat: {rust_compat(feature.compat)} "
            "},"
        )
    lines.extend(["];", ""])
    return lines


def write_report(
    output: Path | None,
    bcd_meta: dict[str, Any],
    css_properties: list[CssProperty],
    html_elements: list[Feature],
    js_features: list[Feature],
    api_features: list[Feature],
) -> None:
    if output is None:
        return
    output.parent.mkdir(parents=True, exist_ok=True)
    report = {
        "mdn_bcd_version": bcd_meta.get("version"),
        "mdn_bcd_timestamp": bcd_meta.get("timestamp"),
        "counts": {
            "css_properties": len(css_properties),
            "html_elements": len(html_elements),
            "js_features": len(js_features),
            "web_api_features": len(api_features),
        },
        "sample_css": [property_data.name for property_data in css_properties[:20]],
        "sample_html": [feature.path for feature in html_elements[:20]],
        "sample_js": [feature.path for feature in js_features[:20]],
        "sample_api": [feature.path for feature in api_features[:20]],
    }
    output.write_text(json.dumps(report, ensure_ascii=False, indent=2), encoding="utf-8")


def main() -> int:
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8", errors="replace")

    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--output",
        default="src/generated_rust/web_platform_data.rs",
        help="Rust output path",
    )
    parser.add_argument(
        "--cache-dir",
        default="tools/.cache/web_platform",
        help="download cache directory",
    )
    parser.add_argument("--offline", action="store_true", help="use cached JSON only")
    parser.add_argument("--json-report", help="optional summary report path")
    args = parser.parse_args()

    cache_dir = Path(args.cache_dir)
    bcd = fetch_json(BCD_URL, cache_dir / "mdn_browser_compat_data.json", args.offline)
    css_data = fetch_json(CSS_DATA_URL, cache_dir / "mdn_css_properties.json", args.offline)

    css_properties = collect_css_properties(bcd, css_data)
    html_elements = collect_html_elements(bcd)
    js_features = collect_js_features(bcd)
    api_features = collect_api_features(bcd)

    bcd_meta = bcd.get("__meta", {}) if isinstance(bcd.get("__meta"), dict) else {}
    write_rust(
        Path(args.output),
        bcd_meta,
        css_properties,
        html_elements,
        js_features,
        api_features,
    )
    write_report(
        Path(args.json_report) if args.json_report else None,
        bcd_meta,
        css_properties,
        html_elements,
        js_features,
        api_features,
    )

    print(
        "exported "
        f"{len(css_properties)} CSS properties, "
        f"{len(html_elements)} HTML elements, "
        f"{len(js_features)} JS features, "
        f"{len(api_features)} Web APIs"
    )
    print(f"MDN BCD version: {bcd_meta.get('version', 'unknown')}")
    print(f"Rust: {args.output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
