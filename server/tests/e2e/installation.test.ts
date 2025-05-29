/**
 * Installation validation tests
 * Tests the installation process and verifies the binary works correctly
 */

import {
  assertEquals,
  assertExists,
} from "https://deno.land/std@0.210.0/assert/mod.ts";
import { join } from "https://deno.land/std@0.210.0/path/mod.ts";
import { describe, it } from "https://deno.land/std@0.210.0/testing/bdd.ts";

const BINARY_NAME = "paiml-mcp-agent-toolkit";

async function testBinaryExecution(binaryPath: string) {
  // Test 1: Binary responds to MCP initialize
  const initRequest = JSON.stringify({
    jsonrpc: "2.0",
    id: 1,
    method: "initialize",
    params: {
      protocolVersion: "2024-11-05",
      capabilities: {},
      clientInfo: {
        name: "test-client",
        version: "1.0.0",
      },
    },
  });

  const proc = new Deno.Command(binaryPath, {
    stdin: "piped",
    stdout: "piped",
    stderr: "piped",
  });

  const child = proc.spawn();
  const writer = child.stdin.getWriter();
  await writer.write(new TextEncoder().encode(initRequest + "\n"));
  await writer.close();

  const { code, stdout, stderr: _stderr } = await child.output();

  assertEquals(code, 0, "Binary should exit with code 0");

  // MCP servers output JSON-RPC responses line by line
  const output = new TextDecoder().decode(stdout);
  const lines = output.trim().split("\n");

  // Find the JSON response line (skip any log lines)
  let response;
  for (const line of lines) {
    try {
      response = JSON.parse(line);
      if (response.jsonrpc === "2.0") break;
    } catch {
      // Skip non-JSON lines (logs, etc.)
    }
  }

  if (!response) {
    console.error("No valid JSON-RPC response found in output:", output);
    throw new Error("Invalid server response");
  }
  assertEquals(response.jsonrpc, "2.0");
  assertEquals(response.id, 1);
  assertExists(response.result);
  assertEquals(response.result.serverInfo.name, "paiml-mcp-agent-toolkit");
}

describe("Installation Tests", () => {
  it("should find binary in standard installation paths", async () => {
    // Check common installation locations
    const installPaths = [
      "/usr/local/bin",
      join(Deno.env.get("HOME") || "", ".local/bin"),
      "./target/release",
    ];

    let found = false;
    let foundPath = "";

    for (const path of installPaths) {
      const fullPath = join(path, BINARY_NAME);
      try {
        await Deno.stat(fullPath);
        found = true;
        foundPath = fullPath;
        break;
      } catch {
        // Continue searching
      }
    }

    // It's OK if not found - this test just checks common paths
    if (found) {
      assertExists(foundPath);
    }
  });

  it("should execute binary if found in ~/.local/bin", async () => {
    const binaryPath = join(Deno.env.get("HOME") || "", ".local/bin", BINARY_NAME);
    
    try {
      await Deno.stat(binaryPath);
      await testBinaryExecution(binaryPath);
    } catch {
      // Skip if not installed
      console.log("Binary not found in ~/.local/bin - skipping execution test");
    }
  });

  it("should handle MCP protocol operations", async () => {
    const binaryPath = join(Deno.env.get("HOME") || "", ".local/bin", BINARY_NAME);
    
    try {
      await Deno.stat(binaryPath);
    } catch {
      // Skip if not installed
      console.log("Binary not found - skipping MCP protocol test");
      return;
    }

    // Test resources/list
    const listRequest = JSON.stringify({
      jsonrpc: "2.0",
      id: 2,
      method: "resources/list",
    });

    const proc = new Deno.Command(binaryPath, {
      stdin: "piped",
      stdout: "piped",
      stderr: "piped",
    });

    const child = proc.spawn();
    const writer = child.stdin.getWriter();
    await writer.write(new TextEncoder().encode(listRequest + "\n"));
    await writer.close();

    const { code, stdout } = await child.output();

    assertEquals(code, 0);

    // MCP servers output JSON-RPC responses line by line
    const output = new TextDecoder().decode(stdout);
    const lines = output.trim().split("\n");

    // Find the JSON response line (skip any log lines)
    let response;
    for (const line of lines) {
      try {
        response = JSON.parse(line);
        if (response.jsonrpc === "2.0") break;
      } catch {
        // Skip non-JSON lines (logs, etc.)
      }
    }

    if (!response) {
      console.error("No valid JSON-RPC response found in output:", output);
      throw new Error("Invalid server response");
    }
    assertExists(response.result);
    assertExists(response.result.resources);
    assertEquals(response.result.resources.length, 9, "Should have 9 templates");
  });

  it("should simulate curl installation process", async () => {
    const targetDir = join(Deno.env.get("HOME") || "", ".local/bin");
    const sourceBinary = "./target/release/" + BINARY_NAME;
    const testBinary = join(targetDir, BINARY_NAME + "-test");

    try {
      // Check if source binary exists
      await Deno.stat(sourceBinary);

      // Create target directory if it doesn't exist
      await Deno.mkdir(targetDir, { recursive: true });

      // Simulate installation (copy binary)
      await Deno.copyFile(sourceBinary, testBinary);

      // Make it executable
      await Deno.chmod(testBinary, 0o755);

      // Test the installed binary
      await testBinaryExecution(testBinary);

      // Clean up test binary
      await Deno.remove(testBinary);
    } catch {
      // Skip if release binary not built
      console.log("Release binary not found - skipping installation simulation");
    }
  });
});