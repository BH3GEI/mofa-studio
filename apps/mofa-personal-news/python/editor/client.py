from __future__ import annotations

import json
import os
import sys
from pathlib import Path
from typing import Any, Dict, List

import requests

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from utils import load_env_file


def generate_broadcast_script(payload: Dict[str, Any]) -> str:
    load_env_file(Path(__file__).resolve().parents[1] / "env.secret")
    api_key = os.getenv("LLM_API_KEY")
    if not api_key:
        raise RuntimeError("Missing LLM_API_KEY")
    base_url = os.getenv("LLM_API_BASE", "https://api.openai.com")
    model = os.getenv("LLM_MODEL", "gpt-4o-mini")
    temperature = float(payload.get("config", {}).get("llm_temperature", 0.2))
    compact = _shrink_payload(payload)
    prompt = _build_prompt(compact)

    response = requests.post(
        f"{base_url.rstrip('/')}/v1/chat/completions",
        headers={"Authorization": f"Bearer {api_key}"},
        json={
            "model": model,
            "temperature": temperature,
            "messages": prompt,
        },
        timeout=30,
    )
    try:
        response.raise_for_status()
    except requests.HTTPError as exc:
        detail = response.text
        raise RuntimeError(f"LLM API error: {detail}") from exc
    data = response.json()
    return data["choices"][0]["message"]["content"].strip()


def _build_prompt(payload: Dict[str, Any]) -> List[Dict[str, str]]:
    system = _load_prompt_text()
    user = json.dumps(payload, ensure_ascii=False)
    return [
        {"role": "system", "content": system},
        {"role": "user", "content": user},
    ]


def _shrink_payload(payload: Dict[str, Any]) -> Dict[str, Any]:
    config = payload.get("config", {})
    inputs = payload.get("inputs", []) or []
    compact_inputs = []
    for source_block in inputs:
        source = source_block.get("source")
        items = source_block.get("items", []) or []
        trimmed = [_trim_item(source, item) for item in items[:3]]
        compact_inputs.append({"source": source, "items": trimmed})
    return {"config": config, "inputs": compact_inputs}


def _trim_item(source: str, item: Dict[str, Any]) -> Dict[str, Any]:
    if source == "rss":
        return {
            "title": _truncate(item.get("title", ""), 120),
            "summary": _truncate(item.get("summary", ""), 600),
            "published_at": item.get("published_at"),
            "source_name": item.get("source_name"),
        }
    if source == "x":
        return {
            "author": item.get("author"),
            "text": _truncate(item.get("text", ""), 280),
            "engagement": item.get("engagement", {}),
            "created_at": item.get("created_at"),
        }
    if source == "weather":
        return {"summary": _truncate(item.get("summary", ""), 200)}
    return item


def _truncate(text: str, limit: int) -> str:
    if not text:
        return ""
    text = str(text)
    if len(text) <= limit:
        return text
    return text[: limit - 1].rstrip() + "…"


def _load_json(path: str) -> Dict[str, Any]:
    with open(path, "r", encoding="utf-8") as handle:
        data = json.load(handle)
    if isinstance(data, list):
        return {"inputs": data}
    return data


def _load_prompt_text() -> str:
    prompt_path = Path(__file__).resolve().parent / "prompt.txt"
    if not prompt_path.exists():
        raise RuntimeError("Missing editor prompt.txt")
    return prompt_path.read_text(encoding="utf-8").strip()


def _cli() -> None:
    import argparse

    parser = argparse.ArgumentParser(description="Generate broadcast script with LLM")
    parser.add_argument("json_path", help="Path to personal news JSON")
    args = parser.parse_args()
    payload = _load_json(args.json_path)
    script = generate_broadcast_script(payload)
    print(script)


if __name__ == "__main__":
    _cli()
