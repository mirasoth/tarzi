#!/usr/bin/env python3
"""
Basic usage example for the tarzi Python library.
"""

import tarzi


def main():
    html_input = """
    <html>
        <head><title>Example Page</title></head>
        <body>
            <h1>Welcome to Tarzi</h1>
            <p>This is a <strong>test</strong> page with <a href="https://example.com">a link</a>.</p>
            <img src="image.jpg" alt="Test image">
        </body>
    <
    """

    # Converter
    converter = tarzi.Converter()
    yaml_output = converter.convert(html_input, "yaml")
    print(f"YAML output:\n{yaml_output}\n")

    # WebFetcher with different modes
    fetcher = tarzi.WebFetcher()
    try:
        # Plain request mode
        content = fetcher.fetch("https://httpbin.org/html", "plain_request", "html")
        print(f"Plain request - Fetched content length: {len(content)}")

        # Raw fetch mode
        raw_content = fetcher.fetch("https://httpbin.org/html", "plain_request", "html")
        print(f"Raw fetch - Content length: {len(raw_content)}")
    except Exception as e:
        print(f"Fetch failed: {e}")

    # SearchEngine (default: duckduckgo,bing,brave + access cascade)
    search_engine = tarzi.SearchEngine()
    try:
        results = search_engine.search("machine learning", 2)
        print(f"Found {len(results)} search results:")
        for i, result in enumerate(results):
            print(f"  {i+1}. {result.title} ({result.url})")
            print(f"     {result.snippet}")
    except Exception as e:
        print(f"Search failed: {e}")
    finally:
        search_engine.shutdown()

    # Configuration-based usage
    try:
        config_str = """
[fetcher]
timeout = 30
format = "markdown"
web_driver = "chromedriver"

[search]
engine = "brave,duckduckgo"
browser = true
limit = 3
"""
        config = tarzi.Config.from_str(config_str)
        print("Created config from string successfully")

        tarzi.WebFetcher.from_config(config)
        print("Created fetcher from config successfully")

        tarzi.SearchEngine.from_config(config)
        print("Created search engine from config successfully")
        print("Tip: set BRAVE_API_KEY / SERPER_API_KEY for API path; see examples/search_cascade.py")

    except Exception as e:
        print(f"Configuration usage failed: {e}")


if __name__ == "__main__":
    main()
