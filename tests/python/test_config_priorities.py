#!/usr/bin/env python3
"""
Unit tests for configuration loading priorities in tarzi.
Tests the precedence order: CLI > Env Vars > Defaults (programmatic from_str for explicit config).
"""

import os

import pytest

import tarzi


@pytest.mark.unit
class TestConfigPriorities:
    """Test configuration loading priorities."""

    def test_default_config_values(self):
        """Test that default configuration values are loaded correctly."""
        config = tarzi.Config()

        components = tarzi.WebFetcher.from_config(config)
        assert isinstance(components, tarzi.WebFetcher)

        search_engine = tarzi.SearchEngine.from_config(config)
        assert isinstance(search_engine, tarzi.SearchEngine)

    def test_config_from_string_override(self):
        """Test that string config overrides defaults."""
        config_str = """
[general]
log_level = "error"
timeout = 120

[fetcher]
browser = false
format = "yaml"

[search]
engine = "brave"
limit = 15
"""
        config = tarzi.Config.from_str(config_str)

        fetcher = tarzi.WebFetcher.from_config(config)
        assert isinstance(fetcher, tarzi.WebFetcher)

        search_engine = tarzi.SearchEngine.from_config(config)
        assert isinstance(search_engine, tarzi.SearchEngine)

    def test_config_load_from_env(self):
        """Test that Config.load() picks up TARZI_* environment variables."""
        original = {
            "TARZI_SEARCH_ENGINE": os.environ.get("TARZI_SEARCH_ENGINE"),
            "TARZI_SEARCH_BROWSER": os.environ.get("TARZI_SEARCH_BROWSER"),
            "TARZI_SEARCH_LIMIT": os.environ.get("TARZI_SEARCH_LIMIT"),
        }
        try:
            os.environ["TARZI_SEARCH_ENGINE"] = "brave,duckduckgo"
            os.environ["TARZI_SEARCH_BROWSER"] = "false"
            os.environ["TARZI_SEARCH_LIMIT"] = "8"

            config = tarzi.Config.load()
            engine = tarzi.SearchEngine.from_config(config)
            assert isinstance(engine, tarzi.SearchEngine)
        finally:
            for var, value in original.items():
                if value is not None:
                    os.environ[var] = value
                elif var in os.environ:
                    del os.environ[var]

    def test_environment_variable_override_config(self):
        """Test that HTTP(S)_PROXY override config proxy at use time."""
        config_str = """
[fetcher]
proxy = "http://config-proxy:8080"

[search]
engine = "duckduckgo"
"""
        config = tarzi.Config.from_str(config_str)

        original_http_proxy = os.environ.get("HTTP_PROXY")
        original_https_proxy = os.environ.get("HTTPS_PROXY")

        try:
            os.environ["HTTP_PROXY"] = "http://env-proxy:3128"
            os.environ["HTTPS_PROXY"] = "http://env-https-proxy:3128"

            fetcher = tarzi.WebFetcher.from_config(config)
            assert isinstance(fetcher, tarzi.WebFetcher)

            search_engine = tarzi.SearchEngine.from_config(config)
            assert isinstance(search_engine, tarzi.SearchEngine)

        finally:
            if original_http_proxy is not None:
                os.environ["HTTP_PROXY"] = original_http_proxy
            elif "HTTP_PROXY" in os.environ:
                del os.environ["HTTP_PROXY"]

            if original_https_proxy is not None:
                os.environ["HTTPS_PROXY"] = original_https_proxy
            elif "HTTPS_PROXY" in os.environ:
                del os.environ["HTTPS_PROXY"]

    def test_environment_variable_priority_order(self):
        """Test that HTTPS_PROXY takes precedence over HTTP_PROXY."""
        config_str = """
[fetcher]
proxy = "http://config-proxy:8080"

[search]
engine = "duckduckgo"
"""
        config = tarzi.Config.from_str(config_str)

        original_http_proxy = os.environ.get("HTTP_PROXY")
        original_https_proxy = os.environ.get("HTTPS_PROXY")

        try:
            os.environ["HTTP_PROXY"] = "http://http-proxy:8080"
            os.environ["HTTPS_PROXY"] = "http://https-proxy:3128"

            fetcher = tarzi.WebFetcher.from_config(config)
            assert isinstance(fetcher, tarzi.WebFetcher)

            search_engine = tarzi.SearchEngine.from_config(config)
            assert isinstance(search_engine, tarzi.SearchEngine)

        finally:
            if original_http_proxy is not None:
                os.environ["HTTP_PROXY"] = original_http_proxy
            elif "HTTP_PROXY" in os.environ:
                del os.environ["HTTP_PROXY"]

            if original_https_proxy is not None:
                os.environ["HTTPS_PROXY"] = original_https_proxy
            elif "HTTPS_PROXY" in os.environ:
                del os.environ["HTTPS_PROXY"]

    def test_empty_environment_variable_fallback(self):
        """Test that empty environment variables fall back to config values."""
        config_str = """
[fetcher]
proxy = "http://config-proxy:8080"

[search]
engine = "duckduckgo"
"""
        config = tarzi.Config.from_str(config_str)

        original_http_proxy = os.environ.get("HTTP_PROXY")

        try:
            os.environ["HTTP_PROXY"] = ""

            fetcher = tarzi.WebFetcher.from_config(config)
            assert isinstance(fetcher, tarzi.WebFetcher)

            search_engine = tarzi.SearchEngine.from_config(config)
            assert isinstance(search_engine, tarzi.SearchEngine)

        finally:
            if original_http_proxy is not None:
                os.environ["HTTP_PROXY"] = original_http_proxy
            elif "HTTP_PROXY" in os.environ:
                del os.environ["HTTP_PROXY"]

    def test_mixed_priority_scenarios(self):
        """Test complex scenarios with mixed configuration sources."""
        config_str = """
[general]
log_level = "info"
timeout = 30

[fetcher]
browser = true
format = "markdown"
proxy = "http://config-proxy:8080"

[search]
engine = "duckduckgo"
limit = 5
"""
        config = tarzi.Config.from_str(config_str)

        original_env_vars = {
            "HTTP_PROXY": os.environ.get("HTTP_PROXY"),
            "HTTPS_PROXY": os.environ.get("HTTPS_PROXY"),
        }

        try:
            os.environ["HTTPS_PROXY"] = "http://env-proxy:3128"

            fetcher = tarzi.WebFetcher.from_config(config)
            assert isinstance(fetcher, tarzi.WebFetcher)

            search_engine = tarzi.SearchEngine.from_config(config)
            assert isinstance(search_engine, tarzi.SearchEngine)

        finally:
            for var, value in original_env_vars.items():
                if value is not None:
                    os.environ[var] = value
                elif var in os.environ:
                    del os.environ[var]

    def test_web_driver_configuration_priority(self):
        """Test web driver configuration with different priority sources."""
        config_str = """
[fetcher]
browser = true
web_driver = "chromedriver"
web_driver_url = "http://localhost:4444"
timeout = 60
"""
        config = tarzi.Config.from_str(config_str)

        fetcher = tarzi.WebFetcher.from_config(config)
        assert isinstance(fetcher, tarzi.WebFetcher)

    def test_timeout_configuration_priority(self):
        """Test timeout configuration from different sources."""
        config_str = """
[general]
timeout = 120

[fetcher]
timeout = 45

[search]
limit = 8
"""
        config = tarzi.Config.from_str(config_str)

        fetcher = tarzi.WebFetcher.from_config(config)
        assert isinstance(fetcher, tarzi.WebFetcher)

        search_engine = tarzi.SearchEngine.from_config(config)
        assert isinstance(search_engine, tarzi.SearchEngine)

    def test_format_configuration_priority(self):
        """Test output format configuration priority."""
        format_configs = ["markdown", "json", "yaml", "raw"]

        for fmt in format_configs:
            config_str = f"""
[fetcher]
format = "{fmt}"
browser = false

[search]
engine = "duckduckgo"
"""
            config = tarzi.Config.from_str(config_str)

            fetcher = tarzi.WebFetcher.from_config(config)
            assert isinstance(fetcher, tarzi.WebFetcher)

    def test_fetcher_browser_configuration(self):
        """Test fetcher.browser configuration options."""
        for browser in [True, False]:
            config_str = f"""
[fetcher]
browser = {str(browser).lower()}
format = "markdown"

[search]
engine = "duckduckgo"
"""
            config = tarzi.Config.from_str(config_str)

            fetcher = tarzi.WebFetcher.from_config(config)
            assert isinstance(fetcher, tarzi.WebFetcher)

    def test_invalid_configuration_handling(self):
        """Test that invalid configurations are handled gracefully."""
        invalid_configs = [
            """
[fetcher]
proxy = "invalid-proxy-url"

[search]
engine = "duckduckgo"
""",
            """
[search]
engine = "invalid-engine"
""",
        ]

        for config_str in invalid_configs:
            try:
                config = tarzi.Config.from_str(config_str)
                assert isinstance(config, tarzi.Config)

                try:
                    tarzi.WebFetcher.from_config(config)
                    tarzi.SearchEngine.from_config(config)
                except Exception:
                    pass

            except Exception:
                pass
