#!/usr/bin/env deno run --allow-read --allow-env

/**
 * Documentation Validation Script
 *
 * This script validates that the project name is consistently used across all documentation,
 * configuration files, and scripts. It ensures we don't regress from "paiml-mcp-agent-toolkit"
 * back to older names like "mcp-agent-toolkit" or "paiml-agent-toolkit".
 */

import { walk } from "https://deno.land/std@0.220.0/fs/walk.ts";
import { relative } from "https://deno.land/std@0.220.0/path/mod.ts";

const CORRECT_NAME = "paiml-mcp-agent-toolkit";
const INCORRECT_PATTERNS = [
  // Old names that should not appear (excluding test assertions)
  /(?<!paiml-)(?<!contains\(")(?<!Found incorrect ')mcp-agent-toolkit(?![-\w])(?!")(?!' )/g, // Matches "mcp-agent-toolkit" but not in test assertions
  /(?<!contains\(")(?<!Found incorrect ')paiml-agent-toolkit(?![-\w])(?!")(?!' )/g, // Matches "paiml-agent-toolkit" but not in test assertions
  /(?<!contains\(")(?<!Found old binary name ')mcp_server_stateless(?!")(?!' )/g, // Old binary name but not in test assertions
  /(?<!contains\(")(?<!Found old artifact pattern ')mcp-server-(?!")(?!' )/g, // Old release artifact names but not in test assertions
];

const EXCEPTIONS = [
  // Files that might legitimately contain old names (e.g., migration docs, changelogs)
  /CHANGELOG\.md/,
  /\.git/,
  /target/,
  /node_modules/,
  /\.DS_Store/,
  /validate-docs\.ts/, // This script itself contains the patterns we're searching for
];

interface ValidationError {
  file: string;
  line: number;
  column: number;
  pattern: string;
  context: string;
}

const errors: ValidationError[] = [];

async function validateFile(path: string): Promise<void> {
  try {
    const content = await Deno.readTextFile(path);
    const lines = content.split("\n");

    for (let lineNum = 0; lineNum < lines.length; lineNum++) {
      const line = lines[lineNum];

      for (const pattern of INCORRECT_PATTERNS) {
        const matches = [...line.matchAll(pattern)];

        for (const match of matches) {
          if (match.index !== undefined) {
            errors.push({
              file: relative(Deno.cwd(), path),
              line: lineNum + 1,
              column: match.index + 1,
              pattern: match[0],
              context: line.trim(),
            });
          }
        }
      }
    }
  } catch (error) {
    if (!(error instanceof Deno.errors.PermissionDenied)) {
      console.error(`Error reading ${path}: ${error}`);
    }
  }
}

function shouldValidateFile(path: string): boolean {
  // Check if path matches any exception patterns
  for (const exception of EXCEPTIONS) {
    if (exception.test(path)) {
      return false;
    }
  }

  // Validate specific file types
  const validExtensions = [
    ".md",
    ".json",
    ".toml",
    ".yaml",
    ".yml",
    ".ts",
    ".js",
    ".rs",
    ".sh",
    ".bash",
    "Makefile",
    "Dockerfile",
    ".hbs",
  ];

  const fileName = path.split("/").pop() || "";

  // Check if it's a Makefile (no extension)
  if (fileName === "Makefile") {
    return true;
  }

  // Check file extensions
  return validExtensions.some((ext) => path.endsWith(ext));
}

async function validateProject(): Promise<void> {
  console.log(
    `🔍 Validating project documentation for correct usage of "${CORRECT_NAME}"...\n`,
  );

  // Walk through all files
  for await (const entry of walk(Deno.cwd(), { skip: EXCEPTIONS })) {
    if (entry.isFile && shouldValidateFile(entry.path)) {
      await validateFile(entry.path);
    }
  }

  // Report results
  if (errors.length === 0) {
    console.log(`✅ All files use the correct project name "${CORRECT_NAME}"`);
    Deno.exit(0);
  } else {
    console.error(`❌ Found ${errors.length} naming inconsistencies:\n`);

    // Group errors by file
    const errorsByFile = new Map<string, ValidationError[]>();
    for (const error of errors) {
      if (!errorsByFile.has(error.file)) {
        errorsByFile.set(error.file, []);
      }
      errorsByFile.get(error.file)!.push(error);
    }

    // Display errors
    for (const [file, fileErrors] of errorsByFile) {
      console.error(`\n📄 ${file}:`);
      for (const error of fileErrors) {
        console.error(
          `   Line ${error.line}, Column ${error.column}: Found "${error.pattern}"`,
        );
        console.error(`   > ${error.context}`);
      }
    }

    console.error(
      `\n💡 All instances of old names should be replaced with "${CORRECT_NAME}"`,
    );
    Deno.exit(1);
  }
}

// Run validation
if (import.meta.main) {
  validateProject();
}
