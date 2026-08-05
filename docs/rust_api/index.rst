Rust API Reference
===================

Complete reference for the tarzi Rust API.

Quick Reference
---------------

**Core Modules**
   - ``tarzi::converter`` - HTML conversion functionality
   - ``tarzi::fetcher`` - Web page fetching
   - ``tarzi::search`` - Search engine integration
   - ``tarzi::search::parser`` - Search result parsing
   - ``tarzi::search::access`` - Access cascade resolution
   - ``tarzi::search::api`` - Brave / Serper API clients

**Main Structs**
   - ``Converter`` - HTML conversion
   - ``WebFetcher`` - Web page fetching
   - ``SearchEngine`` - Web search operations
   - ``ParserFactory`` - Parser creation and management

**Enums**
   - ``Format`` - Output formats (Markdown, JSON, YAML, HTML)
   - ``WebFetcher`` - Plain HTTP → headless browser cascade
   - ``AccessMethod`` - Search access methods (Api, PlainHttp, Browser)
   - ``AccessMethod`` - Resolved access path (Api, PlainHttp, Browser)
   - ``SearchEngineType`` - Supported search engines
     (Bing, DuckDuckGo, Google, GoogleSerper, BraveSearch, Baidu, SougouWeixin)

Basic Usage
-----------

.. code-block:: rust

   use tarzi::{Converter, WebFetcher, SearchEngine, Format};
   use tarzi::search::{ParserFactory, SearchEngineType};

   #[tokio::main]
   async fn main() -> Result<(), Box<dyn std::error::Error>> {
       // Convert HTML
       let converter = Converter::new();
       let markdown = converter.convert("<h1>Hello</h1>", Format::Markdown).await?;

       // Fetch web page
       let mut fetcher = WebFetcher::new();
       let content = fetcher.fetch(
           "https://example.com",
           Format::Markdown
       ).await?;

       // Search web (cascade from config / defaults)
       let mut search_engine = SearchEngine::new();
       let results = search_engine.search("agentic AI", 10).await?;

       // Parse HTML SERP content
       let factory = ParserFactory::new();
       let parser = factory.get_parser(&SearchEngineType::Google);
       let _parsed = parser.parse(&content, 10)?;

       let _ = (markdown, results);
       Ok(())
   }

Search Engines and Access Cascade
---------------------------------

Configure ``Config.search.engine`` (comma-separated failover OK) and ``Config.search.browser`` (default true). Defaults: ``duckduckgo,bing,brave`` + browser enabled.

.. code-block:: rust

   use tarzi::{config::Config, search::SearchEngine};

   #[tokio::main]
   async fn main() -> Result<(), Box<dyn std::error::Error>> {
       // Brave: API when BRAVE_API_KEY is set, else web cascade
       let mut config = Config::new();
       config.search.engine = "brave".to_string();
       config.search.browser = true;
       let mut engine = SearchEngine::from_config(&config);
       let _ = engine.search("rust ownership", 5).await?;

       // Google via Serper (API-only)
       config.search.engine = "google_serper".to_string();
       config.search.browser = false;
       let mut serper = SearchEngine::from_config(&config);
       let _ = serper.search("tokio runtime", 5).await?;

       // DuckDuckGo with browser fallback enabled
       config.search.engine = "duckduckgo".to_string();
       config.search.browser = true;
       let mut web = SearchEngine::from_config(&config);
       let _ = web.search("cargo workspace", 5).await?;

       Ok(())
   }

Resolve the cascade without searching via ``tarzi::search::resolve_access``.
See :doc:`/configuration` and :doc:`/examples/api_search`.
