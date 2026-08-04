//! Integration tests for the web driver manager
//!
//! These tests verify that the driver manager works correctly with actual driver binaries.
//! Tests will be skipped if the required drivers are not installed.

#[cfg(feature = "test-helpers")]
use std::time::Duration;
#[cfg(feature = "test-helpers")]
use tarzi::fetcher::driver::{DriverConfig, DriverManager, DriverStatus, DriverType, test_helpers};
#[cfg(not(feature = "test-helpers"))]
use tarzi::fetcher::driver::{DriverManager, DriverType};

/// Helper function to test driver lifecycle with proper cleanup
#[cfg(feature = "test-helpers")]
fn test_driver_lifecycle(
    driver_type: DriverType,
    expected_binary_name: &str,
) -> Result<(), String> {
    if !test_helpers::is_driver_available(&driver_type) {
        return Err(format!(
            "{expected_binary_name} driver not available - skipping test"
        ));
    }

    let manager = DriverManager::new();
    let port = test_helpers::find_available_port();

    let args = match driver_type {
        DriverType::Chrome => vec![
            "--disable-gpu".to_string(),
            "--no-sandbox".to_string(),
            "--disable-dev-shm-usage".to_string(),
        ],
        DriverType::Firefox => {
            let mut args = vec![
                "--host=127.0.0.1".to_string(),
                "--marionette-port=2828".to_string(),
                "--log=info".to_string(),
            ];

            // Add Firefox binary path for macOS if it exists
            let firefox_paths = vec![
                "/Applications/Firefox.app/Contents/MacOS/firefox",
                "/Applications/Firefox.app/Contents/MacOS/firefox-bin",
                "/opt/homebrew/bin/firefox",
                "/usr/local/bin/firefox",
            ];

            for path in firefox_paths {
                if std::path::Path::new(path).exists() {
                    args.push("--binary".to_string());
                    args.push(path.to_string());
                    println!("Using Firefox binary: {path}");
                    break;
                }
            }

            args
        }
        _ => vec![],
    };

    let timeout = match driver_type {
        DriverType::Firefox => Duration::from_secs(30), // Firefox takes longer to start
        _ => Duration::from_secs(15),
    };

    let config = DriverConfig {
        driver_type: driver_type.clone(),
        port,
        args,
        timeout,
        verbose: false,
    };

    match manager.start_driver_with_config(config.clone()) {
        Ok(info) => {
            // Verify driver info
            if info.config.driver_type != driver_type {
                return Err(format!(
                    "Expected driver type {:?}, got {:?}",
                    driver_type, info.config.driver_type
                ));
            }
            if info.config.port != port {
                return Err(format!("Expected port {}, got {}", port, info.config.port));
            }
            if info.status != DriverStatus::Running {
                return Err(format!("Expected status Running, got {:?}", info.status));
            }
            if info.pid.is_none() {
                return Err("Expected PID to be Some, got None".to_string());
            }
            if info.endpoint != format!("http://127.0.0.1:{port}") {
                return Err(format!(
                    "Expected endpoint http://127.0.0.1:{}, got {}",
                    port, info.endpoint
                ));
            }

            // Verify the driver is listed
            let drivers = manager.list_drivers();
            if drivers.len() != 1 {
                return Err(format!("Expected 1 driver in list, got {}", drivers.len()));
            }
            if drivers[0].config.port != port {
                return Err(format!(
                    "Expected listed driver port {}, got {}",
                    port, drivers[0].config.port
                ));
            }

            // Verify we can get driver info
            let retrieved_info = manager.get_driver_info(port);
            if retrieved_info.is_none() {
                return Err("Expected driver info to be Some, got None".to_string());
            }

            // Stop the driver
            if let Err(e) = manager.stop_driver(port) {
                return Err(format!("Failed to stop driver: {e}"));
            }

            // Verify the driver is no longer listed
            let drivers_after_stop = manager.list_drivers();
            if !drivers_after_stop.is_empty() {
                return Err(format!(
                    "Expected no drivers after stop, got {}",
                    drivers_after_stop.len()
                ));
            }

            Ok(())
        }
        Err(e) => {
            let error_msg = format!("{e}");
            if error_msg.contains(expected_binary_name)
                || error_msg.contains(&driver_type.to_string())
            {
                Err(format!("Driver startup failed (expected): {error_msg}"))
            } else {
                Err(format!("Unexpected error: {error_msg}"))
            }
        }
    }
}

/// Test that we can create a driver manager
#[test]
fn test_create_driver_manager() {
    let manager = DriverManager::new();
    assert_eq!(manager.list_drivers().len(), 0);
}

/// Test driver binary detection
#[test]
fn test_driver_binary_detection() {
    let manager = DriverManager::new();

    // Test Chrome driver detection
    let chrome_result = manager.check_driver_binary(&DriverType::Chrome);
    match &chrome_result {
        Ok(()) => println!("ChromeDriver is available"),
        Err(e) => println!("ChromeDriver not available: {e}"),
    }

    // Test Firefox driver detection
    let firefox_result = manager.check_driver_binary(&DriverType::Firefox);
    match &firefox_result {
        Ok(()) => println!("GeckoDriver is available"),
        Err(e) => println!("GeckoDriver not available: {e}"),
    }

    // At least one of the results should provide useful error messages
    if let Err(err) = chrome_result {
        let error_msg = format!("{err}");
        assert!(error_msg.contains("chromedriver") || error_msg.contains("ChromeDriver"));
    }

    if let Err(err) = firefox_result {
        let error_msg = format!("{err}");
        assert!(error_msg.contains("geckodriver") || error_msg.contains("GeckoDriver"));
    }
}

/// Test starting and stopping a Chrome driver (if available)
#[test]
#[cfg(feature = "test-helpers")]
fn test_chrome_driver_lifecycle() {
    match test_driver_lifecycle(DriverType::Chrome, "chromedriver") {
        Ok(()) => {
            println!("✓ Chrome driver test completed successfully");
        }
        Err(msg) => {
            if msg.contains("not available - skipping test") {
                println!("✓ Chrome driver test skipped: {msg}");
            } else {
                panic!("Chrome driver test failed unexpectedly: {msg}");
            }
        }
    }
}

/// Test starting and stopping a Firefox driver (if available)
#[test]
#[cfg(feature = "test-helpers")]
fn test_firefox_driver_lifecycle() {
    match test_driver_lifecycle(DriverType::Firefox, "geckodriver") {
        Ok(()) => {
            println!("✓ Firefox driver test completed successfully");
        }
        Err(msg) => {
            if msg.contains("not available - skipping test") {
                println!("✓ Firefox driver test skipped: {msg}");
            } else {
                panic!("Firefox driver test failed unexpectedly: {msg}");
            }
        }
    }
}

/// Test error handling for non-existent driver
#[test]
#[cfg(feature = "test-helpers")]
fn test_nonexistent_driver() {
    let manager = DriverManager::new();
    let port = test_helpers::find_available_port();

    let config = DriverConfig {
        driver_type: DriverType::Generic("nonexistent-driver".to_string()),
        port,
        args: vec![],
        timeout: Duration::from_secs(5),
        verbose: false,
    };

    let result = manager.start_driver_with_config(config);

    match result {
        Ok(_) => {
            panic!("Expected error for non-existent driver, but got success");
        }
        Err(e) => {
            let error_msg = format!("{e}");
            if error_msg.contains("not found") || error_msg.contains("nonexistent-driver") {
                println!("✓ Non-existent driver error handling works correctly: {error_msg}");
            } else {
                panic!("Unexpected error message for non-existent driver: {error_msg}");
            }
        }
    }

    // Verify no driver is listed after failure
    let drivers = manager.list_drivers();
    assert_eq!(drivers.len(), 0, "Expected no drivers after failed start");
}
