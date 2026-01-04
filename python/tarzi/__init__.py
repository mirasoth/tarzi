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

__all__ = [
    "Config",
    "Converter",
    "WebFetcher",
    "SearchEngine",
    "SearchResult",
]
