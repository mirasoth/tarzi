API Search Examples
===================

This guide demonstrates tarzi's search access cascade: API → plain HTTP → headless browser.

Access Cascade
--------------

With ``search.mode = "auto"`` (default), tarzi tries:

1. **API** when the engine supports apiquery and a key is available
2. **Plain HTTP** to the engine's public search URL
3. **Headless browser** as a last resort

Forced modes:

- ``apiquery`` — API only (errors if unsupported or key missing)
- ``webquery`` — plain HTTP then browser (never uses API)

Engine Capabilities
-------------------

.. list-table::
   :header-rows: 1
   :widths: 22 15 15 48

   * - Engine
     - Web
     - API
     - Notes
   * - ``bing``
     - Yes
     - No
     - Default engine
   * - ``google``
     - Yes
     - No
     - HTML only; use ``google_serper`` for API
   * - ``google_serper`` / ``serper``
     - No
     - Yes
     - Requires ``SERPER_API_KEY``
   * - ``brave``
     - Yes
     - Yes
     - ``BRAVE_API_KEY`` for API path
   * - ``duckduckgo``
     - Yes
     - No
     - Plain HTML URL differs from browser SERP
   * - ``baidu`` / ``sogou_weixin``
     - Yes
     - No
     - Web cascade only

Supported API Engines
---------------------

- **Brave** (``brave``): Brave Search API via ``BRAVE_API_KEY`` or ``search.api_key``
- **Google Serper** (``google_serper`` / ``serper``): Serper API via ``SERPER_API_KEY`` or ``search.api_key``

``google`` remains web-only. There is no Google Custom Search (CSE) integration.

Basic API Search
----------------

Python
~~~~~~

.. code-block:: python

   import tarzi

   config_str = """
   [search]
   engine = "brave"
   mode = "auto"
   limit = 5
   api_key = "your-brave-api-key"
   """

   config = tarzi.Config.from_str(config_str)
   search_engine = tarzi.SearchEngine.from_config(config)

   try:
       results = search_engine.search("artificial intelligence trends", 5)
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

   use tarzi::{config::Config, search::SearchEngine};

   #[tokio::main]
   async fn main() -> Result<(), Box<dyn std::error::Error>> {
       let mut config = Config::new();
       config.search.engine = "brave".to_string();
       config.search.mode = "apiquery".to_string();
       config.search.limit = 5;
       config.search.api_key = Some("your-brave-api-key".to_string());

       let mut search_engine = SearchEngine::from_config(&config);

       match search_engine.search("machine learning applications", 5).await {
           Ok(results) => {
               println!("Found {} results:", results.len());
               for (i, result) in results.iter().enumerate() {
                   println!("{}. {}", i + 1, result.title);
                   println!("   URL: {}", result.url);
               }
           }
           Err(e) => println!("Search failed: {}", e),
       }

       Ok(())
   }

Google via Serper
-----------------

.. code-block:: toml

   [search]
   engine = "google_serper"
   mode = "apiquery"
   limit = 10
   # Prefer env: export SERPER_API_KEY=...
   # api_key = "your-serper-api-key"

Web-only Mode
-------------

Force the HTTP → browser path (useful to avoid spending API quota):

.. code-block:: python

   import tarzi

   config = tarzi.Config.from_str(
       """
   [search]
   engine = "brave"
   mode = "webquery"
   limit = 5
   """
   )
   engine = tarzi.SearchEngine.from_config(config)
   results = engine.search("rust async", 5)

Environment Variables
---------------------

- ``BRAVE_API_KEY`` — Brave Search API
- ``SERPER_API_KEY`` — Google Serper API

Environment variables take precedence over ``search.api_key`` in config.

Runnable Examples
-----------------

From the repository ``examples/`` directory:

.. code-block:: bash

   cargo run --example search_modes
   cargo run --example search_engine_brave
   cargo run --example search_engine_serper

   python examples/search_modes.py
   python examples/search_engine_serper.py

See also :doc:`/configuration` for the full engine capability table.
