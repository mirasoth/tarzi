#!/usr/bin/env python3
"""Demonstrate search access cascade, browser toggle, and multi-engine failover."""

import tarzi


def run_search(engine: str, browser: bool, query: str, limit: int = 3) -> None:
    config = tarzi.Config.load()
    config_str = f"""
[search]
engine = "{engine}"
browser = {str(browser).lower()}
limit = {limit}

[fetcher]
browser = false
"""
    config = tarzi.Config.from_str(config_str)
    search = tarzi.SearchEngine.from_config(config)
    try:
        results = search.search(query, limit)
        print(f"OK engine={engine} browser={browser} results={len(results)}")
        for i, r in enumerate(results, 1):
            print(f"  {i}. {r.title} — {r.url}")
    except Exception as e:
        print(f"ERR engine={engine}: {e}")
    finally:
        search.shutdown()


def main() -> None:
    query = "rust programming language"
    print("=== Bing (browser enabled) ===")
    run_search("bing", True, query)

    print("\n=== DuckDuckGo (browser disabled) ===")
    run_search("duckduckgo", False, query)

    print("\n=== Brave (API if BRAVE_API_KEY set) ===")
    run_search("brave", True, query)

    print("\n=== google_serper (API-only) ===")
    run_search("google_serper", True, query)

    print("\n=== Multi-engine: google_serper,duckduckgo,bing ===")
    run_search("google_serper,duckduckgo,bing", False, query)


if __name__ == "__main__":
    main()
