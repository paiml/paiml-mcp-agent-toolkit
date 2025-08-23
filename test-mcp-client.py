#!/usr/bin/env python3
"""
Simple MCP client test for PMAT agent functionality.
This script tests the MCP (Model Context Protocol) server implementation.
"""

import subprocess
import json
import asyncio
import sys
import time


async def test_mcp_server():
    """Test the MCP server functionality."""
    print("🧪 Testing PMAT MCP Server Integration")
    print("=" * 50)
    
    # Start the MCP server process
    print("🚀 Starting MCP server...")
    proc = subprocess.Popen(
        ["cargo", "run", "--", "agent", "mcp-server", "--debug"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        cwd="/home/noah/src/paiml-mcp-agent-toolkit"
    )
    
    # Give it a moment to start
    await asyncio.sleep(2)
    
    try:
        print("📡 Testing MCP protocol communication...")
        
        # Test 1: Initialize request
        init_request = {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "pmat-test-client",
                    "version": "1.0.0"
                }
            }
        }
        
        print("→ Sending initialize request...")
        proc.stdin.write(json.dumps(init_request) + "\n")
        proc.stdin.flush()
        
        # Read response
        response_line = proc.stdout.readline()
        if response_line:
            print(f"← Received: {response_line.strip()}")
            try:
                response = json.loads(response_line.strip())
                if response.get("id") == 1:
                    print("✅ Initialize request successful")
                else:
                    print("❌ Initialize request failed")
            except json.JSONDecodeError:
                print(f"❌ Invalid JSON response: {response_line}")
        
        # Test 2: Health check
        health_request = {
            "jsonrpc": "2.0",
            "id": 2,
            "method": "health_check",
            "params": {}
        }
        
        print("→ Sending health check request...")
        proc.stdin.write(json.dumps(health_request) + "\n")
        proc.stdin.flush()
        
        # Read response
        response_line = proc.stdout.readline()
        if response_line:
            print(f"← Received: {response_line.strip()}")
            try:
                response = json.loads(response_line.strip())
                if response.get("id") == 2:
                    print("✅ Health check successful")
                else:
                    print("❌ Health check failed")
            except json.JSONDecodeError:
                print(f"❌ Invalid JSON response: {response_line}")
        
        # Test 3: List tools
        tools_request = {
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/list",
            "params": {}
        }
        
        print("→ Sending tools list request...")
        proc.stdin.write(json.dumps(tools_request) + "\n")
        proc.stdin.flush()
        
        # Read response
        response_line = proc.stdout.readline()
        if response_line:
            print(f"← Received: {response_line.strip()}")
            try:
                response = json.loads(response_line.strip())
                if response.get("id") == 3 and "result" in response:
                    tools = response["result"].get("tools", [])
                    print(f"✅ Tools list successful - Found {len(tools)} tools:")
                    for tool in tools:
                        print(f"   - {tool.get('name', 'unknown')}: {tool.get('description', 'no description')}")
                else:
                    print("❌ Tools list failed")
            except json.JSONDecodeError:
                print(f"❌ Invalid JSON response: {response_line}")
        
        print("\n🎉 MCP Server testing completed!")
        
    except Exception as e:
        print(f"❌ Error during testing: {e}")
        
    finally:
        # Clean shutdown
        print("🛑 Shutting down MCP server...")
        proc.terminate()
        try:
            proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            proc.kill()
            proc.wait()


if __name__ == "__main__":
    print("PMAT MCP Server Integration Test")
    print("================================")
    
    # Check if we have the required dependencies
    try:
        result = subprocess.run(["cargo", "--version"], capture_output=True, text=True)
        if result.returncode != 0:
            print("❌ Cargo not found. Please install Rust and Cargo.")
            sys.exit(1)
        print(f"✅ Cargo found: {result.stdout.strip()}")
    except FileNotFoundError:
        print("❌ Cargo not found. Please install Rust and Cargo.")
        sys.exit(1)
    
    # Run the async test
    asyncio.run(test_mcp_server())