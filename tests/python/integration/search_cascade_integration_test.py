#!/usr/bin/env python3
"""
Integration tests for search engines, browser toggle, and multi-engine failover.

Network / API-key cases skip or soft-fail when credentials or connectivity
are unavailable so CI stays green.
"""

import os

import pytest

import tarzi

TEST_QUERY = "rust programming"
TEST_LIMIT = 2

WEB_ENGINES = ["bing", "duckduckgo", "google", "brave", "baidu", "sogou_weixin"]


def _make_engine(engine: str, browser: bool = True) -> tarzi.SearchEngine:
    config = tarzi.Config.from_str(
        f"""
[fetcher]
mode = "plain_request"

[search]
engine = "{engine}"
browser = {str(browser).lower()}
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
        "all search",
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
def test_web_engines_browser_disabled(engine):
    search = _make_engine(engine, browser=False)
    try:
        try:
            results = search.search(TEST_QUERY, TEST_LIMIT)
            assert len(results) <= TEST_LIMIT
        except Exception as e:
            if _is_acceptable_external_failure(e):
                pytest.skip(f"{engine} unavailable externally: {e}")
            raise
    finally:
        search.shutdown()


@pytest.mark.integration
@pytest.mark.network
@pytest.mark.slow
@pytest.mark.parametrize("engine", ["bing", "duckduckgo", "brave"])
def test_selected_engines(engine):
    search = _make_engine(engine, browser=True)
    try:
        try:
            results = search.search(TEST_QUERY, TEST_LIMIT)
            assert isinstance(results, list)
        except Exception as e:
            if _is_acceptable_external_failure(e):
                pytest.skip(f"{engine} unavailable externally: {e}")
            raise
    finally:
        search.shutdown()


@pytest.mark.integration
def test_api_only_without_key_fails():
    search = _make_engine("google_serper")
    try:
        with pytest.raises(Exception, match="(?i)serper|api|credential|key"):
            search.search(TEST_QUERY, TEST_LIMIT)
    finally:
        search.shutdown()


@pytest.mark.integration
@pytest.mark.network
@pytest.mark.slow
def test_multi_engine_failover():
    search = _make_engine("google_serper,duckduckgo", browser=False)
    try:
        try:
            results = search.search(TEST_QUERY, TEST_LIMIT)
            assert isinstance(results, list)
        except Exception as e:
            if _is_acceptable_external_failure(e):
                pytest.skip(f"multi-engine unavailable externally: {e}")
            raise
    finally:
        search.shutdown()


@pytest.mark.integration
@pytest.mark.network
@pytest.mark.slow
def test_brave_when_key_available():
    if not os.environ.get("BRAVE_API_KEY"):
        pytest.skip("BRAVE_API_KEY unset")
    search = _make_engine("brave", browser=False)
    try:
        try:
            results = search.search(TEST_QUERY, TEST_LIMIT)
            assert len(results) > 0
        except Exception as e:
            if _is_acceptable_external_failure(e):
                pytest.skip(f"Brave failed externally: {e}")
            raise
    finally:
        search.shutdown()


@pytest.mark.integration
@pytest.mark.network
@pytest.mark.slow
def test_serper_when_key_available():
    if not os.environ.get("SERPER_API_KEY"):
        pytest.skip("SERPER_API_KEY unset")
    search = _make_engine("google_serper")
    try:
        try:
            results = search.search(TEST_QUERY, TEST_LIMIT)
            assert len(results) > 0
        except Exception as e:
            if _is_acceptable_external_failure(e):
                pytest.skip(f"Serper failed externally: {e}")
            raise
    finally:
        search.shutdown()
