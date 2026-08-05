Python API Reference
====================

Complete reference for the tarzi Python API.

Quick Reference
---------------

**Core Functions**
   - :func:`tarzi.convert_html` - Convert HTML to various formats
   - :func:`tarzi.fetch` / :func:`tarzi.fetch_url` - Fetch web page content
   - :func:`tarzi.search_web` - Search the web
   - :func:`tarzi.search_with_content` - Search and fetch result pages

**Classes**
   - :class:`tarzi.Converter` - HTML conversion
   - :class:`tarzi.WebFetcher` - Web page fetching
   - :class:`tarzi.SearchEngine` - Web search
   - :class:`tarzi.Config` - Configuration management

**Data Types**
   - :class:`tarzi.SearchResult` - Search result data

Basic Usage
-----------

.. code-block:: python

   import tarzi

   # Convert HTML
   markdown = tarzi.convert_html("<h1>Hello</h1>", "markdown")

   # Fetch web page
   content = tarzi.fetch_url("https://example.com", mode="plain_request")

   # Search web (access mode comes from config; default auto cascade)
   results = tarzi.search_web("python programming", 10)

Search Engines and Modes
------------------------

``SearchEngine`` reads ``search.engine`` and ``search.mode`` from :class:`tarzi.Config`.

Supported engines: ``bing``, ``google``, ``google_serper`` (alias ``serper``), ``brave``,
``duckduckgo``, ``baidu``, ``sogou_weixin``, ``tavily``, ``googleai`` (alias ``google_ai``),
``searxng``.

Modes: ``auto`` (default), ``apiquery``, ``webquery``. See :doc:`/configuration`.

.. code-block:: python

   import tarzi

   # Brave cascade (API when BRAVE_API_KEY is set)
   config = tarzi.Config.from_str(
       """
   [search]
   engine = "brave"
   mode = "auto"
   limit = 5
   """
   )
   engine = tarzi.SearchEngine.from_config(config)
   results = engine.search("rust async", 5)

   # Force Serper API
   config = tarzi.Config.from_str(
       """
   [search]
   engine = "google_serper"
   mode = "apiquery"
   """
   )
   engine = tarzi.SearchEngine.from_config(config)
   results = engine.search("agentic AI", 5)

   # Tavily API
   config = tarzi.Config.from_str(
       """
   [search]
   engine = "tavily"
   mode = "apiquery"
   """
   )
   engine = tarzi.SearchEngine.from_config(config)
   results = engine.search("latest AI news", 5)

   # Web-only (never call APIs)
   config = tarzi.Config.from_str(
       """
   [search]
   engine = "duckduckgo"
   mode = "webquery"
   """
   )
   engine = tarzi.SearchEngine.from_config(config)
   results = engine.search("python packaging", 5)

Environment keys ``BRAVE_API_KEY``, ``SERPER_API_KEY``, ``TAVILY_API_KEY``, and
``GEMINI_API_KEY`` override ``search.api_key``. ``SEARX_HOST`` overrides
``search.base_url`` for ``searxng``.

More examples: :doc:`/examples/api_search` and ``examples/search_modes.py``.
