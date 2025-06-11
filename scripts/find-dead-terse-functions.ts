#!/usr/bin/env -S deno run --allow-read --allow-run

// Find dead terse formatting functions

const deadTerseFunctions = [
  "format_terse_executive_summary",
  "format_terse_key_metrics",
  "format_terse_complexity_metrics",
  "format_terse_churn_metrics",
  "format_terse_satd_metrics",
  "format_terse_duplicates_metrics",
  "format_terse_dead_code_metrics",
  "format_terse_ast_network_analysis",
];

async function checkFunction(funcName: string) {
  const cmd = new Deno.Command("rg", {
    args: [`${funcName}\\(`, "--type", "rust"],
  });

  const output = await cmd.output();
  const text = new TextDecoder().decode(output.stdout);
  const lines = text.split("\n").filter((line) =>
    line.trim() && !line.includes(`fn ${funcName}`)
  );

  return {
    name: funcName,
    isUsed: lines.length > 0,
    usages: lines.slice(0, 3),
  };
}

console.log("Checking terse format functions...\n");

const results = await Promise.all(deadTerseFunctions.map(checkFunction));

console.log("=== DEAD TERSE FUNCTIONS ===");
const deadFunctions = results.filter((r) => !r.isUsed);
deadFunctions.forEach((r) => console.log(`- ${r.name}`));

console.log("\n=== USED TERSE FUNCTIONS ===");
const usedFunctions = results.filter((r) => r.isUsed);
usedFunctions.forEach((r) => {
  console.log(`- ${r.name}`);
  r.usages.forEach((usage) => console.log(`    ${usage.trim()}`));
});

// Now find line numbers for dead functions
if (deadFunctions.length > 0) {
  console.log("\n=== LINE NUMBERS FOR DEAD FUNCTIONS ===");
  for (const func of deadFunctions) {
    const grepCmd = new Deno.Command("grep", {
      args: ["-n", `fn ${func.name}`, "server/src/cli/mod.rs"],
    });
    const grepOutput = await grepCmd.output();
    const grepText = new TextDecoder().decode(grepOutput.stdout);
    if (grepText) {
      console.log(grepText.trim());
    }
  }
}
