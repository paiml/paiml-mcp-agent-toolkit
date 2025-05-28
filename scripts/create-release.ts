#!/usr/bin/env -S deno run --allow-read --allow-write --allow-env --allow-run --allow-net

/**
 * Script to create a GitHub release with binary artifacts
 */

import {
  buildReleaseBinary,
  checkGhCli,
  checkProjectStructure,
  createGitHubRelease,
  createTarball,
  detectPlatform,
  generateReleaseNotes,
  getVersion,
  GREEN,
  isRunningInGitHubActions,
  NC,
  RED,
  YELLOW,
} from "./lib/create-release-utils.ts";

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
    await checkProjectStructure();

    // Build release binary
    await buildReleaseBinary();

    // Detect platform
    const platform = detectPlatform();

    // Create tarball
    const binaryName = "paiml-mcp-agent-toolkit";
    const tarball = await createTarball(binaryName, platform);
    console.log(`${YELLOW}Created tarball: ${tarball}${NC}`);

    // Generate release notes
    const releaseNotes = generateReleaseNotes(version);

    // Create GitHub release
    await createGitHubRelease(version, releaseNotes, tarball);

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
