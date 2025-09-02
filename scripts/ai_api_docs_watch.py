#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
每日抓取各大 AI Provider API 文档，检测变更并输出 changes_out.json
首次运行仅建立 baseline，不创建 Issue。
改进：
- OpenAI 使用官方 openapi.yaml，避免网页 403。
- 增强请求头，增加备用浏览器 UA。
- 自动识别非 HTML (yaml/json/plain) 文件，直接原文处理。
"""

import os
import json
import time
import hashlib
import datetime
import difflib
from pathlib import Path
from typing import Optional
import requests
from bs4 import BeautifulSoup

SNAPSHOT_PATH = Path("data/api_doc_snapshots.json")
OUTPUT_PATH = Path("changes_out.json")

PROVIDER_PAGES = [
    # Independent adapters
    {"provider": "OpenAI", "url": "https://raw.githubusercontent.com/openai/openai-openapi/master/openapi.yaml", "title": "OpenAI OpenAPI Spec"},
    {"provider": "Anthropic", "url": "https://docs.anthropic.com/claude/docs", "title": "Anthropic Claude API Documentation"},
    {"provider": "Google Gemini", "url": "https://ai.google.dev/gemini-api/docs", "title": "Google Gemini API Documentation"},
    {"provider": "Cohere", "url": "https://docs.cohere.com/reference/about", "title": "Cohere API Reference Overview"},
    {"provider": "Mistral", "url": "https://docs.mistral.ai/api/", "title": "Mistral API Reference"},
    
    # Config-driven providers
    {"provider": "Groq", "url": "https://groq.com/docs", "title": "Groq API Documentation"},
    {"provider": "DeepSeek", "url": "https://deepseek.com/docs", "title": "DeepSeek API Documentation"},
    {"provider": "Qwen", "url": "https://qwen.com/docs", "title": "Qwen API Documentation"},
    {"provider": "HuggingFace", "url": "https://huggingface.co/docs", "title": "HuggingFace API Documentation"},
    {"provider": "TogetherAI", "url": "https://docs.together.ai/docs", "title": "TogetherAI API Documentation"},
    {"provider": "Ollama", "url": "https://docs.ollama.ai/", "title": "Ollama API Documentation"},
    {"provider": "xAI Grok", "url": "https://x.ai/docs", "title": "xAI Grok API Documentation"},
    
    # Chinese providers
    {"provider": "Baidu Wenxin", "url": "https://cloud.baidu.com/doc/WENXINWORKSHOP/s/1lilb2u4t", "title": "Baidu Wenxin API Documentation"},
    {"provider": "Tencent Hunyuan", "url": "https://cloud.tencent.com/document/product/1129/74712", "title": "Tencent Hunyuan API Documentation"},
    {"provider": "iFlytek Spark", "url": "https://www.xfyun.cn/doc/spark/introduce.html", "title": "iFlytek Spark API Documentation"},
    {"provider": "Moonshot Kimi", "url": "https://docs.moonshot.cn/docs", "title": "Moonshot Kimi API Documentation"},
    
    # Enterprise providers
    {"provider": "Azure OpenAI", "url": "https://learn.microsoft.com/en-us/azure/ai-services/openai/reference", "title": "Azure OpenAI REST API Reference"},
    {"provider": "AWS Bedrock", "url": "https://docs.aws.amazon.com/bedrock/latest/userguide/api-methods.html", "title": "AWS Bedrock API Methods"},
]

PRIMARY_UA = "DocsWatchBot/1.0 (+https://github.com/hiddenpath/ai-lib)"
BROWSER_UA = (
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 "
    "(KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36"
)

BASE_HEADERS = {
    "User-Agent": PRIMARY_UA,
    "Accept": "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
    "Accept-Language": "en-US,en;q=0.9",
    "Connection": "close",
}

DIFF_MAX_LINES = 120
REQUEST_TIMEOUT = 40
RETRY = 2
RETRY_SLEEP = 3


def is_text_raw(url: str, resp: Optional[requests.Response]) -> bool:
    lowered = url.lower()
    if any(lowered.endswith(ext) for ext in (".yaml", ".yml", ".json", ".txt")):
        return True
    if resp is not None:
        ct = resp.headers.get("Content-Type", "").lower()
        if any(x in ct for x in ("yaml", "json", "text/plain")):
            return True
    return False


def fetch_page(url: str) -> str:
    last_exc = None
    for attempt in range(RETRY + 1):
        headers = dict(BASE_HEADERS)
        if attempt > 0:
            headers["User-Agent"] = BROWSER_UA
        try:
            r = requests.get(url, headers=headers, timeout=REQUEST_TIMEOUT)
            if r.status_code == 403 and attempt < RETRY:
                last_exc = f"HTTP 403 (attempt {attempt+1})"
                time.sleep(RETRY_SLEEP)
                continue
            r.raise_for_status()
            if is_text_raw(url, r):
                return r.text
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
    diff_lines = list(
        difflib.unified_diff(
            old.splitlines(),
            new.splitlines(),
            fromfile="previous",
            tofile="current",
            lineterm="",
        )
    )
    if len(diff_lines) > DIFF_MAX_LINES:
        truncated = diff_lines[:DIFF_MAX_LINES]
        truncated.append(f"...(diff truncated, total {len(diff_lines)} lines)")
        return "\n".join(truncated)
    return "\n".join(diff_lines)

def main():
    now_iso = datetime.datetime.utcnow().isoformat() + "Z"
    snapshots = load_snapshots()
    is_baseline = not SNAPSHOT_PATH.exists()
    changes = []
    fetch_errors = []

    for page in PROVIDER_PAGES:
        provider = page["provider"]
        url = page["url"]
        title = page["title"]
        try:
            content = fetch_page(url)
        except Exception as e:
            print(f"[WARN] Fetch failed {provider} {url}: {e}")
            fetch_errors.append({"provider": provider, "url": url, "error": str(e)})
            continue
        h = compute_hash(content)
        prev = snapshots.get(url)
        if prev is None:
            snapshots[url] = {
                "provider": provider,
                "title": title,
                "hash": h,
                "content": content,
                "fetched_at": now_iso,
            }
            print(f"[INFO] Baseline added {provider}: {title}")
        else:
            if prev.get("hash") != h:
                diff_text = unified_diff(prev.get("content", ""), content)
                changes.append(
                    {
                        "provider": provider,
                        "url": url,
                        "title": title,
                        "old_hash": prev.get("hash"),
                        "new_hash": h,
                        "diff": diff_text,
                    }
                )
                snapshots[url].update(
                    {
                        "hash": h,
                        "content": content,
                        "fetched_at": now_iso,
                    }
                )
                print(f"[CHANGE] {provider}: {title}")
            else:
                print(f"[NOCHANGE] {provider}: {title}")

    save_snapshots(snapshots)

    if is_baseline:
        print("[INFO] First run (baseline). No issue will be created.")
        if fetch_errors:
            print(f"[INFO] {len(fetch_errors)} provider(s) failed during baseline.")
        OUTPUT_PATH.write_text(
            json.dumps({"baseline": True, "changes": [], "fetch_errors": fetch_errors}, ensure_ascii=False, indent=2),
            encoding="utf-8",
        )
        return

    OUTPUT_PATH.write_text(
        json.dumps(
            {
                "baseline": False,
                "generated_at": now_iso,
                "changes": changes,
                "fetch_errors": fetch_errors,
            },
            ensure_ascii=False,
            indent=2,
        ),
        encoding="utf-8",
    )
    print(f"[INFO] {len(changes)} changes recorded. Fetch errors: {len(fetch_errors)}")


if __name__ == "__main__":
    main()