#!/usr/bin/env -S deno run --allow-read --allow-write --allow-run --allow-env

/**
 * MCP Server Universal Install Script
 *
 * A comprehensive installation script that handles:
 * - Automatic rebuild when source files change
 * - Clean uninstall and reinstall
 * - Installation status checking
 * - Force rebuild options
 *
 * Usage:
 *   ./scripts/mcp-install.ts              # Smart install (rebuild if needed)
 *   ./scripts/mcp-install.ts --status     # Check installation status
 *   ./scripts/mcp-install.ts --uninstall  # Uninstall only
 *   ./scripts/mcp-install.ts --reinstall  # Force uninstall and reinstall
 *   ./scripts/mcp-install.ts --force      # Force rebuild even if no changes
 *   ./scripts/mcp-install.ts --check      # Check if rebuild needed (no install)
 */

import { parse } from "https://deno.land/std@0.208.0/flags/mod.ts";
import { exists } from "https://deno.land/std@0.208.0/fs/mod.ts";
import { join } from "https://deno.land/std@0.208.0/path/mod.ts";
import { walk } from "https://deno.land/std@0.208.0/fs/walk.ts";

const args = parse(Deno.args, {
  boolean: [
    "help",
    "status",
    "uninstall",
    "reinstall",
    "force",
    "check",
    "clean",
  ],
  alias: {
    h: "help",
    s: "status",
    u: "uninstall",
    r: "reinstall",
    f: "force",
    c: "check",
  },
});

interface InstallLocation {
  path: string;
  type: "system" | "user";
}

interface BuildInfo {
  lastBuildTime?: number;
  sourceHash?: string;
}

const BINARY_NAME = "paiml-mcp-agent-toolkit";
const SERVER_DIR = join(Deno.cwd(), "server");
const SOURCE_PATTERNS = ["**/*.rs", "**/*.hbs", "**/*.json"];
const IGNORED_DIRS = ["target", "node_modules", ".git"];
const INSTALL_LOCATIONS: InstallLocation[] = [
  { path: "/usr/local/bin", type: "system" },
  { path: join(Deno.env.get("HOME") || "", ".local/bin"), type: "user" },
];

async function runCommand(
  command: string,
  args: string[],
  options?: { cwd?: string; silent?: boolean },
): Promise<{ success: boolean; stdout: string; stderr: string }> {
  try {
    const cmd = new Deno.Command(command, {
      args,
      cwd: options?.cwd,
      stdout: "piped",
      stderr: "piped",
    });

    const { success, stdout, stderr } = await cmd.output();
    const stdoutStr = new TextDecoder().decode(stdout);
    const stderrStr = new TextDecoder().decode(stderr);

    if (!options?.silent) {
      if (stdoutStr) console.log(stdoutStr);
      if (stderrStr && !success) console.error(stderrStr);
    }

    return { success, stdout: stdoutStr, stderr: stderrStr };
  } catch (error) {
    return {
      success: false,
      stdout: "",
      stderr: error instanceof Error ? error.message : String(error),
    };
  }
}

async function getSourceFilesHash(): Promise<string> {
  const encoder = new TextEncoder();
  let combinedContent = "";

  // Walk through server directory and collect source files
  for await (
    const entry of walk(SERVER_DIR, {
      includeFiles: true,
      includeDirs: false,
      skip: IGNORED_DIRS.map((d) => new RegExp(d)),
    })
  ) {
    const relativePath = entry.path.replace(SERVER_DIR + "/", "");

    // Check if file matches our patterns
    const isSourceFile = SOURCE_PATTERNS.some((pattern) => {
      const regex = new RegExp(
        pattern.replace("**", ".*").replace("*", "[^/]*"),
      );
      return regex.test(relativePath);
    });

    if (isSourceFile) {
      const stat = await Deno.stat(entry.path);
      combinedContent += `${relativePath}:${stat.mtime?.getTime() || 0}\n`;
    }
  }

  // Create a simple hash from the content
  const data = encoder.encode(combinedContent);
  const hashBuffer = await crypto.subtle.digest("SHA-256", data);
  const hashArray = Array.from(new Uint8Array(hashBuffer));
  return hashArray.map((b) => b.toString(16).padStart(2, "0")).join("");
}

async function getBuildInfo(): Promise<BuildInfo> {
  const buildInfoPath = join(SERVER_DIR, ".build-info.json");

  try {
    if (await exists(buildInfoPath)) {
      const content = await Deno.readTextFile(buildInfoPath);
      return JSON.parse(content);
    }
  } catch {
    // Ignore errors
  }

  return {};
}

async function saveBuildInfo(info: BuildInfo): Promise<void> {
  const buildInfoPath = join(SERVER_DIR, ".build-info.json");
  await Deno.writeTextFile(buildInfoPath, JSON.stringify(info, null, 2));
}

async function findInstalledBinary(): Promise<string | null> {
  // Check standard installation locations
  for (const location of INSTALL_LOCATIONS) {
    const binaryPath = join(location.path, BINARY_NAME);
    if (await exists(binaryPath)) {
      return binaryPath;
    }
  }

  // Check if binary is in PATH
  const { success, stdout } = await runCommand("which", [BINARY_NAME], {
    silent: true,
  });
  if (success && stdout.trim()) {
    return stdout.trim();
  }

  return null;
}

async function checkIfRebuildNeeded(): Promise<boolean> {
  // Check if binary exists
  const releaseBinary = join(SERVER_DIR, "target/release", BINARY_NAME);
  if (!await exists(releaseBinary)) {
    console.log("⚠️  Binary not found, rebuild needed.");
    return true;
  }

  // Get current source hash
  const currentHash = await getSourceFilesHash();
  const buildInfo = await getBuildInfo();

  if (buildInfo.sourceHash !== currentHash) {
    console.log("🔍 Source files have changed since last build.");
    return true;
  }

  // Check binary modification time
  const binaryStat = await Deno.stat(releaseBinary);
  const binaryTime = binaryStat.mtime?.getTime() || 0;

  if (buildInfo.lastBuildTime && binaryTime < buildInfo.lastBuildTime) {
    console.log("⚠️  Binary is older than expected, rebuild needed.");
    return true;
  }

  console.log("✅ Binary is up to date.");
  return false;
}

async function checkInstallationStatus(): Promise<void> {
  console.log("🔍 Checking MCP server installation status...\n");

  const installedPath = await findInstalledBinary();

  if (installedPath) {
    console.log(`✅ MCP server is installed at: ${installedPath}`);

    // Check version if possible
    const { success, stdout } = await runCommand(installedPath, ["--version"], {
      silent: true,
    });
    if (success && stdout.trim()) {
      console.log(`📌 Version: ${stdout.trim()}`);
    }

    // Check if it's in PATH
    const { success: inPath } = await runCommand("which", [BINARY_NAME], {
      silent: true,
    });
    if (!inPath) {
      console.log(
        "⚠️  Binary is not in PATH. You may need to add its directory to PATH.",
      );
    }
  } else {
    console.log("❌ MCP server is not installed.");
  }

  // Check build artifacts
  const releaseBinary = join(SERVER_DIR, "target/release", BINARY_NAME);

  if (await exists(releaseBinary)) {
    console.log(`\n📦 Build artifact found at: ${releaseBinary}`);

    // Check if rebuild needed
    const needsRebuild = await checkIfRebuildNeeded();
    if (needsRebuild) {
      console.log("🔄 Source files have changed - rebuild recommended.");
    }
  } else {
    console.log(
      "\n📦 No build artifacts found. Run 'make build' in server/ to build.",
    );
  }

  // Check Claude Code configuration
  const configPaths = [
    join(
      Deno.env.get("HOME") || "",
      ".config/claude/claude_desktop_config.json",
    ),
    join(Deno.env.get("HOME") || "", ".config/claude/claude.json"),
  ];

  console.log("\n🔧 Claude Code configuration:");
  for (const configPath of configPaths) {
    if (await exists(configPath)) {
      console.log(`   Found: ${configPath}`);
      try {
        const config = await Deno.readTextFile(configPath);
        if (
          config.includes("paiml-mcp-agent-toolkit") ||
          config.includes(BINARY_NAME)
        ) {
          console.log("   ✅ MCP server is configured in Claude Code");
        } else {
          console.log("   ⚠️  MCP server not found in Claude Code config");
        }
      } catch {
        console.log("   ⚠️  Could not read config file");
      }
    }
  }
}

async function uninstallMcpServer(): Promise<boolean> {
  console.log("🗑️  Uninstalling MCP server...\n");

  let uninstalled = false;

  // Find and remove installed binary
  const installedPath = await findInstalledBinary();

  if (installedPath) {
    console.log(`Found installation at: ${installedPath}`);

    try {
      await Deno.remove(installedPath);
      console.log(`✅ Removed binary: ${installedPath}`);
      uninstalled = true;
    } catch (error) {
      if (error instanceof Deno.errors.PermissionDenied) {
        console.log(`⚠️  Permission denied. Trying with sudo...`);
        const { success } = await runCommand("sudo", [
          "rm",
          "-f",
          installedPath,
        ]);
        if (success) {
          console.log(`✅ Removed binary with sudo: ${installedPath}`);
          uninstalled = true;
        } else {
          console.error(`❌ Failed to remove binary: ${installedPath}`);
          return false;
        }
      } else {
        console.error(`❌ Error removing binary: ${error}`);
        return false;
      }
    }
  } else {
    console.log("ℹ️  No installed binary found.");
  }

  // Clean up any additional locations
  for (const location of INSTALL_LOCATIONS) {
    const binaryPath = join(location.path, BINARY_NAME);
    if (await exists(binaryPath)) {
      try {
        await Deno.remove(binaryPath);
        console.log(`✅ Cleaned up: ${binaryPath}`);
        uninstalled = true;
      } catch {
        // Ignore errors for additional cleanup
      }
    }
  }

  // Clean build artifacts if requested
  if (args.clean) {
    console.log("\n🧹 Cleaning build artifacts (--clean flag)...");
    const { success } = await runCommand("make", ["clean"], {
      cwd: SERVER_DIR,
    });
    if (success) {
      console.log("✅ Build artifacts cleaned");
    }
  }

  if (uninstalled) {
    console.log("\n✅ MCP server uninstalled successfully!");
  } else {
    console.log("\n✅ MCP server was not installed (nothing to uninstall).");
  }

  return true;
}

async function buildAndInstall(): Promise<boolean> {
  console.log("🔨 Building MCP server...");

  // Build the binary
  const buildResult = await runCommand("cargo", [
    "build",
    "--release",
    "--bin",
    BINARY_NAME,
  ], {
    cwd: SERVER_DIR,
  });

  if (!buildResult.success) {
    console.error("❌ Build failed. Please check the error messages above.");
    return false;
  }

  // Save build info
  const currentHash = await getSourceFilesHash();
  await saveBuildInfo({
    lastBuildTime: Date.now(),
    sourceHash: currentHash,
  });

  // Install the binary
  console.log("\n📦 Installing binary...");
  const installResult = await runCommand("make", ["install"], {
    cwd: SERVER_DIR,
  });

  if (!installResult.success) {
    console.error("❌ Installation failed. Trying with cargo install...");

    // Fallback to cargo install
    const cargoInstallResult = await runCommand("cargo", [
      "install",
      "--path",
      ".",
      "--bin",
      BINARY_NAME,
      "--force",
    ], { cwd: SERVER_DIR });

    if (!cargoInstallResult.success) {
      console.error("❌ Cargo install also failed.");
      return false;
    }
  }

  console.log("\n✅ MCP server installed successfully!");

  // Verify installation
  const { success, stdout } = await runCommand(BINARY_NAME, ["--version"], {
    silent: true,
  });
  if (success && stdout.trim()) {
    console.log(`📌 Installed version: ${stdout.trim()}`);
  }

  return true;
}

async function smartInstall(): Promise<boolean> {
  console.log("🚀 MCP Server Smart Install\n");

  // Check if rebuild is needed
  const needsRebuild = args.force || await checkIfRebuildNeeded();

  if (!needsRebuild) {
    console.log("\n✨ No changes detected, binary is already up to date!");

    // Still check if it's installed in the system
    const installedPath = await findInstalledBinary();
    if (!installedPath) {
      console.log("⚠️  But binary is not installed in system. Installing...");
      const installResult = await runCommand("make", ["install"], {
        cwd: SERVER_DIR,
      });
      if (!installResult.success) {
        console.error("❌ Installation failed.");
        return false;
      }
    } else {
      const { success } = await runCommand("which", [BINARY_NAME], {
        silent: true,
      });
      if (!success) {
        console.log(
          "⚠️  But binary is not in PATH. You may need to add its directory to PATH.",
        );
      }
    }

    return true;
  }

  if (args.force) {
    console.log("🔄 Force rebuild requested.\n");
  }

  return await buildAndInstall();
}

function showHelp(): void {
  console.log(`
MCP Server Universal Install Script

A comprehensive installation script that automatically rebuilds when source
files change and handles all installation scenarios.

Usage:
  ./scripts/mcp-install.ts              # Smart install (rebuild if needed)
  ./scripts/mcp-install.ts --status     # Check installation status
  ./scripts/mcp-install.ts --uninstall  # Uninstall only
  ./scripts/mcp-install.ts --reinstall  # Force uninstall and reinstall
  ./scripts/mcp-install.ts --force      # Force rebuild even if no changes
  ./scripts/mcp-install.ts --check      # Check if rebuild needed (no install)

Options:
  -h, --help       Show this help message
  -s, --status     Check installation status
  -u, --uninstall  Uninstall MCP server only
  -r, --reinstall  Force complete reinstall
  -f, --force      Force rebuild even if sources unchanged
  -c, --check      Check if rebuild needed without installing
      --clean      Clean build artifacts during uninstall

The script monitors:
  - Rust source files (*.rs)
  - Template files (*.hbs) 
  - Template metadata (*.json)

And automatically rebuilds when any of these change.

Examples:
  # Normal usage - install/update as needed
  ./scripts/mcp-install.ts
  
  # Check current status
  ./scripts/mcp-install.ts --status
  
  # Clean reinstall
  ./scripts/mcp-install.ts --reinstall --clean
  
  # Just check if update needed
  ./scripts/mcp-install.ts --check
`);
}

async function main(): Promise<void> {
  if (args.help) {
    showHelp();
    Deno.exit(0);
  }

  // Check if we're in the right directory
  const cargoFile = join(SERVER_DIR, "Cargo.toml");
  if (!await exists(cargoFile)) {
    console.error(
      "❌ Error: This script must be run from the project root directory.",
    );
    console.error("   Current directory:", Deno.cwd());
    Deno.exit(1);
  }

  // Handle various command modes
  if (args.status) {
    await checkInstallationStatus();
    Deno.exit(0);
  }

  if (args.check) {
    const needsRebuild = await checkIfRebuildNeeded();
    console.log(`\nRebuild needed: ${needsRebuild ? "Yes" : "No"}`);
    Deno.exit(needsRebuild ? 1 : 0);
  }

  if (args.uninstall) {
    const success = await uninstallMcpServer();
    Deno.exit(success ? 0 : 1);
  }

  if (args.reinstall) {
    console.log("🔄 Performing complete reinstall...\n");

    const uninstallSuccess = await uninstallMcpServer();
    if (!uninstallSuccess) {
      console.error("\n❌ Uninstall failed. Aborting reinstall.");
      Deno.exit(1);
    }

    console.log("\n" + "=".repeat(60) + "\n");

    // Force rebuild for reinstall
    args.force = true;
    const installSuccess = await buildAndInstall();
    if (!installSuccess) {
      console.error("\n❌ Install failed.");
      Deno.exit(1);
    }

    console.log("\n✅ MCP server reinstalled successfully!");
  } else {
    // Default: smart install
    const success = await smartInstall();

    if (success) {
      console.log("\n✅ Done! MCP server is ready to use.");
      console.log("\n💡 Next steps:");
      console.log("   1. Restart Claude Code to use the updated server");
      console.log("   2. Test with: paiml-mcp-agent-toolkit --version");
    }

    Deno.exit(success ? 0 : 1);
  }
}

if (import.meta.main) {
  main();
}
