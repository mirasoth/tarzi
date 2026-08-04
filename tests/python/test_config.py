#!/usr/bin/env python3
"""
Unit tests for the Config class in tarzi.
"""

import pytest

import tarzi


@pytest.fixture
def config():
    """Fixture for creating a Config instance."""
    return tarzi.Config()


@pytest.fixture
def sample_config():
    """Fixture for sample configuration string."""
    return """
[fetcher]
timeout = 30
user_agent = "Test Agent"
format = "html"
proxy = "http://proxy.example.com:8080"

[search]
engine = "brave"
"""


@pytest.mark.unit
class TestConfig:
    """Test cases for the Config class."""

    def test_config_creation(self, config):
        """Test Config can be created."""
        assert isinstance(config, tarzi.Config)
        assert str(config) == "Tarzi configuration"
        assert repr(config) == "Config()"

    def test_config_from_str(self, sample_config):
        """Test creating Config from string."""
        config = tarzi.Config.from_str(sample_config)
        assert isinstance(config, tarzi.Config)

    def test_config_from_str_invalid(self):
        """Test invalid config string raises RuntimeError."""
        with pytest.raises(RuntimeError, match="Failed to parse config"):
            tarzi.Config.from_str("invalid toml content")

    def test_config_load(self):
        """Test Config.load() returns a config from env/defaults."""
        config = tarzi.Config.load()
        assert isinstance(config, tarzi.Config)


@pytest.mark.unit
class TestConfigIntegration:
    """Test using config with different components."""

    def test_config_with_components(self, sample_config):
        """Test using config with different components."""
        config = tarzi.Config.from_str(sample_config)

        # Test with converter
        converter = tarzi.Converter.from_config(config)
        assert isinstance(converter, tarzi.Converter)

        # Test with fetcher
        fetcher = tarzi.WebFetcher.from_config(config)
        assert isinstance(fetcher, tarzi.WebFetcher)

        # Test with search engine
        engine = tarzi.SearchEngine.from_config(config)
        assert isinstance(engine, tarzi.SearchEngine)
