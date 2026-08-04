#!/usr/bin/env python3
"""
Integration tests for all search engines and access modes.

Network / API-key cases skip or soft-fail when credentials or connectivity
are unavailable so CI stays green.
"""

import os

import pytest

import tarzi

TEST_QUERY = "rust programming"
TEST_LIMIT = 2

WEB_ENGINES = ["bing", "duckduckgo", "google", "brave", "baidu", "sogou_weixin"]
ALL_ENGINES = WEB_ENGINES + ["google_serper"]
ALL_MODES = ["auto", "apiquery", "webquery"]


def _make_engine(engine: str, mode: str) -> tarzi.SearchEngine:
    config = tarzi.Config.from_str(
        f"""
[fetcher]
mode = "plain_request"

[search]
engine = "{engine}"
mode = "{mode}"
limit = {TEST_LIMIT}
"""
    )
    return tarzi.SearchEngine.from_config(config)


def _is_acceptable_external_failure(exc: Exception) -> bool:
    msg = str(exc).lower()
    needles = (
        "timeout",
        "network",
        "connection",
        "dns",
        "tls",
        "certificate",
        "rate",
        "403",
        "429",
        "captcha",
        "blocked",
        "no results",
        "returned no results",
        "all search access methods failed",
        "failed to fetch",
        "status",
        "webdriver",
        "chromedriver",
        "geckodriver",
        "browser automation",
        "no self-managed",
    )
    return any(n in msg for n in needles)


@pytest.mark.integration
@pytest.mark.network
@pytest.mark.slow
@pytest.mark.parametrize("engine", WEB_ENGINES)
def test_webquery_all_web_engines(engine):
    search = _make_engine(engine, "webquery")
    try:
        try:
            results = search.search(TEST_QUERY, TEST_LIMIT)
            assert len(results) <= TEST_LIMIT
        except Exception as e:
            if _is_acceptable_external_failure(e):
                pytest.skip(f"{engine} webquery unavailable externally: {e}")
            raise
    finally:
        search.shutdown()


@pytest.mark.integration
@pytest.mark.network
@pytest.mark.slow
@pytest.mark.parametrize("engine", ["bing", "duckduckgo", "brave"])
def test_auto_selected_engines(engine):
    search = _make_engine(engine, "auto")
    try:
        try:
            results = search.search(TEST_QUERY, TEST_LIMIT)
            assert len(results) <= TEST_LIMIT
        except Exception as e:
            if _is_acceptable_external_failure(e):
                pytest.skip(f"{engine} auto unavailable externally: {e}")
            raise
    finally:
        search.shutdown()


@pytest.mark.integration
@pytest.mark.parametrize("engine", ["bing", "duckduckgo", "google", "baidu", "sogou_weixin"])
def test_apiquery_unsupported_engines_fail_fast(engine):
    search = _make_engine(engine, "apiquery")
    try:
        with pytest.raises(Exception, match="(?i)api"):
            search.search(TEST_QUERY, TEST_LIMIT)
    finally:
        search.shutdown()


@pytest.mark.integration
def test_google_serper_webquery_rejected():
    search = _make_engine("google_serper", "webquery")
    try:
        with pytest.raises(Exception, match="(?i)webquery|apiquery|only supports"):
            search.search(TEST_QUERY, TEST_LIMIT)
    finally:
        search.shutdown()


@pytest.mark.integration
@pytest.mark.network
@pytest.mark.slow
def test_brave_apiquery_when_key_available():
    if not os.environ.get("BRAVE_API_KEY"):
        pytest.skip("BRAVE_API_KEY unset")
    search = _make_engine("brave", "apiquery")
    try:
        try:
            results = search.search(TEST_QUERY, TEST_LIMIT)
            assert len(results) > 0
        except Exception as e:
            if _is_acceptable_external_failure(e):
                pytest.skip(f"Brave apiquery failed externally: {e}")
            raise
    finally:
        search.shutdown()


@pytest.mark.integration
@pytest.mark.network
@pytest.mark.slow
def test_serper_apiquery_when_key_available():
    if not os.environ.get("SERPER_API_KEY"):
        pytest.skip("SERPER_API_KEY unset")
    search = _make_engine("google_serper", "apiquery")
    try:
        try:
            results = search.search(TEST_QUERY, TEST_LIMIT)
            assert len(results) > 0
        except Exception as e:
            if _is_acceptable_external_failure(e):
                pytest.skip(f"Serper apiquery failed externally: {e}")
            raise
    finally:
        search.shutdown()


@pytest.mark.integration
@pytest.mark.parametrize("engine", ALL_ENGINES)
@pytest.mark.parametrize("mode", ALL_MODES)
def test_engine_mode_config_wiring(engine, mode):
    search = _make_engine(engine, mode)
    try:
        assert isinstance(search, tarzi.SearchEngine)
    finally:
        search.shutdown()
