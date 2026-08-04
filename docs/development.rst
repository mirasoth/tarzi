Development Guide
=================

This guide covers development setup, building from source, and contributing to tarzi.

Architecture Overview
=====================

Parser System
~~~~~~~~~~~~~

Tarzi search uses:

- **BaseParser** trait for HTML/JSON result parsing
- **ParserFactory** to select an engine-specific parser
- **search::access** to resolve API → plain HTTP → browser
- **search::api** for Brave and Serper REST clients

To add a new search engine:

1. Create a new parser file (e.g., ``src/search/parser/newengine.rs``)
2. Implement ``BaseParser``
3. Add the parser to ``ParserFactory::get_parser()``
4. Update ``SearchEngineType`` and query patterns in ``constants.rs``
5. Declare ``supports_api`` / ``supports_web`` / ``is_api_only`` behavior on the engine type
6. If the engine has an official API, add a client under ``src/search/api/`` and wire it into
   ``resolve_access`` / ``SearchEngine::search_via_api``
7. Add unit coverage in the engine×mode matrix (``src/search/access.rs``) and examples/docs

Development Setup
-----------------

Prerequisites
~~~~~~~~~~~~~

- Rust stable toolchain
- Python 3.10 or higher
- Git
- Cargo and uv / pip

Clone and Setup
~~~~~~~~~~~~~~~

.. code-block:: bash

   git clone https://github.com/mirasoth/tarzi.git
   cd tarzi

   cargo build
   make install-dev

Building from Source
--------------------

Rust Library
~~~~~~~~~~~~

.. code-block:: bash

   cargo build
   cargo build --release
   cargo test

Python Bindings
~~~~~~~~~~~~~~~

.. code-block:: bash

   maturin build --release
   maturin develop --release

CLI Tool
~~~~~~~~

.. code-block:: bash

   cargo build --release --bin tarzi
   cargo install --path .

Testing
-------

Prefer Makefile targets:

.. code-block:: bash

   make test-unit          # Rust + Python unit tests
   make test-integration   # Rust + Python integration tests
   make check              # format + lint

Rust Tests
~~~~~~~~~~

.. code-block:: bash

   # Library unit tests (includes access cascade matrix)
   cargo test --lib

   # Search mode integration (all engines × modes; network soft-fails)
   cargo test --test search_mode_integration_tests

   # Run with output
   cargo test -- --nocapture

Python Tests
~~~~~~~~~~~~

.. code-block:: bash

   # Unit tests (includes tests/python/test_search_modes.py)
   make test-unit-python

   # Integration (includes search_mode_integration_test.py)
   make test-integration-python

   # Focused
   uv run pytest tests/python/test_search_modes.py -q
   uv run pytest tests/python/integration/search_mode_integration_test.py -q

Documentation
-------------

Building Docs
~~~~~~~~~~~~~

.. code-block:: bash

   # Install documentation dependencies
   pip install -r ../docs/requirements.txt

   # Build documentation
   cd ../docs
   make html

   # View documentation
   open _build/html/index.html

   # Build all formats
   make all

Development Workflow
--------------------

1. **Feature Development**
   .. code-block:: bash

      # Create feature branch
      git checkout -b feature/new-feature

      # Make changes and test
      cargo test
      pytest tarzi/tests/python/

      # Build and test Python bindings
      maturin develop --release

2. **Documentation Updates**
   .. code-block:: bash

      # Update documentation
      cd ../docs
      make html
      # Check generated docs

3. **Testing Changes**
   .. code-block:: bash

      # Run full test suite
      cargo test
      pytest tarzi/tests/python/
      cargo clippy
      cargo fmt --check

4. **Commit and Push**
   .. code-block:: bash

      git add .
      git commit -m "feat: add new feature"
      git push origin feature/new-feature

Code Style
----------

Rust
~~~~~

- Follow Rust formatting: ``cargo fmt``
- Use clippy for linting: ``cargo clippy``
- Document public APIs with doc comments
- Use meaningful variable and function names

Python
~~~~~~~

- Follow PEP 8 style guide
- Use type hints for function parameters
- Document functions with docstrings
- Use meaningful variable names

Contributing
------------

1. **Fork the repository**
2. **Create a feature branch**
3. **Make your changes**
4. **Add tests for new functionality**
5. **Update documentation**
6. **Run the full test suite**
7. **Submit a pull request**

Issue Reporting
---------------

When reporting issues, please include:

- Operating system and version
- Rust/Python versions
- Steps to reproduce
- Expected vs actual behavior
- Error messages and stack traces

Release Process
---------------

1. **Update version numbers**
   - ``Cargo.toml``
   - ``pyproject.toml``
   - ``docs/conf.py``

2. **Update changelog**
   - Add new features and fixes
   - Note breaking changes

3. **Build and test**
   .. code-block:: bash

      cargo build --release
      maturin build --release
      cargo test
      pytest tarzi/tests/python/

4. **Create release**
   - Tag the release
   - Upload to crates.io and PyPI
   - Update documentation