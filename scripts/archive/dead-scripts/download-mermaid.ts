#!/usr/bin/env deno run --allow-net --allow-write

/**
 * Download Mermaid.js for the demo mode
 */

const MERMAID_VERSION = "10.6.1";
const MERMAID_URL =
  `https://cdn.jsdelivr.net/npm/mermaid@${MERMAID_VERSION}/dist/mermaid.min.js`;
const OUTPUT_PATH = "./assets/vendor/mermaid-10.6.1.min.js";

console.log(`📥 Downloading Mermaid.js v${MERMAID_VERSION}...`);

try {
  const response = await fetch(MERMAID_URL);
  if (!response.ok) {
    throw new Error(
      `Failed to download: ${response.status} ${response.statusText}`,
    );
  }

  const content = await response.text();

  // Ensure directory exists
  await Deno.mkdir("./assets/vendor", { recursive: true });

  // Write the file
  await Deno.writeTextFile(OUTPUT_PATH, content);

  // Get file size
  const fileInfo = await Deno.stat(OUTPUT_PATH);
  const sizeKB = Math.round(fileInfo.size / 1024);

  console.log(`✅ Downloaded Mermaid.js (${sizeKB} KB) to ${OUTPUT_PATH}`);
} catch (error) {
  console.error(`❌ Failed to download Mermaid.js: ${error.message}`);
  Deno.exit(1);
}
