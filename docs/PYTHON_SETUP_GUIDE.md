# Tarzi Python Wrapper - Quick Start

Fast setup guide for building and using the tarzi Python extension.

## Status ✅

✅ **Core Rust Library**: Working (68/68 tests pass)  
✅ **Python Bindings**: Complete with enhanced features  
✅ **PyO3 Build**: Fixed and working  

## Quick Setup

### 1. Build the Python Extension

```bash
# Get your Python library info
python3 -c "import sysconfig; print('Library:', sysconfig.get_config_var('LIBDIR')); print('Python lib:', sysconfig.get_config_var('LDLIBRARY'))"

# Set environment variables (adjust paths for your system)
export RUSTFLAGS="-L/Users/xiamingchen/.pyenv/versions/3.11.10/lib -lpython3.11"
export PYO3_PYTHON=/Users/xiamingchen/.pyenv/versions/3.11.10/envs/tarzi/bin/python3

# Build
cargo clean
cargo build --features pyo3
```

### 2. Test the Module

```bash
# Test import
python3 -c "import tarzi; print('✅ Success!')"

# Test basic functionality
python3 -c "
import tarzi
converter = tarzi.Converter()
result = converter.convert('<h1>Test</h1>', 'markdown')
print('Result:', result)
"
```

## Quick Usage Examples

### Basic HTML Conversion
```python
import tarzi

# Create converter
converter = tarzi.Converter()

# Convert HTML to markdown
html = '<h1>Hello</h1><p>World!</p>'
markdown = converter.convert(html, 'markdown')
print(markdown)  # # Hello\n\nWorld!
```

### Web Fetching
```python
import tarzi

# Create web fetcher
fetcher = tarzi.WebFetcher()

# Fetch and convert a webpage
content = fetcher.fetch('https://example.com', 'plain_request', 'markdown')
print(content)
```

### Web Search
```python
import tarzi

# Create search engine
engine = tarzi.SearchEngine()

# Search the web
results = engine.search('python programming', 5)
for result in results:
    print(f"{result.title}: {result.url}")
```

## Available Classes and Functions

### Classes
- `Converter()` - HTML/text conversion
- `WebFetcher()` - Web page fetching  
- `SearchEngine()` - Web search functionality
- `Config()` - Configuration management

### Standalone Functions
- `convert_html(html, format)` - Quick HTML conversion
- `fetch` / `fetch_url(url, mode, format)` - Quick URL fetching
- `search_web(query, limit)` - Quick web search
- `search_with_content(query, limit, fetch_mode, format)` - Search and fetch pages

### Supported Formats
- `html` - Raw HTML
- `markdown` - Markdown text
- `json` - JSON structure
- `yaml` - YAML format

### Fetch Modes
- `plain_request` - Simple HTTP request
- `browser_head` - Browser with head (faster)
- `browser_headless` - Full headless browser

## Development Commands

```bash
# Run Rust tests
cargo test --features "default"

# Test Python bindings (Rust tests)
cargo test --features pyo3

# Run Python unit tests
python3 test_tarzi.py
python3 run_python_tests.py --verbose

# Build release wheel
maturin build --features pyo3 --release

# Run examples
python3 examples/basic_usage.py
python3 examples/sogou_weixin_search.py

# Run all tests (Rust + Python)
make test-all
```

## Troubleshooting

### Build Issues
If you get linking errors, adjust the paths in RUSTFLAGS:
```bash
# For Homebrew Python
export RUSTFLAGS="-L/opt/homebrew/lib -lpython3.11"
export PYO3_PYTHON=/opt/homebrew/bin/python3.11

# For system Python
export RUSTFLAGS="-L/usr/lib -lpython3.11"
export PYO3_PYTHON=/usr/bin/python3
```

### Import Issues
```bash
# Check if module was built
ls target/debug/deps/libtarzi.dylib

# Check Python can find it
python3 -c "import sys; print(sys.path)"
```

## Next Steps

1. **Run Python unit tests**: `python3 test_tarzi.py` for comprehensive testing
2. **Try the example scripts**: `python3 examples/basic_usage.py` and `python3 examples/sogou_weixin_search.py`
3. **Check the API documentation**: `python3 -c "import tarzi; help(tarzi.Converter)"`
4. **Run development tests**: `python3 run_python_tests.py --verbose`
5. **Build wheels for distribution**: `maturin build --release`
6. **Read testing guide**: See `PYTHON_TESTING.md` for detailed test information

**Ready to develop!** 🚀 