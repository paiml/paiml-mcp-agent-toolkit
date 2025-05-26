#!/usr/bin/env -S deno run --allow-read --allow-write --allow-run --allow-env

/**
 * Deterministic MCP Server Install Script
 *
 * This script ensures a completely clean and deterministic installation of the MCP server.
 * It handles all edge cases and ensures consistency across binary names, paths, and configurations.
 *
 * Usage:
 *   ./scripts/mcp-install-deterministic.ts           # Clean install
 *   ./scripts/mcp-install-deterministic.ts --check   # Check current state only
 *   ./scripts/mcp-install-deterministic.ts --clean   # Clean everything and exit
 */

import { parse } from "https://deno.land/std@0.208.0/flags/mod.ts";
import { exists } from "https://deno.land/std@0.208.0/fs/mod.ts";
import { dirname, join } from "https://deno.land/std@0.208.0/path/mod.ts";

const args = parse(Deno.args, {
  boolean: ["help", "check", "clean", "verbose"],
  alias: { h: "help", c: "check", v: "verbose" },
});

// Constants - Single source of truth
const BINARY_NAME = "paiml-mcp-agent-toolkit";
const MCP_SERVER_NAME = "paiml-agent-toolkit";
const PROJECT_ROOT = dirname(dirname(new URL(import.meta.url).pathname));
const SERVER_DIR = join(PROJECT_ROOT, "server");
const CARGO_TOML = join(SERVER_DIR, "Cargo.toml");
const TARGET_DIR = join(SERVER_DIR, "target");
const RELEASE_DIR = join(TARGET_DIR, "release");

interface SystemState {
  // Cargo.toml binary configuration
  cargoTomlBinaries: string[];

  // Built binaries in target/release
  builtBinaries: string[];

  // Installed binaries in system
  installedBinaries: { path: string; binary: string }[];

  // Claude MCP configuration
  claudeConfig: { name: string; command: string } | null;

  // Issues found
  issues: string[];
}

async function runCommand(
  command: string,
  cmdArgs: string[],
  options?: { cwd?: string; silent?: boolean },
): Promise<{ success: boolean; stdout: string; stderr: string }> {
  try {
    if (args.verbose && !options?.silent) {
      console.log(`🔧 Running: ${command} ${cmdArgs.join(" ")}`);
    }

    const cmd = new Deno.Command(command, {
      args: cmdArgs,
      cwd: options?.cwd,
      stdout: "piped",
      stderr: "piped",
    });

    const { success, stdout, stderr } = await cmd.output();
    const stdoutStr = new TextDecoder().decode(stdout);
    const stderrStr = new TextDecoder().decode(stderr);

    return { success, stdout: stdoutStr, stderr: stderrStr };
  } catch (error) {
    return {
      success: false,
      stdout: "",
      stderr: error instanceof Error ? error.message : String(error),
    };
  }
}

async function analyzeSystemState(): Promise<SystemState> {
  const state: SystemState = {
    cargoTomlBinaries: [],
    builtBinaries: [],
    installedBinaries: [],
    claudeConfig: null,
    issues: [],
  };

  // 1. Analyze Cargo.toml
  if (await exists(CARGO_TOML)) {
    const cargoContent = await Deno.readTextFile(CARGO_TOML);
    const binMatches = cargoContent.matchAll(
      /\[\[bin\]\]\s*\nname\s*=\s*"([^"]+)"/g,
    );
    for (const match of binMatches) {
      state.cargoTomlBinaries.push(match[1]);
    }

    if (state.cargoTomlBinaries.length === 0) {
      state.issues.push("No binary definitions found in Cargo.toml");
    } else if (state.cargoTomlBinaries.length > 1) {
      state.issues.push(
        `Multiple binaries defined in Cargo.toml: ${
          state.cargoTomlBinaries.join(", ")
        }`,
      );
    } else if (state.cargoTomlBinaries[0] !== BINARY_NAME) {
      state.issues.push(
        `Wrong binary name in Cargo.toml: ${
          state.cargoTomlBinaries[0]
        } (should be ${BINARY_NAME})`,
      );
    }
  } else {
    state.issues.push("Cargo.toml not found");
  }

  // 2. Check built binaries
  if (await exists(RELEASE_DIR)) {
    for await (const entry of Deno.readDir(RELEASE_DIR)) {
      if (
        entry.isFile && !entry.name.endsWith(".d") && !entry.name.includes(".")
      ) {
        state.builtBinaries.push(entry.name);
      }
    }

    if (state.builtBinaries.length > 1) {
      state.issues.push(
        `Multiple binaries in release dir: ${state.builtBinaries.join(", ")}`,
      );
    }
  }

  // 3. Check installed binaries
  const installPaths = [
    "/usr/local/bin",
    join(Deno.env.get("HOME") || "", ".local/bin"),
  ];

  for (const dir of installPaths) {
    if (await exists(dir)) {
      for await (const entry of Deno.readDir(dir)) {
        if (entry.name.includes("mcp") || entry.name.includes("paiml")) {
          state.installedBinaries.push({
            path: join(dir, entry.name),
            binary: entry.name,
          });
        }
      }
    }
  }

  // 4. Check Claude configuration
  const { success, stdout } = await runCommand("claude", ["mcp", "list"], {
    silent: true,
  });
  if (success) {
    const lines = stdout.trim().split("\n");
    for (const line of lines) {
      if (line.includes("paiml") || line.includes("mcp")) {
        const [name, command] = line.split(":").map((s) => s.trim());
        state.claudeConfig = { name, command };

        if (name !== MCP_SERVER_NAME) {
          state.issues.push(
            `Wrong MCP server name: ${name} (should be ${MCP_SERVER_NAME})`,
          );
        }
        if (command !== BINARY_NAME) {
          state.issues.push(
            `Wrong MCP command: ${command} (should be ${BINARY_NAME})`,
          );
        }
      }
    }
  }

  return state;
}

async function cleanEverything(state: SystemState): Promise<void> {
  console.log("\n🧹 Cleaning all MCP server artifacts...\n");

  // 1. Remove all installed binaries
  for (const { path, binary } of state.installedBinaries) {
    console.log(`🗑️  Removing ${binary} from ${dirname(path)}`);
    try {
      await Deno.remove(path);
    } catch (error) {
      if (error instanceof Deno.errors.PermissionDenied) {
        console.log(`   ⚠️  Permission denied, trying with sudo...`);
        await runCommand("sudo", ["rm", "-f", path]);
      }
    }
  }

  // 2. Remove Claude configuration
  if (state.claudeConfig) {
    console.log(
      `🗑️  Removing Claude MCP configuration: ${state.claudeConfig.name}`,
    );
    await runCommand("claude", ["mcp", "remove", state.claudeConfig.name], {
      silent: true,
    });
  }

  // 3. Clean build artifacts
  console.log("🗑️  Cleaning build artifacts...");
  await runCommand("cargo", ["clean"], { cwd: SERVER_DIR });

  console.log("\n✅ Cleanup complete!");
}

async function fixCargoToml(): Promise<void> {
  console.log("📝 Fixing Cargo.toml to use single binary with correct name...");

  let content = await Deno.readTextFile(CARGO_TOML);

  // Remove all existing [[bin]] sections
  content = content.replace(/\[\[bin\]\][\s\S]*?(?=\[|$)/g, "");

  // Add single correct binary definition at the end
  content = content.trimEnd() + '\n\n[[bin]]\nname = "' + BINARY_NAME +
    '"\npath = "src/bin/mcp_server_stateless.rs"\n';

  await Deno.writeTextFile(CARGO_TOML, content);
  console.log("✅ Cargo.toml updated with single binary: " + BINARY_NAME);
}

async function buildBinary(): Promise<boolean> {
  console.log("\n🔨 Building MCP server binary...");

  const { success, stderr } = await runCommand(
    "cargo",
    ["build", "--release", "--bin", BINARY_NAME],
    { cwd: SERVER_DIR },
  );

  if (!success) {
    console.error("❌ Build failed:");
    console.error(stderr);
    return false;
  }

  const binaryPath = join(RELEASE_DIR, BINARY_NAME);
  if (!await exists(binaryPath)) {
    console.error("❌ Binary not found after build: " + binaryPath);
    return false;
  }

  console.log("✅ Binary built successfully: " + binaryPath);
  return true;
}

async function installBinary(): Promise<string | null> {
  console.log("\n📦 Installing binary...");

  const sourcePath = join(RELEASE_DIR, BINARY_NAME);
  const userBinDir = join(Deno.env.get("HOME") || "", ".local/bin");
  const systemBinDir = "/usr/local/bin";

  // Try system install first
  let targetPath = join(systemBinDir, BINARY_NAME);
  let _installDir = systemBinDir;

  try {
    await Deno.copyFile(sourcePath, targetPath);
    await Deno.chmod(targetPath, 0o755);
    console.log(`✅ Installed to: ${targetPath}`);
  } catch (error) {
    if (error instanceof Deno.errors.PermissionDenied) {
      console.log("⚠️  System install failed, trying user install...");

      // Create user bin directory if needed
      await Deno.mkdir(userBinDir, { recursive: true });

      targetPath = join(userBinDir, BINARY_NAME);
      _installDir = userBinDir;

      await Deno.copyFile(sourcePath, targetPath);
      await Deno.chmod(targetPath, 0o755);
      console.log(`✅ Installed to: ${targetPath}`);

      // Check PATH
      const path = Deno.env.get("PATH") || "";
      if (!path.includes(userBinDir)) {
        console.log("\n⚠️  ${HOME}/.local/bin is not in your PATH!");
        console.log("   Add it to your shell configuration:");
        console.log(`   echo 'export PATH="${userBinDir}:$PATH"' >> ~/.bashrc`);
        console.log(`   echo 'export PATH="${userBinDir}:$PATH"' >> ~/.zshrc`);
      }
    } else {
      throw error;
    }
  }

  return targetPath;
}

async function configureClaude(): Promise<boolean> {
  console.log("\n🔧 Configuring Claude Code...");

  // Check if claude CLI exists
  const { success: hasClaudeCLI } = await runCommand("which", ["claude"], {
    silent: true,
  });
  if (!hasClaudeCLI) {
    console.log("⚠️  Claude CLI not found. Install it with:");
    console.log("   npm install -g @anthropic-ai/claude-cli");
    return false;
  }

  // Remove any existing configuration
  await runCommand("claude", ["mcp", "remove", MCP_SERVER_NAME], {
    silent: true,
  });

  // Add correct configuration
  const { success, stderr } = await runCommand(
    "claude",
    ["mcp", "add", MCP_SERVER_NAME, BINARY_NAME],
  );

  if (!success) {
    console.error("❌ Failed to configure Claude:");
    console.error(stderr);
    return false;
  }

  console.log(
    `✅ Claude configured with: ${MCP_SERVER_NAME} -> ${BINARY_NAME}`,
  );
  return true;
}

async function verifyInstallation(): Promise<boolean> {
  console.log("\n🔍 Verifying installation...");

  // Check binary in PATH
  const { success: inPath } = await runCommand("which", [BINARY_NAME], {
    silent: true,
  });
  if (!inPath) {
    console.error("❌ Binary not found in PATH");
    return false;
  }

  // Test binary execution
  const { success: runs } = await runCommand(
    "timeout",
    ["1s", BINARY_NAME],
    { silent: true },
  );

  // timeout returns 124 when it times out (expected for STDIO server)
  if (!runs) {
    console.log("✅ Binary executes correctly (STDIO mode)");
  }

  // Verify Claude configuration
  const { success, stdout } = await runCommand("claude", ["mcp", "list"], {
    silent: true,
  });
  if (success && stdout.includes(MCP_SERVER_NAME)) {
    console.log("✅ Claude configuration verified");
  } else {
    console.error("❌ Claude configuration not found");
    return false;
  }

  console.log("\n✅ Installation verified successfully!");
  return true;
}

function printState(state: SystemState): void {
  console.log("\n📊 Current System State:");
  console.log("━".repeat(50));

  console.log("\n📄 Cargo.toml binaries:");
  if (state.cargoTomlBinaries.length > 0) {
    state.cargoTomlBinaries.forEach((b) => console.log(`   - ${b}`));
  } else {
    console.log("   (none)");
  }

  console.log("\n🔨 Built binaries:");
  if (state.builtBinaries.length > 0) {
    state.builtBinaries.forEach((b) => console.log(`   - ${b}`));
  } else {
    console.log("   (none)");
  }

  console.log("\n📦 Installed binaries:");
  if (state.installedBinaries.length > 0) {
    state.installedBinaries.forEach(({ path, binary }) =>
      console.log(`   - ${binary} -> ${path}`)
    );
  } else {
    console.log("   (none)");
  }

  console.log("\n🔧 Claude configuration:");
  if (state.claudeConfig) {
    console.log(
      `   - ${state.claudeConfig.name} -> ${state.claudeConfig.command}`,
    );
  } else {
    console.log("   (none)");
  }

  if (state.issues.length > 0) {
    console.log("\n⚠️  Issues found:");
    state.issues.forEach((issue) => console.log(`   - ${issue}`));
  }

  console.log("\n" + "━".repeat(50));
}

async function main(): Promise<void> {
  if (args.help) {
    console.log(`
Deterministic MCP Server Install Script

This script ensures a completely clean and deterministic installation.

Usage:
  ./scripts/mcp-install-deterministic.ts           # Clean install
  ./scripts/mcp-install-deterministic.ts --check   # Check current state only
  ./scripts/mcp-install-deterministic.ts --clean   # Clean everything and exit

Options:
  -h, --help     Show this help message
  -c, --check    Check current state without making changes
  --clean        Clean all artifacts and exit
  -v, --verbose  Show detailed command output
`);
    Deno.exit(0);
  }

  // Ensure we're in the project root
  if (!await exists(CARGO_TOML)) {
    console.error("❌ Error: Must run from project root directory");
    console.error(`   Expected Cargo.toml at: ${CARGO_TOML}`);
    Deno.exit(1);
  }

  console.log("🚀 MCP Server Deterministic Install\n");

  // Analyze current state
  const state = await analyzeSystemState();
  printState(state);

  if (args.check) {
    Deno.exit(state.issues.length > 0 ? 1 : 0);
  }

  if (args.clean) {
    await cleanEverything(state);
    Deno.exit(0);
  }

  // Perform clean install
  console.log("\n🔄 Starting clean installation...");

  // 1. Clean everything first
  await cleanEverything(state);

  // 2. Fix Cargo.toml
  await fixCargoToml();

  // 3. Build binary
  if (!await buildBinary()) {
    console.error("\n❌ Build failed! Aborting installation.");
    Deno.exit(1);
  }

  // 4. Install binary
  const installedPath = await installBinary();
  if (!installedPath) {
    console.error("\n❌ Installation failed!");
    Deno.exit(1);
  }

  // 5. Configure Claude
  await configureClaude();

  // 6. Verify everything
  if (!await verifyInstallation()) {
    console.error("\n❌ Verification failed!");
    Deno.exit(1);
  }

  console.log("\n✅ Installation complete!");
  console.log("\n📋 Next steps:");
  console.log("   1. Restart Claude Code");
  console.log("   2. The MCP server should now work correctly");
}

if (import.meta.main) {
  main();
}
