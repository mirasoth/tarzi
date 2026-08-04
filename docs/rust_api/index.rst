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
   - ``FetchMode`` - Fetching strategies
   - ``SearchMode`` - Search access modes (Auto, ApiQuery, WebQuery)
   - ``AccessMethod`` - Resolved access path (Api, PlainHttp, Browser)
   - ``SearchEngineType`` - Supported search engines
     (Bing, DuckDuckGo, Google, GoogleSerper, BraveSearch, Baidu, SougouWeixin)

Basic Usage
-----------

.. code-block:: rust

   use tarzi::{Converter, WebFetcher, SearchEngine, Format, FetchMode};
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
           FetchMode::PlainRequest,
           Format::Markdown
       ).await?;

       // Search web (auto cascade from config / defaults)
       let mut search_engine = SearchEngine::new();
       let results = search_engine.search("agentic AI", 10).await?;

       // Parse HTML SERP content
       let factory = ParserFactory::new();
       let parser = factory.get_parser(&SearchEngineType::Google);
       let _parsed = parser.parse(&content, 10)?;

       let _ = (markdown, results);
       Ok(())
   }

Search Engines and Modes
------------------------

Configure ``Config.search.engine`` and ``Config.search.mode``
(``auto`` | ``apiquery`` | ``webquery``). Defaults: Bing + ``auto``.

.. code-block:: rust

   use tarzi::{config::Config, search::SearchEngine};

   #[tokio::main]
   async fn main() -> Result<(), Box<dyn std::error::Error>> {
       // Brave: API when BRAVE_API_KEY is set, else web cascade
       let mut config = Config::new();
       config.search.engine = "brave".to_string();
       config.search.mode = "auto".to_string();
       let mut engine = SearchEngine::from_config(&config);
       let _ = engine.search("rust ownership", 5).await?;

       // Google via Serper (API-only)
       config.search.engine = "google_serper".to_string();
       config.search.mode = "apiquery".to_string();
       let mut serper = SearchEngine::from_config(&config);
       let _ = serper.search("tokio runtime", 5).await?;

       // Force web path
       config.search.engine = "duckduckgo".to_string();
       config.search.mode = "webquery".to_string();
       let mut web = SearchEngine::from_config(&config);
       let _ = web.search("cargo workspace", 5).await?;

       Ok(())
   }

Resolve the cascade without searching via ``tarzi::search::resolve_access``.
See :doc:`/configuration` and :doc:`/examples/api_search`.
