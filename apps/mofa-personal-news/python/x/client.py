from __future__ import annotations

import os
import sys
from pathlib import Path
from typing import Any, Dict, List, Optional

import requests

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from utils import load_env_file


def fetch_x_items(config: Dict[str, Any]) -> List[Dict[str, Any]]:
    load_env_file(Path(__file__).resolve().parents[1] / "env.secret")
    token = os.getenv("X_BEARER_TOKEN")
    if not token:
        return []
    accounts = config.get("x_priority_accounts", []) or []
    if not accounts:
        return []
    headers = {"Authorization": f"Bearer {token}"}
    items: List[Dict[str, Any]] = []
    for account in accounts:
        user_id, author = _resolve_user_id(account, headers)
        if not user_id:
            continue
        items.extend(_fetch_user_tweets(user_id, headers, author))
    return items


def _load_config(path: str) -> Dict[str, Any]:
    import json

    with open(path, "r", encoding="utf-8") as handle:
        return json.load(handle)


def _cli() -> None:
    import argparse
    import json

    parser = argparse.ArgumentParser(description="Fetch X items")
    parser.add_argument("config", help="Path to config.json")
    args = parser.parse_args()
    config = _load_config(args.config)
    items = fetch_x_items(config)
    print(json.dumps(items, ensure_ascii=False, indent=2))


def _resolve_user_id(account: str, headers: Dict[str, str]) -> tuple[Optional[str], str]:
    if account.isdigit():
        return account, account
    try:
        response = requests.get(
            f"https://api.x.com/2/users/by/username/{account}",
            headers=headers,
            timeout=10,
        )
        if response.status_code == 404:
            return None, account
        response.raise_for_status()
        data = response.json()
    except requests.RequestException:
        return None, account
    user = data.get("data")
    if isinstance(user, list):
        user = user[0] if user else {}
    if not isinstance(user, dict):
        return None, account
    user_id = user.get("id")
    if not user_id or not str(user_id).isdigit():
        return None, account
    return str(user_id), user.get("username", account)


def _fetch_user_tweets(user_id: str, headers: Dict[str, str], author: str) -> List[Dict[str, Any]]:
    try:
        response = requests.get(
            f"https://api.x.com/2/users/{user_id}/tweets",
            headers=headers,
            params={
                "max_results": 5,
                "tweet.fields": "created_at,public_metrics",
            },
            timeout=10,
        )
        response.raise_for_status()
        data = response.json()
    except requests.RequestException:
        return []
    tweets = data.get("data", []) or []
    items = []
    for tweet in tweets:
        metrics = tweet.get("public_metrics", {})
        items.append(
            {
                "author": author,
                "text": tweet.get("text", ""),
                "engagement": {
                    "likes": metrics.get("like_count", 0),
                    "retweets": metrics.get("retweet_count", 0),
                },
                "created_at": tweet.get("created_at"),
            }
        )
    return items


if __name__ == "__main__":
    _cli()
