#!/usr/bin/env python3
"""Google search via Serper API (engine=google_serper).

Requires:
  export SERPER_API_KEY=your-key
"""

import tarzi


def main():
    config = tarzi.Config.from_str(
        """
[search]
engine = "google_serper"
limit = 5
"""
    )
    # Prefer SERPER_API_KEY env over embedding keys in config.

    search_engine = tarzi.SearchEngine.from_config(config)
    query = "agentic AI frameworks"
    try:
        results = search_engine.search(query, 5)
        print(f"Found {len(results)} Serper results:")
        for i, result in enumerate(results):
            print(f"{i + 1}. {result.title}")
            print(f"   URL: {result.url}")
            print(f"   Snippet: {result.snippet}")
            print()
    except Exception as e:
        print(f"Serper search failed: {e}")
        print("Set SERPER_API_KEY (or search.api_key) and retry.")
    finally:
        search_engine.shutdown()


if __name__ == "__main__":
    main()
