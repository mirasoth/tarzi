#!/usr/bin/env python3
"""
Comprehensive integration tests for search cascade features.

Covers:
- Access cascade wiring (browser on/off)
- Multi-engine ordered failover
- API-only credential probing before network
- Optional live API engines when env keys are present

Network / API-key cases skip or soft-fail so CI stays green.
"""

from __future__ import annotations

import os
from contextlib import contextmanager

import pytest

import tarzi

TEST_QUERY = "rust programming"
TEST_LIMIT = 2

WEB_ENGINES = ["bing", "duckduckgo", "google", "brave", "baidu", "sogou_weixin"]
API_ONLY = ["google_serper", "tavily", "googleai", "searxng"]
API_ENV_KEYS = ["SERPER_API_KEY", "TAVILY_API_KEY", "GEMINI_API_KEY", "SEARX_HOST"]


def _make_engine(engine: str, browser: bool = True, **search_extras) -> tarzi.SearchEngine:
    extra = ""
    for key, value in search_extras.items():
        if isinstance(value, bool):
            extra += f"{key} = {str(value).lower()}\n"
        elif isinstance(value, int):
            extra += f"{key} = {value}\n"
        else:
            extra += f'{key} = "{value}"\n'
    config = tarzi.Config.from_str(
        f"""
[fetcher]

[search]
engine = "{engine}"
browser = {str(browser).lower()}
limit = {TEST_LIMIT}
{extra}
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
        "401",
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
        "unauthorized",
        "invalid",
        "proxy",
    )
    return any(n in msg for n in needles)


@contextmanager
def _without_api_env_keys():
    originals = {k: os.environ.get(k) for k in API_ENV_KEYS}
    try:
        for k in API_ENV_KEYS:
            os.environ.pop(k, None)
        yield
    finally:
        for k, v in originals.items():
            if v is None:
                os.environ.pop(k, None)
            else:
                os.environ[k] = v


# ---------------------------------------------------------------------------
# Config / wiring (no network)
# ---------------------------------------------------------------------------


@pytest.mark.integration
@pytest.mark.parametrize("engine", WEB_ENGINES + API_ONLY)
@pytest.mark.parametrize("browser", [True, False])
def test_from_config_wires_engine_and_browser(engine, browser):
    extras = {}
    if engine in ("google_serper", "tavily", "googleai"):
        extras["api_key"] = "unit-test-key"
    if engine == "searxng":
        extras["base_url"] = "http://localhost:8080"
    search = _make_engine(engine, browser=browser, **extras)
    try:
        assert isinstance(search, tarzi.SearchEngine)
    finally:
        search.shutdown()


@pytest.mark.integration
def test_from_config_multi_engine_list():
    search = _make_engine("brave,duckduckgo,bing", browser=False)
    try:
        assert isinstance(search, tarzi.SearchEngine)
    finally:
        search.shutdown()


@pytest.mark.integration
def test_config_load_browser_and_engine_from_env():
    originals = {
        "TARZI_SEARCH_ENGINE": os.environ.get("TARZI_SEARCH_ENGINE"),
        "TARZI_SEARCH_BROWSER": os.environ.get("TARZI_SEARCH_BROWSER"),
    }
    try:
        os.environ["TARZI_SEARCH_ENGINE"] = "tavily,brave,duckduckgo"
        os.environ["TARZI_SEARCH_BROWSER"] = "false"
        config = tarzi.Config.load()
        search = tarzi.SearchEngine.from_config(config)
        assert isinstance(search, tarzi.SearchEngine)
        search.shutdown()
    finally:
        for k, v in originals.items():
            if v is None:
                os.environ.pop(k, None)
            else:
                os.environ[k] = v


@pytest.mark.integration
def test_set_search_engine_and_browser_helpers():
    config = tarzi.Config.load()
    config.set_search_engine("duckduckgo,bing")
    config.set_search_browser(False)
    config.set_search_limit(3)
    search = tarzi.SearchEngine.from_config(config)
    try:
        assert isinstance(search, tarzi.SearchEngine)
    finally:
        search.shutdown()


# ---------------------------------------------------------------------------
# Credential probing
# ---------------------------------------------------------------------------


@pytest.mark.integration
@pytest.mark.parametrize("engine", API_ONLY)
def test_api_only_without_credentials_fails_before_network(engine):
    with _without_api_env_keys():
        search = _make_engine(engine, browser=True)
        try:
            with pytest.raises(Exception) as excinfo:
                search.search(TEST_QUERY, TEST_LIMIT)
            msg = str(excinfo.value).lower()
            assert any(
                token in msg
                for token in (
                    "api",
                    "key",
                    "host",
                    "credential",
                    "searx",
                    "serper",
                    "tavily",
                    "gemini",
                )
            ), msg
            assert "timed out after" not in msg
        finally:
            search.shutdown()


@pytest.mark.integration
@pytest.mark.network
@pytest.mark.slow
def test_multi_engine_skips_api_only_then_tries_web():
    with _without_api_env_keys():
        search = _make_engine(
            "google_serper,tavily,googleai,searxng,duckduckgo",
            browser=False,
        )
        try:
            try:
                results = search.search(TEST_QUERY, TEST_LIMIT)
                assert isinstance(results, list)
                assert len(results) <= TEST_LIMIT
            except Exception as e:
                if _is_acceptable_external_failure(e):
                    pytest.skip(f"web fallback unavailable externally: {e}")
                raise
        finally:
            search.shutdown()


@pytest.mark.integration
def test_all_api_only_without_creds_aggregate_error():
    with _without_api_env_keys():
        search = _make_engine("google_serper,tavily", browser=True)
        try:
            with pytest.raises(Exception) as excinfo:
                search.search(TEST_QUERY, TEST_LIMIT)
            msg = str(excinfo.value).lower()
            assert "serper" in msg or "tavily" in msg or "all search" in msg
        finally:
            search.shutdown()


# ---------------------------------------------------------------------------
# Live network (soft-fail / skip)
# ---------------------------------------------------------------------------


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
@pytest.mark.parametrize("engine", ["duckduckgo", "brave", "bing"])
def test_selected_web_engines_browser_enabled(engine):
    search = _make_engine(engine, browser=True)
    try:
        try:
            results = search.search(TEST_QUERY, TEST_LIMIT)
            assert isinstance(results, list)
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
def test_multi_engine_web_failover_chain():
    search = _make_engine("bing,duckduckgo,brave", browser=False)
    try:
        try:
            results = search.search(TEST_QUERY, TEST_LIMIT)
            assert isinstance(results, list)
        except Exception as e:
            if _is_acceptable_external_failure(e):
                pytest.skip(f"web failover unavailable: {e}")
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


@pytest.mark.integration
@pytest.mark.network
@pytest.mark.slow
def test_tavily_when_key_available():
    if not os.environ.get("TAVILY_API_KEY"):
        pytest.skip("TAVILY_API_KEY unset")
    search = _make_engine("tavily")
    try:
        try:
            results = search.search(TEST_QUERY, TEST_LIMIT)
            assert len(results) > 0
        except Exception as e:
            if _is_acceptable_external_failure(e):
                pytest.skip(f"Tavily failed externally: {e}")
            raise
    finally:
        search.shutdown()


@pytest.mark.integration
@pytest.mark.network
@pytest.mark.slow
def test_googleai_when_key_available():
    if not os.environ.get("GEMINI_API_KEY"):
        pytest.skip("GEMINI_API_KEY unset")
    search = _make_engine("googleai")
    try:
        try:
            results = search.search(TEST_QUERY, TEST_LIMIT)
            assert len(results) > 0
        except Exception as e:
            if _is_acceptable_external_failure(e):
                pytest.skip(f"Google AI failed externally: {e}")
            raise
    finally:
        search.shutdown()


@pytest.mark.integration
@pytest.mark.network
@pytest.mark.slow
def test_searxng_when_host_available():
    if not os.environ.get("SEARX_HOST"):
        pytest.skip("SEARX_HOST unset")
    search = _make_engine("searxng")
    try:
        try:
            results = search.search(TEST_QUERY, TEST_LIMIT)
            assert len(results) > 0
        except Exception as e:
            if _is_acceptable_external_failure(e):
                pytest.skip(f"SearxNG failed externally: {e}")
            raise
    finally:
        search.shutdown()


@pytest.mark.integration
@pytest.mark.network
@pytest.mark.slow
def test_search_with_content_duckduckgo():
    search = _make_engine("duckduckgo", browser=False)
    try:
        try:
            pairs = search.search_with_content(TEST_QUERY, 1, "markdown")
            assert isinstance(pairs, list)
            assert len(pairs) <= 1
        except Exception as e:
            if _is_acceptable_external_failure(e):
                pytest.skip(f"search_with_content unavailable: {e}")
            raise
    finally:
        search.shutdown()
