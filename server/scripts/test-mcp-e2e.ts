#!/usr/bin/env -S deno run --allow-run --allow-read --allow-write

/**
 * End-to-end test for MCP server binary
 * Tests core MCP protocol functionality via STDIO transport
 */

import {
  assertEquals,
  assertExists,
} from "https://deno.land/std@0.208.0/assert/mod.ts";

interface JsonRpcRequest {
  jsonrpc: "2.0";
  method: string;
  params?: unknown;
  id: number | string;
}

interface JsonRpcResponse {
  jsonrpc: "2.0";
  result?: unknown;
  error?: {
    code: number;
    message: string;
    data?: unknown;
  };
  id: number | string;
}

class McpClient {
  private process: Deno.ChildProcess;
  private writer: WritableStreamDefaultWriter<Uint8Array>;
  private encoder = new TextEncoder();
  private decoder = new TextDecoder();
  private buffer = "";

  constructor(command: string[], args: string[] = []) {
    this.process = new Deno.Command(command[0], {
      args: [...command.slice(1), ...args],
      stdin: "piped",
      stdout: "piped",
      stderr: "piped",
    }).spawn();
    this.writer = this.process.stdin.getWriter();
  }

  async sendRequest(
    method: string,
    params?: unknown,
    id: number | string = 1,
  ): Promise<JsonRpcResponse> {
    const request: JsonRpcRequest = {
      jsonrpc: "2.0",
      method,
      params,
      id,
    };

    const requestStr = JSON.stringify(request) + "\n";
    await this.writer.write(this.encoder.encode(requestStr));

    // Read response
    const response = await this.readResponse();
    return JSON.parse(response) as JsonRpcResponse;
  }

  private async readResponse(): Promise<string> {
    const reader = this.process.stdout.getReader();

    while (true) {
      const { value, done } = await reader.read();
      if (done) break;

      this.buffer += this.decoder.decode(value);
      const lines = this.buffer.split("\n");

      if (lines.length > 1) {
        // Process all complete lines
        for (let i = 0; i < lines.length - 1; i++) {
          const line = lines[i].trim();

          // Skip empty lines and log lines (they start with timestamp)
          if (!line || line.match(/^\[\d{4}-\d{2}-\d{2}/)) {
            continue;
          }

          // Try to parse as JSON
          try {
            JSON.parse(line);
            this.buffer = lines.slice(i + 1).join("\n");
            reader.releaseLock();
            return line;
          } catch {
            // Not JSON, skip
            continue;
          }
        }

        // Keep the incomplete line
        this.buffer = lines[lines.length - 1];
      }
    }

    reader.releaseLock();
    throw new Error("No response received");
  }

  async close() {
    await this.writer.close();
    await this.process.status;
  }
}

async function runE2ETests() {
  console.log("🧪 Running MCP Server E2E Tests");

  // Test 1: Initialize handshake
  console.log("\n📋 Test 1: Initialize handshake");
  const client = new McpClient(["./target/debug/mcp_server_stateless"]);

  try {
    const initResponse = await client.sendRequest("initialize", {
      protocolVersion: "0.1.0",
      capabilities: {
        tools: true,
        resources: { subscribe: true, list: true },
      },
      clientInfo: {
        name: "mcp-e2e-test",
        version: "1.0.0",
      },
    });

    assertExists(initResponse.result);
    assertEquals(initResponse.error, undefined);

    // Check server metadata
    const initResult = initResponse.result as {
      serverInfo?: {
        supportedTemplates?: string[];
        supportedToolchains?: string[];
      };
    };
    assertExists(initResult.serverInfo);
    assertExists(initResult.serverInfo.supportedTemplates);
    assertExists(initResult.serverInfo.supportedToolchains);
    console.log("✅ Initialize handshake successful with server metadata");

    // Test 2: List resources
    console.log("\n📋 Test 2: List resources");
    const resourcesResponse = await client.sendRequest("resources/list", {}, 2);

    assertExists(resourcesResponse.result);
    assertEquals(resourcesResponse.error, undefined);
    const resources = resourcesResponse.result as {
      resources: Array<{ uri: string }>;
    };
    assertExists(resources.resources);
    console.log(`✅ Listed ${resources.resources.length} resources`);

    // Test 3: List tools
    console.log("\n📋 Test 3: List tools");
    const toolsResponse = await client.sendRequest("tools/list", {}, 3);

    assertExists(toolsResponse.result);
    assertEquals(toolsResponse.error, undefined);
    const tools = toolsResponse.result as { tools: Array<{ name: string }> };
    assertExists(tools.tools);
    assertEquals(tools.tools.length, 5);
    const toolNames = tools.tools.map((t) => t.name).sort();
    assertEquals(toolNames, [
      "generate_template",
      "list_templates",
      "scaffold_project",
      "search_templates",
      "validate_template",
    ]);
    console.log("✅ Listed tools successfully");

    // Test 4: Generate a template
    console.log("\n📋 Test 4: Generate template");
    const templateResponse = await client.sendRequest("tools/call", {
      name: "generate_template",
      arguments: {
        resource_uri: "template://makefile/rust/cli-binary",
        parameters: {
          project_name: "test-project",
          has_tests: true,
          has_benchmarks: false,
        },
      },
    }, 4);

    assertExists(templateResponse.result);
    assertEquals(templateResponse.error, undefined);

    // Check if result has content array
    const result = templateResponse.result as {
      content?: Array<{ type: string; text: string }>;
    };
    assertExists(result.content, "Result should have content array");
    assertExists(
      result.content[0],
      "Content array should have at least one item",
    );
    assertEquals(result.content[0].type, "text");
    assertExists(result.content[0].text, "Text content should exist");
    console.log("✅ Template generated successfully");

    // Test 5: Read resource
    console.log("\n📋 Test 5: Read resource");
    const readResponse = await client.sendRequest("resources/read", {
      uri: "template://makefile/rust/cli-binary",
    }, 5);

    assertExists(readResponse.result);
    assertEquals(readResponse.error, undefined);
    const readResult = readResponse.result as {
      contents: Array<{ text: string }>;
    };
    assertExists(readResult.contents);
    assertExists(readResult.contents[0].text);
    console.log("✅ Resource read successfully");

    // Test 6: List prompts
    console.log("\n📋 Test 6: List prompts");
    const promptsResponse = await client.sendRequest("prompts/list", {}, 6);

    assertExists(promptsResponse.result);
    assertEquals(promptsResponse.error, undefined);
    const prompts = promptsResponse.result as {
      prompts: Array<{ name: string }>;
    };
    assertExists(prompts.prompts);
    assertEquals(prompts.prompts.length, 3);
    console.log("✅ Listed prompts successfully");

    // Test 7: Validate template
    console.log("\n📋 Test 7: Validate template");
    const validateResponse = await client.sendRequest("tools/call", {
      name: "validate_template",
      arguments: {
        resource_uri: "template://makefile/rust/cli-binary",
        parameters: {
          // Missing required parameter project_name
          has_tests: true,
        },
      },
    }, 7);

    assertExists(validateResponse.result);
    assertEquals(validateResponse.error, undefined);
    const validateResult = validateResponse.result as {
      valid: boolean;
      missing_required: string[];
      content: Array<{ text: string }>;
    };
    assertEquals(validateResult.valid, false);
    assertExists(validateResult.missing_required);
    assertEquals(validateResult.missing_required.length > 0, true);
    console.log("✅ Template validation works correctly");

    // Test 8: Search templates
    console.log("\n📋 Test 8: Search templates");
    const searchResponse = await client.sendRequest("tools/call", {
      name: "search_templates",
      arguments: {
        query: "rust",
      },
    }, 8);

    assertExists(searchResponse.result);
    assertEquals(searchResponse.error, undefined);
    const searchResult = searchResponse.result as { templates: Array<unknown> };
    assertExists(searchResult.templates);
    console.log("✅ Template search works correctly");

    // Test 9: Handle invalid request
    console.log("\n📋 Test 9: Handle invalid request");
    const errorResponse = await client.sendRequest("invalid/method", {}, 9);

    assertExists(errorResponse.error);
    assertEquals(errorResponse.result, undefined);
    console.log("✅ Error handling works correctly");
  } catch (error) {
    console.error("❌ Test failed:", error);
    await client.close();
    Deno.exit(1);
  }

  await client.close();
  console.log("\n✅ All E2E tests passed!");
}

// Run tests
if (import.meta.main) {
  try {
    await runE2ETests();
  } catch (error) {
    console.error("Test suite failed:", error);
    Deno.exit(1);
  }
}
