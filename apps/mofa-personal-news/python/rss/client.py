from __future__ import annotations

import re
import html
from typing import Any, Dict, List, Optional
from urllib.parse import urlparse
from xml.etree import ElementTree

import requests


def fetch_rss_items(config: Dict[str, Any]) -> List[Dict[str, Any]]:
    sources = config.get("rss_sources", []) or []
    items: List[Dict[str, Any]] = []
    for source in sources:
        try:
            items.extend(_fetch_feed(source, config))
        except requests.RequestException:
            continue
    return items


def _fetch_feed(url: str, config: Dict[str, Any]) -> List[Dict[str, Any]]:
    response = requests.get(url, timeout=10)
    response.raise_for_status()
    content = response.text
    root = ElementTree.fromstring(content)
    channel = root.find("channel")
    source_name = _guess_source_name(url, channel)
    entries: List[Dict[str, Any]] = []
    for item in root.findall(".//item"):
        title = _text(item.find("title"))
        summary = _text(item.find("description"))
        link = _text(item.find("link"))
        if config.get("rss_fetch_full_text", False) and link:
            full_text = _fetch_article_summary(link)
            if full_text:
                summary = full_text
        published_at = _parse_pub_date(_text(item.find("pubDate")))
        entries.append(
            {
                "title": title,
                "summary": summary,
                "published_at": published_at,
                "source_name": source_name,
                "link": link,
            }
        )
    return entries


def _text(node: ElementTree.Element | None) -> str:
    if node is None or node.text is None:
        return ""
    return node.text.strip()


def _guess_source_name(url: str, channel: ElementTree.Element | None) -> str:
    if channel is not None:
        title = _text(channel.find("title"))
        if title:
            return title
    hostname = urlparse(url).hostname or "rss"
    return hostname.replace("www.", "")


def _parse_pub_date(value: str) -> str:
    if not value:
        return ""
    normalized = re.sub(r"\s+", " ", value).strip()
    return normalized


def _fetch_article_summary(url: str) -> Optional[str]:
    try:
        response = requests.get(url, timeout=10)
        response.raise_for_status()
    except requests.RequestException:
        return None
    text = _extract_text_from_html(response.text)
    if not text:
        return None
    return _truncate_text(text, 360)


def _extract_text_from_html(content: str) -> str:
    content = re.sub(r"(?is)<script.*?>.*?</script>", " ", content)
    content = re.sub(r"(?is)<style.*?>.*?</style>", " ", content)
    content = re.sub(r"(?is)<noscript.*?>.*?</noscript>", " ", content)
    content = re.sub(r"(?is)<[^>]+>", " ", content)
    content = html.unescape(content)
    content = re.sub(r"\s+", " ", content).strip()
    return content


def _truncate_text(text: str, max_len: int) -> str:
    if len(text) <= max_len:
        return text
    return text[: max_len - 1].rstrip() + "…"


def _load_config(path: str) -> Dict[str, Any]:
    import json

    with open(path, "r", encoding="utf-8") as handle:
        return json.load(handle)


def _cli() -> None:
    import argparse
    import json

    parser = argparse.ArgumentParser(description="Fetch RSS items")
    parser.add_argument("config", help="Path to config.json")
    args = parser.parse_args()
    config = _load_config(args.config)
    items = fetch_rss_items(config)
    print(json.dumps(items, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    _cli()
