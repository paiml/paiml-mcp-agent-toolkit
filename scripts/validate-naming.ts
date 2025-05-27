#!/usr/bin/env -S deno run --allow-read --allow-run

/**
 * Comprehensive naming validation for the entire project
 * Checks for consistency in binary names, package names, and repository references
 */

// Load project state from central source of truth
const projectStateJson = await Deno.readTextFile("assets/project-state.json");
const projectState = JSON.parse(projectStateJson);

interface ValidationResult {
  category: string;
  passed: boolean;
  message: string;
  details?: string[];
}

const CORRECT_BINARY_NAME = projectState.package.binary.main;
const CORRECT_PACKAGE_NAME = projectState.package.name;
const ALLOWED_ADDITIONAL_BINARIES =
  projectState.package.binary.allowed_additional;

const OLD_NAMES = projectState.deprecated.binaryNames;
const OLD_REPO_URLS = projectState.deprecated.repositoryUrls;

async function runCommand(
  cmd: string[],
): Promise<{ stdout: string; stderr: string; success: boolean }> {
  const process = new Deno.Command(cmd[0], {
    args: cmd.slice(1),
    stdout: "piped",
    stderr: "piped",
  });

  const { stdout, stderr, success } = await process.output();

  return {
    stdout: new TextDecoder().decode(stdout),
    stderr: new TextDecoder().decode(stderr),
    success,
  };
}

async function checkCargoToml(): Promise<ValidationResult> {
  try {
    const cargoToml = await Deno.readTextFile("server/Cargo.toml");
    const lines = cargoToml.split("\n");

    let packageName = "";
    const binaryNames: string[] = [];
    const issues: string[] = [];

    for (let i = 0; i < lines.length; i++) {
      const line = lines[i];
      if (line.startsWith("name = ") && !packageName) {
        packageName = line.match(/name = "(.+)"/)?.[1] || "";
      }
      if (line.startsWith("name = ") && lines[i - 1]?.includes("[[bin]]")) {
        const binName = line.match(/name = "(.+)"/)?.[1] || "";
        if (binName) binaryNames.push(binName);
      }
    }

    if (packageName !== CORRECT_PACKAGE_NAME) {
      issues.push(
        `Package name is '${packageName}', should be '${CORRECT_PACKAGE_NAME}'`,
      );
    }

    if (binaryNames.length === 0) {
      issues.push("No binary targets found");
    } else {
      // Check if the main binary is present
      if (!binaryNames.includes(CORRECT_BINARY_NAME)) {
        issues.push(
          `Main binary '${CORRECT_BINARY_NAME}' not found. Found: ${
            binaryNames.join(", ")
          }`,
        );
      }
      // Allow additional binaries from the allowed list
      const unexpectedBinaries = binaryNames.filter(
        (name) =>
          name !== CORRECT_BINARY_NAME &&
          !ALLOWED_ADDITIONAL_BINARIES.includes(name),
      );
      if (unexpectedBinaries.length > 0) {
        issues.push(
          `Unexpected binary targets found: ${unexpectedBinaries.join(", ")}`,
        );
      }
    }

    return {
      category: "Cargo.toml",
      passed: issues.length === 0,
      message: issues.length === 0
        ? "✅ Package and binary names are correct"
        : "❌ Issues found in Cargo.toml",
      details: issues,
    };
  } catch (error) {
    return {
      category: "Cargo.toml",
      passed: false,
      message: "❌ Failed to read Cargo.toml",
      details: [String(error)],
    };
  }
}

async function checkSourceCode(): Promise<ValidationResult> {
  const issues: string[] = [];

  for (const oldName of OLD_NAMES) {
    // Use word boundaries to match exact names, not parts of other identifiers
    const result = await runCommand([
      "grep",
      "-r",
      "-w", // Match whole words only
      oldName,
      "server/src/",
      "--include=*.rs",
      "--exclude=build_naming_validation.rs",
      "--exclude=claude_code_e2e.rs",
    ]);

    if (result.stdout.trim()) {
      // Filter out legitimate uses like "run_mcp_server" function names
      const lines = result.stdout.trim().split("\n");
      const actualIssues = lines.filter((line) => {
        // Check if this is a function/method name containing the old name
        return !line.match(/fn\s+\w*mcp_server\w*/i) &&
          !line.match(/async\s+fn\s+\w*mcp_server\w*/i) &&
          !line.match(/::\w*mcp_server\w*/i);
      });

      if (actualIssues.length > 0) {
        issues.push(
          `Found references to '${oldName}' in source files:\n${
            actualIssues.join("\n")
          }`,
        );
      }
    }
  }

  return {
    category: "Source Code",
    passed: issues.length === 0,
    message: issues.length === 0
      ? "✅ No old package references in source code"
      : "❌ Found old references in source code",
    details: issues,
  };
}

async function checkGitHubWorkflows(): Promise<ValidationResult> {
  const issues: string[] = [];

  // Check for old binary names
  for (const oldName of OLD_NAMES) {
    const result = await runCommand([
      "grep",
      "-r",
      oldName,
      ".github/workflows/",
      "--include=*.yml",
      "--include=*.yaml",
    ]);

    if (result.stdout.trim()) {
      issues.push(
        `Found references to old binary '${oldName}' in workflows:\n${result.stdout}`,
      );
    }
  }

  // Check for old repository URLs
  for (const oldUrl of OLD_REPO_URLS) {
    const result = await runCommand([
      "grep",
      "-r",
      oldUrl,
      ".github/workflows/",
      "--include=*.yml",
      "--include=*.yaml",
    ]);

    if (result.stdout.trim()) {
      issues.push(
        `Found references to old repository URL '${oldUrl}' in workflows:\n${result.stdout}`,
      );
    }
  }

  // Verify correct binary name is used
  const correctBinaryResult = await runCommand([
    "grep",
    "-r",
    CORRECT_BINARY_NAME,
    ".github/workflows/",
    "--include=*.yml",
    "--include=*.yaml",
  ]);

  if (!correctBinaryResult.stdout.trim()) {
    issues.push(
      `No references to correct binary name '${CORRECT_BINARY_NAME}' found in workflows`,
    );
  }

  return {
    category: "GitHub Workflows",
    passed: issues.length === 0,
    message: issues.length === 0
      ? "✅ GitHub workflows use correct names"
      : "❌ Found naming issues in workflows",
    details: issues,
  };
}

async function checkDocumentation(): Promise<ValidationResult> {
  const issues: string[] = [];
  const docsToCheck = ["README.md", "CLAUDE.md", "scripts/README.md"];

  for (const doc of docsToCheck) {
    try {
      const content = await Deno.readTextFile(doc);

      // Check for old binary names
      for (const oldName of OLD_NAMES) {
        if (content.includes(oldName)) {
          issues.push(`Found reference to '${oldName}' in ${doc}`);
        }
      }

      // Check for old repository URLs
      for (const oldUrl of OLD_REPO_URLS) {
        if (content.includes(oldUrl)) {
          issues.push(`Found reference to old URL '${oldUrl}' in ${doc}`);
        }
      }
    } catch {
      // File doesn't exist, skip
    }
  }

  return {
    category: "Documentation",
    passed: issues.length === 0,
    message: issues.length === 0
      ? "✅ Documentation uses correct names"
      : "❌ Found old references in documentation",
    details: issues,
  };
}

async function checkMakefiles(): Promise<ValidationResult> {
  const issues: string[] = [];
  const makefiles = ["Makefile", "server/Makefile"];

  for (const makefile of makefiles) {
    try {
      const content = await Deno.readTextFile(makefile);

      for (const oldName of OLD_NAMES) {
        if (content.includes(oldName)) {
          issues.push(`Found reference to '${oldName}' in ${makefile}`);
        }
      }
    } catch {
      // File doesn't exist, skip
    }
  }

  return {
    category: "Makefiles",
    passed: issues.length === 0,
    message: issues.length === 0
      ? "✅ Makefiles use correct names"
      : "❌ Found old references in Makefiles",
    details: issues,
  };
}

async function runValidation() {
  console.log("🔍 Running comprehensive naming validation...\n");

  const results: ValidationResult[] = [];

  // Run all checks
  results.push(await checkCargoToml());
  results.push(await checkSourceCode());
  results.push(await checkGitHubWorkflows());
  results.push(await checkDocumentation());
  results.push(await checkMakefiles());

  // Display results
  let hasFailures = false;

  for (const result of results) {
    console.log(`${result.message}`);

    if (!result.passed) {
      hasFailures = true;
      if (result.details && result.details.length > 0) {
        for (const detail of result.details) {
          console.log(`  ${detail}`);
        }
      }
    }
    console.log();
  }

  // Summary
  console.log("📊 Summary:");
  console.log(`  Total checks: ${results.length}`);
  console.log(`  Passed: ${results.filter((r) => r.passed).length}`);
  console.log(`  Failed: ${results.filter((r) => !r.passed).length}`);

  if (hasFailures) {
    console.log("\n❌ Naming validation failed!");
    console.log("\n💡 To fix:");
    console.log("  1. Update all references to use 'paiml-mcp-agent-toolkit'");
    console.log(
      "  2. Update repository URLs to 'paiml/paiml-mcp-agent-toolkit'",
    );
    console.log("  3. Remove any references to old binary names");
    Deno.exit(1);
  } else {
    console.log("\n✅ All naming validations passed!");
  }
}

// Run validation
if (import.meta.main) {
  await runValidation();
}
