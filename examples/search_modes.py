#!/usr/bin/env python3
"""Demonstrate search access modes: auto, apiquery, and webquery.

Optional API keys:
  export BRAVE_API_KEY=...
  export SERPER_API_KEY=...
"""

import tarzi


def run_search(engine: str, mode: str, query: str = "rust programming language", limit: int = 3):
    config = tarzi.Config.from_str(
        f"""
[search]
engine = "{engine}"
mode = "{mode}"
limit = {limit}
"""
    )
    search_engine = tarzi.SearchEngine.from_config(config)
    print(f"=== {engine} / mode={mode} ===")
    try:
        results = search_engine.search(query, limit)
        print(f"Found {len(results)} results:")
        for i, result in enumerate(results):
            print(f"  {i + 1}. {result.title} — {result.url}")
    except Exception as e:
        print(f"Search failed (expected if key/network missing): {e}")
    finally:
        search_engine.shutdown()
    print()


def main():
    # Bing has no public Search API; auto falls back to plain HTTP → browser
    run_search("bing", "auto")

    # Force web path for DuckDuckGo (plain HTML then browser)
    run_search("duckduckgo", "webquery")

    # Brave: API first when BRAVE_API_KEY is set
    run_search("brave", "auto")
    run_search("brave", "apiquery")

    # Google via Serper (API-only)
    run_search("google_serper", "apiquery")

    # Google web-only
    run_search("google", "webquery")


if __name__ == "__main__":
    main()
