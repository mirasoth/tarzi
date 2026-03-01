API Search Examples
===================

This guide demonstrates how to use tarzi's API search features with multiple providers and automatic fallback capabilities.

Parser Architecture
-------------------

Tarzi uses a unified parser architecture where all search engines inherit from base parser traits:

- **BaseSearchParser**: Core trait with name, engine type, and support checking
- **WebSearchParser**: HTML-based parsing for browser scraping
- **ApiSearchParser**: JSON-based parsing for API responses
- **UnifiedParser**: Combines web and API parsing capabilities
- **ParserFactory**: Mode-aware parser selection (WebQuery vs ApiQuery)

This architecture ensures consistent parsing across all search engines and makes it easy to add new engines.

Basic API Search
----------------

Python
~~~~~~~

.. code-block:: python

   import tarzi

   # Configure API search with multiple providers
   config_str = """
   [search]
   engine = "brave"
   mode = "apiquery"
   autoswitch = "smart"
   limit = 5
   
   brave_api_key = "your-brave-api-key"
   exa_api_key = "your-exa-api-key"
   """

   config = tarzi.Config.from_str(config_str)
   search_engine = tarzi.SearchEngine.from_config(config)

   # Perform API search
   try:
       results = search_engine.search(
           "artificial intelligence trends 2024",
           mode="apiquery",
           limit=5
       )
       
       print(f"Found {len(results)} results:")
       for i, result in enumerate(results):
           print(f"{i+1}. {result.title}")
           print(f"   URL: {result.url}")
           print(f"   Snippet: {result.snippet[:150]}...")
           
   except Exception as e:
       print(f"Search failed: {e}")

Rust
~~~~

.. code-block:: rust

   use tarzi::{Config, SearchEngine, SearchMode};

   #[tokio::main]
   async fn main() -> Result<(), Box<dyn std::error::Error>> {
       // Create configuration with API providers
       let mut config = Config::new();
       config.search.engine = "brave".to_string();
       config.search.mode = "apiquery".to_string();
       config.search.autoswitch = "smart".to_string();
       config.search.limit = 5;
       
       // Set API keys
       config.search.brave_api_key = Some("your-brave-api-key".to_string());
       config.search.exa_api_key = Some("your-exa-api-key".to_string());

       let mut search_engine = SearchEngine::from_config(&config);

       // Perform API search
       match search_engine.search(
           "machine learning applications",
           SearchMode::ApiQuery,
           5
       ).await {
           Ok(results) => {
               println!("Found {} results:", results.len());
               for (i, result) in results.iter().enumerate() {
                   println!("{}. {}", i + 1, result.title);
                   println!("   URL: {}", result.url);
                   println!("   Snippet: {}...", &result.snippet[..150.min(result.snippet.len())]);
               }
           }
           Err(e) => println!("Search failed: {}", e),
       }

       Ok(())
   }

Autoswitch Strategies
---------------------

Smart Fallback
~~~~~~~~~~~~~~

The smart autoswitch strategy automatically falls back to available providers when the primary provider fails:

.. code-block:: python

   import tarzi

   # Configure with smart autoswitch
   config_str = """
   [search]
   engine = "brave"  # Primary provider
   mode = "apiquery"
   autoswitch = "smart"  # Enable automatic fallback
   
   brave_api_key = "your-brave-api-key"
   exa_api_key = "your-exa-api-key"
   """

   config = tarzi.Config.from_str(config_str)
   search_engine = tarzi.SearchEngine.from_config(config)

   # If Brave fails, it will automatically try Brave, then Exa
   results = search_engine.search("quantum computing", mode="apiquery", limit=3)

No Fallback
~~~~~~~~~~~

The none strategy only uses the configured primary provider:

.. code-block:: python

   import tarzi

   # Configure with no autoswitch
   config_str = """
   [search]
   engine = "brave"
   mode = "apiquery"
   autoswitch = "none"  # Disable automatic fallback
   
   brave_api_key = "your-brave-api-key"
   """

   config = tarzi.Config.from_str(config_str)
   search_engine = tarzi.SearchEngine.from_config(config)

   # Will only use Brave, no fallback to other providers
   results = search_engine.search("blockchain technology", mode="apiquery", limit=3)

Provider-Specific Examples
--------------------------

Brave Search API
~~~~~~~~~~~~~~~~

.. code-block:: python

   import tarzi

   config_str = """
   [search]
   engine = "brave"
   mode = "apiquery"
   autoswitch = "none"
   
   brave_api_key = "your-brave-api-key"
   """

   config = tarzi.Config.from_str(config_str)
   search_engine = tarzi.SearchEngine.from_config(config)

   # Brave Search is fast and privacy-focused
   results = search_engine.search("privacy tools", mode="apiquery", limit=5)

Exa Search API
~~~~~~~~~~~~~~

.. code-block:: python

   import tarzi

   config_str = """
   [search]
   engine = "exa"
   mode = "apiquery"
   autoswitch = "none"
   
   exa_api_key = "your-exa-api-key"
   """

   config = tarzi.Config.from_str(config_str)
   search_engine = tarzi.SearchEngine.from_config(config)

   # Exa provides AI-powered semantic search
   results = search_engine.search("sustainable energy solutions", mode="apiquery", limit=5)

Travily API
~~~~~~~~~~~

.. code-block:: python

   import tarzi

   config_str = """
   [search]
   engine = "travily"
   mode = "apiquery"
   autoswitch = "none"
   
   travily_api_key = "your-travily-api-key"
   """

   config = tarzi.Config.from_str(config_str)
   search_engine = tarzi.SearchEngine.from_config(config)

   # Travily specializes in travel and location-based search
   results = search_engine.search("best restaurants in Paris", mode="apiquery", limit=5)

DuckDuckGo API
~~~~~~~~~~~~~~

.. code-block:: python

   import tarzi

   config_str = """
   [search]
   engine = "duckduckgo"
   mode = "apiquery"
   autoswitch = "none"
   """

   config = tarzi.Config.from_str(config_str)
   search_engine = tarzi.SearchEngine.from_config(config)

   # DuckDuckGo doesn't require an API key but has limited functionality
   results = search_engine.search("weather forecast", mode="apiquery", limit=3)

Environment Variable Configuration
----------------------------------

You can also configure API keys using environment variables:

.. code-block:: bash

   # Set API keys via environment variables
   export BRAVE_API_KEY=your-brave-api-key
   export EXA_API_KEY=your-exa-api-key
   export TRAVILY_API_KEY=your-travily-api-key

   # Run your application
   python your_app.py

.. code-block:: python

   import tarzi
   import os

   # Create config that will use environment variables
   config = tarzi.Config.new()
   config.search.engine = "brave"
   config.search.mode = "apiquery"
   config.search.autoswitch = "smart"

   # API keys will be automatically loaded from environment variables
   search_engine = tarzi.SearchEngine.from_config(config)
   results = search_engine.search("climate change", mode="apiquery", limit=5)

Error Handling
--------------

API search includes comprehensive error handling:

.. code-block:: python

   import tarzi

   try:
       results = search_engine.search("test query", mode="apiquery", limit=5)
       print(f"Success: {len(results)} results")
   except tarzi.TarziError as e:
       if "API key" in str(e):
           print("Error: Invalid or missing API key")
       elif "rate limit" in str(e):
           print("Error: Rate limit exceeded")
       elif "network" in str(e):
           print("Error: Network connection failed")
       else:
           print(f"Error: {e}")

Performance Comparison
----------------------

Compare browser-based vs API-based search:

.. code-block:: python

   import tarzi
   import time

   # Browser-based search (no API key needed)
   start_time = time.time()
   browser_results = search_engine.search("python tutorial", mode="webquery", limit=5)
   browser_time = time.time() - start_time

   # API-based search (requires API key)
   start_time = time.time()
   api_results = search_engine.search("python tutorial", mode="apiquery", limit=5)
   api_time = time.time() - start_time

   print(f"Browser search: {browser_time:.2f}s")
   print(f"API search: {api_time:.2f}s")
   print(f"API is {browser_time/api_time:.1f}x faster")

Best Practices
--------------

1. **Use Smart Autoswitch**: Enable smart autoswitch for production applications to ensure reliability
2. **Configure Multiple Providers**: Set up multiple API keys for better fallback options
3. **Monitor Rate Limits**: Be aware of API rate limits for each provider
4. **Error Handling**: Always implement proper error handling for API failures
5. **Environment Variables**: Use environment variables for API keys in production
6. **Proxy Support**: Configure proxies if needed for enterprise environments

For more advanced usage patterns, see the :doc:`../configuration` guide. 