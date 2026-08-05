Overview
========

.. note::
   tarzi supports only Linux and macOS. Windows is not supported.

What is tarzi?
--------------

**Tarzi** is a unified search interface designed for **Retrieval-Augmented Generation (RAG)** and **agentic systems** built on large language models. Search is a core functionality in these systems, yet most search engine providers (SEPs) impose API paywalls or strict rate limits. **Tarzi**, empowered by browser automation and web crawling technologies, removes these barriers by supporting token-free queries across multiple search engines. With a single dependency, you can integrate and switch between different SEPs as needed—seamlessly and efficiently.

Key Components
--------------

Converter Module
~~~~~~~~~~~~~~~~

The Converter module is responsible for transforming raw HTML content into various structured formats:

- **HTML to Markdown**: Clean, readable text format perfect for AI training data
- **HTML to JSON**: Structured data with metadata (title, links, images, content)
- **HTML to YAML**: Human-readable structured format for configuration and data storage

Key features:

- Intelligent content extraction
- Metadata preservation
- Customizable output formatting
- Memory-efficient processing

Fetcher Module
~~~~~~~~~~~~~~

The Fetcher module handles web page retrieval with multiple strategies:

**HTTP Mode**
   Fast, lightweight HTTP requests for static content

**Browser Automation**
   Full browser automation for JavaScript-heavy sites:
   
   - Headless mode for server environments
   - Headed mode for debugging

**Proxy Support**
   Custom proxy support for HTTP and headless-browser fetches

Key features:

- Multiple fetch strategies
- Automatic retry logic
- Custom user agent support
- Timeout configuration
- Cookie and session management

Search Module
~~~~~~~~~~~~~

The Search module provides comprehensive search engine integration:

**Access Cascade**
   Each query resolves in priority order:

   - **API** when a key is available (Brave, Google Serper)
   - **Plain HTTP** to the engine public search URL
   - **Headless browser** as last resort

**HTML Parsers**
   Engine-specific HTML parsers via ``BaseParser`` + ``ParserFactory``:

   - Bing, Google, DuckDuckGo, Brave, Baidu, Sogou Weixin

**API Clients**
   REST JSON clients for:

   - Brave Search API (``BRAVE_API_KEY``, engine ``brave``)
   - Google Serper (``SERPER_API_KEY``, engine ``google_serper`` / ``serper``)

   ``google`` is web-only; API Google results use ``google_serper`` (no CSE).

**Configuration**
   Environment variables (``TARZI_*``); file config (``tarzi.toml``) was removed.
   ``TARZI_SEARCH_BROWSER`` = ``true`` | ``false`` (default true); multi-engine via comma-separated ``TARZI_SEARCH_ENGINE``;
   API keys via ``BRAVE_API_KEY`` / ``SERPER_API_KEY`` only.

Key features:

- Multiple search engine support
- Configurable access cascade and result limits
- Search result ranking
- Snippet extraction
- URL validation and cleaning
- Extensible parser architecture

Getting Started
---------------

Ready to get started? Check out our :doc:`installation` guide and :doc:`quickstart` tutorial 
to begin using tarzi in your projects.

For detailed examples and advanced usage patterns, see our :doc:`examples/index` section. 