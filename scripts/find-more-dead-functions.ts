#!/usr/bin/env -S deno run --allow-read --allow-run

// Find more dead functions

const functionsToCheck = [
  // Helper functions from the checklist
  "extract_rust_identifiers",
  "extract_python_identifiers",
  "extract_typescript_identifiers",
  "extract_java_identifiers",
  "extract_c_identifiers",
  "extract_generic_identifiers",
  "discover_rust_files_sync",
  "discover_rust_files",
  "write_output",

  // Full format functions
  "format_full_report_header",
  "format_full_executive_summary",
  "format_full_complexity_analysis",
  "format_full_churn_analysis",
  "format_full_satd_analysis",
  "format_full_dead_code_analysis",
  "format_full_risk_prediction",
  "format_full_recommendations",

  // Other formatting functions
  "format_deep_context_comprehensive",
  "format_annotated_tree",
  "format_tree_node",
  "format_churn_analysis",
  "format_technical_debt",
  "format_dead_code_analysis",
  "format_defect_predictions",
  "format_prioritized_recommendations",
];

async function checkFunction(funcName: string) {
  const cmd = new Deno.Command("rg", {
    args: [`${funcName}\\(`, "--type", "rust", "-C", "1"],
  });

  const output = await cmd.output();
  const text = new TextDecoder().decode(output.stdout);
  const lines = text.split("\n").filter((line) =>
    line.trim() && !line.includes(`fn ${funcName}`) &&
    !line.includes(`// ${funcName}`)
  );

  return {
    name: funcName,
    isUsed: lines.length > 0,
    usageCount: lines.length,
    usages: lines.slice(0, 2),
  };
}

console.log("Checking additional functions...\n");

const results = await Promise.all(functionsToCheck.map(checkFunction));

console.log("=== DEAD FUNCTIONS ===");
const deadFunctions = results.filter((r) => !r.isUsed);
deadFunctions.forEach((r) => console.log(`- ${r.name}`));

console.log("\n=== USED FUNCTIONS ===");
const usedFunctions = results.filter((r) => r.isUsed);
usedFunctions.forEach((r) => {
  console.log(`- ${r.name} (${r.usageCount} uses)`);
});

console.log(
  `\nSummary: ${deadFunctions.length} dead, ${usedFunctions.length} used`,
);

// Find line numbers for dead functions
if (deadFunctions.length > 0) {
  console.log("\n=== FINDING LINE NUMBERS ===");
  for (const func of deadFunctions) {
    const grepCmd = new Deno.Command("grep", {
      args: ["-n", `fn ${func.name}`, "server/src/cli/mod.rs"],
    });
    const grepOutput = await grepCmd.output();
    const grepText = new TextDecoder().decode(grepOutput.stdout);
    if (grepText) {
      const lineNum = grepText.split(":")[0];
      console.log(`${func.name}: line ${lineNum}`);
    }
  }
}
