Installation
============

.. note::
   tarzi supports only Linux and macOS. Windows is not supported.

tarzi is available as both a Python package and a Rust crate, with optional CLI tools.
Choose the installation method that best fits your use case.

Python Installation
-------------------

PyPI (Recommended)
~~~~~~~~~~~~~~~~~~

The easiest way to install tarzi for Python is via pip:

.. code-block:: bash

   pip install tarzi

This will install the pre-compiled Python wheel with all necessary dependencies.

From Source
~~~~~~~~~~~

If you need to build from source or want the latest development version:

.. code-block:: bash

   # Clone the repository
   git clone https://github.com/mirasoth/tarzi.git
   cd tarzi
   cd tarzi

   # Install Rust if not already installed
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env

   # Install maturin for building Python wheels
   pip install maturin

   # Build and install in development mode
   maturin develop --release

Virtual Environment (Recommended)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

It's recommended to use a virtual environment:

.. code-block:: bash

   # Using venv
   python -m venv tarzi-env
   source tarzi-env/bin/activate  # On Windows: tarzi-env\Scripts\activate
   pip install tarzi

   # Using conda
   conda create -n tarzi-env python=3.11
   conda activate tarzi-env
   pip install tarzi

Python Requirements
~~~~~~~~~~~~~~~~~~~

- Python 3.10 or higher
- Operating system: Linux, macOS

Optional dependencies for development:

.. code-block:: bash

   # Install with development dependencies
   pip install tarzi[dev]

   # Install with test dependencies
   pip install tarzi[test]

Rust Installation
-----------------

As a Rust Crate
~~~~~~~~~~~~~~~~

Add tarzi to your `tarzi/Cargo.toml`:

.. code-block:: toml

   [dependencies]
   tarzi = "0.0.11"
   tokio = { version = "1.0", features = ["full"] }

Or add it using cargo:

.. code-block:: bash

   cargo add tarzi

CLI Installation
~~~~~~~~~~~~~~~~

.. code-block:: bash

   # Build CLI
   cargo build --release --bin tarzi

   # Install CLI locally
   cargo install --path .

CLI Installation
~~~~~~~~~~~~~~~~

Install the command-line interface:

.. code-block:: bash

   cargo install tarzi

This will install the `tarzi` binary to your cargo bin directory.

From Source
~~~~~~~~~~~

.. code-block:: bash

   # Clone the repository
   git clone https://github.com/mirasoth/tarzi.git
   cd tarzi

   # Build the project
   cargo build --release

   # Install the CLI (optional)
   cargo install --path .

Rust Requirements
~~~~~~~~~~~~~~~~~

- Rust 1.70 or higher
- Cargo package manager

System Dependencies
-------------------

Browser Dependencies (Optional)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

For browser-based fetching, you'll need a WebDriver:

**Firefox** (Recommended):

.. tabs::

   .. tab:: Linux

      .. code-block:: bash

         # Ubuntu/Debian
         sudo apt-get install firefox

         # Download GeckoDriver
         wget https://github.com/mozilla/geckodriver/releases/latest/download/geckodriver-linux64.tar.gz
         tar -xzf geckodriver-linux64.tar.gz
         sudo mv geckodriver /usr/local/bin/

   .. tab:: macOS

      .. code-block:: bash

         # Using Homebrew
         brew install firefox
         brew install geckodriver

**Chrome/Chromium** (Alternative):

.. tabs::

   .. tab:: Linux

      .. code-block:: bash

         # Ubuntu/Debian
         sudo apt-get update
         sudo apt-get install -y chromium-browser

         # Download ChromeDriver
         wget https://chromedriver.storage.googleapis.com/LATEST_RELEASE_114/chromedriver_linux64.zip
         unzip chromedriver_linux64.zip
         sudo mv chromedriver /usr/local/bin/

   .. tab:: macOS

      .. code-block:: bash

         # Using Homebrew
         brew install chromium
         brew install chromedriver

Verification
------------

After installation, verify that tarzi is working correctly:

Python
~~~~~~~

.. code-block:: python

   import tarzi
   print(tarzi.__version__)

   # Test basic functionality
   html = "<h1>Test</h1>"
   result = tarzi.convert_html(html, "markdown")
   print(result)

Rust
~~~~

.. code-block:: rust

   use tarzi::Converter;

   #[tokio::main]
   async fn main() {
       let converter = Converter::new();
       let html = "<h1>Test</h1>";
       match converter.convert(html, tarzi::Format::Markdown).await {
           Ok(result) => println!("{}", result),
           Err(e) => eprintln!("Error: {}", e),
       }
   }

CLI
~~~

.. code-block:: bash

   tarzi --version
   tarzi convert --input "<h1>Test</h1>" --format markdown

Getting Help
~~~~~~~~~~~~

If you encounter issues:

1. Search existing `GitHub issues <https://github.com/mirasoth/tarzi/issues>`_
2. Create a new issue with detailed error information
3. Join our community discussions

Next Steps
----------

Now that you have tarzi installed, check out the :doc:`quickstart` guide to learn 
the basic usage patterns and start building your first application. 