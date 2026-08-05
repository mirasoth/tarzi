#!/usr/bin/env python3
"""
Integration tests asserting that blocking calls release the GIL.

These use a local HTTP server only: no external network, no WebDriver. A binding
that blocks while holding the GIL freezes every other Python thread in the
embedding process, which is fatal for hosts that run an event loop alongside
tarzi calls.
"""

import asyncio
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

import pytest

import tarzi

RESPONSE_DELAY_SEC = 1.5
TICK_INTERVAL_SEC = 0.05


class _SlowHandler(BaseHTTPRequestHandler):
    """Answer after a delay so the fetch spends real time blocked on I/O."""

    protocol_version = "HTTP/1.1"

    def do_GET(self):  # noqa: N802  # BaseHTTPRequestHandler API
        time.sleep(RESPONSE_DELAY_SEC)
        body = b"<html><body>slow</body></html>"
        self.send_response(200)
        self.send_header("Content-Type", "text/html")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def log_message(self, format, *args):  # noqa: A002  # BaseHTTPRequestHandler API
        pass


@pytest.fixture
def slow_server():
    """Run a delaying HTTP server for the duration of a test."""
    server = ThreadingHTTPServer(("127.0.0.1", 0), _SlowHandler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    try:
        host, port = server.server_address[:2]
        yield f"http://{host}:{port}/"
    finally:
        server.shutdown()
        server.server_close()
        thread.join(timeout=5)


def _count_event_loop_ticks(call, timeout_sec=30.0):
    """Return (ticks, call_duration) for `call` running on a worker thread.

    Ticks are 50ms asyncio sleeps completed on the main thread while the worker
    is inside the native call. A GIL-holding binding yields zero ticks.
    """
    done = threading.Event()
    duration = []

    def worker():
        started = time.monotonic()
        try:
            call()
        finally:
            duration.append(time.monotonic() - started)
            done.set()

    async def probe():
        ticks = 0
        threading.Thread(target=worker, daemon=True).start()
        deadline = time.monotonic() + timeout_sec
        while not done.is_set() and time.monotonic() < deadline:
            await asyncio.sleep(TICK_INTERVAL_SEC)
            ticks += 1
        return ticks

    ticks = asyncio.run(probe())
    done.wait(timeout=timeout_sec)
    return ticks, (duration[0] if duration else 0.0)


@pytest.mark.integration
class TestGilRelease:
    """The GIL must be released around blocking fetch and convert calls."""

    def test_fetch_raw_releases_gil(self, slow_server):
        """Other Python threads keep running while a fetch is in flight."""
        fetcher = tarzi.WebFetcher()

        ticks, duration = _count_event_loop_ticks(
            lambda: fetcher.fetch_raw(slow_server)
        )

        assert duration >= RESPONSE_DELAY_SEC, f"fetch returned too early ({duration:.2f}s)"
        # A cooperative binding yields ~duration/interval ticks; allow wide slack
        # for slow CI, but zero ticks means the GIL was held for the whole call.
        expected_ticks = duration / TICK_INTERVAL_SEC
        assert ticks >= expected_ticks / 4, (
            f"main thread starved: {ticks} ticks in {duration:.2f}s "
            f"(expected ~{expected_ticks:.0f}); GIL held across blocking call"
        )

    def test_repeated_calls_share_one_runtime(self, slow_server):
        """Sequential calls succeed against the shared runtime."""
        fetcher = tarzi.WebFetcher()

        first = fetcher.fetch_raw(slow_server)
        second = fetcher.fetch_raw(slow_server)

        assert "slow" in first
        assert "slow" in second
