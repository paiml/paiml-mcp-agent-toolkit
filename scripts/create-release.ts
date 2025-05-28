#!/usr/bin/env -S deno run --allow-read --allow-write --allow-env --allow-run --allow-net

/**
 * Script to create a GitHub release with binary artifacts
 */

// Colors
const RED = "\x1b[31m";
const GREEN = "\x1b[32m";
const YELLOW = "\x1b[33m";
const NC = "\x1b[0m";

async function getVersion(): Promise<string> {
  const cargoToml = await Deno.readTextFile("server/Cargo.toml");
  const match = cargoToml.match(/^version = "(.*)"/m);
  if (!match) {
    throw new Error("Could not find version in Cargo.toml");
  }
  return match[1];
}

async function runCommand(
  cmd: string,
  args: string[],
  options?: { cwd?: string },
): Promise<{ success: boolean; output: string; error: string }> {
  const command = new Deno.Command(cmd, {
    args,
    cwd: options?.cwd,
    stdout: "piped",
    stderr: "piped",
  });

  const { code, stdout, stderr } = await command.output();

  return {
    success: code === 0,
    output: new TextDecoder().decode(stdout),
    error: new TextDecoder().decode(stderr),
  };
}

async function checkGhCli(): Promise<boolean> {
  try {
    const { success } = await runCommand("gh", ["--version"]);
    return success;
  } catch {
    return false;
  }
}

function isRunningInGitHubActions(): boolean {
  return Deno.env.get("GITHUB_ACTIONS") === "true";
}

function detectPlatform(): { os: string; arch: string } {
  const osMap: Record<string, string> = {
    "linux": "linux",
    "darwin": "macos",
    "windows": "windows",
  };

  const archMap: Record<string, string> = {
    "x86_64": "x86_64",
    "aarch64": "aarch64",
  };

  const os = osMap[Deno.build.os];
  const arch = archMap[Deno.build.arch];

  if (!os || !arch) {
    throw new Error(
      `Unsupported platform: ${Deno.build.os}-${Deno.build.arch}`,
    );
  }

  return { os, arch };
}

async function createTarball(
  binaryName: string,
  os: string,
  arch: string,
): Promise<string> {
  const tarballName = `${binaryName}-${os}-${arch}.tar.gz`;

  const { success, error } = await runCommand("tar", [
    "-czf",
    tarballName,
    "-C",
    "server/target/release",
    binaryName,
  ]);

  if (!success) {
    throw new Error(`Failed to create tarball: ${error}`);
  }

  return tarballName;
}

async function main(): Promise<void> {
  try {
    // Get version
    const version = await getVersion();
    console.log(`${GREEN}Creating release for v${version}${NC}`);

    // Check if gh CLI is installed
    if (!await checkGhCli()) {
      console.error(`${RED}Error: GitHub CLI (gh) is not installed${NC}`);
      console.error("Install it from: https://cli.github.com/");
      Deno.exit(1);
    }

    // Check if we're in the right directory
    try {
      await Deno.stat("server/Cargo.toml");
    } catch {
      console.error(`${RED}Error: Must run from repository root${NC}`);
      Deno.exit(1);
    }

    // Build release binary using make
    console.log(`${YELLOW}Building release binary...${NC}`);
    const { success: buildSuccess, error: buildError } = await runCommand(
      "make",
      ["server-build-binary"],
    );
    if (!buildSuccess) {
      console.error(`${RED}Build failed: ${buildError}${NC}`);
      Deno.exit(1);
    }

    // Detect platform
    const { os, arch } = detectPlatform();

    // Create tarball
    const binaryName = "paiml-mcp-agent-toolkit";
    const tarball = await createTarball(binaryName, os, arch);
    console.log(`${YELLOW}Created tarball: ${tarball}${NC}`);

    // Create release notes
    const releaseNotes = `## MCP Agent Toolkit v${version}

### Installation

Install using curl:
\`\`\`bash
curl -fsSL https://raw.githubusercontent.com/paiml/paiml-mcp-agent-toolkit/master/scripts/install.ts | deno run --allow-net --allow-read --allow-write --allow-env --allow-run -
\`\`\`

Or with the shell wrapper:
\`\`\`bash
curl -fsSL https://raw.githubusercontent.com/paiml/paiml-mcp-agent-toolkit/master/scripts/install.sh | sh
\`\`\`

Or download the binary directly from the assets below.

### What's New
- Template generation for Makefiles, README files, and .gitignore files
- Support for Rust, Deno/TypeScript, and Python UV toolchains
- Stateless MCP server with embedded templates
- Claude Code integration ready

### Supported Platforms
- Linux x86_64
- macOS x86_64
- macOS aarch64 (Apple Silicon)
`;

    // Create GitHub release
    console.log(`${YELLOW}Creating GitHub release...${NC}`);
    const { success: releaseSuccess, error: releaseError } = await runCommand(
      "gh",
      [
        "release",
        "create",
        `v${version}`,
        "--title",
        `v${version}`,
        "--notes",
        releaseNotes,
        tarball,
      ],
    );

    if (releaseSuccess) {
      console.log(`${GREEN}✅ Release created successfully!${NC}`);
      console.log(
        `${GREEN}View at: https://github.com/paiml/paiml-mcp-agent-toolkit/releases/tag/v${version}${NC}`,
      );
    } else {
      console.error(`${RED}Failed to create release: ${releaseError}${NC}`);
      Deno.exit(1);
    }

    // Cleanup (unless in GitHub Actions where we might need the artifact)
    if (!isRunningInGitHubActions()) {
      await Deno.remove(tarball);
    }

    console.log(`${GREEN}Done! Users can now install with:${NC}`);
    console.log(
      "curl -fsSL https://raw.githubusercontent.com/paiml/paiml-mcp-agent-toolkit/master/scripts/install.sh | sh",
    );
  } catch (e) {
    console.error(
      `${RED}Error: ${e instanceof Error ? e.message : String(e)}${NC}`,
    );
    Deno.exit(1);
  }
}

if (import.meta.main) {
  main();
}
