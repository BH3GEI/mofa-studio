from __future__ import annotations

import importlib.util
from pathlib import Path
from typing import Any, Dict, List

ROOT = Path(__file__).resolve().parent.parent


def _load_module(path: Path, name: str):
    spec = importlib.util.spec_from_file_location(name, path)
    if not spec or not spec.loader:
        raise ImportError(f"Unable to load module: {path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


editor_module = _load_module(ROOT / "editor" / "client.py", "editor_client")
generate_broadcast_script = editor_module.generate_broadcast_script


def generate_broadcast(config: Dict[str, Any], inputs: List[Dict[str, Any]]) -> str:
    payload = {"config": config, "inputs": inputs}
    return generate_broadcast_script(payload)


__all__ = ["generate_broadcast"]
