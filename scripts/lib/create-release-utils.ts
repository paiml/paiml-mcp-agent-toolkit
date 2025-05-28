/**
 * Utilities for create-release script
 * Extracted for testability
 */

// Colors
export const RED = "\x1b[31m";
export const GREEN = "\x1b[32m";
export const YELLOW = "\x1b[33m";
export const NC = "\x1b[0m";

export async function getVersion(): Promise<string> {
  const cargoToml = await Deno.readTextFile("server/Cargo.toml");
  const match = cargoToml.match(/^version = "(.*)"/m);
  if (!match) {
    throw new Error("Could not find version in Cargo.toml");
  }
  return match[1];
}

export async function runCommand(
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

export async function checkGhCli(): Promise<boolean> {
  try {
    const { success } = await runCommand("gh", ["--version"]);
    return success;
  } catch {
    return false;
  }
}

export function isRunningInGitHubActions(): boolean {
  return Deno.env.get("GITHUB_ACTIONS") === "true";
}

export function detectPlatform(): string {
  const os = Deno.build.os;
  const arch = Deno.build.arch;

  // Map to Rust target triples
  if (os === "linux") {
    if (arch === "x86_64") return "x86_64-unknown-linux-gnu";
    if (arch === "aarch64") return "aarch64-unknown-linux-gnu";
  } else if (os === "darwin") {
    if (arch === "x86_64") return "x86_64-apple-darwin";
    if (arch === "aarch64") return "aarch64-apple-darwin";
  } else if (os === "windows") {
    if (arch === "x86_64") return "x86_64-pc-windows-msvc";
  }

  throw new Error(`Unsupported platform: ${os}-${arch}`);
}

export async function createTarball(
  binaryName: string,
  platform: string,
): Promise<string> {
  const tarballName = `${binaryName}-${platform}.tar.gz`;

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

export function generateReleaseNotes(version: string): string {
  return `## MCP Agent Toolkit v${version}

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
}

export async function checkProjectStructure(): Promise<void> {
  try {
    await Deno.stat("server/Cargo.toml");
  } catch {
    throw new Error("Must run from repository root");
  }
}

export async function buildReleaseBinary(): Promise<void> {
  console.log(`${YELLOW}Building release binary...${NC}`);
  const { success, error } = await runCommand(
    "make",
    ["server-build-binary"],
  );
  if (!success) {
    throw new Error(`Build failed: ${error}`);
  }
}

export async function createGitHubRelease(
  version: string,
  releaseNotes: string,
  tarball: string,
): Promise<void> {
  console.log(`${YELLOW}Creating GitHub release...${NC}`);
  const { success, error } = await runCommand(
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

  if (!success) {
    throw new Error(`Failed to create release: ${error}`);
  }

  console.log(`${GREEN}✅ Release created successfully!${NC}`);
  console.log(
    `${GREEN}View at: https://github.com/paiml/paiml-mcp-agent-toolkit/releases/tag/v${version}${NC}`,
  );
}
