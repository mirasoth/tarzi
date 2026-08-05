use tarzi::{Result, config::Config, converter::Format, fetcher::WebFetcher};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("=== Browser Driver Integration Demo ===\n");

    // Load configuration with proper precedence
    let mut config = Config::load().unwrap_or_else(|_| {
        println!("Using default configuration (no config files found)");
        Config::default()
    });

    // Configure to use Chrome driver by default
    config.fetcher.web_driver = "chromedriver".to_string();

    // Create WebFetcher with configuration
    let mut fetcher = WebFetcher::from_config(&config);

    // Demo URL
    let test_url = tarzi::constants::HTTPBIN_HTML_URL;

    println!("Testing browser integration with URL: {test_url}");
    println!();

    // Show current configuration
    if let Some(web_driver_url) = &config.fetcher.web_driver_url {
        if !web_driver_url.is_empty() {
            println!("✓ WebDriver URL is configured: {web_driver_url}");
            println!("  → Will use this URL with highest priority");
        } else {
            println!("ℹ WebDriver URL is not configured");
            println!("  → Will check for default webdriver at localhost:9515");
            println!("  → If not found, will try to start one with DriverManager");
        }
    } else {
        println!("ℹ WebDriver URL is not configured");
        println!("  → Will check for default webdriver at localhost:9515");
        println!("  → If not found, will try to start one with DriverManager");
    }
    println!();

    // Test fetch cascade (plain HTTP → headless browser)
    println!("Attempting to fetch content (plain HTTP → headless browser)...");
    match fetcher.fetch(test_url, Format::Html).await {
        Ok(content) => {
            println!("✓ Successfully fetched content!");
            println!("Content length: {} characters", content.len());

            // Show if we're using a managed driver
            if let Some(driver_info) = fetcher.get_managed_driver_info() {
                println!("📱 Using managed driver:");
                println!("   Type: {:?}", driver_info.config.driver_type);
                println!("   Endpoint: {}", driver_info.endpoint);
                println!("   PID: {:?}", driver_info.pid);
                println!("   Started: {:?}", driver_info.started_at);
            } else {
                println!("🌐 Using external WebDriver or plain HTTP");
            }
        }
        Err(e) => {
            println!("✗ Fetch failed: {e}");
            println!("  This is expected if WebDriver is not available and the site needs JS");
        }
    }

    // Explicit cleanup
    fetcher.shutdown().await;
    println!("\n=== Demo completed ===");

    Ok(())
}
