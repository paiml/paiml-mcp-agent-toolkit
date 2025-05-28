/**
 * Utilities for install script
 * Extracted for testability
 */

export interface InstallConfig {
  repo: string;
  binaryName: string;
  installDir: string;
}

export const DEFAULT_CONFIG: InstallConfig = {
  repo: "paiml/paiml-mcp-agent-toolkit",
  binaryName: "paiml-mcp-agent-toolkit",
  installDir: `${Deno.env.get("HOME")}/.local/bin`,
};

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

export async function getLatestVersion(repo: string): Promise<string> {
  const response = await fetch(
    `https://api.github.com/repos/${repo}/releases/latest`,
  );

  if (!response.ok) {
    throw new Error(`Failed to fetch latest release: ${response.statusText}`);
  }

  const data = await response.json();
  return data.tag_name;
}

export function constructDownloadUrl(
  repo: string,
  version: string,
  binaryName: string,
  platform: string,
): string {
  // Ensure version has 'v' prefix
  const versionWithPrefix = version.startsWith("v") ? version : `v${version}`;
  return `https://github.com/${repo}/releases/download/${versionWithPrefix}/${binaryName}-${platform}.tar.gz`;
}

export async function downloadFile(
  url: string,
  destination: string,
): Promise<void> {
  const response = await fetch(url);

  if (!response.ok) {
    throw new Error(`Failed to download: ${response.statusText}`);
  }

  const data = await response.arrayBuffer();
  await Deno.writeFile(destination, new Uint8Array(data));
}

export async function extractTarball(
  tarballPath: string,
  destination: string,
): Promise<void> {
  const command = new Deno.Command("tar", {
    args: ["-xzf", tarballPath, "-C", destination],
  });

  const { code, stderr } = await command.output();

  if (code !== 0) {
    const error = new TextDecoder().decode(stderr);
    throw new Error(`Failed to extract tarball: ${error}`);
  }
}

export async function ensureDirectoryExists(path: string): Promise<void> {
  try {
    await Deno.mkdir(path, { recursive: true });
  } catch (error) {
    // Directory might already exist, which is fine
    if (!(error instanceof Deno.errors.AlreadyExists)) {
      throw error;
    }
  }
}

export function isInPath(directory: string): boolean {
  const path = Deno.env.get("PATH") || "";
  return path.split(":").includes(directory);
}

export async function verifyInstallation(
  binaryPath: string,
): Promise<boolean> {
  try {
    const stat = await Deno.stat(binaryPath);
    return stat.isFile;
  } catch {
    return false;
  }
}

export function stripVersionPrefix(version: string): string {
  return version.startsWith("v") ? version.slice(1) : version;
}
