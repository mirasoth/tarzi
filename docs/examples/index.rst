Examples
========

Real-world examples and use cases for tarzi.

Getting Started
---------------

All examples are available in the ``examples/`` directory of the repository:

.. code-block:: bash

   git clone https://github.com/mirasoth/tarzi.git
   cd tarzi

   # Python examples
   python examples/basic_usage.py
   python examples/search_cascade.py
   python examples/search_engine_serper.py
   python examples/sogou_weixin_search.py

   # Rust examples
   cargo run --example basic_usage
   cargo run --example search_cascade
   cargo run --example search_engine_brave
   cargo run --example search_engine_serper
   cargo run --example search_engine_default

Example Catalog
---------------

**Search access cascade**
   ``search_cascade`` (Rust / Python) — browser toggle and multi-engine failover.
   Optional: ``BRAVE_API_KEY``, ``SERPER_API_KEY``.

**Brave cascade**
   ``search_engine_brave`` — Brave with API → HTTP → browser (API when keyed).

**Google via Serper**
   ``search_engine_serper`` (Rust / Python) — API-only ``google_serper`` (requires ``SERPER_API_KEY``).
   Requires ``SERPER_API_KEY``.

**Default engine**
   ``search_engine_default`` — uses env/defaults (Bing + ``auto``).

**Sogou Weixin**
   ``sogou_weixin_search`` — WeChat article search via Sogou.

**Basics**
   ``basic_usage``, ``simple_usage``, ``browser_driver_usage`` — fetch, convert, and search wiring.

Documentation Examples
----------------------

.. toctree::
   :maxdepth: 2

   api_search
