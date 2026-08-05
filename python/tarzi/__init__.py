# Re-export everything from the Rust module
from .tarzi import Config, Converter, SearchEngine, SearchResult, WebFetcher

# Get version dynamically
try:
    from importlib.metadata import version

    __version__ = version("tarzi")
except ImportError:
    # Fallback for older Python versions
    try:
        import pkg_resources

        __version__ = pkg_resources.get_distribution("tarzi").version
    except (ImportError, pkg_resources.DistributionNotFound):
        # Final fallback - read from pyproject.toml
        try:
            import tomllib
        except ImportError:
            import tomli as tomllib

        try:
            with open("pyproject.toml", "rb") as f:
                pyproject = tomllib.load(f)
                __version__ = pyproject["project"]["version"]
        except (FileNotFoundError, KeyError):
            # Last resort fallback
            __version__ = "unknown"


def convert_html(html: str, format: str = "markdown") -> str:
    """Convert HTML content to the given format."""
    return Converter().convert(html, format)


def fetch(url: str, format: str = "html") -> str:
    """Fetch a URL using WebFetcher (plain HTTP → browser cascade)."""
    return WebFetcher().fetch(url, format)


def fetch_url(url: str, format: str = "html") -> str:
    """Alias for :func:`fetch`."""
    return fetch(url, format)


def search_web(query: str, limit: int = 10):
    """Search the web using the default SearchEngine configuration."""
    engine = SearchEngine()
    try:
        return engine.search(query, limit)
    finally:
        engine.shutdown()


def search_with_content(
    query: str,
    limit: int = 5,
    format: str = "markdown",
):
    """Search and fetch content for each result (fetcher cascade)."""
    engine = SearchEngine()
    try:
        return engine.search_with_content(query, limit, format)
    finally:
        engine.shutdown()


__all__ = [
    "Config",
    "Converter",
    "WebFetcher",
    "SearchEngine",
    "SearchResult",
    "convert_html",
    "fetch",
    "fetch_url",
    "search_web",
    "search_with_content",
    "__version__",
]
