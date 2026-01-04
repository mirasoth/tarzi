Development Guide
==================

This guide covers development setup, building from source, and contributing to tarzi.

Architecture Overview
====================

Parser System
~~~~~~~~~~~~

Tarzi uses a unified parser architecture for search engines:

- **Base Traits**: `BaseSearchParser`, `WebSearchParser`, `ApiSearchParser`
- **Base Structs**: `BaseWebParser`, `BaseApiParser` for common functionality
- **Parser Factory**: Mode-aware parser selection and management
- **Unified Parser**: Combines web and API parsing capabilities

To add a new search engine:

1. Create a new parser file (e.g., `src/search/parser/newengine.rs`)
2. Implement the appropriate base traits
3. Add the parser to `ParserFactory::get_parser()`
4. Update `SearchEngineType` enum if needed

Development Setup
-----------------

Prerequisites
~~~~~~~~~~~~~

- Rust 1.70 or higher
- Python 3.10 or higher
- Git
- Cargo and pip package managers

Clone and Setup
~~~~~~~~~~~~~~~

.. code-block:: bash

   # Clone the repository
   git clone https://github.com/mirasoth/tarzi.git
   cd tarzi
   cd tarzi

   # Install Rust dependencies
   cargo build

   # Install Python development dependencies
   pip install -e .[dev]

   # Install maturin for Python bindings
   pip install maturin

Building from Source
--------------------

Rust Library
~~~~~~~~~~~~

.. code-block:: bash

   # Build in debug mode
   cargo build

   # Build in release mode
   cargo build --release

   # Run tests
   cargo test

   # Run with specific features
   cargo build --features "full"

Python Bindings
~~~~~~~~~~~~~~~

.. code-block:: bash

   # Build Python wheel
   maturin build --release

   # Install in development mode
   maturin develop --release

   # Build for specific Python version
   maturin build --release --interpreter python3.11

CLI Tool
~~~~~~~~

.. code-block:: bash

   # Build CLI
   cargo build --release --bin tarzi

   # Install CLI locally
   cargo install --path .

Testing
-------

Rust Tests
~~~~~~~~~~

.. code-block:: bash

   # Run all tests
   cargo test

   # Run specific test
   cargo test test_name

   # Run integration tests
   cargo test --test integration_test_name

   # Run with output
   cargo test -- --nocapture

Python Tests
~~~~~~~~~~~~

.. code-block:: bash

   # Run Python tests
   pytest tarzi/tests/python/

   # Run with coverage
   pytest tarzi/tests/python/ --cov=tarzi

   # Run specific test file
   pytest tarzi/tests/python/unit/test_converter.py

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