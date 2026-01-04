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

**Supported Providers:**
- **Brave Search API**: Fast, privacy-focused search results
- **Exa Search API**: AI-powered semantic search with enhanced relevance
- **Travily API**: Specialized travel and location-based search
- **DuckDuckGo API**: Privacy-focused search (limited functionality, no API key required)
- **Baidu API**: Chinese search engine with API support

**Engine Capabilities:**

| Engine        | Web Query | API Query | API Key Required |
|---------------|-----------|-----------|------------------|
| Bing          | Yes       | No        | N/A              |
| Google        | Yes       | No        | N/A              |
| Brave         | Yes       | Yes       | Yes              |
| DuckDuckGo    | Yes       | Yes       | No               |
| Exa           | Yes       | Yes       | Yes              |
| Travily       | No        | Yes       | Yes              |
| Baidu         | Yes       | Yes       | Yes              |

**Autoswitch Strategies:**
- **smart**: Automatically fallback to available providers if primary fails
- **none**: Only use the configured primary search engine

**Configuration Example:**

.. code-block:: toml

   [search]
   engine = "brave"
   mode = "apiquery"
   autoswitch = "smart"
   limit = 10
   
   # API keys for different providers
   brave_api_key = "your-brave-api-key"
   exa_api_key = "your-exa-api-key"
   travily_api_key = "your-travily-api-key"
   baidu_api_key = "your-baidu-api-key"
