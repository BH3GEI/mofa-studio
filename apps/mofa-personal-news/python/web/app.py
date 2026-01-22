from __future__ import annotations

import json
import sys
from http import HTTPStatus
from http.server import HTTPServer, SimpleHTTPRequestHandler
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple
import re
from urllib.parse import parse_qs, urlparse

BASE_DIR = Path(__file__).resolve().parent
STATIC_DIR = BASE_DIR / "static"
PROJECT_DIR = BASE_DIR.parent

sys.path.insert(0, str(PROJECT_DIR))

from personal_news import generate_broadcast  # noqa: E402


def _json_response(handler: SimpleHTTPRequestHandler, status: int, payload: Dict[str, Any]) -> None:
    data = json.dumps(payload, ensure_ascii=False).encode("utf-8")
    handler.send_response(status)
    handler.send_header("Content-Type", "application/json; charset=utf-8")
    handler.send_header("Content-Length", str(len(data)))
    handler.end_headers()
    handler.wfile.write(data)


def _read_body(handler: SimpleHTTPRequestHandler) -> Dict[str, Any]:
    length = int(handler.headers.get("Content-Length", "0"))
    body = handler.rfile.read(length).decode("utf-8")
    if not body:
        return {}
    return json.loads(body)


def _run_sample_tests() -> List[Dict[str, Any]]:
    from datetime import datetime, timedelta, timezone

    now = datetime.now(timezone.utc)
    inputs = [
        {
            "source": "rss",
            "items": [
                {
                    "title": "AI policy update",
                    "summary": "Regulators发布新的AI政策框架。",
                    "published_at": (now - timedelta(hours=2)).isoformat(),
                    "source_name": "TechDaily",
                }
            ],
        },
        {
            "source": "x",
            "items": [
                {
                    "author": "founderA",
                    "text": "我们刚发布新版本，细节见链接。",
                    "engagement": {"likes": 120, "retweets": 30},
                    "created_at": (now - timedelta(hours=1)).isoformat(),
                }
            ],
        },
        {
            "source": "weather",
            "items": [{"summary": "多云转小雨，最高气温18度。"}],
        },
    ]
    script = generate_broadcast({"max_duration_seconds": 240}, inputs)
    results = []
    results.append(
        _evaluate_script(
            "section-order",
            script,
            order=[("今日要闻", "与个人相关的动态"), ("与个人相关的动态", "生活服务")],
        )
    )
    results.append(_evaluate_script("weather-present", script, contains=["生活服务方面"]))

    inputs = [
        {
            "source": "rss",
            "items": [
                {
                    "title": "SpaceX launches new rocket",
                    "summary": "SpaceX launches new rocket successfully.",
                    "published_at": (now - timedelta(hours=1)).isoformat(),
                    "source_name": "SpaceNews",
                }
            ],
        },
        {
            "source": "x",
            "items": [
                {
                    "author": "spacex",
                    "text": "SpaceX launches new rocket successfully.",
                    "engagement": {"likes": 500, "retweets": 200},
                    "created_at": (now - timedelta(hours=1)).isoformat(),
                }
            ],
        },
    ]
    script = generate_broadcast({}, inputs)
    results.append(_evaluate_script("dedupe-rss-x", script, contains=["与个人相关的动态中"], not_contains=["今日要闻方面"] ))
    return results


def _evaluate_script(
    name: str,
    script: str,
    *,
    contains: Optional[List[str]] = None,
    not_contains: Optional[List[str]] = None,
    order: Optional[List[Tuple[str, str]]] = None,
) -> Dict[str, Any]:
    errors: List[str] = []
    contains = contains or []
    not_contains = not_contains or []
    order = order or []
    for token in contains:
        if token not in script:
            errors.append(f"missing: {token}")
    for token in not_contains:
        if token in script:
            errors.append(f"should not contain: {token}")
    for earlier, later in order:
        pos_earlier = script.find(earlier)
        pos_later = script.find(later)
        if pos_earlier == -1 or pos_later == -1 or pos_earlier >= pos_later:
            errors.append(f"order: {earlier} -> {later}")
    return {
        "name": name,
        "passed": not errors,
        "detail": "通过" if not errors else "; ".join(errors),
    }


def _run_custom_tests(cases: List[Dict[str, Any]]) -> List[Dict[str, Any]]:
    results: List[Dict[str, Any]] = []
    for case in cases:
        name = case.get("name", "unnamed")
        config = case.get("config", {})
        inputs = case.get("inputs", [])
        try:
            script = generate_broadcast(config, inputs)
        except Exception as exc:  # noqa: BLE001
            results.append({"name": name, "passed": False, "detail": f"error: {exc}"})
            continue
        results.append(
            _evaluate_script(
                name,
                script,
                contains=case.get("contains"),
                not_contains=case.get("not_contains"),
                order=case.get("order"),
            )
        )
    return results


class NewsRequestHandler(SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory=str(STATIC_DIR), **kwargs)

    def end_headers(self) -> None:
        self.send_header("Cache-Control", "no-store")
        super().end_headers()

    def do_GET(self) -> None:
        parsed = urlparse(self.path)
        if parsed.path == "/api/readme":
            query = parse_qs(parsed.query)
            target = query.get("path", ["README.md"])[0]
            readme_path = _resolve_readme_path(target)
            if not readme_path or not readme_path.exists():
                _json_response(self, HTTPStatus.NOT_FOUND, {"error": "README not found"})
                return
            content = readme_path.read_text(encoding="utf-8")
            _json_response(self, HTTPStatus.OK, {"content": content})
            return
        if parsed.path == "/":
            self.path = "/index.html"
        return super().do_GET()

    def do_POST(self) -> None:
        parsed = urlparse(self.path)
        if parsed.path == "/api/generate":
            try:
                payload = _read_body(self)
                config = payload.get("config", {})
                inputs = payload.get("inputs", [])
                script = generate_broadcast(config, inputs)
            except (ValueError, TypeError, json.JSONDecodeError) as exc:
                _json_response(self, HTTPStatus.BAD_REQUEST, {"error": f"Invalid payload: {exc}"})
                return
            _json_response(self, HTTPStatus.OK, {"script": script})
            return
        if parsed.path == "/api/run-tests":
            try:
                payload = _read_body(self)
            except (ValueError, TypeError, json.JSONDecodeError) as exc:
                _json_response(self, HTTPStatus.BAD_REQUEST, {"error": f"Invalid payload: {exc}"})
                return
            cases = payload.get("tests") if isinstance(payload, dict) else None
            if cases:
                results = _run_custom_tests(cases)
            else:
                results = _run_sample_tests()
            _json_response(self, HTTPStatus.OK, {"results": results})
            return
        _json_response(self, HTTPStatus.NOT_FOUND, {"error": "Unknown endpoint"})


def _resolve_readme_path(target: str) -> Path | None:
    if target == "README.md":
        return PROJECT_DIR / "README.md"
    if not re.fullmatch(r"[A-Za-z0-9_-]+/README\.md", target):
        return None
    candidate = (PROJECT_DIR / target).resolve()
    if PROJECT_DIR.resolve() not in candidate.parents:
        return None
    return candidate


def main() -> None:
    host = "127.0.0.1"
    port = 5173
    server = HTTPServer((host, port), NewsRequestHandler)
    print(f"Serving helper UI at http://{host}:{port}")
    server.serve_forever()


if __name__ == "__main__":
    main()
