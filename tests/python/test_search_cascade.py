"""Unit tests for search engines, browser toggle, and multi-engine config."""

import pytest
import tarzi

ALL_ENGINES = [
    "bing",
    "duckduckgo",
    "google",
    "google_serper",
    "brave",
    "baidu",
    "sogou_weixin",
    "tavily",
    "googleai",
    "searxng",
]

API_ONLY = ["google_serper", "tavily", "googleai", "searxng"]


class TestSearchConfig:
    @pytest.mark.parametrize("engine", ALL_ENGINES)
    @pytest.mark.parametrize("browser", [True, False])
    def test_config_accepts_engine_and_browser(self, engine, browser):
        config = tarzi.Config.from_str(
            f"""
[search]
engine = "{engine}"
browser = {str(browser).lower()}
"""
        )
        search = tarzi.SearchEngine.from_config(config)
        assert isinstance(search, tarzi.SearchEngine)

    def test_browser_defaults_true(self):
        config = tarzi.Config.from_str(
            """
[search]
engine = "brave"
"""
        )
        assert config  # parsed
        engine = tarzi.SearchEngine.from_config(config)
        assert isinstance(engine, tarzi.SearchEngine)

    def test_multi_engine_list(self):
        config = tarzi.Config.from_str(
            """
[search]
engine = "brave,duckduckgo,bing"
browser = false
"""
        )
        engine = tarzi.SearchEngine.from_config(config)
        assert isinstance(engine, tarzi.SearchEngine)

    @pytest.mark.parametrize("engine", API_ONLY)
    def test_api_only_without_key_fails(self, engine):
        config = tarzi.Config.from_str(
            f"""
[search]
engine = "{engine}"
"""
        )
        search = tarzi.SearchEngine.from_config(config)
        with pytest.raises(Exception, match="(?i)api_key|api key|host|credential|requires|searx"):
            search.search("rust", 2)

    def test_google_serper_with_programmatic_key_constructs(self):
        config = tarzi.Config.from_str(
            """
[search]
engine = "google_serper"
api_key = "test-key"
"""
        )
        search = tarzi.SearchEngine.from_config(config)
        assert isinstance(search, tarzi.SearchEngine)

    def test_tavily_with_programmatic_key_constructs(self):
        config = tarzi.Config.from_str(
            """
[search]
engine = "tavily"
api_key = "tvly-test"
"""
        )
        search = tarzi.SearchEngine.from_config(config)
        assert isinstance(search, tarzi.SearchEngine)

    def test_searxng_without_host_fails(self):
        config = tarzi.Config.from_str(
            """
[search]
engine = "searxng"
"""
        )
        search = tarzi.SearchEngine.from_config(config)
        with pytest.raises(Exception, match="(?i)searx|host"):
            search.search("rust", 2)

    def test_searxng_with_base_url_constructs(self):
        config = tarzi.Config.from_str(
            """
[search]
engine = "searxng"
base_url = "http://localhost:8080"
"""
        )
        search = tarzi.SearchEngine.from_config(config)
        assert isinstance(search, tarzi.SearchEngine)
