#!/usr/bin/env -S deno run --allow-read --allow-write

/**
 * Centralized version management script
 * Updates version in all required files to maintain consistency
 */

interface VersionUpdate {
  file: string;
  pattern: RegExp;
  replacement: (version: string) => string;
}

// Files that need version updates
const VERSION_UPDATES: VersionUpdate[] = [
  {
    file: "Cargo.toml",
    pattern: /^version = ".*"$/m,
    replacement: (v) => `version = "${v}"`,
  },
  {
    file: "server/Cargo.toml",
    pattern: /^version = ".*"$/m,
    replacement: (v) => `version = "${v}"`,
  },
  {
    file: "installer-macro/Cargo.toml",
    pattern: /^version = ".*"$/m,
    replacement: (v) => `version = "${v}"`,
  },
  {
    file: "assets/project-state.json",
    pattern: /"version": ".*"/,
    replacement: (v) => `"version": "${v}"`,
  },
];

// Semantic version parsing
interface Version {
  major: number;
  minor: number;
  patch: number;
}

function parseVersion(versionStr: string): Version {
  const match = versionStr.match(/^(\d+)\.(\d+)\.(\d+)$/);
  if (!match) {
    throw new Error(`Invalid version format: ${versionStr}`);
  }
  return {
    major: parseInt(match[1]),
    minor: parseInt(match[2]),
    patch: parseInt(match[3]),
  };
}

function formatVersion(version: Version): string {
  return `${version.major}.${version.minor}.${version.patch}`;
}

// Get current version from the source of truth
async function getCurrentVersion(): Promise<string> {
  const projectState = JSON.parse(
    await Deno.readTextFile("assets/project-state.json"),
  );
  return projectState.project.version;
}

// Bump version based on type
function bumpVersion(currentVersion: string, bumpType: string): string {
  const version = parseVersion(currentVersion);

  switch (bumpType) {
    case "major":
      version.major++;
      version.minor = 0;
      version.patch = 0;
      break;
    case "minor":
      version.minor++;
      version.patch = 0;
      break;
    case "patch":
      version.patch++;
      break;
    default:
      throw new Error(`Unknown bump type: ${bumpType}`);
  }

  return formatVersion(version);
}

// Update version in a single file
async function updateVersionInFile(
  update: VersionUpdate,
  newVersion: string,
): Promise<boolean> {
  try {
    const content = await Deno.readTextFile(update.file);
    const updatedContent = content.replace(
      update.pattern,
      update.replacement(newVersion),
    );

    if (content === updatedContent) {
      console.log(`⚠️  No version found in ${update.file}`);
      return false;
    }

    await Deno.writeTextFile(update.file, updatedContent);
    console.log(`✅ Updated ${update.file}`);
    return true;
  } catch (error) {
    console.error(`❌ Failed to update ${update.file}: ${error}`);
    return false;
  }
}

// Update all files atomically
async function updateAllVersions(newVersion: string): Promise<void> {
  console.log(`📦 Updating all versions to ${newVersion}...`);

  // First, verify all files exist and are writable
  for (const update of VERSION_UPDATES) {
    try {
      await Deno.stat(update.file);
    } catch {
      throw new Error(`File not found: ${update.file}`);
    }
  }

  // Update all files
  const results = await Promise.all(
    VERSION_UPDATES.map((update) => updateVersionInFile(update, newVersion)),
  );

  const successCount = results.filter((r) => r).length;
  console.log(`\n✅ Updated ${successCount}/${VERSION_UPDATES.length} files`);

  // Update Cargo.lock
  console.log("\n🔄 Updating Cargo.lock...");
  const cargoUpdate = new Deno.Command("cargo", {
    args: ["update", "-p", "paiml-mcp-agent-toolkit"],
    cwd: "server",
  });

  const { success } = await cargoUpdate.output();
  if (success) {
    console.log("✅ Cargo.lock updated");
  } else {
    console.log("⚠️  Failed to update Cargo.lock");
  }
}

// CLI interface
async function main() {
  const args = Deno.args;

  if (args.length === 0) {
    console.log(
      "Usage: update-version.ts <version|patch|minor|major> [--dry-run]",
    );
    console.log("Examples:");
    console.log("  update-version.ts 1.2.3      # Set specific version");
    console.log("  update-version.ts patch      # Bump patch version");
    console.log("  update-version.ts minor      # Bump minor version");
    console.log("  update-version.ts major      # Bump major version");
    console.log("  update-version.ts current    # Show current version");
    Deno.exit(1);
  }

  const command = args[0];
  const isDryRun = args.includes("--dry-run");

  try {
    const currentVersion = await getCurrentVersion();
    console.log(`📌 Current version: ${currentVersion}`);

    if (command === "current") {
      Deno.exit(0);
    }

    let newVersion: string;

    // Check if it's a bump type or explicit version
    if (["major", "minor", "patch"].includes(command)) {
      newVersion = bumpVersion(currentVersion, command);
    } else {
      // Validate explicit version
      parseVersion(command); // Will throw if invalid
      newVersion = command;
    }

    console.log(`🆕 New version: ${newVersion}`);

    if (isDryRun) {
      console.log("\n🔍 Dry run - no files will be modified");
      Deno.exit(0);
    }

    await updateAllVersions(newVersion);

    console.log("\n✅ Version update complete!");
    console.log("\n💡 Next steps:");
    console.log("   1. Review the changes: git diff");
    console.log(
      "   2. Commit: git commit -am 'chore: bump version to " + newVersion +
        "'",
    );
    console.log("   3. Tag: git tag v" + newVersion);
    console.log("   4. Push: git push && git push --tags");
  } catch (error) {
    console.error(`\n❌ Error: ${error instanceof Error ? error.message : String(error)}`);
    Deno.exit(1);
  }
}

if (import.meta.main) {
  await main();
}
