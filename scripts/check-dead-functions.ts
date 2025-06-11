#!/usr/bin/env -S deno run --allow-read --allow-run

// Script to check if functions are dead code

const functionsToCheck = [
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
  "format_terse_header",
  "format_terse_metadata",
  "format_terse_files_by_complexity",
  "format_terse_files_by_churn",
  "format_terse_files_by_tech_debt",
  "format_terse_dead_code_files",
  "format_terse_predicted_defect_files",
  "format_proof_output",
  "format_proof_annotations_summary",
  "format_proof_annotations_full",
  "format_proof_annotations_markdown",
  "format_proof_annotations_sarif",
  "format_incremental_coverage_summary",
  "format_incremental_coverage_json",
  "format_incremental_coverage_markdown",
  "format_incremental_coverage_sarif",
];

async function checkFunction(funcName: string) {
  // Check for function calls (not definitions)
  const cmd = new Deno.Command("rg", {
    args: [`${funcName}\\(`, "--type", "rust"],
  });

  const output = await cmd.output();
  const text = new TextDecoder().decode(output.stdout);

  // Filter out the function definition line
  const lines = text.split("\n").filter((line) =>
    line.trim() && !line.includes(`fn ${funcName}`)
  );

  return {
    name: funcName,
    isUsed: lines.length > 0,
    usageCount: lines.length,
    usages: lines.slice(0, 3), // First 3 usages
  };
}

console.log("Checking for dead functions in cli/mod.rs...\n");

const results = await Promise.all(functionsToCheck.map(checkFunction));

console.log("=== DEAD FUNCTIONS (not used anywhere) ===");
const deadFunctions = results.filter((r) => !r.isUsed);
deadFunctions.forEach((r) => console.log(`- ${r.name}`));

console.log("\n=== USED FUNCTIONS ===");
const usedFunctions = results.filter((r) => r.isUsed);
usedFunctions.forEach((r) => {
  console.log(`- ${r.name} (${r.usageCount} uses)`);
  r.usages.forEach((usage) => console.log(`    ${usage.trim()}`));
});

console.log(
  `\nSummary: ${deadFunctions.length} dead, ${usedFunctions.length} used`,
);
