#!/usr/bin/env -S deno run --allow-net --allow-read --allow-write --allow-env --allow-run

/**
 * MCP Agent Toolkit Installation Script
 *
 * This script installs the paiml-mcp-agent-toolkit binary from GitHub releases.
 *
 * Usage:
 *   deno run --allow-net --allow-read --allow-write --allow-env --allow-run https://raw.githubusercontent.com/paiml/paiml-mcp-agent-toolkit/master/scripts/install.ts
 *
 * Or to install a specific version:
 *   deno run --allow-net --allow-read --allow-write --allow-env --allow-run https://raw.githubusercontent.com/paiml/paiml-mcp-agent-toolkit/master/scripts/install.ts v0.1.0
 */

// Configuration
const REPO = "paiml/paiml-mcp-agent-toolkit";
const BINARY_NAME = "paiml-mcp-agent-toolkit";
const INSTALL_DIR = `${Deno.env.get("HOME")}/.local/bin`;

// Colors for output
const RED = "\x1b[31m";
const GREEN = "\x1b[32m";
const YELLOW = "\x1b[33m";
const NC = "\x1b[0m"; // No Color

// Helper functions
function error(message: string): never {
  console.error(`${RED}Error: ${message}${NC}`);
  Deno.exit(1);
}

function info(message: string): void {
  console.log(`${GREEN}${message}${NC}`);
}

function warn(message: string): void {
  console.log(`${YELLOW}${message}${NC}`);
}

// Detect OS
function detectOS(): string {
  switch (Deno.build.os) {
    case "linux":
      return "linux";
    case "darwin":
      return "macos";
    case "windows":
      return "windows";
    default:
      error(`Unsupported operating system: ${Deno.build.os}`);
  }
}

// Detect architecture
function detectArch(): string {
  switch (Deno.build.arch) {
    case "x86_64":
      return "x86_64";
    case "aarch64":
      return "aarch64";
    default:
      error(`Unsupported architecture: ${Deno.build.arch}`);
  }
}

// Get latest version from GitHub
async function getLatestVersion(): Promise<string> {
  try {
    const response = await fetch(
      `https://api.github.com/repos/${REPO}/releases/latest`,
    );
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }
    const data = await response.json();
    return data.tag_name;
  } catch (e) {
    error(`Failed to fetch latest version: ${e}`);
  }
}

// Download file
async function downloadFile(url: string, destination: string): Promise<void> {
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(
      `Failed to download: ${response.status} ${response.statusText}`,
    );
  }

  const data = await response.arrayBuffer();
  await Deno.writeFile(destination, new Uint8Array(data));
}

// Extract tar.gz file
async function extractTarGz(
  archivePath: string,
  destDir: string,
): Promise<void> {
  const cmd = new Deno.Command("tar", {
    args: ["-xzf", archivePath, "-C", destDir],
  });
  const { code, stderr } = await cmd.output();

  if (code !== 0) {
    throw new Error(
      `Failed to extract archive: ${new TextDecoder().decode(stderr)}`,
    );
  }
}

// Main installation function
async function install(version?: string): Promise<void> {
  const os = detectOS();
  const arch = detectArch();

  // Get version
  if (!version) {
    version = await getLatestVersion();
  }

  // Remove 'v' prefix if present
  version = version.replace(/^v/, "");

  info(`Installing ${BINARY_NAME} v${version} for ${os}-${arch}...`);

  // Construct download URL
  const downloadUrl =
    `https://github.com/${REPO}/releases/download/v${version}/${BINARY_NAME}-${os}-${arch}.tar.gz`;

  // Create temp directory
  const tmpDir = await Deno.makeTempDir();

  try {
    // Download binary
    info(`Downloading from ${downloadUrl}...`);
    const archivePath = `${tmpDir}/archive.tar.gz`;

    try {
      await downloadFile(downloadUrl, archivePath);
    } catch (_e) {
      error(
        `Failed to download binary. Please check if version ${version} exists for ${os}-${arch}.`,
      );
    }

    // Extract binary
    await extractTarGz(archivePath, tmpDir);

    // Create install directory
    await Deno.mkdir(INSTALL_DIR, { recursive: true });

    // Install binary
    const binaryPath = `${tmpDir}/${BINARY_NAME}`;
    const installPath = `${INSTALL_DIR}/${BINARY_NAME}`;

    try {
      await Deno.stat(binaryPath);
    } catch {
      error("Binary not found in archive");
    }

    await Deno.copyFile(binaryPath, installPath);
    await Deno.chmod(installPath, 0o755);

    info(`Successfully installed ${BINARY_NAME} to ${installPath}`);

    // Check if install dir is in PATH
    const path = Deno.env.get("PATH") || "";
    if (!path.split(":").includes(INSTALL_DIR)) {
      warn(`Warning: ${INSTALL_DIR} is not in your PATH.`);
      warn("Add the following to your shell profile:");
      warn(`  export PATH="$PATH:${INSTALL_DIR}"`);
    }

    // Verify installation
    try {
      const cmd = new Deno.Command(BINARY_NAME, { args: ["--version"] });
      await cmd.output();
      info(`Installation complete! Run '${BINARY_NAME} --version' to verify.`);
    } catch {
      warn(
        `Installation complete, but ${BINARY_NAME} is not in your PATH yet.`,
      );
      warn(`Please restart your shell or add ${INSTALL_DIR} to your PATH.`);
    }
  } finally {
    // Cleanup
    await Deno.remove(tmpDir, { recursive: true });
  }
}

// Main entry point
async function main(): Promise<void> {
  info("MCP Agent Toolkit Installer");

  const version = Deno.args[0];
  await install(version);
}

// Run main
if (import.meta.main) {
  main().catch((e) => {
    error(e.message);
  });
}
