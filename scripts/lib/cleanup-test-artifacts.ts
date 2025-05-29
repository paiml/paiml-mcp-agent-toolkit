#!/usr/bin/env -S deno run --allow-read --allow-write

/**
 * Utility function to clean up test artifacts created by installation scripts.
 * This helps prevent tar.gz files from accumulating in the working directory.
 */

export function cleanupTestArtifacts(directory: string = "."): void {
  const testArtifactPatterns = [
    /^.*-platform\.tar\.gz$/,
    /^nonexistent-platform\.tar\.gz$/,
    /^undefined-platform\.tar\.gz$/,
  ];

  try {
    for (const entry of Deno.readDirSync(directory)) {
      if (entry.isFile) {
        for (const pattern of testArtifactPatterns) {
          if (pattern.test(entry.name)) {
            const filePath = `${directory}/${entry.name}`;
            try {
              Deno.removeSync(filePath);
              console.log(`Cleaned up test artifact: ${entry.name}`);
            } catch (err) {
              console.error(`Failed to remove ${entry.name}: ${err}`);
            }
            break;
          }
        }
      }
    }
  } catch (err) {
    console.error(`Error reading directory ${directory}: ${err}`);
  }
}

/**
 * Hook to be used in test files with afterEach or afterAll
 */
export function setupTestCleanup(): () => void {
  return () => cleanupTestArtifacts();
}

// If run directly, clean current directory
if (import.meta.main) {
  cleanupTestArtifacts();
}
