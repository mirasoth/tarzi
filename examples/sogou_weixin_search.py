#!/usr/bin/env python3
"""
SougouWeixin search example for the tarzi Python library.
Demonstrates searching WeChat articles using Sogou's WeChat search engine.
"""

import tarzi


def main():
    """Main function to demonstrate SougouWeixin search functionality."""
    print("=== Tarzi Python SougouWeixin Search Example ===\n")

    # Create configuration for SougouWeixin search
    config_str = """
[search]
engine = "sogou_weixin"
limit = 10

[fetcher]
web_driver = "chromedriver"
browser = true
format = "markdown"
"""

    try:
        config = tarzi.Config.from_str(config_str)
    except Exception as e:
        print(f"Failed to create config: {e}")
        # Fallback to default config
        config = tarzi.Config()
        print("Using default configuration")

    # Create search engine from config
    search_engine = tarzi.SearchEngine.from_config(config)

    # Query about Oracle Corporation stock price in Chinese
    query = "甲骨文股价"
    print(f"\nSearching WeChat articles for: '{query}'")
    # Perform the search
    try:
        results = search_engine.search(query, 10)
        print(f"\nFound {len(results)} WeChat articles:")
        if not results:
            print("No results found.")
        else:
            for i, result in enumerate(results):
                print(f"\n{i + 1}. {result.title}")
                print(f"   URL: {result.url}")
                if result.snippet:
                    print(f"   Snippet: {result.snippet}")
                print(f"   Rank: {result.rank}")

            print("\n=== Search Summary ===")
            print(f"Total results: {len(results)}")
            print("All results are from mp.weixin.qq.com (WeChat articles)")

    except Exception as e:
        print(f"Search failed: {e}")
        raise

    # Ensure clean shutdown of browser and driver resources
    try:
        search_engine.shutdown()
    except Exception as e:
        print(f"Warning: Failed to properly cleanup search engine: {e}")

    print("\n=== SougouWeixin Search Example completed successfully! ===")


if __name__ == "__main__":
    main()
