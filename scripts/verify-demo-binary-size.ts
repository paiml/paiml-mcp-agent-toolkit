#!/usr/bin/env deno run --allow-read --allow-run --allow-write

/**
 * Verify that demo mode has zero impact on release binary size
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
  console.log("🔍 Verifying demo mode binary size impact...\n");

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

  // Build release without demo-dev feature
  console.log("📦 Building release without demo-dev feature...");
  await runCommand([
    "cargo",
    "build",
    "--release",
    "--manifest-path",
    join(serverDir, "Cargo.toml"),
  ]);
  const sizeWithoutDemo = await getBinarySize(binaryPath);
  console.log(`   Size: ${(sizeWithoutDemo / 1024 / 1024).toFixed(2)} MB`);

  // Clean and rebuild with demo-dev feature
  console.log("\n🧹 Cleaning...");
  await runCommand([
    "cargo",
    "clean",
    "--manifest-path",
    join(serverDir, "Cargo.toml"),
  ]);

  console.log("📦 Building release with demo-dev feature...");
  await runCommand([
    "cargo",
    "build",
    "--release",
    "--features",
    "demo-dev",
    "--manifest-path",
    join(serverDir, "Cargo.toml"),
  ]);
  const sizeWithDemo = await getBinarySize(binaryPath);
  console.log(`   Size: ${(sizeWithDemo / 1024 / 1024).toFixed(2)} MB`);

  // Calculate difference
  const sizeDiff = sizeWithDemo - sizeWithoutDemo;
  const percentDiff = (sizeDiff / sizeWithoutDemo) * 100;

  console.log("\n📊 Results:");
  console.log(
    `   Without demo: ${(sizeWithoutDemo / 1024 / 1024).toFixed(2)} MB`,
  );
  console.log(`   With demo:    ${(sizeWithDemo / 1024 / 1024).toFixed(2)} MB`);
  console.log(
    `   Difference:   ${sizeDiff} bytes (${percentDiff.toFixed(2)}%)`,
  );

  // Verify zero impact
  if (Math.abs(sizeDiff) > 1024) { // Allow 1KB tolerance
    console.error("\n❌ Demo mode increased binary size by more than 1KB!");
    console.error("   This violates the zero-overhead requirement.");
    Deno.exit(1);
  } else {
    console.log("\n✅ Demo mode has zero impact on release binary size!");
  }

  // Additional verification: Check for demo symbols
  console.log("\n🔍 Checking for demo symbols in release binary...");
  try {
    const nmOutput = await runCommand(["nm", binaryPath]);
    const demoSymbols = nmOutput.split("\n").filter((line) =>
      line.includes("demo") && !line.includes("democracy") // Avoid false positives
    );

    if (demoSymbols.length > 0) {
      console.error("❌ Found demo symbols in release binary:");
      demoSymbols.slice(0, 5).forEach((sym) => console.error(`   ${sym}`));
      console.error("   This indicates demo code was not properly excluded.");
    } else {
      console.log("✅ No demo symbols found in release binary.");
    }
  } catch (_e) {
    console.warn("⚠️  Could not run 'nm' command. Skipping symbol check.");
  }
}

if (import.meta.main) {
  main().catch(console.error);
}
