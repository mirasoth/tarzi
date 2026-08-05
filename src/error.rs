use thiserror::Error;

#[derive(Error, Debug)]
pub enum TarziError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("HTTP request error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("URL parsing error: {0}")]
    Url(#[from] url::ParseError),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("YAML serialization error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("Browser automation error: {0}")]
    Browser(String),

    #[error("Browser error: {0}")]
    BrowserError(String),

    #[error("Conversion error: {0}")]
    Conversion(String),

    #[error("Search error: {0}")]
    Search(String),

    #[error("Invalid format: {0}")]
    InvalidFormat(String),

    #[error("Invalid engine: {0}")]
    InvalidEngine(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("WebDriver error: {0}")]
    WebDriver(#[from] thirtyfour::error::WebDriverError),

    #[error("Driver error: {0}")]
    Driver(String),

    #[error("Driver not found: {0}")]
    DriverNotFound(String),

    #[error("Driver process error: {0}")]
    DriverProcess(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Parse error: {0}")]
    Parse(String),
}

pub type Result<T> = std::result::Result<T, TarziError>;
