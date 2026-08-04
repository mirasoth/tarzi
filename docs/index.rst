.. warning::
   tarzi currently supports only Linux and macOS. Windows is not supported.

tarzi - Rust-native lite search for AI applications
====================================================

.. image:: https://img.shields.io/crates/v/tarzi.svg
   :target: https://crates.io/crates/tarzi
   :alt: Crate Version

.. image:: https://img.shields.io/pypi/v/tarzi.svg
   :target: https://pypi.org/project/tarzi/
   :alt: PyPI Version

.. image:: https://img.shields.io/badge/License-Apache%202.0-blue.svg
   :target: https://www.apache.org/licenses/LICENSE-2.0
   :alt: License

.. image:: https://img.shields.io/github/actions/workflow/status/mirasoth/tarzi/rust-ci.yml?branch=main
   :target: https://github.com/mirasoth/tarzi/actions
   :alt: Build Status

**tarzi** is a powerful, Rust-native search library designed specifically for AI applications. 
It provides a comprehensive toolkit for content conversion, web fetching, and search engine integration 
with both browser-based and API-based approaches.

.. toctree::
   :maxdepth: 1
   :caption: Contents:

   overview
   installation
   quickstart
   python_api/index
   rust_api/index
   examples/index
   configuration
   development

Key Features
============

🔧 **Dual Implementation**
   Native Rust library with Python bindings and CLI tools

🔄 **Content Conversion**
   Convert raw HTML to Markdown, JSON, or YAML formats

🌐 **Web Fetching**
   Fetch web pages with optional JavaScript rendering support

🔍 **Search Integration**
   Access cascade: API key → plain HTTP → headless browser

🎯 **Web Search Engines**
   Bing, Google, DuckDuckGo, Brave, Baidu, Sogou Weixin, Google Serper

🚀 **API Search Providers**
   Brave Search API and Google Serper (``SERPER_API_KEY`` / ``BRAVE_API_KEY``)

🔒 **Proxy Support**
   Use proxies for plain HTTP and API paths (browser proxy is limited)

⚡ **End-to-End Pipeline**
   Complete workflow from search queries to content extraction for AI applications

Quick Start
===========

Python
------

.. code-block:: bash

   pip install tarzi

.. code-block:: python

   import tarzi

   # Convert HTML to Markdown
   markdown = tarzi.convert_html("<h1>Hello</h1>", "markdown")

   # Fetch web page
   content = tarzi.fetch_url("https://example.com", mode="browser_headless")

   # Search web (auto: API → plain HTTP → browser)
   results = tarzi.search_web("python programming", 10)

   # Prefer Serper / Brave via config + env keys
   config = tarzi.Config.from_str("""
[search]
engine = "google_serper"
mode = "auto"
""")
   engine = tarzi.SearchEngine.from_config(config)
   results = engine.search("machine learning", 10)

Rust
----

.. code-block:: bash

   cargo add tarzi

.. code-block:: rust

   use tarzi::{config::Config, Converter, WebFetcher, SearchEngine, Format, FetchMode};

   #[tokio::main]
   async fn main() -> Result<(), Box<dyn std::error::Error>> {
       // Convert HTML to Markdown
       let converter = Converter::new();
       let markdown = converter.convert("<h1>Hello</h1>", Format::Markdown).await?;

       // Fetch web page
       let mut fetcher = WebFetcher::new();
       let content = fetcher.fetch(
           "https://example.com",
           FetchMode::BrowserHeadless,
           Format::Markdown
       ).await?;

       // Search web (auto cascade)
       let mut search_engine = SearchEngine::new();
       let results = search_engine.search("agentic AI", 5).await?;

       // Prefer API when configured
       let mut config = Config::new();
       config.search.engine = "brave".to_string();
       config.search.mode = "auto".to_string();
       let mut api_search_engine = SearchEngine::from_config(&config);
       let api_results = api_search_engine.search("machine learning", 5).await?;

       Ok(())
   }

CLI
---

.. code-block:: bash

   # Install the CLI tool
   cargo install tarzi

   # Convert HTML to Markdown
   tarzi convert --input "<h1>Hello</h1>" --format markdown

   # Fetch web page with JavaScript rendering
   tarzi fetch --url "https://example.com" --mode browser_headless --format json

   # Search and fetch content
   tarzi search-and-fetch \
     --query "agentic AI" \
     --fetch-mode plain_request \
     --format markdown \
     --limit 5

Use Cases
=========

🤖 **AI Data Collection**
   Gather and process web content for training data or knowledge bases

📊 **Research Automation**
   Automate web research workflows for academic or business intelligence

🔍 **Content Aggregation**
   Build content aggregation systems that convert web pages to structured data

🕷️ **Web Scraping Pipelines**
   Create robust web scraping pipelines with built-in retry logic and format conversion

🔄 **API Development**
   Use as a backend service for search and content extraction APIs

⚡ **High-Performance Search**
   Leverage API providers for faster, more reliable search results

🛡️ **Enterprise Search Solutions**
   Deploy with proxy support and multiple API providers for enterprise environments

Support
=======

- **Documentation**: https://tarzirs.readthedocs.io/
- **Source Code**: https://github.com/mirasoth/tarzi
- **Issues**: https://github.com/mirasoth/tarzi/issues
- **PyPI**: https://pypi.org/project/tarzi/
- **Crates.io**: https://crates.io/crates/tarzi

License
=======

This project is licensed under the Apache License 2.0 - see the `LICENSE <https://github.com/mirasoth/tarzi/blob/main/LICENSE>`_ file for details.

Indices and tables
==================

* :ref:`genindex`
* :ref:`modindex`
* :ref:`search` 