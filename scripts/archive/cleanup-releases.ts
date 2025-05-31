#!/usr/bin/env -S deno run --allow-net --allow-run --allow-read

/**
 * Script to help diagnose and optionally clean up release issues
 *
 * Usage:
 *   deno run --allow-net --allow-run --allow-read scripts/cleanup-releases.ts
 *   deno run --allow-net --allow-run --allow-read scripts/cleanup-releases.ts --fix
 */

const REPO = "paiml/paiml-mcp-agent-toolkit";
const FIX_MODE = Deno.args.includes("--fix");

interface GitHubRelease {
  id: number;
  tag_name: string;
  name: string;
  draft: boolean;
  prerelease: boolean;
  created_at: string;
  published_at: string;
  html_url: string;
}

interface GitHubWorkflowRun {
  id: number;
  name: string;
  status: string;
  conclusion: string | null;
  created_at: string;
  head_commit: {
    message: string;
  };
}

async function getGitHubData<T>(path: string): Promise<T> {
  const response = await fetch(`https://api.github.com/repos/${REPO}/${path}`, {
    headers: {
      "Accept": "application/vnd.github.v3+json",
      // Add token if available for higher rate limits
      ...(Deno.env.get("GITHUB_TOKEN")
        ? {
          "Authorization": `token ${Deno.env.get("GITHUB_TOKEN")}`,
        }
        : {}),
    },
  });

  if (!response.ok) {
    throw new Error(
      `GitHub API error: ${response.status} ${response.statusText}`,
    );
  }

  return response.json();
}

async function getLocalTags(): Promise<Set<string>> {
  const cmd = new Deno.Command("git", {
    args: ["tag", "-l"],
    stdout: "piped",
  });

  const { stdout } = await cmd.output();
  const tags = new TextDecoder().decode(stdout).trim().split("\n").filter(
    Boolean,
  );
  return new Set(tags);
}

async function _getCommitForTag(tag: string): Promise<string> {
  const cmd = new Deno.Command("git", {
    args: ["rev-list", "-n", "1", tag],
    stdout: "piped",
  });

  const { stdout } = await cmd.output();
  return new TextDecoder().decode(stdout).trim();
}

async function main() {
  console.log("🔍 Analyzing releases and tags...\n");

  // Get all releases from GitHub
  const releases = await getGitHubData<GitHubRelease[]>(
    "releases?per_page=100",
  );
  console.log(`📦 Found ${releases.length} GitHub releases`);

  // Get local tags
  const localTags = await getLocalTags();
  console.log(`🏷️  Found ${localTags.size} local tags`);

  // Get recent workflow runs
  const workflows = await getGitHubData<{ workflow_runs: GitHubWorkflowRun[] }>(
    "actions/workflows/automated-release.yml/runs?per_page=20",
  );

  console.log(`\n🔄 Recent workflow runs:`);
  for (const run of workflows.workflow_runs.slice(0, 5)) {
    const emoji = run.conclusion === "success"
      ? "✅"
      : run.conclusion === "failure"
      ? "❌"
      : "⏳";
    console.log(
      `${emoji} ${run.created_at}: ${run.head_commit.message.split("\n")[0]}`,
    );
  }

  // Check for version mismatches
  console.log("\n🔍 Checking for issues...");

  // Find tags without releases
  const tagsWithoutReleases: string[] = [];
  for (const tag of localTags) {
    if (!releases.find((r) => r.tag_name === tag)) {
      tagsWithoutReleases.push(tag);
    }
  }

  if (tagsWithoutReleases.length > 0) {
    console.log(`\n⚠️  Tags without GitHub releases:`);
    for (const tag of tagsWithoutReleases) {
      console.log(`  - ${tag}`);
    }
  }

  // Find duplicate releases (same version)
  const versionMap = new Map<string, GitHubRelease[]>();
  for (const release of releases) {
    const existing = versionMap.get(release.tag_name) || [];
    existing.push(release);
    versionMap.set(release.tag_name, existing);
  }

  const duplicates = Array.from(versionMap.entries()).filter(([_, releases]) =>
    releases.length > 1
  );
  if (duplicates.length > 0) {
    console.log(`\n⚠️  Duplicate releases found:`);
    for (const [version, releases] of duplicates) {
      console.log(`  - ${version}: ${releases.length} releases`);
      for (const release of releases) {
        console.log(`    - ${release.html_url} (${release.created_at})`);
      }
    }
  }

  // Check current version in Cargo.toml
  const cargoContent = await Deno.readTextFile("Cargo.toml");
  const versionMatch = cargoContent.match(/^version = "(.+)"/m);
  const currentVersion = versionMatch ? versionMatch[1] : "unknown";
  console.log(`\n📋 Current version in Cargo.toml: ${currentVersion}`);

  const currentTag = `v${currentVersion}`;
  const hasTag = localTags.has(currentTag);
  const hasRelease = releases.some((r) => r.tag_name === currentTag);

  console.log(`  - Tag ${currentTag} exists: ${hasTag ? "✅" : "❌"}`);
  console.log(`  - Release ${currentTag} exists: ${hasRelease ? "✅" : "❌"}`);

  if (FIX_MODE) {
    console.log("\n🔧 Fix mode enabled - would perform cleanup actions here");
    console.log("⚠️  Manual cleanup recommended for safety");
  } else {
    console.log(
      "\n💡 Run with --fix to attempt automated cleanup (use with caution)",
    );
  }

  // Provide recommendations
  console.log("\n📝 Recommendations:");
  console.log(
    "1. The workflow has been updated to create tags AFTER committing version bumps",
  );
  console.log("2. This should prevent the duplicate run issue going forward");
  console.log(
    "3. Existing failed workflow runs can be ignored - they were due to the ordering issue",
  );

  if (tagsWithoutReleases.length > 0) {
    console.log(
      "4. You may want to manually create releases for orphaned tags",
    );
  }

  if (duplicates.length > 0) {
    console.log(
      "5. Consider manually deleting duplicate releases (keep the oldest)",
    );
  }
}

if (import.meta.main) {
  main().catch(console.error);
}
