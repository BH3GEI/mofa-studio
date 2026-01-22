from __future__ import annotations

import os
import sys
from pathlib import Path
from typing import Any, Dict, List, Optional

import requests

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from utils import load_env_file


def fetch_weather(config: Dict[str, Any]) -> List[Dict[str, Any]]:
    load_env_file(Path(__file__).resolve().parents[1] / "env.secret")
    city = config.get("city")
    if not city:
        return []
    provider = config.get("weather_provider", "wttr")
    if provider != "wttr":
        return []
    response = requests.get(
        f"https://wttr.in/{city}",
        params={
            "format": "j1",
            "lang": "zh-cn",
        },
        timeout=10,
    )
    response.raise_for_status()
    data = response.json()
    summary = _build_summary(data)
    if not summary:
        return []
    return [
        {
            "summary": summary,
        }
    ]


def _build_summary(data: Dict[str, Any]) -> Optional[str]:
    current = (data.get("current_condition") or [{}])[0]
    weather_desc = (current.get("weatherDesc") or [{}])[0].get("value")
    temp_c = current.get("temp_C")
    feels_like = current.get("FeelsLikeC")
    wind_kph = current.get("windspeedKmph")
    parts = []
    if weather_desc:
        parts.append(weather_desc)
    if temp_c is not None:
        parts.append(f"当前气温{temp_c}度")
    if feels_like is not None:
        parts.append(f"体感{feels_like}度")
    if wind_kph is not None:
        parts.append(f"风速约{wind_kph}公里每小时")
    if not parts:
        return None
    return "，".join(parts) + "。"


def _load_config(path: str) -> Dict[str, Any]:
    import json

    with open(path, "r", encoding="utf-8") as handle:
        return json.load(handle)


def _cli() -> None:
    import argparse
    import json

    parser = argparse.ArgumentParser(description="Fetch weather items")
    parser.add_argument("config", help="Path to config.json")
    args = parser.parse_args()
    config = _load_config(args.config)
    items = fetch_weather(config)
    print(json.dumps(items, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    _cli()
