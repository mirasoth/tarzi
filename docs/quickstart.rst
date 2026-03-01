Quick Start Guide
=================

This guide will get you up and running with tarzi in just a few minutes. 
We'll cover the most common use cases and basic functionality.

.. note::
   tarzi supports only Linux and macOS. Windows is not supported.

Your First tarzi Program
-------------------------

Python
~~~~~~

Let's start with a simple example that demonstrates the core functionality:

.. code-block:: python

   import tarzi

   # 1. Convert HTML to Markdown
   html = "<h1>Hello World</h1><p>This is a <strong>test</strong>.</p>"
   markdown = tarzi.convert_html(html, "markdown")
   print("Converted to Markdown:")
   print(markdown)

   # 2. Fetch a web page
   try:
       content = tarzi.fetch_url(
           "https://httpbin.org/html", 
           mode="plain_request", 
           format="markdown"
       )
       print("\nFetched content:")
       print(content[:200] + "...")
   except Exception as e:
       print(f"Fetch failed: {e}")

   # 3. Search the web (browser-based)
   try:
       results = tarzi.search_web(
           "python web scraping", 
           mode="webquery", 
           limit=3
       )
       print(f"\nFound {len(results)} search results:")
       for i, result in enumerate(results):
           print(f"{i+1}. {result.title}")
           print(f"   URL: {result.url}")
           print(f"   Snippet: {result.snippet[:100]}...")
   except Exception as e:
       print(f"Search failed: {e}")

   # 4. Search using API providers (requires API keys)
   try:
       results = tarzi.search_web(
           "machine learning trends", 
           mode="apiquery", 
           limit=3
       )
       print(f"\nAPI search found {len(results)} results:")
       for i, result in enumerate(results):
           print(f"{i+1}. {result.title}")
           print(f"   URL: {result.url}")
           print(f"   Snippet: {result.snippet[:100]}...")
   except Exception as e:
       print(f"API search failed: {e}")

Save this as `quickstart.py` and run it:

.. code-block:: bash

   python quickstart.py

Rust
~~~~

Here's the equivalent Rust program:

.. code-block:: rust

   use tarzi::{Converter, WebFetcher, SearchEngine, Format, FetchMode, SearchMode};

   #[tokio::main]
   async fn main() -> Result<(), Box<dyn std::error::Error>> {
       // 1. Convert HTML to Markdown
       let converter = Converter::new();
       let html = "<h1>Hello World</h1><p>This is a <strong>test</strong>.</p>";
       let markdown = converter.convert(html, Format::Markdown).await?;
       println!("Converted to Markdown:\n{}", markdown);

       // 2. Fetch a web page
       let mut fetcher = WebFetcher::new();
       match fetcher.fetch(
           "https://httpbin.org/html",
           FetchMode::PlainRequest,
           Format::Markdown
       ).await {
           Ok(content) => {
               println!("\nFetched content:\n{}...", &content[..200.min(content.len())]);
           }
           Err(e) => println!("Fetch failed: {}", e),
       }

       // 3. Search the web (browser-based)
       let mut search_engine = SearchEngine::new();
       match search_engine.search(
           "agentic AI",
           SearchMode::WebQuery,
           3
       ).await {
           Ok(results) => {
               println!("\nFound {} search results:", results.len());
               for (i, result) in results.iter().enumerate() {
                   println!("{}. {}", i + 1, result.title);
                   println!("   URL: {}", result.url);
                   println!("   Snippet: {}...", &result.snippet[..100.min(result.snippet.len())]);
               }
           }
           Err(e) => println!("Search failed: {}", e),
       }

       // 4. Search using API providers (requires API keys)
       let mut api_search_engine = SearchEngine::from_config(&Config::new());
       match api_search_engine.search(
           "machine learning trends",
           SearchMode::ApiQuery,
           3
       ).await {
           Ok(results) => {
               println!("\nAPI search found {} results:", results.len());
               for (i, result) in results.iter().enumerate() {
                   println!("{}. {}", i + 1, result.title);
                   println!("   URL: {}", result.url);
                   println!("   Snippet: {}...", &result.snippet[..100.min(result.snippet.len())]);
               }
           }
           Err(e) => println!("API search failed: {}", e),
       }

       Ok(())
   }

Save this as `src/main.rs` in a new Cargo project and run:

.. code-block:: bash

   cargo run

CLI
~~~

You can also use the command-line interface:

.. code-block:: bash

   # Convert HTML to Markdown
   tarzi convert --input "<h1>Hello</h1>" --format markdown

   # Fetch a web page
   tarzi fetch --url "https://httpbin.org/html" --format markdown

   # Search the web
   tarzi search --query "agentic AI" --limit 3

Core Concepts
-------------

Formats
~~~~~~~

tarzi supports multiple output formats:

- **Markdown**: Clean, readable text format
- **JSON**: Structured data with metadata
- **YAML**: Human-readable structured format

.. code-block:: python

   # Try different formats
   html = "<h1>Title</h1><p>Content with <a href='#'>link</a>.</p>"
   
   markdown = tarzi.convert_html(html, "markdown")
   json_data = tarzi.convert_html(html, "json")
   yaml_data = tarzi.convert_html(html, "yaml")
   
   print("Markdown:", markdown)
   print("JSON:", json_data)
   print("YAML:", yaml_data)

Fetch Modes
~~~~~~~~~~~

Different modes for fetching web content:

- **plain_request**: Fast HTTP GET request (no JavaScript)
- **browser_headless**: Full browser automation (supports JavaScript)
- **browser_head**: Browser automation with visible window (for debugging)

.. code-block:: python

   # Static content (fast)
   content = tarzi.fetch_url(
       "https://example.com", 
       mode="plain_request"
   )

   # JavaScript-heavy sites (slower but more complete)
   content = tarzi.fetch_url(
       "https://spa-example.com", 
       mode="browser_headless"
   )

Search Modes
~~~~~~~~~~~~

Two approaches to web search:

- **webquery**: Scrape search engine results pages (no API key needed)
- **apiquery**: Use official search APIs (requires API key)

API Search Providers
~~~~~~~~~~~~~~~~~~~~

tarzi supports multiple API search providers with automatic fallback:

- **Brave Search API**: Fast, privacy-focused search
- **Google API**: Google search results via API
- **Exa Search API**: AI-powered semantic search
- **Travily API**: Travel-focused search engine
- **DuckDuckGo API**: Privacy-focused search (limited functionality)

Autoswitch Strategy
~~~~~~~~~~~~~~~~~~~

When using API search, tarzi can automatically switch between providers:

- **smart**: Automatically fallback to available providers if primary fails
- **none**: Only use the configured primary search engine

.. code-block:: python

   # Browser-based search (no API key needed)
   results = tarzi.search_web(
       "machine learning", 
       mode="webquery", 
       limit=10
   )

   # API-based search (requires API key configuration)
   results = tarzi.search_web(
       "artificial intelligence", 
       mode="apiquery", 
       limit=10
   )

Configuration
-------------

Basic configuration can be done through environment variables or a `tarzi.toml` file:

.. code-block:: toml

   [search]
   engine = "brave"
   mode = "apiquery"
   autoswitch = "smart"
   limit = 5
   
   # API keys for different providers
   brave_api_key = "your-brave-api-key"
   exa_api_key = "your-exa-api-key"
   travily_api_key = "your-travily-api-key"

   [fetcher]
   user_agent = "Mozilla/5.0 (compatible; Tarzi/1.0)"
   timeout = 30
   proxy = "http://proxy.example.com:8080"

Environment Variables
~~~~~~~~~~~~~~~~~~~~~

.. code-block:: bash

   # Proxy configuration (standard environment variables)
   export http_proxy=http://proxy.example.com:8080
   export https_proxy=http://proxy.example.com:8080

   # Debug mode (for development/testing)
   export TARZI_DEBUG=1

Next Steps
----------

- Read the configuration and development guides for detailed usage patterns
- Check out the :doc:`examples/index` for more examples
- Explore the :doc:`python_api/index` or :doc:`rust_api/index` for API reference
- Configure advanced options in :doc:`configuration` 