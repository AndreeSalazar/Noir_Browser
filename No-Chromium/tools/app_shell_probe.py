#!/usr/bin/env python3
"""Inspect JS-heavy app shells and summarize embedded renderable data.

Usage:
  python tools/app_shell_probe.py profile/cache/resources/document/page.body
  python tools/app_shell_probe.py https://www.youtube.com/results?search_query=rust
  python tools/app_shell_probe.py --json profile/cache/resources/document/page.body
  python tools/app_shell_probe.py --yt-dlp https://www.youtube.com/watch?v=VIDEO_ID
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from urllib.request import Request, urlopen


VIDEO_RENDERERS = (
    "videoRenderer",
    "compactVideoRenderer",
    "gridVideoRenderer",
    "playlistVideoRenderer",
    "reelItemRenderer",
)


def read_source(source: str) -> str:
    if source.startswith(("http://", "https://")):
        request = Request(source, headers={"User-Agent": "No-Chromium app shell probe"})
        return urlopen(request, timeout=25).read().decode("utf-8", "replace")
    return Path(source).read_text(encoding="utf-8", errors="replace")


def extract_assigned_json(html: str, variable: str) -> dict | None:
    marker = f"{variable} = "
    start = html.find(marker)
    if start < 0:
        return None
    start = html.find("{", start + len(marker))
    if start < 0:
        return None

    depth = 0
    in_string = False
    escaped = False
    for index, char in enumerate(html[start:], start):
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
        elif char == "{":
            depth += 1
        elif char == "}":
            depth -= 1
            if depth == 0:
                return json.loads(html[start : index + 1])
    return None


def text_from_json_text(value: object) -> str | None:
    if not isinstance(value, dict):
        return None
    simple = value.get("simpleText")
    if isinstance(simple, str):
        return " ".join(simple.split())
    runs = value.get("runs")
    if isinstance(runs, list):
        text = "".join(run.get("text", "") for run in runs if isinstance(run, dict))
        text = " ".join(text.split())
        return text or None
    return None


def walk_video_cards(value: object, out: list[tuple[str, str, str]]) -> None:
    if isinstance(value, dict):
        for renderer_name in VIDEO_RENDERERS:
            renderer = value.get(renderer_name)
            if isinstance(renderer, dict) and isinstance(renderer.get("videoId"), str):
                title = text_from_json_text(renderer.get("title")) or text_from_json_text(
                    renderer.get("headline")
                )
                duration = text_from_json_text(renderer.get("lengthText")) or ""
                if title:
                    out.append((renderer["videoId"], title, duration))
        for child in value.values():
            walk_video_cards(child, out)
    elif isinstance(value, list):
        for child in value:
            walk_video_cards(child, out)


def summarize_player_response(data: dict) -> dict:
    details = data.get("videoDetails") if isinstance(data.get("videoDetails"), dict) else {}
    streaming = data.get("streamingData") if isinstance(data.get("streamingData"), dict) else {}
    formats = streaming.get("formats") if isinstance(streaming.get("formats"), list) else []
    adaptive = (
        streaming.get("adaptiveFormats")
        if isinstance(streaming.get("adaptiveFormats"), list)
        else []
    )
    all_formats = formats + adaptive
    direct = [item for item in all_formats if isinstance(item, dict) and item.get("url")]
    ciphered = [
        item
        for item in all_formats
        if isinstance(item, dict) and (item.get("signatureCipher") or item.get("cipher"))
    ]

    return {
        "title": details.get("title", ""),
        "author": details.get("author", ""),
        "duration_seconds": details.get("lengthSeconds", ""),
        "muxed_formats": len(formats),
        "adaptive_formats": len(adaptive),
        "direct_urls": [
            {
                "quality": item.get("qualityLabel") or item.get("audioQuality") or item.get("quality"),
                "mime": str(item.get("mimeType", "")).split(";")[0],
            }
            for item in direct[:8]
        ],
        "ciphered_formats": len(ciphered),
    }


def describe_player_response(data: dict) -> None:
    summary = summarize_player_response(data)
    print("player:")
    print(f"  title: {summary['title']}")
    print(f"  author: {summary['author']}")
    print(f"  duration: {summary['duration_seconds']}s")
    print(
        f"  formats: {summary['muxed_formats']} muxed / {summary['adaptive_formats']} adaptive"
    )
    print(
        f"  direct URLs: {len(summary['direct_urls'])} / ciphered: {summary['ciphered_formats']}"
    )
    for item in summary["direct_urls"]:
        print(f"  - direct {item['quality'] or 'auto'} {item['mime']}")


def build_report(html: str) -> dict:
    data = extract_assigned_json(html, "ytInitialData")
    cards: list[tuple[str, str, str]] = []
    if data is not None:
        walk_video_cards(data, cards)

    player = extract_assigned_json(html, "ytInitialPlayerResponse")
    return {
        "bytes": len(html),
        "markers": {
            marker: html.count(marker)
            for marker in ("ytInitialData", '"videoId"', "videoRenderer", "feedNudgeRenderer")
        },
        "video_cards": [
            {
                "url": f"https://www.youtube.com/watch?v={video_id}",
                "video_id": video_id,
                "title": title,
                "duration": duration,
            }
            for video_id, title, duration in cards
        ],
        "player": summarize_player_response(player) if player is not None else None,
    }


def describe_with_yt_dlp(source: str) -> int:
    try:
        import yt_dlp  # type: ignore
    except ImportError:
        print("yt-dlp no esta instalado. Instala con: python -m pip install yt-dlp")
        return 1

    with yt_dlp.YoutubeDL({"quiet": True, "skip_download": True, "noplaylist": True}) as ydl:
        info = ydl.extract_info(source, download=False)

    formats = info.get("formats") if isinstance(info, dict) else []
    playable = [
        item
        for item in formats
        if isinstance(item, dict) and isinstance(item.get("url"), str)
    ]
    print("yt-dlp:")
    print(f"  title: {info.get('title', '')}")
    print(f"  uploader: {info.get('uploader', '')}")
    print(f"  duration: {info.get('duration', '')}s")
    print(f"  playable formats: {len(playable)}")
    for item in playable[:12]:
        quality = item.get("format_note") or item.get("height") or item.get("abr") or "auto"
        ext = item.get("ext", "")
        protocol = item.get("protocol", "")
        print(f"  - {quality} {ext} {protocol}")
    return 0


def main() -> int:
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8", errors="replace")

    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", help="HTML file/cache body or URL to inspect")
    parser.add_argument("--json", action="store_true", help="emit structured JSON")
    parser.add_argument(
        "--yt-dlp",
        action="store_true",
        help="use yt-dlp when installed for heavy YouTube stream extraction",
    )
    args = parser.parse_args()

    if args.yt_dlp:
        return describe_with_yt_dlp(args.source)

    html = read_source(args.source)
    report = build_report(html)
    if args.json:
        print(json.dumps(report, ensure_ascii=False, indent=2))
        return 0

    print(f"bytes: {len(html)}")
    for marker in ("ytInitialData", '"videoId"', "videoRenderer", "feedNudgeRenderer"):
        print(f"{marker}: {html.count(marker)}")

    data = extract_assigned_json(html, "ytInitialData")
    if data is None:
        print("ytInitialData: not found")
    else:
        cards: list[tuple[str, str, str]] = []
        walk_video_cards(data, cards)
        print(f"video cards: {len(cards)}")
        for video_id, title, duration in cards[:20]:
            suffix = f" [{duration}]" if duration else ""
            print(f"- https://www.youtube.com/watch?v={video_id} :: {title}{suffix}")

    player = extract_assigned_json(html, "ytInitialPlayerResponse")
    if player is not None:
        describe_player_response(player)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
