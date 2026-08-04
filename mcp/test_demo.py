#!/usr/bin/env python3
"""Demo script to test MCP server structure without full tarzi installation."""

import asyncio
import logging
from typing import List, Dict, Any
from mcp.server.fastmcp import FastMCP
from pydantic import BaseModel, Field

# Mock tarzi functionality for demonstration
class MockSearchResult:
    def __init__(self, title: str, url: str, snippet: str, rank: int):
        self.title = title
        self.url = url
        self.snippet = snippet
        self.rank = rank

class MockTarzi:
    @staticmethod
    def search_web(query: str, limit: int) -> List[MockSearchResult]:
        """Mock search function for demo."""
        return [
            MockSearchResult(f"Result {i+1} for '{query}'", f"https://example{i+1}.com", 
                           f"Snippet {i+1} about {query}", i+1)
            for i in range(min(limit, 3))
        ]
    
    @staticmethod
    def fetch(url: str, mode: str, format: str) -> str:
        """Mock fetch function for demo."""
        if format == "markdown":
            return f"# Mock Content\n\nThis is mock content from {url}"
        elif format == "json":
            return f'{{"url": "{url}", "content": "Mock JSON content"}}'
        else:
            return f"<html><body><h1>Mock Content from {url}</h1></body></html>"
    
    @staticmethod
    def convert_html(html: str, format: str) -> str:
        """Mock conversion function for demo."""
        if format == "markdown":
            return "# Converted Title\n\nConverted content from HTML"
        return '{"converted": "Mock converted content"}'

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# Create the MCP server
mcp = FastMCP(
    "Tarzi Demo Server",
    description="Demo MCP server showing Tarzi functionality structure"
)

class SearchResult(BaseModel):
    """Structured search result."""
    title: str = Field(description="Page title")
    url: str = Field(description="Page URL")
    snippet: str = Field(description="Page snippet/description")
    rank: int = Field(description="Search result ranking")

@mcp.tool()
def demo_search_web(query: str, limit: int = 10) -> List[SearchResult]:
    """Demo search web tool."""
    try:
        mock_results = MockTarzi.search_web(query, limit)
        structured_results = []
        for result in mock_results:
            structured_results.append(SearchResult(
                title=result.title,
                url=result.url,
                snippet=result.snippet,
                rank=result.rank
            ))
        logger.info(f"Demo search completed: {len(structured_results)} results for '{query}'")
        return structured_results
    except Exception as e:
        logger.error(f"Demo search failed: {str(e)}")
        raise ValueError(f"Demo search failed: {str(e)}")

@mcp.tool()
def demo_fetch(url: str, format: str = "html") -> str:
    """Demo fetch URL tool."""
    try:
        content = MockTarzi.fetch(url, "plain_request", format)
        logger.info(f"Demo fetch completed: {url} in {format} format")
        return content
    except Exception as e:
        logger.error(f"Demo fetch failed: {str(e)}")
        raise ValueError(f"Demo fetch failed: {str(e)}")

@mcp.tool()
def demo_convert_html(html_content: str, output_format: str = "markdown") -> str:
    """Demo HTML conversion tool."""
    try:
        converted = MockTarzi.convert_html(html_content, output_format)
        logger.info(f"Demo conversion completed to {output_format}")
        return converted
    except Exception as e:
        logger.error(f"Demo conversion failed: {str(e)}")
        raise ValueError(f"Demo conversion failed: {str(e)}")

@mcp.resource("demo://status")
def demo_status() -> str:
    """Demo status resource."""
    return """Tarzi MCP Demo Server Status: HEALTHY
- Mock Search: Available
- Mock Fetch: Available
- Mock Converter: Available
- MCP Server: Running
- This is a demonstration without actual Tarzi installation
"""

@mcp.resource("demo://config")
def demo_config() -> str:
    """Demo config resource."""
    return """Demo Tarzi Configuration:
- Mode: Demo/Mock
- Available tools: demo_search_web, demo_fetch, demo_convert_html
- Transport: streamable-http
- This demonstrates the MCP server structure
"""

async def main():
    """Main entry point for demo."""
    logger.info("Starting Tarzi MCP Demo Server...")
    
    # Test the tools programmatically
    print("\n=== Demo Tool Tests ===")
    
    # Test search
    search_results = demo_search_web("python programming", 2)
    print(f"Search results: {len(search_results)} found")
    for result in search_results:
        print(f"  - {result.title} ({result.url})")
    
    # Test fetch
    fetch_result = demo_fetch("https://example.com", "markdown")
    print(f"Fetch result: {fetch_result[:50]}...")
    
    # Test convert
    convert_result = demo_convert_html("<h1>Test</h1><p>Content</p>", "markdown")
    print(f"Convert result: {convert_result}")
    
    # Test resources
    print(f"Status: {demo_status()}")
    print(f"Config: {demo_config()}")
    
    print("\n=== Starting MCP Server ===")
    print("Demo completed! The real server would run with:")
    print("mcp.run(transport='streamable-http', host='0.0.0.0', port=8000)")
    print("\nTo run the actual server, install tarzi and use:")
    print("python -m tarzi_mcp_server.server")

if __name__ == "__main__":
    asyncio.run(main())