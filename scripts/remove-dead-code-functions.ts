#!/usr/bin/env -S deno run --allow-read --allow-write --allow-run=rg

// Script to remove functions marked with #[allow(dead_code)]

const filePath = "server/src/cli/mod.rs";
const content = await Deno.readTextFile(filePath);
const lines = content.split("\n");

// Find all functions with #[allow(dead_code)]
const deadCodeRanges: Array<{ start: number; end: number; name: string }> = [];
let i = 0;

while (i < lines.length) {
  if (lines[i].includes("#[allow(dead_code)]")) {
    // Look for function on next line
    let funcLine = i + 1;
    while (funcLine < lines.length && !lines[funcLine].includes(" fn ")) {
      funcLine++;
    }

    if (funcLine < lines.length && lines[funcLine].includes(" fn ")) {
      // Extract function name
      const match = lines[funcLine].match(/fn\s+(\w+)/);
      const funcName = match ? match[1] : "unknown";

      // Find end of function
      let braceCount = 0;
      let inFunction = false;
      let endLine = funcLine;

      for (let j = funcLine; j < lines.length; j++) {
        const line = lines[j];
        for (const char of line) {
          if (char === "{") {
            braceCount++;
            inFunction = true;
          }
          if (char === "}") braceCount--;
        }

        if (inFunction && braceCount === 0) {
          endLine = j;
          break;
        }
      }

      // Check if function is actually used
      const checkCmd = new Deno.Command("rg", {
        args: [`${funcName}\\(`, "--type", "rust"],
      });

      const output = await checkCmd.output();
      const usageText = new TextDecoder().decode(output.stdout);
      const usages = usageText.split("\n").filter((line) =>
        line.trim() &&
        !line.includes(`fn ${funcName}`) &&
        !line.includes(`#[allow(dead_code)]`)
      );

      if (usages.length === 0) {
        deadCodeRanges.push({
          start: i,
          end: endLine,
          name: funcName,
        });
        console.log(
          `Found dead function: ${funcName} (lines ${i + 1}-${endLine + 1})`,
        );
      } else {
        console.log(
          `Skipping ${funcName} - it's actually used (${usages.length} uses)`,
        );
      }

      i = endLine + 1;
    } else {
      i++;
    }
  } else {
    i++;
  }
}

// Sort by start line in reverse order
deadCodeRanges.sort((a, b) => b.start - a.start);

console.log(`\nFound ${deadCodeRanges.length} dead functions to remove`);

if (deadCodeRanges.length > 0) {
  // Remove dead functions
  const newLines = [...lines];

  for (const range of deadCodeRanges) {
    console.log(
      `Removing ${range.name} (lines ${range.start + 1}-${range.end + 1})`,
    );
    newLines.splice(range.start, range.end - range.start + 1);
  }

  // Write back
  await Deno.writeTextFile(filePath, newLines.join("\n"));
  console.log(`\nRemoved ${deadCodeRanges.length} dead functions`);
  console.log(`File reduced from ${lines.length} to ${newLines.length} lines`);
}
