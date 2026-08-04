Configuration
=============

tarzi can be configured through configuration files, environment variables, and programmatic configuration.

.. note::
   tarzi supports only Linux and macOS. Windows is not supported.

Configuration File
------------------

tarzi reads configuration from the following sources in order of precedence (highest to lowest):

1. **CLI parameters** (highest priority)
2. **~/.tarzi.toml** (user home directory)
3. **tarzi.toml** (current project root)
4. **Default values** (lowest priority)

You can refer to `tarzi.toml <https://github.com/mirasoth/tarzi/blob/main/tarzi.toml>`_ for the default values.

**Note**: The Python CLI is available as `pytarzi` command, while the Rust CLI remains as `tarzi` command.

Environment Variables
---------------------

Currently supported environment variables:

.. code-block:: bash

   # Proxy configuration (standard environment variables)
   export http_proxy=http://proxy.example.com:8080
   export https_proxy=http://proxy.example.com:8080

   # Debug mode (for development/testing)
   export TARZI_DEBUG=1

Programmatic Configuration
--------------------------

Python
~~~~~~

.. code-block:: python

   import tarzi

   # Load from file
   config = tarzi.Config.from_file("tarzi.toml")

   # Create from string
   config_str = """
   [fetcher]
   timeout = 60
   format = "json"
   """
   config = tarzi.Config.from_str(config_str)

   # Use with components
   fetcher = tarzi.WebFetcher.from_config(config)
   search_engine = tarzi.SearchEngine.from_config(config)

Rust
~~~~

.. code-block:: rust

   use tarzi::{Config, WebFetcher, SearchEngine};

   // Load from file
   let config = Config::from_file("tarzi.toml")?;

   // Create programmatically
   let mut config = Config::default();
   config.fetcher.timeout = 60;
   config.fetcher.format = Format::Json;

   // Use with components
   let fetcher = WebFetcher::from_config(&config);
   let search_engine = SearchEngine::from_config(&config);

Configuration Precedence
-------------------------

Configuration values are applied in the following order (highest to lowest priority):

1. **CLI parameters** (command line arguments)
2. **Environment variables** (limited support - see above)
3. **~/.tarzi.toml** (user configuration file)
4. **tarzi.toml** (project configuration file)
5. **Default values** (hardcoded defaults)

**Note**: Environment variables currently only override proxy settings and API keys. 
All other configuration must be set via TOML file, CLI parameters, or programmatically.

API Search Configuration
------------------------

tarzi supports multiple API search providers with automatic fallback capabilities:

**Supported API providers:**
- **Brave Search API**: REST API with ``BRAVE_API_KEY`` (or ``search.api_key``)
- **Google Serper**: Google results via Serper with ``SERPER_API_KEY`` (engine ``google_serper``)

**Access cascade** (``search.mode = "auto"``): API key if available → plain HTTP → headless browser.

**Engine Capabilities:**

.. list-table::
   :header-rows: 1
   :widths: 25 20 20 35

   * - Engine
     - Web Query
     - API Query
     - API Key
   * - Bing
     - Yes
     - No
     - N/A (Bing Search API retired)
   * - Google
     - Yes
     - No
     - N/A (use ``google_serper`` for API)
   * - Google Serper
     - No
     - Yes
     - Yes (``SERPER_API_KEY``)
   * - Brave
     - Yes
     - Yes
     - Yes for API (``BRAVE_API_KEY``)
   * - DuckDuckGo
     - Yes
     - No
     - N/A
   * - Baidu
     - Yes
     - No
     - N/A
   * - Sogou Weixin
     - Yes
     - No
     - N/A

**Configuration Example:**

.. code-block:: toml

   [search]
   engine = "brave"
   mode = "auto"          # auto | apiquery | webquery
   limit = 10
   api_key = "your-brave-api-key"
   # Prefer env vars: BRAVE_API_KEY / SERPER_API_KEY
