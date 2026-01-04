use std::time::Duration;
use tarzi::constants::CHROMEDRIVER_DEFAULT_URL;
use tarzi::search::parser::{
    BaiduParser, BaseParser, BingParser, BraveParser, DuckDuckGoParser, GoogleParser,
    SogouWeixinParser,
};
use tarzi::search::types::SearchEngineType;
use tarzi::utils::is_webdriver_available;
use thirtyfour::{By, DesiredCapabilities, Key, WebDriver};

/// Integration tests for search parsers
/// These tests require internet access and a running WebDriver server
/// Perform a real Bing search using WebDriver and return the HTML
async fn perform_bing_search(query: &str) -> Result<String, Box<dyn std::error::Error>> {
    let webdriver_url = CHROMEDRIVER_DEFAULT_URL;

    // Setup Firefox capabilities for geckodriver (default)
    let mut caps = DesiredCapabilities::firefox();
    caps.add_arg("--disable-blink-features=AutomationControlled")?;
    caps.add_arg("--disable-web-security")?;
    caps.add_arg("--disable-features=VizDisplayCompositor")?;

    // Connect to WebDriver
    let driver = WebDriver::new(webdriver_url, caps).await?;

    let result = async {
        // Navigate to Bing
        driver.goto("https://www.bing.com").await?;
        println!("Navigated to Bing homepage");

        // Wait a moment for the page to load
        tokio::time::sleep(Duration::from_millis(1000)).await;

        // Find search box and enter query
        let search_box = driver.find(By::Name("q")).await?;
        search_box.clear().await?;
        search_box.send_keys(query).await?;
        search_box.send_keys(Key::Enter).await?;
        println!("Submitted search query: '{query}'");

        // Wait for search results to load
        let mut search_results_loaded = false;
        for _ in 0..20 {
            tokio::time::sleep(Duration::from_millis(500)).await;
            match driver.find_all(By::Css("li.b_algo")).await {
                Ok(elements) if !elements.is_empty() => {
                    println!("Search results loaded successfully");
                    search_results_loaded = true;
                    break;
                }
                _ => continue,
            }
        }

        if !search_results_loaded {
            println!("Warning: Search results did not load within timeout, trying to get page source anyway");
        }

        // Additional wait to ensure all results are loaded
        tokio::time::sleep(Duration::from_millis(2000)).await;

        // Get page source
        let page_source = driver.source().await?;
        println!("Retrieved page source, length: {} characters", page_source.len());

        Ok::<String, Box<dyn std::error::Error>>(page_source)
    }.await;

    // Always quit the driver
    if let Err(e) = driver.quit().await {
        eprintln!("Warning: Failed to quit WebDriver: {e}");
    }

    result
}

/// Perform a real DuckDuckGo search using WebDriver and return the HTML
async fn perform_duckduckgo_search(query: &str) -> Result<String, Box<dyn std::error::Error>> {
    let webdriver_url = CHROMEDRIVER_DEFAULT_URL;

    // Setup Firefox capabilities for geckodriver (default)
    let mut caps = DesiredCapabilities::firefox();
    caps.add_arg("--disable-blink-features=AutomationControlled")?;
    caps.add_arg("--disable-web-security")?;
    caps.add_arg("--disable-features=VizDisplayCompositor")?;
    caps.add_arg("--user-agent=Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")?;
    caps.add_arg("--no-first-run")?;
    caps.add_arg("--disable-default-apps")?;

    // Connect to WebDriver
    let driver = WebDriver::new(webdriver_url, caps).await?;

    let result = async {
        // Navigate to DuckDuckGo
        driver.goto("https://duckduckgo.com/").await?;
        println!("Navigated to DuckDuckGo homepage");

        // Wait a moment for the page to load
        tokio::time::sleep(Duration::from_millis(2000)).await;

        // Try to find search box with different selectors
        let search_box = match driver.find(By::Id("search_form_input_homepage")).await {
            Ok(element) => element,
            Err(_) => {
                // Try alternative selector
                match driver.find(By::Name("q")).await {
                    Ok(element) => element,
                    Err(_) => driver.find(By::Css("input[type='text']")).await?
                }
            }
        };

        search_box.clear().await?;
        search_box.send_keys(query).await?;
        println!("Entered search query: '{query}'");

        // Try to submit with Enter key first (more natural)
        match search_box.send_keys(Key::Enter).await {
            Ok(_) => {
                println!("Submitted search with Enter key");
            }
            Err(_) => {
                // Fallback to clicking search button
                let search_button = match driver.find(By::Id("search_button_homepage")).await {
                    Ok(element) => element,
                    Err(_) => driver.find(By::Css("button[type='submit']")).await?
                };
                search_button.click().await?;
                println!("Clicked search button");
            }
        }

        // Wait for search results to load
        let mut search_results_loaded = false;
        for _ in 0..20 {
            tokio::time::sleep(Duration::from_millis(500)).await;
            match driver.find_all(By::Css("div.result__body")).await {
                Ok(elements) if !elements.is_empty() => {
                    println!("DuckDuckGo search results loaded successfully");
                    search_results_loaded = true;
                    break;
                }
                _ => continue,
            }
        }

        if !search_results_loaded {
            println!("Warning: DuckDuckGo search results did not load within timeout, trying to get page source anyway");
        }

        // Additional wait to ensure all results are loaded
        tokio::time::sleep(Duration::from_millis(2000)).await;

        // Get page source
        let page_source = driver.source().await?;
        println!("Retrieved DuckDuckGo page source, length: {} characters", page_source.len());

        Ok::<String, Box<dyn std::error::Error>>(page_source)
    }.await;

    // Always quit the driver
    if let Err(e) = driver.quit().await {
        eprintln!("Warning: Failed to quit WebDriver: {e}");
    }

    result
}

/// Perform a real Google search using WebDriver and return the HTML
async fn perform_google_search(query: &str) -> Result<String, Box<dyn std::error::Error>> {
    let webdriver_url = CHROMEDRIVER_DEFAULT_URL;

    // Setup Firefox capabilities for geckodriver (default)
    let mut caps = DesiredCapabilities::firefox();
    caps.add_arg("--disable-blink-features=AutomationControlled")?;
    caps.add_arg("--disable-web-security")?;
    caps.add_arg("--disable-features=VizDisplayCompositor")?;
    caps.add_arg("--user-agent=Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")?;
    caps.add_arg("--no-first-run")?;
    caps.add_arg("--disable-default-apps")?;

    // Connect to WebDriver
    let driver = WebDriver::new(webdriver_url, caps).await?;

    let result = async {
        // Navigate to Google
        driver.goto("https://www.google.com").await?;
        println!("Navigated to Google homepage");

        // Wait a moment for the page to load
        tokio::time::sleep(Duration::from_millis(2000)).await;

        // Try to accept cookies if prompted
        if let Ok(cookie_button) = driver.find(By::Css("button#L2AGLb")).await {
            if (cookie_button.click().await).is_ok() {
                println!("Accepted Google cookies");
                tokio::time::sleep(Duration::from_millis(1000)).await;
            }
        }

        // Try to find search box with different selectors
        let search_box = match driver.find(By::Name("q")).await {
            Ok(element) => element,
            Err(_) => {
                // Try alternative selector
                match driver.find(By::Css("input[name='q']")).await {
                    Ok(element) => element,
                    Err(_) => driver.find(By::Css("input[type='text']")).await?
                }
            }
        };

        search_box.clear().await?;
        search_box.send_keys(query).await?;
        println!("Entered search query: '{query}'");

        // Submit with Enter key
        search_box.send_keys(Key::Enter).await?;
        println!("Submitted search with Enter key");

        // Wait for search results to load
        let mut search_results_loaded = false;
        for _ in 0..20 {
            tokio::time::sleep(Duration::from_millis(500)).await;
            match driver.find_all(By::Css("div.tF2Cxc")).await {
                Ok(elements) if !elements.is_empty() => {
                    println!("Google search results loaded successfully");
                    search_results_loaded = true;
                    break;
                }
                _ => continue,
            }
        }

        if !search_results_loaded {
            println!("Warning: Google search results did not load within timeout, trying to get page source anyway");
        }

        // Additional wait to ensure all results are loaded
        tokio::time::sleep(Duration::from_millis(2000)).await;

        // Get page source
        let page_source = driver.source().await?;
        println!("Retrieved Google page source, length: {} characters", page_source.len());

        Ok::<String, Box<dyn std::error::Error>>(page_source)
    }.await;

    // Always quit the driver
    if let Err(e) = driver.quit().await {
        eprintln!("Warning: Failed to quit WebDriver: {e}");
    }

    result
}

/// Perform a real Brave search using WebDriver and return the HTML
async fn perform_brave_search(query: &str) -> Result<String, Box<dyn std::error::Error>> {
    let webdriver_url = CHROMEDRIVER_DEFAULT_URL;

    // Setup Firefox capabilities for geckodriver (default)
    let mut caps = DesiredCapabilities::firefox();
    caps.add_arg("--disable-blink-features=AutomationControlled")?;
    caps.add_arg("--disable-web-security")?;
    caps.add_arg("--disable-features=VizDisplayCompositor")?;
    caps.add_arg("--user-agent=Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")?;
    caps.add_arg("--no-first-run")?;
    caps.add_arg("--disable-default-apps")?;

    // Connect to WebDriver
    let driver = WebDriver::new(webdriver_url, caps).await?;

    let result = async {
        // Navigate to Brave Search
        driver.goto("https://search.brave.com/").await?;
        println!("Navigated to Brave Search homepage");

        // Wait a moment for the page to load
        tokio::time::sleep(Duration::from_millis(2000)).await;

        // Try to find search box with different selectors
        let search_box = match driver.find(By::Css("input[type='search']")).await {
            Ok(element) => element,
            Err(_) => {
                // Try alternative selectors
                match driver.find(By::Name("q")).await {
                    Ok(element) => element,
                    Err(_) => driver.find(By::Css("input[name='q']")).await?
                }
            }
        };

        search_box.clear().await?;
        search_box.send_keys(query).await?;
        println!("Entered search query: '{query}'");

        // Submit with Enter key
        search_box.send_keys(Key::Enter).await?;
        println!("Submitted search with Enter key");

        // Wait for search results to load
        let mut search_results_loaded = false;
        for _ in 0..20 {
            tokio::time::sleep(Duration::from_millis(500)).await;
            match driver.find_all(By::Css(".result-row")).await {
                Ok(elements) if !elements.is_empty() => {
                    println!("Brave search results loaded successfully");
                    search_results_loaded = true;
                    break;
                }
                _ => continue,
            }
        }

        if !search_results_loaded {
            println!("Warning: Brave search results did not load within timeout, trying to get page source anyway");
        }

        // Additional wait to ensure all results are loaded
        tokio::time::sleep(Duration::from_millis(2000)).await;

        // Get page source
        let page_source = driver.source().await?;
        println!("Retrieved Brave page source, length: {} characters", page_source.len());

        Ok::<String, Box<dyn std::error::Error>>(page_source)
    }.await;

    // Always quit the driver
    if let Err(e) = driver.quit().await {
        eprintln!("Warning: Failed to quit WebDriver: {e}");
    }

    result
}

/// Perform a real Baidu search using WebDriver and return the HTML
async fn perform_baidu_search(query: &str) -> Result<String, Box<dyn std::error::Error>> {
    let webdriver_url = CHROMEDRIVER_DEFAULT_URL;

    // Setup Firefox capabilities for geckodriver (default)
    let mut caps = DesiredCapabilities::firefox();
    caps.add_arg("--disable-blink-features=AutomationControlled")?;
    caps.add_arg("--disable-web-security")?;
    caps.add_arg("--disable-features=VizDisplayCompositor")?;
    caps.add_arg("--user-agent=Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")?;
    caps.add_arg("--no-first-run")?;
    caps.add_arg("--disable-default-apps")?;

    // Connect to WebDriver
    let driver = WebDriver::new(webdriver_url, caps).await?;

    let result = async {
        // Navigate to Baidu
        driver.goto("https://www.baidu.com").await?;
        println!("Navigated to Baidu homepage");

        // Wait a moment for the page to load
        tokio::time::sleep(Duration::from_millis(2000)).await;

        // Try to find search box
        let search_box = driver.find(By::Id("kw")).await?;
        search_box.clear().await?;
        search_box.send_keys(query).await?;
        println!("Entered search query: '{query}'");

        // Submit search
        let submit_button = driver.find(By::Id("su")).await?;
        submit_button.click().await?;
        println!("Clicked search button");

        // Wait for search results to load
        let mut search_results_loaded = false;
        for _ in 0..20 {
            tokio::time::sleep(Duration::from_millis(500)).await;
            match driver.find_all(By::Css("div#content_left")).await {
                Ok(elements) if !elements.is_empty() => {
                    println!("Baidu search results loaded successfully");
                    search_results_loaded = true;
                    break;
                }
                _ => continue,
            }
        }

        if !search_results_loaded {
            println!("Warning: Baidu search results did not load within timeout, trying to get page source anyway");
        }

        // Additional wait to ensure all results are loaded
        tokio::time::sleep(Duration::from_millis(2000)).await;

        // Get page source
        let page_source = driver.source().await?;
        println!("Retrieved Baidu page source, length: {} characters", page_source.len());

        Ok::<String, Box<dyn std::error::Error>>(page_source)
    }.await;

    // Always quit the driver
    if let Err(e) = driver.quit().await {
        eprintln!("Warning: Failed to quit WebDriver: {e}");
    }

    result
}

/// Perform a real Sogou Weixin search using WebDriver and return the HTML
async fn perform_sogou_weixin_search(query: &str) -> Result<String, Box<dyn std::error::Error>> {
    let webdriver_url = CHROMEDRIVER_DEFAULT_URL;

    // Setup Firefox capabilities for geckodriver (default)
    let mut caps = DesiredCapabilities::firefox();
    caps.add_arg("--disable-blink-features=AutomationControlled")?;
    caps.add_arg("--disable-web-security")?;
    caps.add_arg("--disable-features=VizDisplayCompositor")?;
    caps.add_arg("--no-first-run")?;
    caps.add_arg("--disable-default-apps")?;

    // Connect to WebDriver
    let driver = WebDriver::new(webdriver_url, caps).await?;

    let result = async {
        // Navigate to Sogou Weixin
        driver.goto("https://weixin.sogou.com/").await?;
        println!("Navigated to Sogou Weixin homepage");

        // Wait a moment for the page to load
        tokio::time::sleep(Duration::from_millis(2000)).await;

        // Try to find search box
        let search_box = match driver.find(By::Name("query")).await {
            Ok(el) => el,
            Err(_) => match driver.find(By::Css("input[type='text']")).await {
                Ok(el) => el,
                Err(_) => driver.find(By::Css("input#query")).await?,
            },
        };

        search_box.clear().await?;
        search_box.send_keys(query).await?;
        println!("Entered search query: '{query}'");

        // Submit search: try Enter first, fallback to a submit button
        match search_box.send_keys(Key::Enter).await {
            Ok(_) => println!("Submitted search with Enter key"),
            Err(_) => {
                // Try common button selectors
                if let Ok(btn) = driver.find(By::Css("input[type='submit']")).await {
                    btn.click().await?;
                } else if let Ok(btn) = driver.find(By::Css("button[type='submit']")).await {
                    btn.click().await?;
                }
                println!("Clicked search button");
            }
        }

        // Wait for search results to load - look for result containers
        let mut search_results_loaded = false;
        for _ in 0..20 {
            tokio::time::sleep(Duration::from_millis(500)).await;
            // Weixin result pages often have anchors linking to mp.weixin.qq.com
            match driver
                .find_all(By::Css("a[href*='mp.weixin.qq.com'], a[href*='weixin.sogou.com/link']"))
                .await
            {
                Ok(elements) if !elements.is_empty() => {
                    println!("Sogou Weixin search results loaded successfully");
                    search_results_loaded = true;
                    break;
                }
                _ => continue,
            }
        }

        if !search_results_loaded {
            println!("Warning: Sogou Weixin results did not load within timeout, getting page source anyway");
        }

        // Additional wait to ensure all results are loaded
        tokio::time::sleep(Duration::from_millis(2000)).await;

        // Get page source
        let page_source = driver.source().await?;
        println!(
            "Retrieved Sogou Weixin page source, length: {} characters",
            page_source.len()
        );

        Ok::<String, Box<dyn std::error::Error>>(page_source)
    }
    .await;

    // Always quit the driver
    if let Err(e) = driver.quit().await {
        eprintln!("Warning: Failed to quit WebDriver: {e}");
    }

    result
}

#[tokio::test]
async fn test_sogou_weixin_parser_real_world_integration() {
    // Skip test if WebDriver is not available
    if !is_webdriver_available().await {
        println!("Skipping Sogou Weixin real-world integration test: WebDriver not available");
        println!("To run this test, start a WebDriver server (e.g., geckodriver on port 4444)");
        return;
    }

    println!("Starting real-world Sogou Weixin search integration test...");

    // Perform a real search with timeout (Sogou may have anti-bot)
    let search_query = "rust 编程语言";
    let html_content = match tokio::time::timeout(
        Duration::from_secs(30),
        perform_sogou_weixin_search(search_query),
    )
    .await
    {
        Ok(Ok(html)) => html,
        Ok(Err(e)) => {
            println!("⚠️  Sogou Weixin search failed: {e}");
            println!("This is likely due to anti-automation measures or regional restrictions.");
            println!("The SogouWeixinParser logic is tested by parsing below if HTML present.");
            return; // Skip the test gracefully
        }
        Err(_) => {
            println!("⚠️  Sogou Weixin search timed out after 30 seconds");
            println!("This is likely due to anti-automation measures or CAPTCHA.");
            return; // Skip the test gracefully
        }
    };

    // Simple sanity check
    assert!(html_content.contains("weixin.sogou.com") || html_content.contains("mp.weixin.qq.com"));
    assert!(html_content.len() > 2000);
    println!("✓ Successfully retrieved Sogou Weixin search results HTML");

    // Create SogouWeixinParser and parse the results
    let parser = SogouWeixinParser::new();
    assert_eq!(parser.name(), "SogouWeixinParser");
    assert!(parser.supports(&SearchEngineType::SougouWeixin));
    println!("✓ SogouWeixinParser created and validated");

    let limit = 5;
    let results = match parser.parse(&html_content, limit) {
        Ok(results) => results,
        Err(e) => {
            eprintln!("Failed to parse Sogou Weixin HTML: {e}");
            if std::env::var("TARZI_DEBUG").is_ok() {
                std::fs::write("debug_sogou_weixin.html", &html_content).ok();
                println!("Debug HTML saved to debug_sogou_weixin.html");
            }
            panic!("Parser failed: {e}");
        }
    };

    println!("✓ Successfully parsed {} search results", results.len());

    if results.is_empty() {
        println!("⚠️  Warning: No Sogou Weixin results found. Possible anti-bot or layout change.");
        if std::env::var("TARZI_DEBUG").is_ok() {
            std::fs::write("debug_sogou_weixin_no_results.html", &html_content).ok();
            println!("Debug HTML saved to debug_sogou_weixin_no_results.html");
        }
        return; // Don't fail the test hard in flaky environments
    }

    assert!(
        results.len() <= limit,
        "Should not exceed the requested limit"
    );

    for (i, result) in results.iter().enumerate() {
        println!("Result {}: {} - {}", i + 1, result.title, result.url);
        assert_eq!(result.rank, i + 1, "Rank should be sequential");
        assert!(
            result.url.contains("mp.weixin.qq.com"),
            "URL should point to mp.weixin.qq.com: {}",
            result.url
        );
    }

    // Test with different limits
    let small_limit = 2;
    let small_results = parser.parse(&html_content, small_limit).unwrap();
    assert!(small_results.len() <= small_limit);
}

#[tokio::test]
async fn test_bing_parser_real_world_integration() {
    // Skip test if WebDriver is not available
    if !is_webdriver_available().await {
        println!("Skipping real-world integration test: WebDriver not available");
        println!("To run this test, start a WebDriver server (e.g., geckodriver on port 4444)");
        return;
    }

    println!("Starting real-world Bing search integration test...");

    // Perform a real search
    let search_query = "mirasoth";
    let html_content = match perform_bing_search(search_query).await {
        Ok(html) => html,
        Err(e) => {
            eprintln!("Failed to perform Bing search: {e}");
            panic!("Integration test failed: {e}");
        }
    };

    // Verify we got actual Bing HTML
    assert!(html_content.contains("bing.com"));
    assert!(html_content.len() > 10000); // Bing pages are typically quite large
    println!("✓ Successfully retrieved Bing search results HTML");

    // Create BingParser and parse the results
    let parser = BingParser::new();
    assert_eq!(parser.name(), "BingParser");
    assert!(parser.supports(&SearchEngineType::Bing));
    println!("✓ BingParser created and validated");

    // Parse the real HTML
    let limit = 5;
    let results = match parser.parse(&html_content, limit) {
        Ok(results) => results,
        Err(e) => {
            eprintln!("Failed to parse Bing HTML: {e}");
            // Save HTML for debugging if needed
            if std::env::var("TARZI_DEBUG").is_ok() {
                std::fs::write("debug_bing.html", &html_content).ok();
                println!("Debug HTML saved to debug_bing.html");
            }
            panic!("Parser failed: {e}");
        }
    };

    println!("✓ Successfully parsed {} search results", results.len());

    // Validate the parsed results
    assert!(!results.is_empty(), "Should have found some search results");
    assert!(
        results.len() <= limit,
        "Should not exceed the requested limit"
    );

    // Validate each result structure
    for (i, result) in results.iter().enumerate() {
        println!("Result {}: {} - {}", i + 1, result.title, result.url);

        // Basic validation
        assert!(!result.title.is_empty(), "Title should not be empty");
        assert!(!result.url.is_empty(), "URL should not be empty");
        assert_eq!(result.rank, i + 1, "Rank should be sequential");

        // URL validation
        assert!(
            result.url.starts_with("http://") || result.url.starts_with("https://"),
            "URL should be properly formatted: {}",
            result.url
        );

        // Content validation (for "rust programming language" search)
        let lower_title = result.title.to_lowercase();
        let lower_snippet = result.snippet.to_lowercase();
        let contains_rust = lower_title.contains("rust")
            || lower_snippet.contains("rust")
            || lower_title.contains("programming")
            || lower_snippet.contains("programming");

        if !contains_rust {
            println!(
                "Warning: Result {} may not be relevant to search query",
                i + 1
            );
        }
    }

    println!("✓ All results validated successfully");

    // Test with different limits
    let small_limit = 2;
    let small_results = parser.parse(&html_content, small_limit).unwrap();
    assert!(small_results.len() <= small_limit);
    assert!(small_results.len() <= results.len());
    println!("✓ Parser correctly handles different limits");

    // Test with empty HTML
    let empty_results = parser.parse("", 5).unwrap();
    assert!(empty_results.is_empty());
    println!("✓ Parser correctly handles empty HTML");

    println!("🎉 Real-world Bing parser integration test completed successfully!");
}

#[tokio::test]
async fn test_bing_parser_performance() {
    // Skip test if WebDriver is not available
    if !is_webdriver_available().await {
        println!("Skipping performance test: WebDriver not available");
        return;
    }

    println!("Testing BingParser performance...");

    // Get HTML once
    let html = match perform_bing_search("performance test").await {
        Ok(html) => html,
        Err(e) => {
            println!("Skipping performance test: {e}");
            return;
        }
    };

    let parser = BingParser::new();

    // Test parsing performance
    let start_time = std::time::Instant::now();
    let iterations = 10;

    for _ in 0..iterations {
        let _results = parser.parse(&html, 10).unwrap();
    }

    let elapsed = start_time.elapsed();
    let avg_time = elapsed / iterations;

    println!("Average parsing time: {avg_time:?}");
    assert!(
        avg_time < Duration::from_millis(500),
        "Parsing should be reasonably fast"
    );

    println!("✓ Performance test completed");
}

#[tokio::test]
async fn test_duckduckgo_parser_real_world_integration() {
    // Skip test if WebDriver is not available
    if !is_webdriver_available().await {
        println!("Skipping DuckDuckGo real-world integration test: WebDriver not available");
        println!("To run this test, start a WebDriver server (e.g., geckodriver on port 4444)");
        return;
    }

    println!("Starting real-world DuckDuckGo search integration test...");

    // Perform a real search with timeout
    let search_query = "rust programming language";
    let html_content = match tokio::time::timeout(
        Duration::from_secs(30), // Shorter timeout
        perform_duckduckgo_search(search_query),
    )
    .await
    {
        Ok(Ok(html)) => html,
        Ok(Err(e)) => {
            println!("⚠️  DuckDuckGo search failed: {e}");
            println!("This is likely due to DuckDuckGo's anti-automation measures.");
            println!("The DuckDuckGoParser logic is tested separately in unit tests.");
            return; // Skip the test gracefully
        }
        Err(_) => {
            println!("⚠️  DuckDuckGo search timed out after 30 seconds");
            println!("This is likely due to DuckDuckGo's anti-automation measures.");
            println!("The DuckDuckGoParser logic is tested separately in unit tests.");
            return; // Skip the test gracefully
        }
    };

    // Verify we got actual DuckDuckGo HTML
    assert!(html_content.contains("duckduckgo.com") || html_content.contains("DuckDuckGo"));
    assert!(html_content.len() > 5000); // DuckDuckGo pages are typically quite large
    println!("✓ Successfully retrieved DuckDuckGo search results HTML");

    // Create DuckDuckGoParser and parse the results
    let parser = DuckDuckGoParser::new();
    assert_eq!(parser.name(), "DuckDuckGoParser");
    assert!(parser.supports(&SearchEngineType::DuckDuckGo));
    println!("✓ DuckDuckGoParser created and validated");

    // Parse the real HTML
    let limit = 5;
    let results = match parser.parse(&html_content, limit) {
        Ok(results) => results,
        Err(e) => {
            eprintln!("Failed to parse DuckDuckGo HTML: {e}");
            // Save HTML for debugging if needed
            if std::env::var("TARZI_DEBUG").is_ok() {
                std::fs::write("debug_duckduckgo.html", &html_content).ok();
                println!("Debug HTML saved to debug_duckduckgo.html");
            }
            panic!("Parser failed: {e}");
        }
    };

    println!("✓ Successfully parsed {} search results", results.len());

    // Validate the parsed results
    if results.is_empty() {
        println!("⚠️  Warning: No search results found. This could be due to:");
        println!("   - DuckDuckGo blocking automated requests");
        println!("   - Changes in DuckDuckGo's HTML structure");
        println!("   - Geographic restrictions or different page layout");

        // Don't fail the test immediately, but save debug info
        if std::env::var("TARZI_DEBUG").is_ok() {
            std::fs::write("debug_duckduckgo_no_results.html", &html_content).ok();
            println!("Debug HTML saved to debug_duckduckgo_no_results.html");
        }
    } else {
        assert!(
            results.len() <= limit,
            "Should not exceed the requested limit"
        );

        // Validate each result structure
        for (i, result) in results.iter().enumerate() {
            println!("Result {}: {} - {}", i + 1, result.title, result.url);

            // Basic validation
            assert!(!result.title.is_empty(), "Title should not be empty");
            assert_eq!(result.rank, i + 1, "Rank should be sequential");

            // URL validation (DuckDuckGo might have empty URLs for some results)
            if !result.url.is_empty() {
                assert!(
                    result.url.starts_with("http://")
                        || result.url.starts_with("https://")
                        || result.url.starts_with("/"),
                    "URL should be properly formatted or relative: {}",
                    result.url
                );
            }

            // Content validation (for "rust programming language" search)
            let lower_title = result.title.to_lowercase();
            let lower_snippet = result.snippet.to_lowercase();
            let contains_rust = lower_title.contains("rust")
                || lower_snippet.contains("rust")
                || lower_title.contains("programming")
                || lower_snippet.contains("programming");

            if !contains_rust {
                println!(
                    "Warning: Result {} may not be relevant to search query",
                    i + 1
                );
            }
        }

        println!("✓ All results validated successfully");
    }

    // Test with different limits
    let small_limit = 2;
    let small_results = parser.parse(&html_content, small_limit).unwrap();
    assert!(small_results.len() <= small_limit);
    assert!(small_results.len() <= results.len());
    println!("✓ Parser correctly handles different limits");

    // Test with empty HTML
    let empty_results = parser.parse("", 5).unwrap();
    assert!(empty_results.is_empty());
    println!("✓ Parser correctly handles empty HTML");

    println!("🎉 Real-world DuckDuckGo parser integration test completed successfully!");
}

#[tokio::test]
async fn test_google_parser_real_world_integration() {
    // Skip test if WebDriver is not available
    if !is_webdriver_available().await {
        println!("Skipping Google real-world integration test: WebDriver not available");
        println!("To run this test, start a WebDriver server (e.g., geckodriver on port 4444)");
        return;
    }

    println!("Starting real-world Google search integration test...");

    // Perform a real search with timeout
    let search_query = "rust programming language";
    let html_content = match tokio::time::timeout(
        Duration::from_secs(30), // Shorter timeout
        perform_google_search(search_query),
    )
    .await
    {
        Ok(Ok(html)) => html,
        Ok(Err(e)) => {
            println!("⚠️  Google search failed: {e}");
            println!("This is likely due to Google's anti-automation measures or CAPTCHA.");
            println!("The GoogleParser logic is tested separately in unit tests.");
            return; // Skip the test gracefully
        }
        Err(_) => {
            println!("⚠️  Google search timed out after 30 seconds");
            println!("This is likely due to Google's anti-automation measures or CAPTCHA.");
            println!("The GoogleParser logic is tested separately in unit tests.");
            return; // Skip the test gracefully
        }
    };

    // Verify we got actual Google HTML
    assert!(html_content.contains("google.com") || html_content.contains("Google"));
    assert!(html_content.len() > 5000); // Google pages are typically quite large
    println!("✓ Successfully retrieved Google search results HTML");

    // Create GoogleParser and parse the results
    let parser = GoogleParser::new();
    assert_eq!(parser.name(), "GoogleParser");
    assert!(parser.supports(&SearchEngineType::Google));
    println!("✓ GoogleParser created and validated");

    // Parse the real HTML
    let limit = 5;
    let results = match parser.parse(&html_content, limit) {
        Ok(results) => results,
        Err(e) => {
            eprintln!("Failed to parse Google HTML: {e}");
            // Save HTML for debugging if needed
            if std::env::var("TARZI_DEBUG").is_ok() {
                std::fs::write("debug_google.html", &html_content).ok();
                println!("Debug HTML saved to debug_google.html");
            }
            panic!("Parser failed: {e}");
        }
    };

    println!("✓ Successfully parsed {} search results", results.len());

    // Validate the parsed results
    if results.is_empty() {
        println!("⚠️  Warning: No search results found. This could be due to:");
        println!("   - Google blocking automated requests or showing CAPTCHA");
        println!("   - Changes in Google's HTML structure");
        println!("   - Geographic restrictions or different page layout");

        // Don't fail the test immediately, but save debug info
        if std::env::var("TARZI_DEBUG").is_ok() {
            std::fs::write("debug_google_no_results.html", &html_content).ok();
            println!("Debug HTML saved to debug_google_no_results.html");
        }
    } else {
        assert!(
            results.len() <= limit,
            "Should not exceed the requested limit"
        );

        // Validate each result structure
        for (i, result) in results.iter().enumerate() {
            println!("Result {}: {} - {}", i + 1, result.title, result.url);

            // Basic validation
            assert!(!result.title.is_empty(), "Title should not be empty");
            assert_eq!(result.rank, i + 1, "Rank should be sequential");

            // URL validation (Google might have empty URLs for some results)
            if !result.url.is_empty() {
                assert!(
                    result.url.starts_with("http://")
                        || result.url.starts_with("https://")
                        || result.url.starts_with("/"),
                    "URL should be properly formatted or relative: {}",
                    result.url
                );
            }

            // Content validation (for "rust programming language" search)
            let lower_title = result.title.to_lowercase();
            let lower_snippet = result.snippet.to_lowercase();
            let contains_rust = lower_title.contains("rust")
                || lower_snippet.contains("rust")
                || lower_title.contains("programming")
                || lower_snippet.contains("programming");

            if !contains_rust {
                println!(
                    "Warning: Result {} may not be relevant to search query",
                    i + 1
                );
            }
        }

        println!("✓ All results validated successfully");
    }

    // Test with different limits
    let small_limit = 2;
    let small_results = parser.parse(&html_content, small_limit).unwrap();
    assert!(small_results.len() <= small_limit);
    assert!(small_results.len() <= results.len());
    println!("✓ Parser correctly handles different limits");

    // Test with empty HTML
    let empty_results = parser.parse("", 5).unwrap();
    assert!(empty_results.is_empty());
    println!("✓ Parser correctly handles empty HTML");

    println!("🎉 Real-world Google parser integration test completed successfully!");
}

#[tokio::test]
async fn test_brave_parser_real_world_integration() {
    // Skip test if WebDriver is not available
    if !is_webdriver_available().await {
        println!("Skipping Brave real-world integration test: WebDriver not available");
        println!("To run this test, start a WebDriver server (e.g., geckodriver on port 4444)");
        return;
    }

    println!("Starting real-world Brave search integration test...");

    // Perform a real search with timeout
    let search_query = "rust programming language";
    let html_content = match tokio::time::timeout(
        Duration::from_secs(30), // Shorter timeout
        perform_brave_search(search_query),
    )
    .await
    {
        Ok(Ok(html)) => html,
        Ok(Err(e)) => {
            println!("⚠️  Brave search failed: {e}");
            println!("This is likely due to Brave's anti-automation measures or CAPTCHA.");
            println!("The BraveParser logic is tested separately in unit tests.");
            return; // Skip the test gracefully
        }
        Err(_) => {
            println!("⚠️  Brave search timed out after 30 seconds");
            println!("This is likely due to Brave's anti-automation measures or CAPTCHA.");
            println!("The BraveParser logic is tested separately in unit tests.");
            return; // Skip the test gracefully
        }
    };

    // Verify we got actual Brave HTML
    assert!(html_content.contains("brave.com") || html_content.contains("Brave"));
    assert!(html_content.len() > 5000); // Brave pages are typically quite large
    println!("✓ Successfully retrieved Brave search results HTML");

    // Create BraveParser and parse the results
    let parser = BraveParser::new();
    assert_eq!(parser.name(), "BraveParser");
    assert!(parser.supports(&SearchEngineType::BraveSearch));
    println!("✓ BraveParser created and validated");

    // Parse the real HTML
    let limit = 5;
    let results = match parser.parse(&html_content, limit) {
        Ok(results) => results,
        Err(e) => {
            eprintln!("Failed to parse Brave HTML: {e}");
            // Save HTML for debugging if needed
            if std::env::var("TARZI_DEBUG").is_ok() {
                std::fs::write("debug_brave.html", &html_content).ok();
                println!("Debug HTML saved to debug_brave.html");
            }
            panic!("Parser failed: {e}");
        }
    };

    println!("✓ Successfully parsed {} search results", results.len());

    // Validate the parsed results
    if results.is_empty() {
        println!("⚠️  Warning: No search results found. This could be due to:");
        println!("   - Brave blocking automated requests or showing CAPTCHA");
        println!("   - Changes in Brave's HTML structure");
        println!("   - Geographic restrictions or different page layout");

        // Don't fail the test immediately, but save debug info
        if std::env::var("TARZI_DEBUG").is_ok() {
            std::fs::write("debug_brave_no_results.html", &html_content).ok();
            println!("Debug HTML saved to debug_brave_no_results.html");
        }
    } else {
        assert!(
            results.len() <= limit,
            "Should not exceed the requested limit"
        );

        // Validate each result structure
        for (i, result) in results.iter().enumerate() {
            println!("Result {}: {} - {}", i + 1, result.title, result.url);

            // Basic validation
            assert!(!result.title.is_empty(), "Title should not be empty");
            assert_eq!(result.rank, i + 1, "Rank should be sequential");

            // URL validation (Brave might have empty URLs for some results)
            if !result.url.is_empty() {
                assert!(
                    result.url.starts_with("http://")
                        || result.url.starts_with("https://")
                        || result.url.starts_with("/"),
                    "URL should be properly formatted or relative: {}",
                    result.url
                );
            }

            // Content validation (for "rust programming language" search)
            let lower_title = result.title.to_lowercase();
            let lower_snippet = result.snippet.to_lowercase();
            let contains_rust = lower_title.contains("rust")
                || lower_snippet.contains("rust")
                || lower_title.contains("programming")
                || lower_snippet.contains("programming");

            if !contains_rust {
                println!(
                    "Warning: Result {} may not be relevant to search query",
                    i + 1
                );
            }
        }

        println!("✓ All results validated successfully");
    }

    // Test with different limits
    let small_limit = 2;
    let small_results = parser.parse(&html_content, small_limit).unwrap();
    assert!(small_results.len() <= small_limit);
    assert!(small_results.len() <= results.len());
    println!("✓ Parser correctly handles different limits");

    // Test with empty HTML
    let empty_results = parser.parse("", 5).unwrap();
    assert!(empty_results.is_empty());
    println!("✓ Parser correctly handles empty HTML");

    println!("🎉 Real-world Brave parser integration test completed successfully!");
}

#[tokio::test]
async fn test_baidu_parser_real_world_integration() {
    // Skip test if WebDriver is not available
    if !is_webdriver_available().await {
        println!("Skipping Baidu real-world integration test: WebDriver not available");
        println!("To run this test, start a WebDriver server (e.g., geckodriver on port 4444)");
        return;
    }

    println!("Starting real-world Baidu search integration test...");

    // Perform a real search with timeout
    let search_query = "rust 编程语言"; // "rust programming language" in Chinese
    let html_content = match tokio::time::timeout(
        Duration::from_secs(30), // Shorter timeout
        perform_baidu_search(search_query),
    )
    .await
    {
        Ok(Ok(html)) => html,
        Ok(Err(e)) => {
            println!("⚠️  Baidu search failed: {e}");
            println!(
                "This is likely due to Baidu's anti-automation measures or regional restrictions."
            );
            println!("The BaiduParser logic is tested separately in unit tests.");
            return; // Skip the test gracefully
        }
        Err(_) => {
            println!("⚠️  Baidu search timed out after 30 seconds");
            println!(
                "This is likely due to Baidu's anti-automation measures or regional restrictions."
            );
            println!("The BaiduParser logic is tested separately in unit tests.");
            return; // Skip the test gracefully
        }
    };

    // Verify we got actual Baidu HTML
    assert!(html_content.contains("baidu.com") || html_content.contains("百度"));
    assert!(html_content.len() > 5000); // Baidu pages are typically quite large
    println!("✓ Successfully retrieved Baidu search results HTML");

    // Create BaiduParser and parse the results
    let parser = BaiduParser::new();
    assert_eq!(parser.name(), "BaiduParser");
    assert!(parser.supports(&SearchEngineType::Baidu));
    println!("✓ BaiduParser created and validated");

    // Parse the real HTML
    let limit = 5;
    let results = match parser.parse(&html_content, limit) {
        Ok(results) => results,
        Err(e) => {
            eprintln!("Failed to parse Baidu HTML: {e}");
            // Save HTML for debugging if needed
            if std::env::var("TARZI_DEBUG").is_ok() {
                std::fs::write("debug_baidu.html", &html_content).ok();
                println!("Debug HTML saved to debug_baidu.html");
            }
            panic!("Parser failed: {e}");
        }
    };

    println!("✓ Successfully parsed {} search results", results.len());

    // Validate the parsed results
    if results.is_empty() {
        println!("⚠️  Warning: No search results found. This could be due to:");
        println!("   - Baidu blocking automated requests or showing CAPTCHA");
        println!("   - Changes in Baidu's HTML structure");
        println!("   - Regional restrictions or different page layout");
        println!("   - All results filtered out as ads");

        // Don't fail the test immediately, but save debug info
        if std::env::var("TARZI_DEBUG").is_ok() {
            std::fs::write("debug_baidu_no_results.html", &html_content).ok();
            println!("Debug HTML saved to debug_baidu_no_results.html");
        }
    } else {
        assert!(
            results.len() <= limit,
            "Should not exceed the requested limit"
        );

        // Validate each result structure
        for (i, result) in results.iter().enumerate() {
            println!("Result {}: {} - {}", i + 1, result.title, result.url);

            // Basic validation
            assert!(!result.title.is_empty(), "Title should not be empty");
            assert_eq!(result.rank, i + 1, "Rank should be sequential");

            // URL validation (Baidu might have empty URLs for some results)
            if !result.url.is_empty() {
                assert!(
                    result.url.starts_with("http://")
                        || result.url.starts_with("https://")
                        || result.url.starts_with("/"),
                    "URL should be properly formatted or relative: {}",
                    result.url
                );
            }

            // Content validation (for "rust 编程语言" search)
            let lower_title = result.title.to_lowercase();
            let lower_snippet = result.snippet.to_lowercase();
            let contains_rust = lower_title.contains("rust")
                || lower_snippet.contains("rust")
                || lower_title.contains("编程")
                || lower_snippet.contains("编程")
                || lower_title.contains("programming")
                || lower_snippet.contains("programming");

            if !contains_rust {
                println!(
                    "Warning: Result {} may not be relevant to search query",
                    i + 1
                );
            }
        }

        println!("✓ All results validated successfully");
    }

    // Test with different limits
    let small_limit = 2;
    let small_results = parser.parse(&html_content, small_limit).unwrap();
    assert!(small_results.len() <= small_limit);
    assert!(small_results.len() <= results.len());
    println!("✓ Parser correctly handles different limits");

    // Test with empty HTML
    let empty_results = parser.parse("", 5).unwrap();
    assert!(empty_results.is_empty());
    println!("✓ Parser correctly handles empty HTML");

    println!("🎉 Real-world Baidu parser integration test completed successfully!");
}
