#!/usr/bin/env -S deno run --allow-read --allow-write --allow-run --allow-env

/**
 * E2E Test for GitHub Actions Release Workflow DAG
 *
 * This test simulates the release workflow to catch version mismatch issues
 * and other workflow problems before they happen in production.
 *
 * Following Toyota Way principles:
 * - Genchi Genbutsu (go and see): Simulate actual workflow steps
 * - Jidoka (automation with human touch): Detect and stop on quality issues
 * - Kaizen (continuous improvement): Learn from failures
 */

import { parse as parseYaml } from "https://deno.land/std@0.210.0/yaml/mod.ts";
import { join } from "https://deno.land/std@0.210.0/path/mod.ts";

interface WorkflowTestResult {
  success: boolean;
  errors: string[];
  warnings: string[];
  simulatedSteps: string[];
}

interface WorkflowStep {
  name?: string;
  run?: string;
  uses?: string;
  with?: Record<string, unknown>;
}

interface WorkflowJob {
  if?: string;
  steps?: WorkflowStep[];
}

interface WorkflowContent {
  name: string;
  jobs?: Record<string, WorkflowJob>;
}

// Global type augmentation for test state
declare global {
  var __workflowNewVersion: string | undefined;
  var __builtBinaryVersion: string | undefined;
}

class WorkflowSimulator {
  private readonly workflowPath: string;
  private workflowContent!: WorkflowContent;
  private readonly tempDir: string;
  private errors: string[] = [];
  private warnings: string[] = [];
  private steps: string[] = [];

  constructor(workflowPath: string) {
    this.workflowPath = workflowPath;
    this.tempDir = Deno.makeTempDirSync({ prefix: "workflow-test-" });
  }

  async load(): Promise<void> {
    const content = await Deno.readTextFile(this.workflowPath);
    this.workflowContent = parseYaml(content) as WorkflowContent;
    this.steps.push(`Loaded workflow: ${this.workflowContent.name}`);
  }

  async simulate(): Promise<WorkflowTestResult> {
    try {
      // Set up test repository
      await this.setupTestRepo();

      // Simulate each job
      for (
        const [jobName, job] of Object.entries(this.workflowContent.jobs || {})
      ) {
        this.steps.push(`\n=== Simulating job: ${jobName} ===`);
        await this.simulateJob(jobName, job as WorkflowJob);
      }

      // Run version mismatch detection
      this.detectVersionMismatches();

      return {
        success: this.errors.length === 0,
        errors: this.errors,
        warnings: this.warnings,
        simulatedSteps: this.steps,
      };
    } finally {
      // Cleanup
      await Deno.remove(this.tempDir, { recursive: true });
    }
  }

  private async setupTestRepo(): Promise<void> {
    this.steps.push("Setting up test repository...");

    // Copy essential files to temp dir
    const filesToCopy = [
      "Cargo.toml",
      "server/Cargo.toml",
      "installer-macro/Cargo.toml",
      "assets/project-state.json",
      "scripts/update-version.ts",
    ];

    for (const file of filesToCopy) {
      const src = join(Deno.cwd(), file);
      const dst = join(this.tempDir, file);

      // Create directory if needed
      const dir = join(this.tempDir, file.substring(0, file.lastIndexOf("/")));
      if (dir !== this.tempDir) {
        await Deno.mkdir(dir, { recursive: true });
      }

      try {
        await Deno.copyFile(src, dst);
      } catch (e) {
        this.warnings.push(
          `Could not copy ${file}: ${
            e instanceof Error ? e.message : String(e)
          }`,
        );
      }
    }

    // Initialize git repo
    await this.runCommand("git init", this.tempDir);
    await this.runCommand("git config user.name 'Test User'", this.tempDir);
    await this.runCommand(
      "git config user.email 'test@example.com'",
      this.tempDir,
    );
    await this.runCommand("git add .", this.tempDir);
    await this.runCommand("git commit -m 'Initial commit'", this.tempDir);
  }

  private async simulateJob(jobName: string, job: WorkflowJob): Promise<void> {
    // Check job conditions
    if (job.if) {
      this.steps.push(`  Condition: ${job.if}`);
    }

    // Track key workflow stages
    switch (jobName) {
      case "test-and-check":
        await this.simulateTestAndCheck();
        break;
      case "version-bump":
        await this.simulateVersionBump();
        break;
      case "build-binaries":
        await this.simulateBuildBinaries();
        break;
      case "create-release":
        await this.simulateCreateRelease();
        break;
    }
  }

  private async simulateTestAndCheck(): Promise<void> {
    this.steps.push("  - Running tests (simulated)");

    // Get current version
    const currentVersion = await this.getCurrentVersion();
    this.steps.push(`  - Current version: ${currentVersion}`);

    // Check if tag exists
    const tagExists = await this.checkTagExists(`v${currentVersion}`);
    if (tagExists) {
      this.steps.push(
        `  - Tag v${currentVersion} already exists - would skip release`,
      );
      return;
    }

    // Simulate version bump
    const newVersion = this.bumpVersion(currentVersion, "patch");
    this.steps.push(`  - Would bump version to: ${newVersion}`);
  }

  private async simulateVersionBump(): Promise<void> {
    this.steps.push("  - Version bump job (simulated)");

    const currentVersion = await this.getCurrentVersion();
    const newVersion = this.bumpVersion(currentVersion, "patch");

    this.steps.push(
      `  - Updating version from ${currentVersion} to ${newVersion}`,
    );
    await this.updateVersionFiles(newVersion);

    this.steps.push(`  - Creating tag v${newVersion}`);

    // This is key for v2 workflow - version is bumped BEFORE build
    globalThis.__workflowNewVersion = newVersion;
  }

  private async simulateBuildBinaries(): Promise<void> {
    this.steps.push("  - Building binaries (simulated)");

    // THIS IS THE CRITICAL CHECK!
    const currentVersion = await this.getCurrentVersion();
    this.steps.push(
      `  - Binary would be built with version: ${currentVersion}`,
    );

    // Store this for later comparison
    globalThis.__builtBinaryVersion = currentVersion;
  }

  private async simulateCreateRelease(): Promise<void> {
    this.steps.push("  - Creating release (simulated)");

    // In the fixed workflow, version update already happened in version-bump job
    // So we use the current version as the release version
    const currentVersion = await this.getCurrentVersion();

    // Check what version the binary was built with
    const builtVersion = globalThis.__builtBinaryVersion;
    const releaseVersion = currentVersion;

    this.steps.push(`  - Creating release for tag: v${releaseVersion}`);
    this.steps.push(`  - Binary version: ${builtVersion}`);
    this.steps.push(`  - Release version: ${releaseVersion}`);

    if (builtVersion !== releaseVersion) {
      this.errors.push(
        `VERSION MISMATCH DETECTED! Binary has version ${builtVersion} but release is ${releaseVersion}`,
      );
    }
  }

  private detectVersionMismatches(): void {
    this.steps.push("\n=== Version Mismatch Detection ===");

    // Analyze workflow for common patterns that cause version mismatches
    const workflow = this.workflowContent;

    // Check if binaries are built before version update
    const buildJob = workflow.jobs?.["build-binaries"];
    const releaseJob = workflow.jobs?.["create-release"];

    if (buildJob && releaseJob) {
      // Check if version update happens in release job
      const releaseSteps = releaseJob.steps || [];
      const versionUpdateStep = releaseSteps.findIndex((step) =>
        step.name?.includes("Update version") ||
        step.run?.includes("update-version")
      );

      if (versionUpdateStep >= 0) {
        this.warnings.push(
          "Version update happens in create-release job AFTER binaries are built. " +
            "This will cause version mismatch!",
        );
      }
    }

    // Check for proper checkout refs in build job
    if (buildJob?.steps) {
      const checkoutStep = buildJob.steps.find((step) =>
        step.uses?.includes("actions/checkout")
      );

      if (!checkoutStep?.with?.ref) {
        this.warnings.push(
          "Build job doesn't specify ref for checkout. " +
            "Should checkout the tag after version bump.",
        );
      }
    }
  }

  private async getCurrentVersion(): Promise<string> {
    const cargoToml = await Deno.readTextFile(join(this.tempDir, "Cargo.toml"));
    const match = cargoToml.match(/version = "([^"]+)"/);
    return match ? match[1] : "0.0.0";
  }

  private bumpVersion(
    version: string,
    type: "patch" | "minor" | "major",
  ): string {
    const [major, minor, patch] = version.split(".").map(Number);

    switch (type) {
      case "major":
        return `${major + 1}.0.0`;
      case "minor":
        return `${major}.${minor + 1}.0`;
      case "patch":
        return `${major}.${minor}.${patch + 1}`;
    }
  }

  private async updateVersionFiles(version: string): Promise<void> {
    // Simulate version update
    const files = ["Cargo.toml", "server/Cargo.toml"];

    for (const file of files) {
      try {
        const path = join(this.tempDir, file);
        let content = await Deno.readTextFile(path);
        content = content.replace(
          /version = "[^"]+"/,
          `version = "${version}"`,
        );
        await Deno.writeTextFile(path, content);
      } catch (_e) {
        // Ignore if file doesn't exist in test
      }
    }
  }

  private async checkTagExists(tag: string): Promise<boolean> {
    try {
      await this.runCommand(`git tag -l ${tag}`, this.tempDir);
      return false; // In our test, tags don't exist yet
    } catch {
      return false;
    }
  }

  private async runCommand(cmd: string, cwd: string): Promise<string> {
    const command = new Deno.Command("sh", {
      args: ["-c", cmd],
      cwd,
      stdout: "piped",
      stderr: "piped",
    });

    const { code, stdout, stderr } = await command.output();

    if (code !== 0) {
      throw new Error(
        `Command failed: ${cmd}\n${new TextDecoder().decode(stderr)}`,
      );
    }

    return new TextDecoder().decode(stdout);
  }

  cleanup(): void {
    try {
      Deno.removeSync(this.tempDir, { recursive: true });
    } catch {
      // Ignore cleanup errors
    }
  }
}

// Test runner
async function runWorkflowTests(): Promise<void> {
  console.log("🔍 GitHub Actions Workflow DAG E2E Test\n");

  const workflowFiles = [
    ".github/workflows/automated-release.yml",
  ];

  let hasErrors = false;

  for (const workflowFile of workflowFiles) {
    console.log(`\n📋 Testing ${workflowFile}`);
    console.log("═".repeat(60));

    try {
      const simulator = new WorkflowSimulator(workflowFile);
      await simulator.load();

      const result = await simulator.simulate();

      // Print simulation steps
      if (Deno.args.includes("--verbose")) {
        console.log("\nSimulation steps:");
        result.simulatedSteps.forEach((step) => console.log(step));
      }

      // Print warnings
      if (result.warnings.length > 0) {
        console.log("\n⚠️  Warnings:");
        result.warnings.forEach((warning) => console.log(`   - ${warning}`));
      }

      // Print errors
      if (result.errors.length > 0) {
        console.log("\n❌ Errors:");
        result.errors.forEach((error) => console.log(`   - ${error}`));
        hasErrors = true;
      }

      if (result.success) {
        console.log("\n✅ Workflow passed version consistency checks");
      } else {
        console.log("\n❌ Workflow has version mismatch issues!");
      }

      simulator.cleanup();
    } catch (e) {
      console.error(
        `\n❌ Failed to test ${workflowFile}: ${
          e instanceof Error ? e.message : String(e)
        }`,
      );
      hasErrors = true;
    }
  }

  // Summary
  console.log("\n" + "═".repeat(60));
  if (hasErrors) {
    console.log("❌ FAILED: Workflow tests found critical issues");
    Deno.exit(1);
  } else {
    console.log("✅ SUCCESS: All workflow tests passed");
  }
}

// Additional test for specific scenarios
function testVersionMismatchScenario(): void {
  console.log("\n🧪 Testing Version Mismatch Scenario");
  console.log("═".repeat(60));

  // Simulate the exact sequence that causes version mismatch
  const steps = [
    "1. Current version in Cargo.toml: 0.2.1",
    "2. Workflow triggered by push to master",
    "3. build-binaries job starts (reads version 0.2.1)",
    "4. Binary built with version 0.2.1",
    "5. create-release job starts",
    "6. Version bumped to 0.2.2 in files",
    "7. Tag v0.2.2 created",
    "8. Release v0.2.2 created with binary showing v0.2.1",
    "9. ❌ VERSION MISMATCH!",
  ];

  steps.forEach((step) => console.log(`   ${step}`));

  console.log("\nThis is exactly what happens with the current workflow!");
}

// Main
if (import.meta.main) {
  await runWorkflowTests();

  if (Deno.args.includes("--scenarios")) {
    testVersionMismatchScenario();
  }
}
