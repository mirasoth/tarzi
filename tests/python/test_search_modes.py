#!/usr/bin/env python3
"""Unit tests for search engines and access modes."""

import pytest

import tarzi

ALL_ENGINES = [
    "bing",
    "duckduckgo",
    "google",
    "google_serper",
    "serper",
    "brave",
    "baidu",
    "sogou_weixin",
]

ALL_MODES = ["auto", "apiquery", "webquery"]

WEB_ONLY_ENGINES = ["bing", "duckduckgo", "google", "baidu", "sogou_weixin"]
API_ENGINES = ["brave", "google_serper"]


@pytest.mark.unit
class TestSearchEnginesAndModes:
    """Config and SearchEngine wiring for every engine × mode."""

    @pytest.mark.parametrize("engine", ALL_ENGINES)
    @pytest.mark.parametrize("mode", ALL_MODES)
    def test_config_accepts_engine_and_mode(self, engine, mode):
        config = tarzi.Config.from_str(
            f"""
[search]
engine = "{engine}"
mode = "{mode}"
limit = 3
"""
        )
        assert isinstance(config, tarzi.Config)
        search = tarzi.SearchEngine.from_config(config)
        assert isinstance(search, tarzi.SearchEngine)

    @pytest.mark.parametrize("mode", ALL_MODES)
    def test_search_mode_roundtrip(self, mode):
        config = tarzi.Config.from_str(
            f"""
[search]
engine = "brave"
mode = "{mode}"
"""
        )
        engine = tarzi.SearchEngine.from_config(config)
        assert isinstance(engine, tarzi.SearchEngine)

    def test_default_mode_is_auto(self):
        config = tarzi.Config.from_str(
            """
[search]
engine = "bing"
"""
        )
        engine = tarzi.SearchEngine.from_config(config)
        assert isinstance(engine, tarzi.SearchEngine)

    def test_api_key_in_config(self):
        config = tarzi.Config.from_str(
            """
[search]
engine = "brave"
mode = "apiquery"
api_key = "unit-test-key"
"""
        )
        engine = tarzi.SearchEngine.from_config(config)
        assert isinstance(engine, tarzi.SearchEngine)

    @pytest.mark.parametrize("engine", WEB_ONLY_ENGINES)
    def test_apiquery_web_only_engines_fail_fast(self, engine):
        config = tarzi.Config.from_str(
            f"""
[search]
engine = "{engine}"
mode = "apiquery"
api_key = "unused"
"""
        )
        search = tarzi.SearchEngine.from_config(config)
        with pytest.raises(Exception, match="(?i)api"):
            search.search("rust", 1)
        search.shutdown()

    def test_google_serper_webquery_rejected(self):
        config = tarzi.Config.from_str(
            """
[search]
engine = "google_serper"
mode = "webquery"
api_key = "unused"
"""
        )
        search = tarzi.SearchEngine.from_config(config)
        with pytest.raises(Exception, match="(?i)webquery|apiquery|only supports"):
            search.search("rust", 1)
        search.shutdown()

    def test_google_serper_without_key_fails(self, monkeypatch):
        monkeypatch.delenv("SERPER_API_KEY", raising=False)
        config = tarzi.Config.from_str(
            """
[search]
engine = "google_serper"
mode = "auto"
"""
        )
        search = tarzi.SearchEngine.from_config(config)
        with pytest.raises(Exception, match="(?i)serper|api key"):
            search.search("rust", 1)
        search.shutdown()
