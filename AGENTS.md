# Tarzi Development Rules & Guidelines

This document contains the development rules and guidelines for **Tarzi**—a rust-native lite search for AI applications.

## Project Structure

```
tarzi/
├── src/                        # Rust source code
│   ├── fetcher/                # Web content fetching
│   └── search/                 # Search engine integration
│       └── parser/             # Search result parsers
├── tests/                      # Integration tests
│   └── python/                 # Python integration tests
│       ├── unit/               # Python unit tests
│       └── integration/        # Python integration tests
├── python/                     # Python bindings output
├── examples/                   # Usage examples (Rust & Python)
├── docs/                       # Documentation
├── mcp/                        # MCP server implementation
├── Makefile                    # Build automation
├── Cargo.toml                  # Rust dependencies
├── pyproject.toml              # Python packaging
└── .env.example                # Environment configuration template
```

## General Considerations

- **Long-term Stability**: Build a Rust project that remains runnable after 10 years. This is the fundamental rule for managing the build toolchain.
- **Code Quality**: Always ensure code formatting and linting are performed well after generation.
- **Scope Control**: DO NOT change unrelated parts beyond user requirements.
- **Test Integrity**: Always ensure existing unit tests and integration tests pass.
- **Modular Design**: Prefer modularized design for each module.
- **Constants Management**: Declare constants in dedicated modules. DO NOT treat error messages as constants.
- **Makefile Conventions**: Classify subcommands by category with unified patterns:
  - `make foo` - Run for both Rust and Python
  - `make foo-rust` - Run for Rust only  
  - `make foo-python` - Run for Python only
- **Magic Values**: DO NOT declare magic values in multiple positions. Use constant values instead.

## Implementation Considerations for Rust

- **Minimize external dependencies**: Use only well-maintained, stable crates with a strong community or institutional backing. Prefer the Rust standard library for core functionality like file I/O, threading, and networking.
- **Audit dependencies**: Use `cargo tree` or `cargo audit` to review dependencies and their transitive dependencies. Avoid crates with excessive dependencies or those that haven't been updated recently.
- **Use stable Rust features**: Avoid experimental or nightly-only features (e.g., unstable APIs or `#![feature(...)]`). Stick to the stable Rust release channel to ensure compatibility with future Rust versions.
- **Lock Rust Version and Toolchain**: For example, pin the Rust toolchain, document toolchain installation, consider vendoring the toolchain with `cargo vendor`, and regularly update vendored dependencies
- **Write Robust, Portable Code**: Write code that avoids platform-specific assumptions, avoids external system dependencies. For platform-specific code, use `#[cfg(...)]` to handle differences gracefully.
- Always use `cargo fmt` and `cargo clippy` to format and lint code.
- Always run `make check` before submitting commit.

## Implementation Considerations for Python

- **Package Management**: Always manage Python project with `pyproject.toml`
- **Quality Assurance**: Always run `make check` and fix errors before finishing code modification.
- **Python Bindings**: Use PyO3 for Rust-Python integration with proper error handling.

## Implementation Considerations for Tarzi

- **Default engine configuration**: ChromeDriver as default webdriver. Bing as default engine. Markdown as defalt fetch format.

## Testing Strategy & Guidelines

### Test Architecture
The project uses a comprehensive multi-layer testing approach:

#### **Unit Tests**
- **Location**: Embedded in source files with `#[cfg(test)]` modules
- **Coverage**: Individual functions, modules, and components
- **Commands**: 
  - `make test-unit` - All unit tests
  - `make test-unit-rust` - Rust unit tests only  
  - `make test-unit-python` - Python unit tests only

#### **Integration Tests**
- **Location**: `tests/` directory for Rust, `tests/python/` for Python
- **Coverage**: Multi-component interactions, external dependencies
- **Commands**:
  - `make test-integration` - All integration tests
  - `make test-integration-rust` - Rust integration tests only
  - `make test-integration-python` - Python integration tests only

### Testing Best Practices

#### **Robust Error Handling**
- Tests handle external service unavailability gracefully
- Network timeouts treated as acceptable failures in CI environments
- WebDriver unavailability handled with appropriate skipping

#### **Environment Compatibility**
- Tests work reliably in CI/CD environments
- Graceful degradation when external tools unavailable
- Timeout handling for slow environments

#### **Test Isolation**
- Each test cleans up resources properly
- No shared state between tests
- Proper WebDriver process lifecycle management

### Development Workflow

#### **Pre-commit Checklist**
1. `make format` - Format code (both languages)
2. `make lint` - Lint code with clippy and optional Python tools
3. `make test-unit` - Run all unit tests  
4. `make test-integration` - Run all integration tests
5. `make check` - Combined format check and linting

#### **Quality Gates**
- All existing tests must pass
- New code must include appropriate tests
- No clippy warnings in Rust code
- Proper error handling and resource cleanup

## Search Engine Architecture

Here is the definition and logics about search engines:

- Each engine provider can serve either web query or API query, or both.
- The web query path always requires no API key.
- The API path of some engines may require an API key (probed from env before any request).
- Access cascade (always): API (if available) → plain HTTP → headless browser (if `search.browser` / `TARZI_SEARCH_BROWSER` is true, default true).
- `TARZI_SEARCH_ENGINE` accepts a single engine or a comma-separated ordered failover list (e.g. `brave,duckduckgo,bing`).
- API-only engines without credentials are skipped (multi-engine) or error (single engine) without hitting the network.

### Known Engine List

| Engine         | Web Query | API Query | API Key Required | Notes |
|----------------|-----------|-----------|------------------|-------|
| Bing           | Yes       | No        | N/A              | Bing Search API retired |
| Google         | Yes       | No        | N/A              | HTML web query only |
| Google Serper  | No        | Yes       | Yes (`SERPER_API_KEY`) | Engine id: `google_serper` (alias `serper`) |
| Brave          | Yes       | Yes       | Yes for API (`BRAVE_API_KEY`) | API preferred when key present |
| DuckDuckGo     | Yes       | No        | N/A              | Plain HTML endpoint preferred before browser |
| Baidu          | Yes       | No        | N/A              | |
| Sogou Weixin   | Yes       | No        | N/A              | Engine id: `sogou_weixin` |

### Engine Implementation Guidelines

- Engine `google` and `google_serper` are different search providers.
- Each engine has specific query pattern, and web and API paths always have different query patterns.
- Each engine should have specific parser for web and API paths.
- Each engine should have different implementation about search functionalities in web or API path.
- Do not use Google Custom Search (CSE); Google API results go through `google_serper` only.

## Current System Status

### Test Coverage & Validation ✅
- **Unit Tests**: 96 Rust + 51 Python tests passing
- **Integration Tests**: 44 Rust + 53 Python tests passing  
- **Driver Tests**: Chrome and Firefox WebDriver lifecycle tests validated
- **Browser Automation**: Full browser instance management working
- **Search Engines**: All major search engines (Google, Bing, Brave, DuckDuckGo, Baidu) supported
- **Proxy Support**: HTTP proxy configuration and routing tested

### Development Infrastructure ✅
- **Dual Language Support**: Rust + Python with unified Makefile commands
- **CI/CD Ready**: Graceful handling of missing dependencies and external services
- **Code Quality**: Comprehensive linting, formatting, and error handling
- **Documentation**: Complete API documentation and usage examples
- **MCP Integration**: Model Context Protocol server implementation available

### Key Features Verified ✅
- **WebDriver Management**: Automated Chrome/Firefox driver lifecycle
- **Content Fetching**: HTTP requests, browser automation, format conversion
- **Search Integration**: Multiple search engines with unified API
- **Error Handling**: Robust timeout and failure recovery mechanisms
- **Configuration Management**: Flexible TOML-based configuration system
