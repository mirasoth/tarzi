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
