from __future__ import annotations

import importlib.util
import json
import sys
from pathlib import Path
from typing import Any, Dict, List

ROOT = Path(__file__).resolve().parent


def _load_module(path: Path, name: str):
    spec = importlib.util.spec_from_file_location(name, path)
    if not spec or not spec.loader:
        raise ImportError(f"Unable to load module: {path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


rss_module = _load_module(ROOT / "rss" / "client.py", "rss_client")
weather_module = _load_module(ROOT / "weather" / "client.py", "weather_client")
x_module = _load_module(ROOT / "x" / "client.py", "x_client")
editor_module = _load_module(ROOT / "editor" / "client.py", "editor_client")

fetch_rss_items = rss_module.fetch_rss_items
fetch_weather = weather_module.fetch_weather
fetch_x_items = x_module.fetch_x_items
generate_broadcast_script = editor_module.generate_broadcast_script


def load_config(path: Path) -> Dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def build_inputs(config: Dict[str, Any]) -> List[Dict[str, Any]]:
    inputs: List[Dict[str, Any]] = []
    rss_items = fetch_rss_items(config)
    if rss_items:
        inputs.append({"source": "rss", "items": rss_items})
    x_items = fetch_x_items(config)
    if x_items:
        inputs.append({"source": "x", "items": x_items})
    weather_items = fetch_weather(config)
    if weather_items:
        inputs.append({"source": "weather", "items": weather_items})
    return inputs


def main() -> None:
    if len(sys.argv) < 2:
        print("Usage: python personal-news/main.py <config.json>", file=sys.stderr)
        sys.exit(1)
    config_path = Path(sys.argv[1]).resolve()
    config = load_config(config_path)
    inputs = build_inputs(config)
    payload = {"config": config, "inputs": inputs}
    script = generate_broadcast_script(payload)
    print(script)


if __name__ == "__main__":
    main()
