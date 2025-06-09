#!/usr/bin/env deno run --allow-read --allow-write --allow-run

/**
 * Excellence Tracker - Monitors code quality metrics against excellence standards
 */

// import { exec } from "https://deno.land/std@0.208.0/process/mod.ts";
import { ensureDir } from "https://deno.land/std@0.208.0/fs/mod.ts";
import { join } from "https://deno.land/std@0.208.0/path/mod.ts";

interface ExcellenceMetrics {
  coverage: number;
  complexity: {
    max: number;
    p90: number;
    violations: number;
  };
  tdg: {
    average: number;
    max: number;
  };
  satd: number;
  performance: {
    o1_percentage: number;
  };
  timestamp: string;
}

interface ExcellenceStandards {
  coverage: number;
  maxComplexity: number;
  maxTdg: number;
  maxSatd: number;
  minO1Percentage: number;
}

const STANDARDS: ExcellenceStandards = {
  coverage: 95,
  maxComplexity: 10,
  maxTdg: 1.0,
  maxSatd: 0,
  minO1Percentage: 80,
};

async function runCommand(cmd: string[]): Promise<string> {
  const p = new Deno.Command(cmd[0], {
    args: cmd.slice(1),
    stdout: "piped",
    stderr: "piped",
  });

  const { stdout } = await p.output();
  return new TextDecoder().decode(stdout);
}

function getCoverageMetrics(): Promise<number> {
  try {
    // This is a placeholder - actual coverage command depends on setup
    console.log("📊 Measuring code coverage...");
    // const output = await runCommand(["cargo", "tarpaulin", "--print-summary"]);
    // Parse coverage percentage from output
    return Promise.resolve(0); // Placeholder
  } catch {
    return Promise.resolve(0);
  }
}

async function getComplexityMetrics(
  path: string,
): Promise<ExcellenceMetrics["complexity"]> {
  try {
    console.log("🔍 Analyzing complexity...");
    const output = await runCommand([
      "./target/release/pmat",
      "analyze",
      "complexity",
      "-p",
      path,
      "--format",
      "json",
    ]);

    const data = JSON.parse(output);
    return {
      max: data.summary?.max_cyclomatic || 0,
      p90: data.summary?.p90_cyclomatic || 0,
      violations: data.violations?.filter((v: { value: number }) =>
        v.value > STANDARDS.maxComplexity
      ).length || 0,
    };
  } catch {
    return { max: 0, p90: 0, violations: 0 };
  }
}

async function getTdgMetrics(path: string): Promise<ExcellenceMetrics["tdg"]> {
  try {
    console.log("📈 Analyzing TDG...");
    const output = await runCommand([
      "./target/release/pmat",
      "analyze",
      "tdg",
      "-p",
      path,
      "--format",
      "json",
    ]);

    const data = JSON.parse(output);
    return {
      average: data.average_tdg || 0,
      max: data.p99_tdg || 0,
    };
  } catch {
    return { average: 0, max: 0 };
  }
}

async function getSatdCount(path: string): Promise<number> {
  try {
    console.log("🔍 Checking SATD...");
    const output = await runCommand([
      "./target/release/pmat",
      "analyze",
      "satd",
      "-p",
      path,
      "--format",
      "json",
    ]);

    const data = JSON.parse(output);
    return data.summary?.total_items || 0;
  } catch {
    return 0;
  }
}

function analyzePerformance(
  _path: string,
): Promise<ExcellenceMetrics["performance"]> {
  try {
    console.log("⚡ Analyzing performance characteristics...");
    // This would analyze Big-O complexity of functions
    // For now, returning placeholder
    return Promise.resolve({ o1_percentage: 0 });
  } catch {
    return Promise.resolve({ o1_percentage: 0 });
  }
}

async function trackExcellence(modulePath: string): Promise<ExcellenceMetrics> {
  console.log(`\n🎯 Excellence Tracker - ${modulePath}\n`);

  const metrics: ExcellenceMetrics = {
    coverage: await getCoverageMetrics(),
    complexity: await getComplexityMetrics(modulePath),
    tdg: await getTdgMetrics(modulePath),
    satd: await getSatdCount(modulePath),
    performance: await analyzePerformance(modulePath),
    timestamp: new Date().toISOString(),
  };

  return metrics;
}

function generateReport(metrics: ExcellenceMetrics): string {
  const passed: string[] = [];
  const failed: string[] = [];

  // Check coverage
  if (metrics.coverage >= STANDARDS.coverage) {
    passed.push(
      `✅ Coverage: ${metrics.coverage}% (target: ${STANDARDS.coverage}%)`,
    );
  } else {
    failed.push(
      `❌ Coverage: ${metrics.coverage}% (target: ${STANDARDS.coverage}%)`,
    );
  }

  // Check complexity
  if (metrics.complexity.max <= STANDARDS.maxComplexity) {
    passed.push(
      `✅ Max Complexity: ${metrics.complexity.max} (target: ≤${STANDARDS.maxComplexity})`,
    );
  } else {
    failed.push(
      `❌ Max Complexity: ${metrics.complexity.max} (target: ≤${STANDARDS.maxComplexity}) - ${metrics.complexity.violations} violations`,
    );
  }

  // Check TDG
  if (metrics.tdg.max <= STANDARDS.maxTdg) {
    passed.push(
      `✅ Max TDG: ${
        metrics.tdg.max.toFixed(2)
      } (target: ≤${STANDARDS.maxTdg})`,
    );
  } else {
    failed.push(
      `❌ Max TDG: ${
        metrics.tdg.max.toFixed(2)
      } (target: ≤${STANDARDS.maxTdg})`,
    );
  }

  // Check SATD
  if (metrics.satd <= STANDARDS.maxSatd) {
    passed.push(`✅ SATD: ${metrics.satd} (target: ${STANDARDS.maxSatd})`);
  } else {
    failed.push(`❌ SATD: ${metrics.satd} (target: ${STANDARDS.maxSatd})`);
  }

  // Check performance
  if (metrics.performance.o1_percentage >= STANDARDS.minO1Percentage) {
    passed.push(
      `✅ O(1) Functions: ${metrics.performance.o1_percentage}% (target: ≥${STANDARDS.minO1Percentage}%)`,
    );
  } else {
    failed.push(
      `❌ O(1) Functions: ${metrics.performance.o1_percentage}% (target: ≥${STANDARDS.minO1Percentage}%)`,
    );
  }

  let report = "# Excellence Report\n\n";
  report += `Generated: ${metrics.timestamp}\n\n`;

  if (failed.length === 0) {
    report += "## 🏆 All Excellence Standards Met!\n\n";
  } else {
    report += `## 📊 Status: ${passed.length}/${
      passed.length + failed.length
    } standards met\n\n`;
  }

  if (passed.length > 0) {
    report += "### Passed Standards\n";
    report += passed.map((p) => `- ${p}`).join("\n");
    report += "\n\n";
  }

  if (failed.length > 0) {
    report += "### Failed Standards\n";
    report += failed.map((f) => `- ${f}`).join("\n");
    report += "\n\n";
  }

  return report;
}

async function saveMetrics(metrics: ExcellenceMetrics, moduleName: string) {
  const dir = "excellence-metrics";
  await ensureDir(dir);

  const filename = join(dir, `${moduleName}-${Date.now()}.json`);
  await Deno.writeTextFile(filename, JSON.stringify(metrics, null, 2));

  // Also save latest
  const latestFilename = join(dir, `${moduleName}-latest.json`);
  await Deno.writeTextFile(latestFilename, JSON.stringify(metrics, null, 2));
}

// Main execution
if (import.meta.main) {
  const modulePath = Deno.args[0] || "server/src/services/makefile_linter";
  const moduleName = modulePath.split("/").pop() || "unknown";

  try {
    const metrics = await trackExcellence(modulePath);
    const report = generateReport(metrics);

    console.log("\n" + report);

    await saveMetrics(metrics, moduleName);

    // Exit with error if standards not met
    const failureCount = (metrics.coverage < STANDARDS.coverage ? 1 : 0) +
      (metrics.complexity.max > STANDARDS.maxComplexity ? 1 : 0) +
      (metrics.tdg.max > STANDARDS.maxTdg ? 1 : 0) +
      (metrics.satd > STANDARDS.maxSatd ? 1 : 0) +
      (metrics.performance.o1_percentage < STANDARDS.minO1Percentage ? 1 : 0);

    if (failureCount > 0) {
      console.error(`\n⚠️  ${failureCount} excellence standards not met`);
      Deno.exit(1);
    } else {
      console.log("\n✨ All excellence standards achieved!");
    }
  } catch (error) {
    console.error("Error tracking excellence:", error);
    Deno.exit(1);
  }
}
