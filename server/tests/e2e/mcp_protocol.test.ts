/**
 * MCP Protocol E2E Tests
 * Tests core MCP protocol functionality via STDIO transport
 */

import {
  assertEquals,
  assertExists,
} from "https://deno.land/std@0.208.0/assert/mod.ts";
import { beforeAll, afterEach, describe, it } from "https://deno.land/std@0.208.0/testing/bdd.ts";

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
    // Close stdout and stderr to prevent resource leaks
    await this.process.stdout.cancel();
    await this.process.stderr.cancel();
    await this.process.status;
  }
}

describe("MCP Server E2E Tests", () => {
  let binaryPath: string;
  let client: McpClient | null = null;

  beforeAll(() => {
    // The binary should be in the workspace root target directory
    binaryPath = "../target/debug/paiml-mcp-agent-toolkit";
    
    // Check if binary exists
    try {
      Deno.statSync(binaryPath);
    } catch {
      throw new Error(
        `Binary not found at: ${binaryPath}. Make sure to run 'make server-build' from the repository root.`,
      );
    }
  });

  afterEach(async () => {
    if (client) {
      await client.close();
      client = null;
    }
  });

  it("should complete initialize handshake", async () => {
    client = new McpClient([binaryPath]);

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
  });

  it("should list resources", async () => {
    client = new McpClient([binaryPath]);

    const resourcesResponse = await client.sendRequest("resources/list", {}, 2);

    assertExists(resourcesResponse.result);
    assertEquals(resourcesResponse.error, undefined);
    const resources = resourcesResponse.result as {
      resources: Array<{ uri: string }>;
    };
    assertExists(resources.resources);
    assertEquals(resources.resources.length > 0, true);
  });

  it("should list tools", async () => {
    client = new McpClient([binaryPath]);

    const toolsResponse = await client.sendRequest("tools/list", {}, 3);

    assertExists(toolsResponse.result);
    assertEquals(toolsResponse.error, undefined);
    const tools = toolsResponse.result as { tools: Array<{ name: string }> };
    assertExists(tools.tools);
    assertEquals(tools.tools.length, 8);
    const toolNames = tools.tools.map((t) => t.name).sort();
    assertEquals(toolNames, [
      "analyze_code_churn",
      "analyze_complexity",
      "generate_template",
      "get_server_info",
      "list_templates",
      "scaffold_project",
      "search_templates",
      "validate_template",
    ]);
  });

  it("should generate a template", async () => {
    client = new McpClient([binaryPath]);

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
  });

  it("should read a resource", async () => {
    client = new McpClient([binaryPath]);

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
  });

  it("should list prompts", async () => {
    client = new McpClient([binaryPath]);

    const promptsResponse = await client.sendRequest("prompts/list", {}, 6);

    assertExists(promptsResponse.result);
    assertEquals(promptsResponse.error, undefined);
    const prompts = promptsResponse.result as {
      prompts: Array<{ name: string }>;
    };
    assertExists(prompts.prompts);
    assertEquals(prompts.prompts.length, 3);
  });

  it("should validate template with missing parameters", async () => {
    client = new McpClient([binaryPath]);

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
  });

  it("should search templates", async () => {
    client = new McpClient([binaryPath]);

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
  });

  it("should handle invalid requests", async () => {
    client = new McpClient([binaryPath]);

    const errorResponse = await client.sendRequest("invalid/method", {}, 9);

    assertExists(errorResponse.error);
    assertEquals(errorResponse.result, undefined);
  });
});