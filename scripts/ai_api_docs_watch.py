#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
每日抓取各大 AI Provider API 文档，检测变更并输出 changes_out.json
首次运行仅建立 baseline，不创建 Issue。
"""

import os
import json
import time
import hashlib
import datetime
import difflib
from pathlib import Path
import requests
from bs4 import BeautifulSoup

SNAPSHOT_PATH = Path("data/api_doc_snapshots.json")
OUTPUT_PATH = Path("changes_out.json")

PROVIDER_PAGES = [
    {"provider": "OpenAI", "url": "https://platform.openai.com/docs/api-reference/introduction", "title": "OpenAI API Reference (Introduction)"},
    {"provider": "Anthropic", "url": "https://docs.anthropic.com/en/api/reference", "title": "Anthropic API Reference"},
    {"provider": "Google Gemini", "url": "https://ai.google.dev/api/rest", "title": "Google Gemini REST API"},
    {"provider": "Cohere", "url": "https://docs.cohere.com/reference/about", "title": "Cohere API Reference Overview"},
    {"provider": "Mistral", "url": "https://docs.mistral.ai/api/", "title": "Mistral API Reference"},
    {"provider": "Azure OpenAI", "url": "https://learn.microsoft.com/en-us/azure/ai-services/openai/reference", "title": "Azure OpenAI REST API Reference"},
    {"provider": "AWS Bedrock", "url": "https://docs.aws.amazon.com/bedrock/latest/userguide/api-methods.html", "title": "AWS Bedrock API Methods"},
]

HEADERS = {"User-Agent": "DocsWatchBot/1.0 (+https://github.com/hiddenpath/ai-lib)"}
DIFF_MAX_LINES = 120
REQUEST_TIMEOUT = 30
RETRY = 2
RETRY_SLEEP = 3

def fetch_page(url: str) -> str:
    last_exc = None
    for _ in range(RETRY + 1):
        try:
            r = requests.get(url, headers=HEADERS, timeout=REQUEST_TIMEOUT)
            r.raise_for_status()
            html = r.text
            soup = BeautifulSoup(html, "html.parser")
            for tag in soup(["script", "style", "noscript"]):
                tag.decompose()
            lines = [ln.strip() for ln in soup.get_text("\n").splitlines() if ln.strip()]
            return "\n".join(lines)
        except Exception as e:
            last_exc = e
            time.sleep(RETRY_SLEEP)
    raise RuntimeError(f"Failed to fetch {url}: {last_exc}")

def compute_hash(content: str) -> str:
    return hashlib.sha256(content.encode("utf-8")).hexdigest()

def load_snapshots():
    if SNAPSHOT_PATH.exists():
        try:
            return json.loads(SNAPSHOT_PATH.read_text(encoding="utf-8"))
        except Exception:
            return {}
    return {}

def save_snapshots(data):
    SNAPSHOT_PATH.parent.mkdir(parents=True, exist_ok=True)
    SNAPSHOT_PATH.write_text(json.dumps(data, ensure_ascii=False, indent=2), encoding="utf-8")

def unified_diff(old: str, new: str) -> str:
    diff_lines = list(difflib.unified_diff(old.splitlines(), new.splitlines(),
                                           fromfile="previous", tofile="current", lineterm=""))
    if len(diff_lines) > DIFF_MAX_LINES:
        truncated = diff_lines[:DIFF_MAX_LINES]
        truncated.append(f"...(diff truncated, total {len(diff_lines)} lines)")
        return "\n".join(truncated)
    return "\n".join(diff_lines)

def main():
    now_iso = datetime.datetime.now(datetime.timezone.utc).isoformat().replace('+00:00', 'Z')
    snapshots = load_snapshots()
    is_baseline = not SNAPSHOT_PATH.exists()
    changes = []

    for page in PROVIDER_PAGES:
        provider = page["provider"]
        url = page["url"]
        title = page["title"]
        try:
            content = fetch_page(url)
        except Exception as e:
            print(f"[WARN] Fetch failed {provider} {url}: {e}")
            continue
        h = compute_hash(content)
        prev = snapshots.get(url)
        if prev is None:
            snapshots[url] = {
                "provider": provider,
                "title": title,
                "hash": h,
                "content": content,
                "fetched_at": now_iso
            }
            print(f"[INFO] Baseline added {provider}: {title}")
        else:
            if prev.get("hash") != h:
                diff_text = unified_diff(prev.get("content", ""), content)
                changes.append({
                    "provider": provider,
                    "url": url,
                    "title": title,
                    "old_hash": prev.get("hash"),
                    "new_hash": h,
                    "diff": diff_text
                })
                snapshots[url].update({
                    "hash": h,
                    "content": content,
                    "fetched_at": now_iso
                })
                print(f"[CHANGE] {provider}: {title}")
            else:
                print(f"[NOCHANGE] {provider}: {title}")

    save_snapshots(snapshots)

    if is_baseline:
        print("[INFO] First run (baseline). No issue will be created.")
        OUTPUT_PATH.write_text(json.dumps({"baseline": True, "changes": []}, ensure_ascii=False, indent=2), encoding="utf-8")
        return

    OUTPUT_PATH.write_text(json.dumps({
        "baseline": False,
        "generated_at": now_iso,
        "changes": changes
    }, ensure_ascii=False, indent=2), encoding="utf-8")
    print(f"[INFO] {len(changes)} changes recorded.")

if __name__ == "__main__":
    main()