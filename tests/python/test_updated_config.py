#!/usr/bin/env python3
"""
Unit tests for updated Config structure in tarzi.
"""

import pytest

import tarzi


@pytest.fixture
def modern_config():
    """Fixture for modern configuration with specific API keys."""
    return """
[general]
log_level = "debug"
timeout = 60

[fetcher]
mode = "browser_headless"
format = "markdown"
timeout = 30
proxy = "http://proxy.example.com:8080"

[search]
engine = "google"
limit = 5
"""


@pytest.fixture
def minimal_config():
    """Fixture for minimal configuration."""
    return """
[search]
engine = "duckduckgo"
"""


@pytest.mark.unit
class TestUpdatedConfig:
    """Test cases for updated configuration structure."""

    def test_proxy_configuration_integration(self):
        """Test proxy configuration works with all components."""
        config_str = """
[fetcher]
proxy = "http://proxy.company.com:8080"
timeout = 45

[search]
engine = "brave"
"""
        config = tarzi.Config.from_str(config_str)
        assert isinstance(config, tarzi.Config)

        # All components should handle proxy configuration
        fetcher = tarzi.WebFetcher.from_config(config)
        assert isinstance(fetcher, tarzi.WebFetcher)

        engine = tarzi.SearchEngine.from_config(config)
        assert isinstance(engine, tarzi.SearchEngine)

    def test_web_driver_configuration(self):
        """Test web driver configuration options."""
        driver_configs = [
            ('web_driver = "geckodriver"', None),
            ('web_driver = "chromedriver"', None),
            ('web_driver = "geckodriver"\nweb_driver_url = "http://selenium-hub:4444"', "http://selenium-hub:4444"),
        ]

        for driver_config, _expected_url in driver_configs:
            config_str = f"""
[fetcher]
{driver_config}

[search]
engine = "duckduckgo"
"""
            config = tarzi.Config.from_str(config_str)
            assert isinstance(config, tarzi.Config)

            # Should be able to create fetcher with driver config
            fetcher = tarzi.WebFetcher.from_config(config)
            assert isinstance(fetcher, tarzi.WebFetcher)

    def test_search_engine_options(self):
        """Test different search engine configuration options."""
        engines = [
            "duckduckgo",
            "brave",
            "bing",
            "google",
            "google_serper",
            "serper",
            "baidu",
            "sogou_weixin",
            "tavily",
            "googleai",
            "google_ai",
            "searxng",
        ]

        for engine in engines:
            config_str = f"""
[search]
engine = "{engine}"
"""
            config = tarzi.Config.from_str(config_str)
            assert isinstance(config, tarzi.Config)

            # Should be able to create search engine
            search_engine = tarzi.SearchEngine.from_config(config)
            assert isinstance(search_engine, tarzi.SearchEngine)

    def test_search_mode_options(self):
        """Test different search access mode configuration options."""
        modes = ["auto", "apiquery", "webquery"]

        for mode in modes:
            config_str = f"""
[search]
engine = "brave"
mode = "{mode}"
"""
            config = tarzi.Config.from_str(config_str)
            assert isinstance(config, tarzi.Config)
            search_engine = tarzi.SearchEngine.from_config(config)
            assert isinstance(search_engine, tarzi.SearchEngine)

    def test_fetcher_mode_options(self):
        """Test different fetcher mode configuration options."""
        modes = ["plain_request", "browser_headless", "browser_head"]

        for mode in modes:
            config_str = f"""
[fetcher]
mode = "{mode}"

[search]
engine = "duckduckgo"
"""
            config = tarzi.Config.from_str(config_str)
            assert isinstance(config, tarzi.Config)

            # Should be able to create fetcher
            fetcher = tarzi.WebFetcher.from_config(config)
            assert isinstance(fetcher, tarzi.WebFetcher)

    def test_format_options(self):
        """Test different format configuration options."""
        formats = ["html", "markdown", "json", "yaml"]

        for format_type in formats:
            config_str = f"""
[fetcher]
format = "{format_type}"

[search]
engine = "duckduckgo"
"""
            config = tarzi.Config.from_str(config_str)
            assert isinstance(config, tarzi.Config)

            # Should be able to create components
            fetcher = tarzi.WebFetcher.from_config(config)
            assert isinstance(fetcher, tarzi.WebFetcher)

            converter = tarzi.Converter.from_config(config)
            assert isinstance(converter, tarzi.Converter)

    def test_timeout_configurations(self):
        """Test timeout configuration options."""
        config_str = """
[general]
timeout = 120

[fetcher]
timeout = 60

[search]
engine = "duckduckgo"
"""
        config = tarzi.Config.from_str(config_str)
        assert isinstance(config, tarzi.Config)

        # Components should handle timeout configuration
        fetcher = tarzi.WebFetcher.from_config(config)
        assert isinstance(fetcher, tarzi.WebFetcher)

        engine = tarzi.SearchEngine.from_config(config)
        assert isinstance(engine, tarzi.SearchEngine)

    def test_search_limit_configuration(self):
        """Test search limit configuration."""
        limits = [1, 5, 10, 20, 50]

        for limit in limits:
            config_str = f"""
[search]
engine = "duckduckgo"
limit = {limit}
"""
            config = tarzi.Config.from_str(config_str)
            assert isinstance(config, tarzi.Config)

            # Should be able to create search engine
            engine = tarzi.SearchEngine.from_config(config)
            assert isinstance(engine, tarzi.SearchEngine)

    def test_invalid_configuration_handling(self):
        """Test handling of invalid configuration values."""
        invalid_configs = [
            # Invalid limit should handle gracefully or error appropriately
            """
[search]
engine = "duckduckgo"
limit = -1
""",
            # Invalid timeout should handle gracefully
            """
[general]
timeout = -5

[search]
engine = "duckduckgo"
""",
        ]

        for config_str in invalid_configs:
            try:
                config = tarzi.Config.from_str(config_str)
                # Configuration parsing might succeed even with invalid values
                # (validation happens at runtime)
                assert isinstance(config, tarzi.Config)

                # Try to create components - might succeed or fail
                engine = tarzi.SearchEngine.from_config(config)
                assert isinstance(engine, tarzi.SearchEngine)

            except Exception:
                # Invalid configs may cause failures, which is acceptable
                pass

    def test_empty_configuration_defaults(self):
        """Test that empty configuration uses appropriate defaults."""
        config_str = ""
        config = tarzi.Config.from_str(config_str)
        assert isinstance(config, tarzi.Config)

        # Should be able to create components with default configuration
        fetcher = tarzi.WebFetcher.from_config(config)
        assert isinstance(fetcher, tarzi.WebFetcher)

        engine = tarzi.SearchEngine.from_config(config)
        assert isinstance(engine, tarzi.SearchEngine)

        converter = tarzi.Converter.from_config(config)
        assert isinstance(converter, tarzi.Converter)

    def test_partial_configuration_sections(self):
        """Test configuration with only some sections defined."""
        partial_configs = [
            # Only general section
            """
[general]
log_level = "debug"
""",
            # Only fetcher section
            """
[fetcher]
mode = "plain_request"
""",
            # Only search section
            """
[search]
engine = "brave"
""",
        ]

        for config_str in partial_configs:
            config = tarzi.Config.from_str(config_str)
            assert isinstance(config, tarzi.Config)

            # Should be able to create components even with partial config
            fetcher = tarzi.WebFetcher.from_config(config)
            assert isinstance(fetcher, tarzi.WebFetcher)

            engine = tarzi.SearchEngine.from_config(config)
            assert isinstance(engine, tarzi.SearchEngine)

            converter = tarzi.Converter.from_config(config)
            assert isinstance(converter, tarzi.Converter)
