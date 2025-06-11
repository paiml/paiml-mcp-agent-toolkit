#!/usr/bin/env -S deno run --allow-read

// Script to find line ranges of functions for removal

const filePath = "server/src/cli/mod.rs";
const content = await Deno.readTextFile(filePath);
const lines = content.split("\n");

const deadFunctions = [
  "format_deep_context_full",
  "execute_analyze_command_legacy",
  "handle_analyze_graph_metrics_legacy",
  "format_terse_summary",
  "format_terse_complexity_hotspots",
  "format_terse_churn_analysis",
  "format_terse_technical_debt",
  "format_terse_dead_code",
  "format_terse_defect_predictions",
  "format_terse_recommendations",
  "format_terse_metadata",
  "format_terse_files_by_complexity",
  "format_terse_files_by_churn",
  "format_terse_files_by_tech_debt",
  "format_terse_dead_code_files",
];

function findFunctionRange(
  funcName: string,
): { start: number; end: number } | null {
  let startLine = -1;
  let braceCount = 0;
  let inFunction = false;

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];

    // Look for function definition
    if (line.includes(`fn ${funcName}(`) || line.includes(`fn ${funcName}<`)) {
      startLine = i;
      inFunction = true;

      // Check for #[allow(dead_code)] above
      if (i > 0 && lines[i - 1].includes("#[allow(dead_code)]")) {
        startLine = i - 1;
      }
      // Check for doc comments
      if (i > 1 && lines[i - 2].startsWith("///")) {
        let j = i - 2;
        while (j >= 0 && lines[j].startsWith("///")) {
          startLine = j;
          j--;
        }
      }
    }

    if (inFunction) {
      // Count braces
      for (const char of line) {
        if (char === "{") braceCount++;
        if (char === "}") braceCount--;
      }

      // Function ends when brace count returns to 0
      if (braceCount === 0 && startLine !== i) {
        return { start: startLine + 1, end: i + 1 }; // Convert to 1-based
      }
    }
  }

  return null;
}

console.log("Finding line ranges for dead functions:\n");

const ranges: Array<{ name: string; start: number; end: number }> = [];

for (const func of deadFunctions) {
  const range = findFunctionRange(func);
  if (range) {
    ranges.push({ name: func, ...range });
    console.log(`${func}: lines ${range.start}-${range.end}`);
  } else {
    console.log(`${func}: NOT FOUND`);
  }
}

// Sort by start line in reverse order (remove from bottom to top)
ranges.sort((a, b) => b.start - a.start);

console.log("\nRemoval commands (in order):");
ranges.forEach((r) => {
  console.log(`sed -i '${r.start},${r.end}d' server/src/cli/mod.rs`);
});
