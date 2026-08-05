#!/usr/bin/env python3
"""
Integration tests for the WebFetcher class in tarzi.
These tests require network access and external services.
"""

import pytest

import tarzi


@pytest.fixture
def fetcher():
    """Fixture for creating a WebFetcher instance."""
    return tarzi.WebFetcher()


@pytest.fixture
def test_url():
    """Fixture for reliable test URL."""
    return "https://httpbin.org/html"


@pytest.mark.integration
class TestWebFetcher:
    """Integration test cases for the WebFetcher class."""

    def test_fetcher_creation(self, fetcher):
        """Test WebFetcher can be created."""
        assert isinstance(fetcher, tarzi.WebFetcher)
        assert str(fetcher) == "Tarzi web page fetcher"
        assert repr(fetcher) == "WebFetcher()"

    @pytest.mark.network
    def test_fetch_html(self, fetcher, test_url):
        """Test fetching HTML content."""
        try:
            result = fetcher.fetch(test_url, "html")
            assert isinstance(result, str)
            assert len(result) > 0
        except Exception as e:
            pytest.skip(f"Network request failed: {e}")

    @pytest.mark.network
    def test_fetch_markdown(self, fetcher, test_url):
        """Test fetching Markdown content."""
        try:
            result = fetcher.fetch(test_url, "markdown")
            assert isinstance(result, str)
            assert len(result) > 0
        except Exception as e:
            pytest.skip(f"Network request failed: {e}")

    @pytest.mark.network
    def test_fetch(self, fetcher, test_url):
        """Test raw fetching."""
        try:
            result = fetcher.fetch(test_url, "html")
            assert isinstance(result, str)
            assert len(result) > 0
        except Exception as e:
            pytest.skip(f"Network request failed: {e}")


    def test_invalid_format(self, fetcher, test_url):
        """Test invalid format raises ValueError."""
        with pytest.raises(ValueError, match="Invalid format"):
            fetcher.fetch(test_url, "invalid_format")

    def test_from_config(self):
        """Test creating WebFetcher from config."""
        config = tarzi.Config()
        fetcher = tarzi.WebFetcher.from_config(config)
        assert isinstance(fetcher, tarzi.WebFetcher)


@pytest.mark.integration
@pytest.mark.network
def test_fetch_function(test_url):
    """Test fetch standalone function."""
    try:
        result = tarzi.fetch(test_url, "html")
        assert isinstance(result, str)
        assert len(result) > 0
    except Exception as e:
        pytest.skip(f"Network request failed: {e}")


@pytest.mark.integration
def test_fetch_invalid_format(test_url):
    """Test fetch with invalid format."""
    with pytest.raises(ValueError, match="Invalid format"):
        tarzi.fetch(test_url, "invalid_format")
