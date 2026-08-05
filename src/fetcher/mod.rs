//! Web content fetching module
//!
//! Access cascade (always): plain HTTP → headless browser (when `fetcher.browser` is enabled).

pub mod browser;
pub mod driver;
pub mod webfetcher;

// Re-export main types and functions
pub use driver::{DriverConfig, DriverInfo, DriverManager, DriverStatus, DriverType};
pub use webfetcher::WebFetcher;
