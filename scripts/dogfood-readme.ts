#!/usr/bin/env -S deno run --allow-all

/**
 * Dogfooding Script: Use our own deep-context analysis to maintain README.md
 *
 * This script demonstrates our MCP toolkit's capabilities by analyzing our own codebase
 * and updating README.md with fresh metrics from our deep-context analysis.
 */

const BINARY_PATH = "./target/release/paiml-mcp-agent-toolkit";
const ARTIFACTS_DIR = "./artifacts/dogfooding";

async function ensureDirectoryExists(path: string) {
  try {
    await Deno.mkdir(path, { recursive: true });
  } catch (error) {
    if (!(error instanceof Deno.errors.AlreadyExists)) {
      throw error;
    }
  }
}

async function runDeepContextAnalysis(): Promise<void> {
  console.log("🧬 Running deep-context analysis on our own codebase...");

  // Ensure artifacts directory exists
  await ensureDirectoryExists(ARTIFACTS_DIR);

  const process = new Deno.Command(BINARY_PATH, {
    args: [
      "analyze",
      "deep-context",
      "--format",
      "json",
    ],
    stdout: "piped",
    stderr: "piped",
    env: {
      "NO_COLOR": "1", // Disable ANSI color codes
      "TERM": "dumb", // Force plain output
    },
  });

  const { code, stdout, stderr } = await process.output();

  if (code !== 0) {
    const errorText = new TextDecoder().decode(stderr);
    console.error("❌ Deep-context analysis failed:", errorText);
    throw new Error(`Analysis failed with exit code ${code}`);
  }

  const result = new TextDecoder().decode(stdout);
  const timestamp = new Date().toISOString().split("T")[0];

  // Save the full analysis
  await Deno.writeTextFile(
    `${ARTIFACTS_DIR}/deep-context-${timestamp}.json`,
    result,
  );

  try {
    const analysis = JSON.parse(result);

    // Extract key metrics
    const metrics = {
      timestamp,
      overall_health: analysis.quality_scorecard?.overall_health || 0,
      complexity_score: analysis.quality_scorecard?.complexity_score || 0,
      maintainability_index:
        analysis.quality_scorecard?.maintainability_index || 0,
      technical_debt_hours: analysis.quality_scorecard?.technical_debt_hours ||
        0,
      total_defects: analysis.defect_summary?.total_defects || 0,
      analysis_duration: analysis.metadata?.analysis_duration || {},
    };

    console.log("📊 Analysis Results:");
    console.log(
      `   - Overall Health: ${metrics.overall_health.toFixed(1)}/100`,
    );
    console.log(
      `   - Complexity Score: ${metrics.complexity_score.toFixed(1)}/100`,
    );
    console.log(
      `   - Maintainability: ${metrics.maintainability_index.toFixed(1)}/100`,
    );
    console.log(
      `   - Technical Debt: ${metrics.technical_debt_hours.toFixed(1)} hours`,
    );
    console.log(`   - Total Defects: ${metrics.total_defects}`);

    // Save summary for potential README integration
    await Deno.writeTextFile(
      `${ARTIFACTS_DIR}/summary-${timestamp}.json`,
      JSON.stringify(metrics, null, 2),
    );
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    console.error("⚠️  Could not parse analysis results:", errorMessage);
    throw error;
  }
}

// Main execution
if (import.meta.main) {
  try {
    // Check if binary exists
    try {
      await Deno.stat(BINARY_PATH);
    } catch {
      console.error(`❌ Binary not found at ${BINARY_PATH}`);
      console.error("   Run 'make release' to build the binary first");
      Deno.exit(1);
    }

    await runDeepContextAnalysis();

    console.log(
      "✅ Dogfooding complete! Our tool successfully analyzed itself.",
    );
    console.log(`📁 Detailed analysis saved to: ${ARTIFACTS_DIR}/`);
    console.log("🔍 This demonstrates our deep-context analysis capabilities");
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    console.error("❌ Dogfooding failed:", errorMessage);
    Deno.exit(1);
  }
}
