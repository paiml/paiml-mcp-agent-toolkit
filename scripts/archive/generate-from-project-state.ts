#!/usr/bin/env -S deno run --allow-read

/**
 * Generate various project artifacts from the central project state
 */

// Load project state from central source of truth
const projectStateJson = await Deno.readTextFile("assets/project-state.json");
const projectState = JSON.parse(projectStateJson);

// Badge generation functions
function generateGitHubBadge(workflow: string, label: string): string {
  const { owner, name } = projectState.repository;
  return `[![${label}](https://github.com/${owner}/${name}/actions/workflows/${workflow}/badge.svg)](https://github.com/${owner}/${name}/actions/workflows/${workflow})`;
}

function generateStaticBadge(
  label: string,
  message: string,
  color: string,
): string {
  const encodedLabel = encodeURIComponent(label);
  const encodedMessage = encodeURIComponent(message);
  return `[![${label}](https://img.shields.io/badge/${encodedLabel}-${encodedMessage}-${color})]`;
}

function generateLicenseBadge(): string {
  const license = projectState.badges.license;
  return `[![License: ${license.type}](https://img.shields.io/badge/License-${license.type}-yellow.svg)](${license.url})`;
}

// Generate all badges
export function generateBadges(): string {
  const badges = [];

  // CI badge
  if (projectState.badges.ci) {
    badges.push(
      generateGitHubBadge(
        projectState.badges.ci.workflow,
        projectState.badges.ci.label,
      ),
    );
  }

  // Coverage badge
  if (projectState.badges.coverage) {
    badges.push(
      generateStaticBadge(
        projectState.badges.coverage.label,
        `${projectState.badges.coverage.percentage}%`,
        projectState.badges.coverage.color,
      ) + `(${projectState.repository.url}/actions)`,
    );
  }

  // Organization badge
  badges.push(
    generateStaticBadge(
      `Built by ${projectState.organization.name}`,
      "",
      "blue",
    ).replace("--", "-") + `(${projectState.organization.website})`,
  );

  // MCP Compatible badge
  badges.push(
    generateStaticBadge(
      "MCP Compatible",
      "",
      "green",
    ).replace("--", "-") + "(https://modelcontextprotocol.io)",
  );

  // License badge
  badges.push(generateLicenseBadge());

  return badges.join("\n");
}

// Generate installation URL
export function generateInstallerUrl(): string {
  const { owner, name } = projectState.repository;
  return `https://raw.githubusercontent.com/${owner}/${name}/main/${projectState.installation.installer.script}`;
}

// Generate release URLs
export function generateReleaseUrl(
  platform: string,
  arch: string,
  format: string = "tar.gz",
): string {
  const { owner, name } = projectState.repository;
  const { name: packageName } = projectState.package;
  return `https://github.com/${owner}/${name}/releases/latest/download/${packageName}-${arch}-${platform}.${format}`;
}

// Generate repository URLs
export function generateRepoUrl(path: string = ""): string {
  const { owner, name } = projectState.repository;
  return `https://github.com/${owner}/${name}${path ? "/" + path : ""}`;
}

// Generate Claude Code config example
export function generateClaudeConfig(): string {
  const { name: binaryName } = projectState.package;
  const serverName = projectState.mcp.serverName;

  return `{
  "mcpServers": {
    "${serverName}": {
      "command": "${projectState.installation.paths.global}/${binaryName}",
      "args": [],
      "env": {}
    }
  }
}`;
}

// Main function to demonstrate all generators
if (import.meta.main) {
  console.log("=== Project State Generators ===\n");

  console.log("Badges:");
  console.log(generateBadges());
  console.log();

  console.log("Installer URL:");
  console.log(generateInstallerUrl());
  console.log();

  console.log("Release URLs:");
  console.log(
    "- Linux x64:",
    generateReleaseUrl("unknown-linux-gnu", "x86_64"),
  );
  console.log("- macOS ARM64:", generateReleaseUrl("apple-darwin", "aarch64"));
  console.log();

  console.log("Repository URLs:");
  console.log("- Main:", generateRepoUrl());
  console.log("- Issues:", generateRepoUrl("issues"));
  console.log("- Discussions:", generateRepoUrl("discussions"));
  console.log();

  console.log("Claude Code Config:");
  console.log(generateClaudeConfig());
}
