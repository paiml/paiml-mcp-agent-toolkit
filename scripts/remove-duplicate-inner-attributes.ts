#!/usr/bin/env deno

import { walk } from "https://deno.land/std@0.190.0/fs/walk.ts";

async function removeDuplicateInnerAttributes(
  filePath: string,
): Promise<boolean> {
  try {
    const content = await Deno.readTextFile(filePath);

    // Remove duplicate inner attributes that follow outer attributes
    let modified = content;

    // Remove #![allow(...)] lines that appear after #[allow(...)]
    modified = modified.replace(
      /(#\[cfg\(test\)\]\s*\n#\[allow\(clippy::cast_possible_truncation\)\]\s*\n#\[allow\(clippy::cast_sign_loss\)\]\s*\n#\[allow\(clippy::cast_lossless\)\]\s*\n)(#!\[allow\(clippy::cast_possible_truncation\)\]\s*\n#!\[allow\(clippy::cast_sign_loss\)\]\s*\n#!\[allow\(clippy::cast_lossless\)\]\s*\n+)/g,
      "$1",
    );

    // Also remove standalone duplicate inner attributes
    modified = modified.replace(
      /(#\[allow\([^)]+\)\]\s*\n)(#!\[allow\([^)]+\)\]\s*\n)/g,
      "$1",
    );

    if (content !== modified) {
      await Deno.writeTextFile(filePath, modified);
      return true;
    }
    return false;
  } catch (error) {
    console.error(`Error processing ${filePath}:`, error);
    return false;
  }
}

async function main() {
  console.log("🔧 Removing Duplicate Inner Attributes");
  console.log("======================================\n");

  let fixedCount = 0;
  let totalCount = 0;

  const rustFiles = walk("./server/src", {
    exts: [".rs"],
    skip: [/target/],
  });

  for await (const entry of rustFiles) {
    totalCount++;
    if (await removeDuplicateInnerAttributes(entry.path)) {
      console.log(`✅ Fixed: ${entry.path.replace("./server/", "")}`);
      fixedCount++;
    }
  }

  console.log(`\n📊 Summary: Fixed ${fixedCount} of ${totalCount} files`);
}

if (import.meta.main) {
  await main();
}
