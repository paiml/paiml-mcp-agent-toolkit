#!/usr/bin/env -S deno run --allow-read --allow-write

// Remove specific dead functions from cli/mod.rs

const filePath = "server/src/cli/mod.rs";
const content = await Deno.readTextFile(filePath);
const lines = content.split("\n");

const deadFunctions = [
  "format_full_report_header",
  "format_full_executive_summary",
  "format_full_complexity_analysis",
  "format_full_churn_analysis",
  "format_full_satd_analysis",
  "format_full_dead_code_analysis",
  "format_full_risk_prediction",
  "format_full_recommendations",
  "add_performance_metrics",
  "handle_output",
];

// Find function ranges
const ranges: Array<{ name: string; start: number; end: number }> = [];

for (const funcName of deadFunctions) {
  let startLine = -1;
  let braceCount = 0;
  let inFunction = false;

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];

    // Look for function definition
    if (
      (line.includes(`fn ${funcName}(`) || line.includes(`fn ${funcName}<`)) &&
      !inFunction
    ) {
      startLine = i;
      inFunction = true;

      // Check for doc comments above
      let j = i - 1;
      while (
        j >= 0 &&
        (lines[j].trim().startsWith("///") || lines[j].trim().startsWith("//"))
      ) {
        startLine = j;
        j--;
      }
    }

    if (inFunction) {
      // Count braces
      for (const char of line) {
        if (char === "{") braceCount++;
        if (char === "}") braceCount--;
      }

      // Function ends when brace count returns to 0
      if (braceCount === 0 && i > startLine) {
        ranges.push({ name: funcName, start: startLine, end: i });
        inFunction = false;
        break;
      }
    }
  }
}

// Sort by start line in reverse order (remove from bottom to top)
ranges.sort((a, b) => b.start - a.start);

console.log(`Found ${ranges.length} functions to remove:\n`);
for (const range of ranges) {
  console.log(`${range.name}: lines ${range.start + 1}-${range.end + 1}`);
}

// Remove the functions
const newLines = [...lines];
for (const range of ranges) {
  console.log(`Removing ${range.name}...`);
  // Remove lines and add blank line if needed
  newLines.splice(range.start, range.end - range.start + 1);
}

// Write back
await Deno.writeTextFile(filePath, newLines.join("\n"));

console.log(`\nRemoved ${ranges.length} functions`);
console.log(
  `File reduced from ${lines.length} to ${newLines.length} lines (${
    lines.length - newLines.length
  } lines removed)`,
);
