#!/usr/bin/env -S deno run --allow-all
/**
 * Dogfooding Script: Use our own MCP Agent Toolkit to keep README.md updated
 *
 * This script demonstrates using our own tools to maintain our documentation,
 * providing both automation and real-world testing of our MCP capabilities.
 */

import { join } from "https://deno.land/std@0.208.0/path/mod.ts";

interface AnalysisResult {
  content: string;
  type: string;
  timestamp: string;
}

interface ProjectMetrics {
  complexity: AnalysisResult;
  churn: AnalysisResult;
  dag: AnalysisResult;
  ast_context: AnalysisResult;
  server_info: AnalysisResult;
}

interface ServerInfoResponse {
  result: {
    protocolVersion: string;
    capabilities: {
      tools?: Record<string, unknown>;
      resources?: Record<string, unknown>;
      prompts?: Record<string, unknown>;
    };
    serverInfo: {
      name: string;
      version: string;
      vendor: string;
      author: string;
      description: string;
      capabilities: string[];
      supportedTemplates: string[];
      supportedToolchains: string[];
      examples: string[];
    };
  };
}

const PROJECT_ROOT = Deno.cwd();
const BINARY_PATH = join(
  PROJECT_ROOT,
  "target/release/paiml-mcp-agent-toolkit",
);
const README_PATH = join(PROJECT_ROOT, "README.md");
const ARTIFACTS_DIR = join(PROJECT_ROOT, "artifacts");

class MCPAgentToolkitDogfooder {
  constructor(private binaryPath: string) {}

  async ensureBinaryExists(): Promise<void> {
    try {
      await Deno.stat(this.binaryPath);
    } catch {
      console.log("🔨 Building binary for dogfooding...");
      const buildProcess = new Deno.Command("make", {
        args: ["server-build-binary"],
        cwd: PROJECT_ROOT,
      });
      const result = await buildProcess.output();
      if (!result.success) {
        throw new Error("Failed to build binary");
      }
    }
  }

  async runMCPCommand(
    method: string,
    params: Record<string, unknown> = {},
  ): Promise<unknown> {
    const request = {
      jsonrpc: "2.0",
      id: 1,
      method,
      params,
    };

    const process = new Deno.Command(this.binaryPath, {
      stdin: "piped",
      stdout: "piped",
      stderr: "piped",
    });

    const child = process.spawn();

    const writer = child.stdin.getWriter();
    await writer.write(new TextEncoder().encode(JSON.stringify(request)));
    await writer.close();

    const output = await child.output();

    if (!output.success) {
      const error = new TextDecoder().decode(output.stderr);
      throw new Error(`MCP command failed: ${error}`);
    }

    const response = new TextDecoder().decode(output.stdout);
    // The output may contain log lines before the JSON response
    // Find the JSON response which starts with '{'
    const lines = response.split("\n");
    const jsonLine = lines.find((line: string) => line.trim().startsWith("{"));
    if (!jsonLine) {
      throw new Error(`No JSON response found in output: ${response}`);
    }
    return JSON.parse(jsonLine);
  }

  async runCLICommand(args: string[]): Promise<string> {
    const process = new Deno.Command(this.binaryPath, {
      args,
      stdout: "piped",
      stderr: "piped",
    });

    const output = await process.output();

    if (!output.success) {
      const error = new TextDecoder().decode(output.stderr);
      throw new Error(`CLI command failed: ${error}`);
    }

    return new TextDecoder().decode(output.stdout);
  }

  async analyzeProjectComplexity(): Promise<AnalysisResult> {
    console.log("📊 Analyzing code complexity...");
    const content = await this.runCLICommand([
      "analyze",
      "complexity",
      "--toolchain",
      "rust",
      "--format",
      "summary",
      "--max-cyclomatic",
      "15",
      "--max-cognitive",
      "20",
    ]);

    return {
      content,
      type: "complexity",
      timestamp: new Date().toISOString(),
    };
  }

  async analyzeCodeChurn(): Promise<AnalysisResult> {
    console.log("🔄 Analyzing code churn patterns...");
    const content = await this.runCLICommand([
      "analyze",
      "churn",
      "--days",
      "30",
      "--format",
      "markdown",
    ]);

    return {
      content,
      type: "churn",
      timestamp: new Date().toISOString(),
    };
  }

  async generateDependencyGraph(): Promise<AnalysisResult> {
    console.log("🔀 Generating dependency graph...");

    // Generate the .mmd file
    const outputPath = join(
      ARTIFACTS_DIR,
      "mermaid",
      "current_project_dag.mmd",
    );
    await this.runCLICommand([
      "analyze",
      "dag",
      "--dag-type",
      "call-graph",
      "--show-complexity",
      "--filter-external",
      "--max-depth",
      "3",
      "-o",
      outputPath,
    ]);

    // Read the generated content
    const content = await Deno.readTextFile(outputPath);

    return {
      content,
      type: "dag",
      timestamp: new Date().toISOString(),
    };
  }

  async generateASTContext(): Promise<AnalysisResult> {
    console.log("🌳 Generating AST context...");
    const content = await this.runCLICommand([
      "context",
      "rust",
      "--format",
      "markdown",
    ]);

    return {
      content,
      type: "ast_context",
      timestamp: new Date().toISOString(),
    };
  }

  async getServerInfo(): Promise<AnalysisResult> {
    console.log("ℹ️ Getting server information...");
    const response = await this.runMCPCommand(
      "initialize",
      { capabilities: {} },
    ) as ServerInfoResponse;

    const serverInfo = response.result.serverInfo;
    const content = `## Server Information

**Name:** ${serverInfo.name}
**Version:** ${serverInfo.version}
**Description:** ${serverInfo.description}

**Capabilities:**
- Tools: ${Object.keys(response.result.capabilities.tools || {}).length}
- Resources: ${Object.keys(response.result.capabilities.resources || {}).length}
- Prompts: ${Object.keys(response.result.capabilities.prompts || {}).length}

**Templates Available:** ${serverInfo.supportedTemplates.join(", ")}
**Supported Toolchains:** ${serverInfo.supportedToolchains.join(", ")}
`;

    return {
      content,
      type: "server_info",
      timestamp: new Date().toISOString(),
    };
  }

  async gatherAllMetrics(): Promise<ProjectMetrics> {
    console.log(
      "🚀 Gathering comprehensive project metrics using our own tools...",
    );

    await this.ensureBinaryExists();

    const [complexity, churn, dag, ast_context, server_info] = await Promise
      .all([
        this.analyzeProjectComplexity(),
        this.analyzeCodeChurn(),
        this.generateDependencyGraph(),
        this.generateASTContext(),
        this.getServerInfo(),
      ]);

    return {
      complexity,
      churn,
      dag,
      ast_context,
      server_info,
    };
  }

  async updateREADME(metrics: ProjectMetrics): Promise<void> {
    console.log("📝 Updating README.md with fresh metrics...");

    let readme = await Deno.readTextFile(README_PATH);
    const timestamp = new Date().toISOString().split("T")[0];

    // Update the Architecture section with current DAG
    const dagSection = `
### Current Project Dependency Graph

*Auto-generated on ${timestamp} using our own MCP toolkit*

\`\`\`mermaid
${metrics.dag.content}
\`\`\`

This diagram shows the current code dependency structure of the PAIML MCP Agent Toolkit itself, generated using our own \`analyze dag\` command. The graph includes:
- Function call relationships
- Complexity color coding (green=low, yellow=medium, orange=high, red=very high)
- Module boundaries and interfaces
- External dependency filtering
`;

    // Update or add Performance section with real metrics
    const performanceSection = `
### Dogfooding Metrics

*These metrics are generated using our own toolkit on our own codebase*

#### Code Complexity Analysis
${metrics.complexity.content}

#### Code Churn Analysis (Last 30 Days)
${metrics.churn.content}

#### AST Analysis Summary
\`\`\`
Generated on: ${metrics.ast_context.timestamp}
Tool used: paiml-mcp-agent-toolkit context rust
Cache status: ${metrics.ast_context.content.includes("cache") ? "HIT" : "MISS"}
\`\`\`

*Analysis generated with: \`paiml-mcp-agent-toolkit analyze complexity --toolchain rust\`*
`;

    // Insert dogfooding section after Performance section
    const performanceIndex = readme.indexOf("## Performance");
    if (performanceIndex !== -1) {
      const nextSectionIndex = readme.indexOf("\n## ", performanceIndex + 1);
      const insertPoint = nextSectionIndex !== -1
        ? nextSectionIndex
        : readme.length;

      readme = readme.slice(0, insertPoint) + performanceSection +
        readme.slice(insertPoint);
    }

    // Update the architecture section if it exists, or add it
    const archIndex = readme.indexOf("### System Overview");
    if (archIndex !== -1) {
      const nextSectionIndex = readme.indexOf("\n### ", archIndex + 1);
      const insertPoint = nextSectionIndex !== -1
        ? nextSectionIndex
        : readme.indexOf("\n## ", archIndex);

      readme = readme.slice(0, insertPoint) + dagSection +
        readme.slice(insertPoint);
    }

    // Add footer with generation info
    const footer = `
---

*This README.md is kept up-to-date using our own PAIML MCP Agent Toolkit*  
*Last updated: ${timestamp} • Generated with: \`deno run scripts/dogfood-readme.ts\`*
`;

    if (
      !readme.includes(
        "*This README.md is kept up-to-date using our own PAIML MCP Agent Toolkit*",
      )
    ) {
      readme += footer;
    }

    await Deno.writeTextFile(README_PATH, readme);
    console.log("✅ README.md updated with fresh dogfooding metrics!");
  }

  async saveMetricsArtifacts(metrics: ProjectMetrics): Promise<void> {
    console.log("💾 Saving metrics artifacts...");

    const metricsDir = join(ARTIFACTS_DIR, "dogfooding");
    await Deno.mkdir(metricsDir, { recursive: true });

    const timestamp = new Date().toISOString().split("T")[0];

    // Save individual metric files
    await Promise.all([
      Deno.writeTextFile(
        join(metricsDir, `complexity-${timestamp}.md`),
        metrics.complexity.content,
      ),
      Deno.writeTextFile(
        join(metricsDir, `churn-${timestamp}.md`),
        metrics.churn.content,
      ),
      Deno.writeTextFile(
        join(metricsDir, `dag-${timestamp}.mmd`),
        metrics.dag.content,
      ),
      Deno.writeTextFile(
        join(metricsDir, `ast-context-${timestamp}.md`),
        metrics.ast_context.content,
      ),
      Deno.writeTextFile(
        join(metricsDir, `server-info-${timestamp}.md`),
        metrics.server_info.content,
      ),
    ]);

    // Save combined metrics
    const combinedMetrics = {
      generated_at: new Date().toISOString(),
      generated_by: "paiml-mcp-agent-toolkit",
      metrics,
      tools_used: [
        "analyze complexity --toolchain rust",
        "analyze churn --days 30",
        "analyze dag --show-complexity",
        "context rust",
        "server/info (MCP method)",
      ],
    };

    await Deno.writeTextFile(
      join(metricsDir, `combined-metrics-${timestamp}.json`),
      JSON.stringify(combinedMetrics, null, 2),
    );

    console.log(`📁 Metrics saved to ${metricsDir}/`);
  }
}

// Main execution
if (import.meta.main) {
  const dogfooder = new MCPAgentToolkitDogfooder(BINARY_PATH);

  try {
    console.log(
      "🐕 Starting dogfooding: Using our own MCP toolkit to analyze and update documentation\n",
    );

    const metrics = await dogfooder.gatherAllMetrics();
    await dogfooder.updateREADME(metrics);
    await dogfooder.saveMetricsArtifacts(metrics);

    console.log(
      "\n🎉 Dogfooding complete! Our own tools have updated our documentation.",
    );
    console.log("📋 Summary:");
    console.log("  • Code complexity analyzed");
    console.log("  • Churn patterns identified");
    console.log("  • Dependency graph generated");
    console.log("  • AST context created");
    console.log("  • README.md updated");
    console.log("  • Metrics artifacts saved");
    console.log(
      "\nNext: Commit these changes to demonstrate real-world usage! 🚀",
    );
  } catch (error) {
    console.error("❌ Dogfooding failed:", error.message);
    Deno.exit(1);
  }
}
