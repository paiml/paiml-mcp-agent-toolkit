#!/usr/bin/env -S deno run --allow-read --allow-run

// Comprehensive dead code detection for cli/mod.rs

const filePath = "server/src/cli/mod.rs";
const content = await Deno.readTextFile(filePath);
const lines = content.split("\n");

// Find all functions (both regular and async)
const functionPattern = /^(async\s+)?fn\s+(\w+)/;
const functions: Map<string, number> = new Map();

for (let i = 0; i < lines.length; i++) {
  const match = lines[i].match(functionPattern);
  if (match) {
    const funcName = match[2];
    functions.set(funcName, i + 1);
  }
}

console.log(`Found ${functions.size} functions in cli/mod.rs\n`);

// Check each function for usage
const deadFunctions: Array<{ name: string; line: number }> = [];
const usedFunctions: Array<{ name: string; line: number; usageCount: number }> =
  [];

for (const [funcName, lineNum] of functions) {
  // Skip test functions and main
  if (funcName === "main" || funcName.includes("test_")) {
    continue;
  }

  // Check for usage
  const cmd = new Deno.Command("rg", {
    args: [`\\b${funcName}\\b`, "--type", "rust", "-c"],
  });

  const output = await cmd.output();
  const text = new TextDecoder().decode(output.stdout);

  // Count occurrences (subtract 1 for the definition)
  let totalCount = 0;
  for (const line of text.split("\n")) {
    if (line.includes(":")) {
      const [_, count] = line.split(":");
      totalCount += parseInt(count) || 0;
    }
  }

  // If only found once (the definition), it's dead code
  if (totalCount <= 1) {
    deadFunctions.push({ name: funcName, line: lineNum });
  } else {
    usedFunctions.push({
      name: funcName,
      line: lineNum,
      usageCount: totalCount - 1,
    });
  }
}

// Group dead functions by prefix
const deadByPrefix = new Map<string, number>();
for (const func of deadFunctions) {
  const prefix = func.name.split("_")[0];
  deadByPrefix.set(prefix, (deadByPrefix.get(prefix) || 0) + 1);
}

console.log("=== DEAD FUNCTIONS BY PREFIX ===");
for (
  const [prefix, count] of [...deadByPrefix.entries()].sort((a, b) =>
    b[1] - a[1]
  )
) {
  console.log(`${prefix}_*: ${count} functions`);
}

console.log("\n=== ALL DEAD FUNCTIONS ===");
deadFunctions.sort((a, b) => a.line - b.line);
for (const func of deadFunctions) {
  console.log(`Line ${func.line}: ${func.name}`);
}

console.log(
  `\nTotal: ${deadFunctions.length} dead functions out of ${functions.size}`,
);
