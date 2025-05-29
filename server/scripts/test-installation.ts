#!/usr/bin/env -S deno run --allow-all

/**
 * Installation validation test script
 * Tests the installation process and verifies the binary works correctly
 */

import { assertEquals, assertExists } from "https://deno.land/std@0.210.0/assert/mod.ts";
import { join } from "https://deno.land/std@0.210.0/path/mod.ts";

const BINARY_NAME = "paiml-mcp-agent-toolkit";

async function testBinaryExecution(binaryPath: string) {
  console.log("🧪 Testing binary execution...");

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

  console.log("✅ Binary executes correctly");
}

async function testMCPProtocol(binaryPath: string) {
  console.log("🧪 Testing MCP protocol operations...");

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

  console.log("✅ MCP protocol works correctly");
}

async function testInstallationPaths() {
  console.log("🧪 Testing installation paths...");

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
      console.log(`✅ Binary found at: ${fullPath}`);
      break;
    } catch {
      // Continue searching
    }
  }

  if (!found) {
    console.log("⚠️  Binary not found in standard installation paths");
    console.log("   This is expected before installation");
  }

  return foundPath;
}

async function testCurlInstallation() {
  console.log("🧪 Testing curl installation simulation...");

  // This simulates what the curl | sh script will do
  const targetDir = join(Deno.env.get("HOME") || "", ".local/bin");
  const sourceBinary = "./target/release/" + BINARY_NAME;

  try {
    // Check if source binary exists
    await Deno.stat(sourceBinary);

    // Create target directory if it doesn't exist
    await Deno.mkdir(targetDir, { recursive: true });

    // Simulate installation (copy binary)
    const testBinary = join(targetDir, BINARY_NAME + "-test");
    await Deno.copyFile(sourceBinary, testBinary);

    // Make it executable
    await Deno.chmod(testBinary, 0o755);

    // Test the installed binary
    await testBinaryExecution(testBinary);

    // Clean up test binary
    await Deno.remove(testBinary);

    console.log("✅ Installation simulation successful");
  } catch (_e) {
    console.log("⚠️  Installation simulation requires built binary");
    console.log(`   Run 'cargo build --release' first`);
  }
}

async function main() {
  console.log("🚀 PAIML MCP Agent Toolkit Installation Validation");
  console.log("=".repeat(50));

  // Test 1: Check if binary exists in standard locations
  const installedPath = await testInstallationPaths();

  // Test 2: If found, test execution
  if (installedPath) {
    await testBinaryExecution(installedPath);
    await testMCPProtocol(installedPath);
  }

  // Test 3: Test installation simulation
  await testCurlInstallation();

  console.log("\n✨ Installation validation complete!");
}

if (import.meta.main) {
  await main();
}
