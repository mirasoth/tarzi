#![allow(unsafe_op_in_unsafe_fn)]
#![allow(non_local_definitions)]
use crate::config::Config;
use crate::{Converter, FetchMode, Format, SearchEngine, WebFetcher};
use pyo3::prelude::*;
use pyo3::types::PyType;
use std::future::Future;
use std::str::FromStr;
use std::sync::OnceLock;
use toml;

/// Python module for tarzi - Rust-native lite search for AI applications
#[pymodule]
fn tarzi(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyConverter>()?;
    m.add_class::<PyWebFetcher>()?;
    m.add_class::<PySearchEngine>()?;
    m.add_class::<PySearchResult>()?;
    m.add_class::<PyConfig>()?;
    Ok(())
}

/// Tokio runtime shared by every Python entry point.
///
/// A per-call runtime pays thread-pool setup on every request and prevents the
/// HTTP client from reusing pooled connections between calls.
fn shared_runtime() -> PyResult<&'static tokio::runtime::Runtime> {
    static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

    if let Some(runtime) = RUNTIME.get() {
        return Ok(runtime);
    }

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                "Failed to create async runtime: {e}"
            ))
        })?;
    Ok(RUNTIME.get_or_init(|| runtime))
}

/// Drive `future` to completion on the shared runtime with the GIL released.
///
/// Every method below blocks on network or WebDriver I/O for seconds at a time.
/// Keeping the GIL for that long freezes *all* Python threads in the embedding
/// process — including event loops and watchdogs — so blocking sections must
/// always run through here.
fn block_on_without_gil<F>(py: Python<'_>, future: F) -> PyResult<F::Output>
where
    F: Future + Send,
    F::Output: Send,
{
    let runtime = shared_runtime()?;
    Ok(py.allow_threads(|| runtime.block_on(future)))
}

/// HTML/text content converter
#[pyclass(name = "Converter")]
#[derive(Clone)]
pub struct PyConverter {
    inner: Converter,
}

#[allow(non_local_definitions)]
#[pymethods]
impl PyConverter {
    /// Create a new converter with default settings
    ///
    /// Returns:
    ///     Converter: A new converter instance
    #[new]
    fn new() -> Self {
        Self {
            inner: Converter::new(),
        }
    }

    /// Create a converter from configuration
    ///
    /// Args:
    ///     config (Config): Configuration object
    ///     
    /// Returns:
    ///     Converter: A new converter instance
    #[classmethod]
    fn from_config(_cls: &Bound<'_, PyType>, _config: &PyConfig) -> PyResult<Self> {
        Ok(Self {
            inner: Converter::new(),
        })
    }

    /// Convert HTML/text content to the specified format
    ///
    /// Args:
    ///     input (str): Input HTML or text content
    ///     format (str): Output format ("html", "markdown", "json", "yaml")
    ///     
    /// Returns:
    ///     str: Converted content
    ///     
    /// Raises:
    ///     ValueError: If format is invalid
    ///     RuntimeError: If conversion fails
    fn convert(&self, py: Python<'_>, input: &str, format: &str) -> PyResult<String> {
        let parsed_format = Format::from_str(format).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Invalid format '{format}': {e}"
            ))
        })?;

        let input = input.to_owned();
        let converter = &self.inner;
        block_on_without_gil(
            py,
            async move { converter.convert(&input, parsed_format).await },
        )?
        .map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Conversion failed: {e}"))
        })
    }

    /// Convert content using custom configuration
    ///
    /// Args:
    ///     input (str): Input HTML or text content
    ///     config (Config): Configuration object
    ///     
    /// Returns:
    ///     str: Converted content
    ///     
    /// Raises:
    ///     RuntimeError: If conversion fails
    fn convert_with_config(
        &self,
        py: Python<'_>,
        input: &str,
        config: &PyConfig,
    ) -> PyResult<String> {
        let input = input.to_owned();
        let config = config.inner.clone();
        let converter = &self.inner;
        block_on_without_gil(py, async move {
            converter.convert_with_config(&input, &config).await
        })?
        .map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                "Conversion with config failed: {e}"
            ))
        })
    }

    fn __repr__(&self) -> String {
        "Converter()".to_string()
    }

    fn __str__(&self) -> String {
        "Tarzi HTML/text content converter".to_string()
    }
}

/// Web page fetcher with multiple modes
#[pyclass(name = "WebFetcher")]
pub struct PyWebFetcher {
    inner: WebFetcher,
}

#[allow(non_local_definitions)]
#[pymethods]
impl PyWebFetcher {
    /// Create a new web fetcher with default settings
    ///
    /// Returns:
    ///     WebFetcher: A new fetcher instance
    #[new]
    fn new() -> Self {
        Self {
            inner: WebFetcher::new(),
        }
    }

    /// Create a web fetcher from configuration
    ///
    /// Args:
    ///     config (Config): Configuration object
    ///     
    /// Returns:
    ///     WebFetcher: A new fetcher instance
    #[classmethod]
    fn from_config(_cls: &Bound<'_, PyType>, config: &PyConfig) -> PyResult<Self> {
        Ok(Self {
            inner: WebFetcher::from_config(&config.inner),
        })
    }

    /// Fetch a web page and convert to specified format
    ///
    /// Args:
    ///     url (str): URL to fetch
    ///     mode (str): Fetch mode ("plain_request", "browser_head", "browser_headless")
    ///     format (str): Output format ("html", "markdown", "json", "yaml")
    ///     
    /// Returns:
    ///     str: Fetched and converted content
    ///     
    /// Raises:
    ///     ValueError: If mode or format is invalid
    ///     RuntimeError: If fetching fails
    fn fetch(&mut self, py: Python<'_>, url: &str, mode: &str, format: &str) -> PyResult<String> {
        let parsed_mode = FetchMode::from_str(mode).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Invalid fetch mode '{mode}': {e}"
            ))
        })?;
        let parsed_format = Format::from_str(format).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Invalid format '{format}': {e}"
            ))
        })?;

        let owned_url = url.to_owned();
        let fetcher = &mut self.inner;
        block_on_without_gil(py, async move {
            fetcher.fetch(&owned_url, parsed_mode, parsed_format).await
        })?
        .map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                "Failed to fetch '{url}': {e}"
            ))
        })
    }

    /// Fetch raw HTML content from a web page
    ///
    /// Args:
    ///     url (str): URL to fetch
    ///     mode (str): Fetch mode ("plain_request", "browser_head", "browser_headless")
    ///     
    /// Returns:
    ///     str: Raw HTML content
    ///     
    /// Raises:
    ///     ValueError: If mode is invalid
    ///     RuntimeError: If fetching fails
    fn fetch_raw(&mut self, py: Python<'_>, url: &str, mode: &str) -> PyResult<String> {
        let parsed_mode = FetchMode::from_str(mode).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Invalid fetch mode '{mode}': {e}"
            ))
        })?;

        let owned_url = url.to_owned();
        let fetcher = &mut self.inner;
        block_on_without_gil(py, async move {
            fetcher.fetch_raw(&owned_url, parsed_mode).await
        })?
        .map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                "Failed to fetch raw content from '{url}': {e}"
            ))
        })
    }

    /// Fetch a web page through a proxy
    ///
    /// Args:
    ///     url (str): URL to fetch
    ///     proxy (str): Proxy URL (e.g., "http://proxy:port")
    ///     mode (str): Fetch mode ("plain_request", "browser_head", "browser_headless")
    ///     format (str): Output format ("html", "markdown", "json", "yaml")
    ///     
    /// Returns:
    ///     str: Fetched and converted content
    ///     
    /// Raises:
    ///     ValueError: If mode or format is invalid
    ///     RuntimeError: If fetching fails
    fn fetch_with_proxy(
        &mut self,
        py: Python<'_>,
        url: &str,
        proxy: &str,
        mode: &str,
        format: &str,
    ) -> PyResult<String> {
        let parsed_mode = FetchMode::from_str(mode).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Invalid fetch mode '{mode}': {e}"
            ))
        })?;
        let parsed_format = Format::from_str(format).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Invalid format '{format}': {e}"
            ))
        })?;

        let owned_url = url.to_owned();
        let owned_proxy = proxy.to_owned();
        let fetcher = &mut self.inner;
        block_on_without_gil(py, async move {
            fetcher
                .fetch_with_proxy(&owned_url, &owned_proxy, parsed_mode, parsed_format)
                .await
        })?
        .map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                "Failed to fetch '{url}' via proxy '{proxy}': {e}"
            ))
        })
    }

    fn __repr__(&self) -> String {
        "WebFetcher()".to_string()
    }

    fn __str__(&self) -> String {
        "Tarzi web page fetcher".to_string()
    }
}

/// Search engine with multiple providers and modes
#[pyclass(name = "SearchEngine")]
pub struct PySearchEngine {
    inner: SearchEngine,
}

#[allow(non_local_definitions)]
#[pymethods]
impl PySearchEngine {
    /// Create a new search engine with default settings
    ///
    /// Returns:
    ///     SearchEngine: A new search engine instance
    #[new]
    fn new() -> Self {
        // Use configuration loading with precedence to ensure proper defaults
        let config = crate::config::Config::load().unwrap_or_default();
        Self {
            inner: SearchEngine::from_config(&config),
        }
    }

    /// Create a search engine from configuration
    ///
    /// Args:
    ///     config (Config): Configuration object
    ///     
    /// Returns:
    ///     SearchEngine: A new search engine instance
    #[classmethod]
    fn from_config(_cls: &Bound<'_, PyType>, config: &PyConfig) -> PyResult<Self> {
        Ok(Self {
            inner: SearchEngine::from_config(&config.inner),
        })
    }

    /// Search for web pages
    ///
    /// Args:
    ///     query (str): Search query
    ///     limit (int): Maximum number of results
    ///     
    /// Returns:
    ///     List[SearchResult]: List of search results
    ///     
    /// Raises:
    ///     RuntimeError: If search fails
    fn search(
        &mut self,
        py: Python<'_>,
        query: &str,
        limit: usize,
    ) -> PyResult<Vec<PySearchResult>> {
        let owned_query = query.to_owned();
        let engine = &mut self.inner;
        block_on_without_gil(py, async move { engine.search(&owned_query, limit).await })?
            .map(|results| {
                results
                    .into_iter()
                    .map(|r| PySearchResult {
                        title: r.title,
                        url: r.url,
                        snippet: r.snippet,
                        rank: r.rank,
                    })
                    .collect()
            })
            .map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                    "Search failed for query '{query}': {e}"
                ))
            })
    }

    /// Search for web pages and fetch their content
    ///
    /// Args:
    ///     query (str): Search query
    ///     limit (int): Maximum number of results
    ///     fetch_mode (str): Fetch mode ("plain_request", "browser_head", "browser_headless")
    ///     format (str): Output format ("html", "markdown", "json", "yaml")
    ///     
    /// Returns:
    ///     List[Tuple[SearchResult, str]]: List of (result, content) pairs
    ///     
    /// Raises:
    ///     ValueError: If fetch_mode, or format is invalid
    ///     RuntimeError: If search or fetch fails
    fn search_with_content(
        &mut self,
        py: Python<'_>,
        query: &str,
        limit: usize,
        fetch_mode: &str,
        format: &str,
    ) -> PyResult<Vec<(PySearchResult, String)>> {
        let parsed_fetch_mode = FetchMode::from_str(fetch_mode).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Invalid fetch mode '{fetch_mode}': {e}"
            ))
        })?;
        let parsed_format = Format::from_str(format).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Invalid format '{format}': {e}"
            ))
        })?;

        let owned_query = query.to_owned();
        let engine = &mut self.inner;
        block_on_without_gil(py, async move {
            engine
                .search_with_content(&owned_query, limit, parsed_fetch_mode, parsed_format)
                .await
        })?
        .map(|results| {
            results
                .into_iter()
                .map(|(r, content)| {
                    (
                        PySearchResult {
                            title: r.title,
                            url: r.url,
                            snippet: r.snippet,
                            rank: r.rank,
                        },
                        content,
                    )
                })
                .collect()
        })
        .map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                "Search and fetch failed for query '{query}': {e}"
            ))
        })
    }

    /// Search using a proxy
    ///
    /// Args:
    ///     query (str): Search query
    ///     limit (int): Maximum number of results
    ///     proxy (str): Proxy URL (e.g., "http://proxy:port")
    ///     
    /// Returns:
    ///     List[SearchResult]: List of search results
    ///     
    /// Raises:
    ///     RuntimeError: If search fails
    fn search_with_proxy(
        &mut self,
        py: Python<'_>,
        query: &str,
        limit: usize,
        proxy: &str,
    ) -> PyResult<Vec<PySearchResult>> {
        let owned_query = query.to_owned();
        let owned_proxy = proxy.to_owned();
        let engine = &mut self.inner;
        block_on_without_gil(py, async move {
            engine
                .search_with_proxy(&owned_query, limit, &owned_proxy)
                .await
        })?
        .map(|results| {
            results
                .into_iter()
                .map(|r| PySearchResult {
                    title: r.title,
                    url: r.url,
                    snippet: r.snippet,
                    rank: r.rank,
                })
                .collect()
        })
        .map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                "Search with proxy failed for query '{query}': {e}"
            ))
        })
    }

    /// Shutdown browser and driver resources
    ///
    /// This method ensures proper cleanup of browser instances and WebDriver processes.
    /// It should be called when the search engine is no longer needed to free up system resources.
    ///
    /// Returns:
    ///     None
    ///     
    /// Raises:
    ///     RuntimeError: If shutdown fails
    fn shutdown(&mut self, py: Python<'_>) -> PyResult<()> {
        let engine = &mut self.inner;
        block_on_without_gil(py, async move { engine.shutdown().await })
    }

    fn __repr__(&self) -> String {
        "SearchEngine()".to_string()
    }

    fn __str__(&self) -> String {
        "Tarzi search engine".to_string()
    }
}

/// Search result with metadata
#[pyclass(name = "SearchResult")]
#[derive(Clone, Debug)]
pub struct PySearchResult {
    /// Page title
    #[pyo3(get)]
    pub title: String,
    /// Page URL
    #[pyo3(get)]
    pub url: String,
    /// Page snippet/description
    #[pyo3(get)]
    pub snippet: String,
    /// Search result rank (1-based)
    #[pyo3(get)]
    pub rank: usize,
}

#[pymethods]
impl PySearchResult {
    fn __repr__(&self) -> String {
        format!(
            "SearchResult(title='{}', url='{}', snippet='{}', rank={})",
            self.title, self.url, self.snippet, self.rank
        )
    }

    fn __str__(&self) -> String {
        format!(
            "[{}] {}\n{}\n{}",
            self.rank, self.title, self.url, self.snippet
        )
    }
}

/// Configuration management
#[pyclass(name = "Config")]
#[derive(Clone)]
pub struct PyConfig {
    inner: Config,
}

#[allow(non_local_definitions)]
#[pymethods]
impl PyConfig {
    /// Create a new configuration with default values (no env overlay)
    ///
    /// Returns:
    ///     Config: A new configuration instance
    #[new]
    fn new() -> Self {
        Self {
            inner: Config::new(),
        }
    }

    /// Load configuration from environment variables and defaults
    ///
    /// Returns:
    ///     Config: Configuration from env (`TARZI_*`) over defaults
    ///
    /// Raises:
    ///     RuntimeError: If an environment variable has an invalid value
    #[classmethod]
    fn load(_cls: &Bound<'_, PyType>) -> PyResult<Self> {
        let config = Config::load().map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                "Failed to load config from environment: {e}"
            ))
        })?;
        Ok(Self { inner: config })
    }

    /// Create configuration from TOML string (programmatic)
    ///
    /// Args:
    ///     content (str): TOML configuration content
    ///     
    /// Returns:
    ///     Config: Configuration parsed from string
    ///     
    /// Raises:
    ///     RuntimeError: If content cannot be parsed
    #[classmethod]
    fn from_str(_cls: &Bound<'_, PyType>, content: &str) -> PyResult<Self> {
        let config: Config = toml::from_str(content).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                "Failed to parse config: {e}"
            ))
        })?;
        Ok(Self { inner: config })
    }

    fn __repr__(&self) -> String {
        "Config()".to_string()
    }

    fn __str__(&self) -> String {
        "Tarzi configuration".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_python() {
        pyo3::prepare_freethreaded_python();
    }

    #[test]
    fn test_py_converter_new() {
        let converter = PyConverter::new();
        assert_eq!(converter.inner, Converter::new());
    }

    #[test]
    fn test_py_converter_convert_html() {
        setup_python();
        let converter = PyConverter::new();
        let html = "<h1>Test</h1>";
        let result = Python::with_gil(|py| converter.convert(py, html, "html")).unwrap();
        assert_eq!(result, html);
    }

    #[test]
    fn test_py_converter_convert_markdown() {
        setup_python();
        let converter = PyConverter::new();
        let html = "<h1>Test</h1>";
        let result = Python::with_gil(|py| converter.convert(py, html, "markdown")).unwrap();
        // The HTML to markdown conversion produces "# Test\n"
        assert!(result.contains("# Test") || result.contains("Test"));
    }

    #[test]
    fn test_py_converter_convert_json() {
        setup_python();
        let converter = PyConverter::new();
        let html = "<h1>Test</h1><p>Content</p>";
        let result = Python::with_gil(|py| converter.convert(py, html, "json")).unwrap();
        assert!(result.contains("Test"));
        assert!(result.contains("Content"));
    }

    #[test]
    fn test_py_converter_convert_yaml() {
        setup_python();
        let converter = PyConverter::new();
        let html = "<h1>Test</h1><p>Content</p>";
        let result = Python::with_gil(|py| converter.convert(py, html, "yaml")).unwrap();
        assert!(result.contains("Test"));
        assert!(result.contains("Content"));
    }

    #[test]
    fn test_py_converter_invalid_format() {
        setup_python();
        let converter = PyConverter::new();
        let html = "<h1>Test</h1>";
        let result = Python::with_gil(|py| converter.convert(py, html, "invalid"));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid format"));
    }

    #[test]
    fn test_shared_runtime_is_reused() {
        let first = shared_runtime().expect("runtime");
        let second = shared_runtime().expect("runtime");
        assert!(std::ptr::eq(first, second), "each call built a new runtime");
    }

    #[test]
    fn test_py_webfetcher_new() {
        let _fetcher = PyWebFetcher::new();
        // Just test that it can be created without panicking
    }

    #[test]
    fn test_py_webfetcher_from_config() {
        let config = PyConfig::new();
        // Just test that it can be created without panicking
        let _fetcher = PyWebFetcher {
            inner: WebFetcher::from_config(&config.inner),
        };
    }

    #[test]
    fn test_py_searchengine_new() {
        let _engine = PySearchEngine::new();
        // Just test that it can be created without panicking
    }

    #[test]
    fn test_py_searchengine_from_config() {
        let config = PyConfig::new();
        // Just test that it can be created without panicking
        let _engine = PySearchEngine {
            inner: SearchEngine::from_config(&config.inner),
        };
    }

    #[test]
    fn test_py_search_result() {
        let result = PySearchResult {
            title: "Test Title".to_string(),
            url: "https://example.com".to_string(),
            snippet: "Test snippet".to_string(),
            rank: 1,
        };
        assert_eq!(result.title, "Test Title");
        assert_eq!(result.url, "https://example.com");
        assert_eq!(result.snippet, "Test snippet");
        assert_eq!(result.rank, 1);
    }

    #[test]
    fn test_py_search_result_repr() {
        let result = PySearchResult {
            title: "Test Title".to_string(),
            url: "https://example.com".to_string(),
            snippet: "Test snippet".to_string(),
            rank: 1,
        };
        let repr = result.__repr__();
        assert!(repr.contains("Test Title"));
        assert!(repr.contains("https://example.com"));
        assert!(repr.contains("Test snippet"));
        assert!(repr.contains("1"));
    }

    #[test]
    fn test_py_search_result_str() {
        let result = PySearchResult {
            title: "Test Title".to_string(),
            url: "https://example.com".to_string(),
            snippet: "Test snippet".to_string(),
            rank: 1,
        };
        let str_repr = result.__str__();
        assert!(str_repr.contains("[1]"));
        assert!(str_repr.contains("Test Title"));
        assert!(str_repr.contains("https://example.com"));
        assert!(str_repr.contains("Test snippet"));
    }

    #[test]
    fn test_py_search_result_clone() {
        let result = PySearchResult {
            title: "Test Title".to_string(),
            url: "https://example.com".to_string(),
            snippet: "Test snippet".to_string(),
            rank: 1,
        };
        let cloned = result.clone();
        assert_eq!(result.title, cloned.title);
        assert_eq!(result.url, cloned.url);
        assert_eq!(result.snippet, cloned.snippet);
        assert_eq!(result.rank, cloned.rank);
    }

    #[test]
    fn test_py_config_new() {
        let _config = PyConfig::new();
        // Just test that it can be created without panicking
    }

    #[test]
    fn test_py_config_from_str() {
        let config_str = r#"
[fetcher]
timeout = 30
user_agent = "Test Agent"
format = "html"
proxy = ""

[search]
engine = "bing"
"#;
        let config: Config = toml::from_str(config_str).unwrap();
        assert_eq!(config.fetcher.user_agent, "Test Agent");
        assert_eq!(config.fetcher.timeout, 30);
    }

    #[test]
    fn test_py_config_from_str_invalid() {
        let config_str = "invalid toml content";
        let result = toml::from_str::<Config>(config_str);
        assert!(result.is_err());
    }
}
