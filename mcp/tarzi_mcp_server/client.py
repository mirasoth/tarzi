#!/usr/bin/env python3
"""Simple MCP client to test the Tarzi MCP server."""

import asyncio
import json
import logging
from typing import Any, Dict, List

try:
    from mcp import ClientSession
    from mcp.client.streamable_http import streamablehttp_client
except ImportError:
    raise ImportError("mcp library is required. Install with: pip install mcp")

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


class TarziMCPClient:
    """Simple client for testing Tarzi MCP server."""
    
    def __init__(self, server_url: str = "http://127.0.0.1:8000"):
        """Initialize the client with server URL."""
        self.server_url = server_url
    
    async def test_server_resources(self):
        """Test server resources (status and config)."""
        try:
            async with streamablehttp_client(self.server_url) as (read_stream, write_stream, _):
                async with ClientSession(read_stream, write_stream) as session:
                    await session.initialize()
                    
                    logger.info("Testing server resources...")
                    
                    # List available resources
                    resources = await session.list_resources()
                    logger.info(f"Available resources: {[r.uri for r in resources.resources]}")
                    
                    # Read status resource
                    if any(r.uri == "tarzi://status" for r in resources.resources):
                        status, _ = await session.read_resource("tarzi://status")
                        logger.info(f"Server status:\n{status}")
                    
                    # Read config resource
                    if any(r.uri == "tarzi://config" for r in resources.resources):
                        config, _ = await session.read_resource("tarzi://config")
                        logger.info(f"Server config:\n{config}")
                        
        except Exception as e:
            logger.error(f"Resource test failed: {e}")
    
    async def test_search_tool(self, query: str = "python programming"):
        """Test the search_web tool."""
        try:
            async with streamablehttp_client(self.server_url) as (read_stream, write_stream, _):
                async with ClientSession(read_stream, write_stream) as session:
                    await session.initialize()
                    
                    logger.info(f"Testing search with query: '{query}'")
                    
                    # Call search tool
                    result = await session.call_tool("search_web", {
                        "query": query,
                        "limit": 3
                    })
                    
                    logger.info(f"Search results: {json.dumps(result.content, indent=2)}")
                    
        except Exception as e:
            logger.error(f"Search test failed: {e}")
    
    async def test_fetch_tool(self, url: str = "https://httpbin.org/html"):
        """Test the fetch tool."""
        try:
            async with streamablehttp_client(self.server_url) as (read_stream, write_stream, _):
                async with ClientSession(read_stream, write_stream) as session:
                    await session.initialize()
                    
                    logger.info(f"Testing fetch with URL: {url}")
                    
                    # Call fetch tool
                    result = await session.call_tool("fetch", {
                        "url": url,
                        "format": "markdown",
                    })
                    
                    logger.info(f"Fetch result length: {len(str(result.content))} characters")
                    logger.info(f"Fetch result preview: {str(result.content)[:200]}...")
                    
        except Exception as e:
            logger.error(f"Fetch test failed: {e}")
    
    async def test_convert_tool(self):
        """Test the convert_html tool."""
        try:
            async with streamablehttp_client(self.server_url) as (read_stream, write_stream, _):
                async with ClientSession(read_stream, write_stream) as session:
                    await session.initialize()
                    
                    logger.info("Testing HTML conversion...")
                    
                    test_html = """
                    <html>
                        <head><title>Test Page</title></head>
                        <body>
                            <h1>Hello World</h1>
                            <p>This is a <strong>test</strong> page.</p>
                            <a href="https://example.com">Link</a>
                        </body>
                    </html>
                    """
                    
                    # Call convert tool
                    result = await session.call_tool("convert_html", {
                        "html_content": test_html,
                        "output_format": "markdown"
                    })
                    
                    logger.info(f"Conversion result:\n{result.content}")
                    
        except Exception as e:
            logger.error(f"Convert test failed: {e}")
    
    async def test_all_tools(self):
        """Test all available tools."""
        try:
            async with streamablehttp_client(self.server_url) as (read_stream, write_stream, _):
                async with ClientSession(read_stream, write_stream) as session:
                    await session.initialize()
                    
                    # List available tools
                    tools = await session.list_tools()
                    logger.info(f"Available tools: {[t.name for t in tools.tools]}")
                    
                    for tool in tools.tools:
                        logger.info(f"Tool: {tool.name} - {tool.description}")
                        
        except Exception as e:
            logger.error(f"Tool listing failed: {e}")
    
    async def run_all_tests(self):
        """Run all tests."""
        logger.info("Starting MCP client tests...")
        
        await self.test_all_tools()
        await self.test_server_resources()
        await self.test_convert_tool()
        await self.test_fetch_tool()
        await self.test_search_tool()
        
        logger.info("All tests completed!")


async def main():
    """Main entry point for the client."""
    import argparse
    
    parser = argparse.ArgumentParser(description="Tarzi MCP Client")
    parser.add_argument("--server", default="http://127.0.0.1:8000", help="MCP server URL")
    parser.add_argument("--test", choices=["all", "search", "fetch", "convert", "resources"], 
                       default="all", help="Test to run")
    parser.add_argument("--query", default="python programming", help="Search query for search test")
    parser.add_argument("--url", default="https://httpbin.org/html", help="URL for fetch test")
    
    args = parser.parse_args()
    
    client = TarziMCPClient(args.server)
    
    if args.test == "all":
        await client.run_all_tests()
    elif args.test == "search":
        await client.test_search_tool(args.query)
    elif args.test == "fetch":
        await client.test_fetch_tool(args.url)
    elif args.test == "convert":
        await client.test_convert_tool()
    elif args.test == "resources":
        await client.test_server_resources()


if __name__ == "__main__":
    asyncio.run(main())