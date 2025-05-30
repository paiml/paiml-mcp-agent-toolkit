#!/usr/bin/env deno run --allow-read --allow-run --allow-write

/**
 * Verify the binary size with demo mode included
 * Note: Demo mode is now always included for better developer experience
 */

import { join } from "https://deno.land/std@0.208.0/path/mod.ts";

async function runCommand(cmd: string[]): Promise<string> {
  const command = new Deno.Command(cmd[0], {
    args: cmd.slice(1),
    stdout: "piped",
    stderr: "piped",
  });
  const output = await command.output();
  if (!output.success) {
    throw new Error(
      `Command failed: ${cmd.join(" ")}\n${
        new TextDecoder().decode(output.stderr)
      }`,
    );
  }
  return new TextDecoder().decode(output.stdout);
}

async function getBinarySize(path: string): Promise<number> {
  try {
    const stat = await Deno.stat(path);
    return stat.size;
  } catch {
    return 0;
  }
}

async function main() {
  console.log("🔍 Checking release binary size with demo mode...\n");

  const serverDir = join(Deno.cwd(), "server");
  const binaryPath = join(serverDir, "target/release/paiml-mcp-agent-toolkit");

  // Clean previous builds
  console.log("🧹 Cleaning previous builds...");
  await runCommand([
    "cargo",
    "clean",
    "--manifest-path",
    join(serverDir, "Cargo.toml"),
  ]);

  // Build release (demo mode is now always included)
  console.log("📦 Building release binary...");
  await runCommand([
    "cargo",
    "build",
    "--release",
    "--manifest-path",
    join(serverDir, "Cargo.toml"),
  ]);
  const binarySize = await getBinarySize(binaryPath);
  console.log(`   Size: ${(binarySize / 1024 / 1024).toFixed(2)} MB`);

  console.log("\n📊 Results:");
  console.log(`   Binary size: ${(binarySize / 1024 / 1024).toFixed(2)} MB`);

  // Warn if binary is too large
  const maxSizeMB = 20; // 20MB threshold
  if (binarySize > maxSizeMB * 1024 * 1024) {
    console.error(`\n⚠️  Binary size exceeds ${maxSizeMB}MB threshold!`);
    console.error(
      "   Consider optimizing dependencies or build configuration.",
    );
  } else {
    console.log(
      `\n✅ Binary size is within acceptable limits (<${maxSizeMB}MB).`,
    );
  }

  // Check for demo functionality
  console.log("\n🔍 Verifying demo command is available...");
  try {
    const helpOutput = await runCommand([binaryPath, "--help"]);
    if (
      helpOutput.includes("demo") && helpOutput.includes("Run interactive demo")
    ) {
      console.log("✅ Demo command is available in release binary.");
    } else {
      console.error("❌ Demo command not found in help output!");
      Deno.exit(1);
    }
  } catch (e) {
    console.error("❌ Failed to run binary:", e);
    Deno.exit(1);
  }
}

if (import.meta.main) {
  main().catch(console.error);
}
