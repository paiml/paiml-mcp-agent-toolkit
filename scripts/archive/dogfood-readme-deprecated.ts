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

    // Create the metrics section content
    const metricsSection = `### Current Project Metrics

*Auto-generated metrics using our own toolkit*

**Dependency Graph:**
\`\`\`mermaid
${metrics.dag.content}
\`\`\`

**Latest Analysis:** *Generated on ${timestamp}*`;

    // Find the dogfooding metrics section using HTML comment markers
    const startMarker = "<!-- DOGFOODING_METRICS_START -->";
    const endMarker = "<!-- DOGFOODING_METRICS_END -->";

    const startIndex = readme.indexOf(startMarker);
    const endIndex = readme.indexOf(endMarker);

    if (startIndex !== -1 && endIndex !== -1) {
      // Replace the existing metrics section
      const beforeSection = readme.slice(0, startIndex + startMarker.length);
      const afterSection = readme.slice(endIndex);

      readme = beforeSection + "\n" + metricsSection + "\n" + afterSection;
      console.log("✅ README.md metrics section updated!");
    } else {
      console.log(
        "⚠️  Could not find dogfooding metrics section markers in README.md",
      );
      console.log(
        "   Expected <!-- DOGFOODING_METRICS_START --> and <!-- DOGFOODING_METRICS_END -->",
      );
    }

    await Deno.writeTextFile(README_PATH, readme);
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
