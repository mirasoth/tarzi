Configuration
=============

.. important::
   **Breaking change:** File-based configuration (``tarzi.toml``, ``~/.tarzi.toml``,
   and ``Config.from_file`` / ``Config::from_file``) has been **removed**.
   Configure tarzi with environment variables, CLI flags, or programmatic
   ``Config`` construction (``Config::load()``, field assignment, or Python
   ``Config.from_str``). See `.env.example <https://github.com/mirasoth/tarzi/blob/main/.env.example>`_.

tarzi supports only Linux and macOS. Windows is not supported.

Configuration Precedence
------------------------

Configuration values are applied in the following order (highest to lowest priority):

1. **CLI parameters** (command line arguments)
2. **Environment variables** (``TARZI_*``, plus proxy and search API keys)
3. **Default values** (hardcoded defaults)

**Note**: The Python CLI is available as `pytarzi` command, while the Rust CLI remains as `tarzi` command.

Environment Variables
---------------------

Tarzi-specific settings use the ``TARZI_`` prefix. Proxy and engine API keys keep their standard names.
Tarzi has **no** product API key; use engine keys only (``BRAVE_API_KEY``, ``SERPER_API_KEY``).

.. code-block:: bash

   # Proxy (standard environment variables; win over TARZI_PROXY at use time)
   export HTTPS_PROXY=http://proxy.example.com:8080
   export HTTP_PROXY=http://proxy.example.com:8080

   # Search API keys (engine-specific)
   export BRAVE_API_KEY=your-brave-api-key
   export SERPER_API_KEY=your-serper-api-key

   # Tarzi settings
   export TARZI_SEARCH_ENGINE=brave
   export TARZI_SEARCH_BROWSER=true
   export TARZI_SEARCH_LIMIT=10
   export TARZI_FETCHER_BROWSER=true
   export TARZI_FETCHER_FORMAT=markdown
   export TARZI_USER_AGENT="Mozilla/5.0 ..."
   export TARZI_FETCHER_TIMEOUT=30
   export TARZI_PROXY=http://127.0.0.1:7890
   export TARZI_WEB_DRIVER=chromedriver
   export TARZI_WEB_DRIVER_URL=http://localhost:4444
   export TARZI_LOG_LEVEL=info
   export TARZI_TIMEOUT=30
   export TARZI_QUERY_PATTERN=https://example.com/search?q={query}

``Config::load()`` / ``tarzi.Config.load()`` apply ``TARZI_*`` over defaults.
``HTTP(S)_PROXY``, ``BRAVE_API_KEY``, and ``SERPER_API_KEY`` are resolved at use time and still take precedence over values stored on the config object.

Migrating from ``tarzi.toml``
----------------------------

.. list-table::
   :header-rows: 1
   :widths: 40 60

   * - Old (file)
     - New (environment)
   * - ``[search] engine``
     - ``TARZI_SEARCH_ENGINE`` (comma-separated list supported)
   * - ``[search] browser``
     - ``TARZI_SEARCH_BROWSER``
   * - ``[search] limit``
     - ``TARZI_SEARCH_LIMIT``
   * - ``[search] query_pattern``
     - ``TARZI_QUERY_PATTERN``
   * - ``[search] mode`` (removed)
     - N/A — cascade is always on; use ``TARZI_SEARCH_BROWSER``
   * - ``[search] api_key``
     - ``BRAVE_API_KEY`` / ``SERPER_API_KEY`` / ``TAVILY_API_KEY`` / ``GEMINI_API_KEY``
   * - ``[fetcher] mode`` (removed)
     - N/A — cascade is always on; use ``TARZI_FETCHER_BROWSER``
   * - ``[fetcher] browser``
     - ``TARZI_FETCHER_BROWSER``
   * - ``[fetcher] format``
     - ``TARZI_FETCHER_FORMAT``
   * - ``[fetcher] user_agent``
     - ``TARZI_USER_AGENT``
   * - ``[fetcher] timeout``
     - ``TARZI_FETCHER_TIMEOUT``
   * - ``[fetcher] proxy``
     - ``TARZI_PROXY`` or ``HTTP(S)_PROXY``
   * - ``[fetcher] web_driver``
     - ``TARZI_WEB_DRIVER``
   * - ``[fetcher] web_driver_url``
     - ``TARZI_WEB_DRIVER_URL``
   * - ``[general] log_level``
     - ``TARZI_LOG_LEVEL``
   * - ``[general] timeout``
     - ``TARZI_TIMEOUT``

Delete any project ``tarzi.toml`` / ``~/.tarzi.toml``; they are ignored.

Programmatic Configuration
--------------------------

Python
~~~~~~

.. code-block:: python

   import tarzi

   # Load from environment + defaults
   config = tarzi.Config.load()

   # Or create from TOML string (in-memory only; not a config file path)
   config_str = """
   [fetcher]
   timeout = 60
   format = "json"

   [search]
   engine = "brave"
   browser = true
   limit = 5
   """
   config = tarzi.Config.from_str(config_str)

   # Use with components
   fetcher = tarzi.WebFetcher.from_config(config)
   search_engine = tarzi.SearchEngine.from_config(config)

Rust
~~~~

.. code-block:: rust

   use tarzi::{config::Config, WebFetcher, SearchEngine};

   // Load from environment + defaults
   let config = Config::load()?;

   // Or create programmatically
   let mut config = Config::default();
   config.fetcher.timeout = 60;
   config.search.engine = "brave".to_string();
   config.search.browser = true;

   // Use with components
   let fetcher = WebFetcher::from_config(&config);
   let search_engine = SearchEngine::from_config(&config);

Search Engines and Access Cascade
---------------------------------

``search.engine`` / ``TARZI_SEARCH_ENGINE`` selects the provider, or a
comma-separated **ordered failover list**. Default is ``duckduckgo,bing,brave``.

``search.browser`` / ``TARZI_SEARCH_BROWSER`` (default ``true``) controls whether
browser automation may be used as the last search access step.

**Access cascade** (always):

1. **API** — if the engine supports it and credentials are present (env probed first; no request if missing)
2. **Plain HTTP** — public SERP URL
3. **Browser** — only when ``search.browser`` is true

API-only engines without credentials are skipped in a multi-engine list (or error
when they are the only engine).

**Supported engines:**

.. list-table::
   :header-rows: 1
   :widths: 22 18 18 42

   * - Engine id
     - Web Query
     - API Query
     - API Key / Host
   * - ``bing``
     - Yes
     - No
     - N/A (Bing Search API retired)
   * - ``google``
     - Yes
     - No
     - N/A (use ``google_serper`` for API)
   * - ``google_serper`` / ``serper``
     - No
     - Yes
     - Yes (``SERPER_API_KEY``)
   * - ``brave``
     - Yes
     - Yes
     - Yes for API (``BRAVE_API_KEY``)
   * - ``duckduckgo``
     - Yes
     - No
     - N/A
   * - ``baidu``
     - Yes
     - No
     - N/A
   * - ``sogou_weixin``
     - Yes
     - No
     - N/A
   * - ``tavily``
     - No
     - Yes
     - Yes (``TAVILY_API_KEY``)
   * - ``googleai`` / ``google_ai``
     - No
     - Yes
     - Yes (``GEMINI_API_KEY``)
   * - ``searxng``
     - No
     - Yes
     - Host (``SEARX_HOST`` or ``search.base_url``)

``google`` is HTML web query only. Google API results go through ``google_serper`` (no CSE).
``tavily``, ``googleai``, and ``searxng`` are API-only (no HTML SERP fallback).

**Configuration examples:**

.. code-block:: bash

   # Brave with cascade (API when BRAVE_API_KEY is set)
   export TARZI_SEARCH_ENGINE=brave
   export TARZI_SEARCH_BROWSER=true
   export TARZI_SEARCH_LIMIT=10
   export BRAVE_API_KEY=your-brave-api-key

.. code-block:: bash

   # Serper API (skipped automatically if SERPER_API_KEY is unset in a multi-engine list)
   export TARZI_SEARCH_ENGINE=google_serper
   export SERPER_API_KEY=your-serper-api-key

.. code-block:: bash

   # Ordered multi-engine failover (API-only engines without keys are skipped)
   export TARZI_SEARCH_ENGINE=google_serper,brave,duckduckgo,bing
   export TARZI_SEARCH_BROWSER=false

.. code-block:: bash

   # Tavily API
   export TARZI_SEARCH_ENGINE=tavily
   export TAVILY_API_KEY=your-tavily-api-key

.. code-block:: bash

   # Gemini grounded search
   export TARZI_SEARCH_ENGINE=googleai
   export GEMINI_API_KEY=your-gemini-api-key

.. code-block:: bash

   # Self-hosted SearxNG
   export TARZI_SEARCH_ENGINE=searxng
   export SEARX_HOST=http://localhost:8080

.. code-block:: bash

   # Web engines without browser fallback (plain HTTP only)
   export TARZI_SEARCH_ENGINE=duckduckgo
   export TARZI_SEARCH_BROWSER=false
   export TARZI_SEARCH_LIMIT=5
