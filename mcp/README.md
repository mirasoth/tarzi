# Tarzi MCP Server

A Model Context Protocol (MCP) server that exposes Tarzi's web search, content fetching, and HTML conversion capabilities as tools for LLM applications. **Now with automatic browser automation support!**

## Features

The Tarzi MCP Server provides the following tools and resources:

### Tools
- **search_web**: Search the web using various search engines
- **fetch**: Fetch content from web URLs with different formats (supports both HTTP and browser automation)
- **convert_html**: Convert HTML content to markdown, JSON, or YAML
- **search_and_fetch**: Combined search and content fetching

### Resources
- **tarzi://config**: Current Tarzi configuration
- **tarzi://status**: Server health status

### Browser Automation Features ✨
- **Automatic browser setup** - no manual configuration required
- **Headless Firefox** with geckodriver
- **Full JavaScript rendering** for dynamic content
- **Anti-bot detection bypass** capabilities
- **Custom user agents** and browser profiles
- **Configurable timeouts** and window sizes
- **Download management** and caching

## Installation

### Option 1: Python Package Installation

1. Install the package and dependencies:
```bash
cd mcp
pip install -e .
```

2. Run the server:
```bash
tarzi-mcp-server --host 0.0.0.0 --port 8000
```

### Option 2: Docker Installation (Recommended for Browser Features)

1. Build and run with Docker:
```bash
cd mcp
docker build -t tarzi-mcp-server .
docker run -p 8000:8000 --shm-size=2g tarzi-mcp-server
```

2. Or use docker-compose:
```bash
docker-compose up
```

3. With Nginx proxy:
```bash
docker-compose --profile proxy up
```

### Option 3: Browser Testing

Test browser functionality:
```bash
# Test browser functionality  
docker-compose --profile test run tarzi-mcp-browser-test

# Run with VNC for visual debugging
docker-compose --profile debug up
# Then visit http://localhost:8080 for VNC access
```

## Usage

### Starting the Server

The server supports multiple transport modes:

#### HTTP Transport (Recommended for Production)
```bash
python -m tarzi_mcp_server.server --transport streamable-http --host 0.0.0.0 --port 8000
```

#### SSE Transport (Legacy)
```bash
python -m tarzi_mcp_server.server --transport sse --host 0.0.0.0 --port 8000
```

#### STDIO Transport (Development/Testing)
```bash
python -m tarzi_mcp_server.server --transport stdio
```

### Testing with the Client

Use the included test client to verify functionality:

```bash
# Test all functionality
python -m tarzi_mcp_server.client --test all

# Test specific functionality
python -m tarzi_mcp_server.client --test search --query "machine learning"
python -m tarzi_mcp_server.client --test fetch --url "https://example.com"
python -m tarzi_mcp_server.client --test convert
python -m tarzi_mcp_server.client --test resources
```

## Tool Reference

### search_web

Search the web using Tarzi search engines.

**Parameters:**
- `query` (string): Search query
- `limit` (integer, default: 10): Maximum results to return

**Search access cascade** (configured via tarzi env, always on):
1. **API** when credentials are available (`BRAVE_API_KEY`, `SERPER_API_KEY`, …)
2. **plain HTTP** to the engine public search URL
3. **browser** when `TARZI_SEARCH_BROWSER=true` (default)

```bash
export TARZI_SEARCH_ENGINE=brave,duckduckgo,bing
export TARZI_SEARCH_BROWSER=true
```

Supported engines include `bing`, `google`, `google_serper` (alias `serper`), `brave`, `duckduckgo`, `baidu`, `sogou_weixin`.

**Returns:** List of search results with title, URL, snippet, and rank.

**Example:**
```json
{
  "query": "python programming",
  "limit": 3
}
```

### fetch

Fetch content from a web URL with optional browser automation.

**Parameters:**
- `url` (string): URL to fetch
- `format` (string, default: "html"): Output format ("html", "markdown", "json", "yaml")
- `mode` (string, default: "plain_request"): Fetch mode ("plain_request", "browser_headless", or "browser_headed")

**Returns:** Fetched content in the specified format.

**Browser Mode Benefits:**
- JavaScript execution
- Dynamic content rendering
- Anti-bot detection bypass
- Automatic configuration
- `browser_headless`: Faster execution without GUI
- `browser_headed`: Visible browser window for debugging

**Example:**
```json
{
  "url": "https://example.com",
  "format": "markdown",
  "mode": "browser_headless"
}
```

### convert_html

Convert HTML content to other formats.

**Parameters:**
- `html_content` (string): HTML content to convert
- `output_format` (string, default: "markdown"): Output format ("markdown", "json", "yaml")

**Returns:** Converted content.

**Example:**
```json
{
  "html_content": "<h1>Hello</h1><p>World</p>",
  "output_format": "markdown"
}
```

### search_and_fetch

Search and fetch content from results with browser automation support.

**Parameters:**
- `query` (string): Search query
- `limit` (integer, default: 5): Maximum results to process
- `fetch_mode` (string, default: "plain_request"): Fetch mode ("plain_request", "browser_headless", or "browser_headed")
- `content_format` (string, default: "markdown"): Content format

**Returns:** Search results with fetched content.

## Integration with Claude Desktop

To use this server with Claude Desktop, add the following configuration to your Claude Desktop config file:

**macOS:** `~/Library/Application Support/Claude/claude_desktop_config.json`
**Windows:** `%APPDATA%\Claude\claude_desktop_config.json`

```json
{
  "mcpServers": {
    "tarzi": {
      "command": "python",
      "args": ["-m", "tarzi_mcp_server.server"]
    }
  }
}
```

For HTTP transport:
```json
{
  "mcpServers": {
    "tarzi": {
      "url": "http://localhost:8000",
      "transport": "streamable-http"
    }
  }
}
```

## Docker Configuration

### Standard Deployment
```bash
docker-compose up
```

### With Browser Testing
```bash
docker-compose --profile test up
```

### With Nginx Proxy
```bash
docker-compose --profile proxy up
```

### With VNC Debugging (Visual Browser Access)
```bash
docker-compose --profile debug up
# Access browser via VNC at http://localhost:8080
```

## Docker Environment Variables

Core variables:
- `PYTHONUNBUFFERED=1`: Disable Python output buffering
- `TARZI_TIMEOUT`: Override general timeout
- `TARZI_USER_AGENT`: Override default user agent
- `TARZI_SEARCH_ENGINE` / `TARZI_SEARCH_BROWSER` / `TARZI_FETCHER_*`: See `.env.example`
- `BRAVE_API_KEY` / `SERPER_API_KEY`: Engine API keys
- `HTTP_PROXY` / `HTTPS_PROXY`: Proxy (wins over `TARZI_PROXY`)

Browser variables (automatically configured by tarzi):
- `MOZ_HEADLESS=1`: Enable headless mode
- `FIREFOX_BINARY_PATH`: Firefox binary location
- `GECKODRIVER_PATH`: Geckodriver location

## Development

### Running Tests

```bash
# Install development dependencies
pip install -e ".[dev]"

# Run tests
pytest

# Run specific tests
python -m tarzi_mcp_server.client --test all
```

### Building Docker Image

```bash
docker build -t tarzi-mcp-server .
```

## Troubleshooting

### Common Issues

1. **Import Error**: Ensure Tarzi is installed: `pip install tarzi`
2. **Connection Failed**: Check that the server is running and accessible
3. **Search Failures**: Some search engines may require API keys or have rate limits
4. **Browser Mode Errors**: Browser configuration is handled automatically by tarzi library

### Browser-Specific Troubleshooting

1. **Browser not starting**:
   ```bash
   # Test browser functionality in Docker
   docker-compose --profile test run tarzi-mcp-browser-test
   ```

2. **JavaScript not executing**:
   - Ensure you're using `mode: "browser_headless"` or `mode: "browser_headed"` in fetch
   - Browser configuration is handled automatically by tarzi

3. **Memory issues**:
   ```bash
   # Increase shared memory for Docker
   docker run --shm-size=2g tarzi-mcp-server
   ```

### Logs

The server logs to stdout by default. In Docker, access logs with:
```bash
docker logs container_name
```

For docker-compose:
```bash
docker-compose logs tarzi-mcp-server
```

Browser-specific logs:
```bash
# Browser test logs
docker-compose --profile test logs tarzi-mcp-browser-test
```

## Performance Optimization

### Browser Performance
- Browser configuration is automatically optimized by tarzi
- Configure shared memory size (`--shm-size=2g`)
- Set appropriate timeouts

### Server Performance
- Use HTTP transport for production
- Configure connection pooling
- Set appropriate timeouts
- Monitor memory usage

## Security Considerations

### Browser Security
- Runs as non-root user
- Automatic security configuration by tarzi
- Isolated browser profiles
- No automatic file downloads outside controlled directory

### Network Security
- Use HTTPS in production
- Implement rate limiting
- Configure firewall rules
- Use nginx proxy for additional security

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests (including browser tests)
4. Submit a pull request

## License

This project follows the same license as the Tarzi project (Apache 2.0).

## Support

For issues related to:
- **MCP functionality**: Check MCP documentation
- **Tarzi functionality**: Check Tarzi repository
- **Browser automation**: Handled automatically by tarzi library
- **Docker issues**: Verify container configuration
- **This server**: Open an issue in this repository

---

**✨ Browser automation ready!** The server now supports automatic browser configuration and full JavaScript rendering through the tarzi library.