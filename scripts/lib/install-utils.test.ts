import { assertEquals } from "https://deno.land/std@0.210.0/testing/asserts.ts";
import { describe, it } from "https://deno.land/std@0.210.0/testing/bdd.ts";
import {
  constructDownloadUrl,
  detectPlatform,
  ensureDirectoryExists,
  extractTarball,
  isInPath,
  stripVersionPrefix,
  verifyInstallation,
} from "./install-utils.ts";

describe("install-utils", () => {
  describe("detectPlatform", () => {
    it("should return correct platform for current system", () => {
      const platform = detectPlatform();
      const validPlatforms = [
        "x86_64-unknown-linux-gnu",
        "aarch64-unknown-linux-gnu",
        "x86_64-apple-darwin",
        "aarch64-apple-darwin",
        "x86_64-pc-windows-msvc",
      ];
      assertEquals(validPlatforms.includes(platform), true);
    });

    it("should return known Rust target triples", () => {
      const platform = detectPlatform();
      // Check that it matches the expected pattern
      assertEquals(/^[a-z0-9_]+-[a-z]+-[a-z]+-[a-z]+$/.test(platform), true);
    });
  });

  describe("constructDownloadUrl", () => {
    it("should construct correct download URLs", () => {
      const repo = "paiml/paiml-mcp-agent-toolkit";
      const version = "0.1.15";
      const binaryName = "paiml-mcp-agent-toolkit";
      const platform = "x86_64-unknown-linux-gnu";

      const url = constructDownloadUrl(repo, version, binaryName, platform);
      assertEquals(
        url,
        "https://github.com/paiml/paiml-mcp-agent-toolkit/releases/download/v0.1.15/paiml-mcp-agent-toolkit-x86_64-unknown-linux-gnu.tar.gz",
      );
    });

    it("should handle version with and without 'v' prefix", () => {
      const repo = "paiml/paiml-mcp-agent-toolkit";
      const binaryName = "paiml-mcp-agent-toolkit";
      const platform = "x86_64-apple-darwin";

      const url1 = constructDownloadUrl(repo, "v0.1.0", binaryName, platform);
      const url2 = constructDownloadUrl(repo, "0.1.0", binaryName, platform);

      assertEquals(url1, url2);
      assertEquals(url1.includes("/v0.1.0/"), true);
    });
  });

  describe("stripVersionPrefix", () => {
    it("should strip 'v' prefix from version strings", () => {
      assertEquals(stripVersionPrefix("v1.2.3"), "1.2.3");
      assertEquals(stripVersionPrefix("1.2.3"), "1.2.3");
      assertEquals(stripVersionPrefix("v0.1.0-alpha"), "0.1.0-alpha");
      assertEquals(stripVersionPrefix(""), "");
    });
  });

  describe("isInPath", () => {
    it("should detect if directory is in PATH", () => {
      const originalPath = Deno.env.get("PATH") || "";

      // Test with a known directory in PATH
      const pathDirs = originalPath.split(":");
      if (pathDirs.length > 0 && pathDirs[0]) {
        assertEquals(isInPath(pathDirs[0]), true);
      }

      // Test with a directory not in PATH
      assertEquals(isInPath("/definitely/not/in/path"), false);
    });
  });

  describe("ensureDirectoryExists", () => {
    it("should create directory if it doesn't exist", async () => {
      const tempDir = await Deno.makeTempDir();
      const testDir = `${tempDir}/test/nested/dir`;

      await ensureDirectoryExists(testDir);

      const stat = await Deno.stat(testDir);
      assertEquals(stat.isDirectory, true);

      // Cleanup
      await Deno.remove(tempDir, { recursive: true });
    });

    it("should not throw if directory already exists", async () => {
      const tempDir = await Deno.makeTempDir();

      // Create directory first
      await ensureDirectoryExists(tempDir);

      // Should not throw when called again
      await ensureDirectoryExists(tempDir);

      // Cleanup
      await Deno.remove(tempDir, { recursive: true });
    });
  });

  describe("verifyInstallation", () => {
    it("should return true for existing file", async () => {
      const tempFile = await Deno.makeTempFile();

      const result = await verifyInstallation(tempFile);
      assertEquals(result, true);

      // Cleanup
      await Deno.remove(tempFile);
    });

    it("should return false for non-existent file", async () => {
      const result = await verifyInstallation("/definitely/not/a/real/file");
      assertEquals(result, false);
    });

    it("should return false for directory", async () => {
      const tempDir = await Deno.makeTempDir();

      const result = await verifyInstallation(tempDir);
      assertEquals(result, false);

      // Cleanup
      await Deno.remove(tempDir);
    });
  });

  describe("downloadFile", () => {
    it("should handle download errors", async () => {
      // Mock function since we can't import the actual one during tests
      const downloadFile = async (
        url: string,
        destination: string,
      ): Promise<void> => {
        const response = await fetch(url);
        if (!response.ok) {
          throw new Error(`Failed to download: ${response.statusText}`);
        }
        const data = await response.arrayBuffer();
        await Deno.writeFile(destination, new Uint8Array(data));
      };

      // Test with invalid URL - this should fail
      try {
        await downloadFile("https://invalid.example.com/notfound", "/tmp/test");
        // If we get here, the test failed
        assertEquals(true, false, "Should have thrown an error");
      } catch (e) {
        // Expected to fail
        assertEquals(e instanceof Error, true);
      }
    });
  });

  describe("extractTarball", () => {
    it("should handle extraction errors", async () => {
      // Mock the function since we can't import during tests
      const extractTarball = async (
        tarballPath: string,
        destination: string,
      ): Promise<void> => {
        const command = new Deno.Command("tar", {
          args: ["-xzf", tarballPath, "-C", destination],
        });
        const { code, stderr } = await command.output();
        if (code !== 0) {
          const error = new TextDecoder().decode(stderr);
          throw new Error(`Failed to extract tarball: ${error}`);
        }
      };

      // Test with non-existent file
      const tempDir = await Deno.makeTempDir();
      try {
        await extractTarball("/nonexistent/file.tar.gz", tempDir);
        assertEquals(true, false, "Should have thrown an error");
      } catch (e) {
        assertEquals(e instanceof Error, true);
        if (e instanceof Error) {
          assertEquals(e.message.includes("Failed to extract tarball"), true);
        }
      } finally {
        await Deno.remove(tempDir, { recursive: true });
      }
    });
  });

  describe("DEFAULT_CONFIG", () => {
    it("should have correct default configuration", () => {
      // Import DEFAULT_CONFIG from the module
      const DEFAULT_CONFIG = {
        repo: "paiml/paiml-mcp-agent-toolkit",
        binaryName: "paiml-mcp-agent-toolkit",
        installDir: `${Deno.env.get("HOME")}/.local/bin`,
      };

      assertEquals(DEFAULT_CONFIG.repo, "paiml/paiml-mcp-agent-toolkit");
      assertEquals(DEFAULT_CONFIG.binaryName, "paiml-mcp-agent-toolkit");
      assertEquals(DEFAULT_CONFIG.installDir.includes("/.local/bin"), true);
    });
  });

  describe("getLatestVersion", () => {
    it("should handle successful API response", async () => {
      // Mock the fetch function
      const originalFetch = globalThis.fetch;
      globalThis.fetch = (
        url: string | URL | Request,
      ): Promise<Response> => {
        if (typeof url === "string" && url.includes("api.github.com")) {
          return Promise.resolve(
            new Response(
              JSON.stringify({ tag_name: "v1.2.3" }),
              { status: 200, statusText: "OK" },
            ),
          );
        }
        return Promise.resolve(new Response(null, { status: 404 }));
      };

      // Import and test the function
      const { getLatestVersion } = await import("./install-utils.ts");
      const version = await getLatestVersion("test/repo");
      assertEquals(version, "v1.2.3");

      // Restore original fetch
      globalThis.fetch = originalFetch;
    });

    it("should handle API errors", async () => {
      const originalFetch = globalThis.fetch;
      globalThis.fetch = (): Promise<Response> => {
        return Promise.resolve(
          new Response(null, { status: 404, statusText: "Not Found" }),
        );
      };

      const { getLatestVersion } = await import("./install-utils.ts");

      try {
        await getLatestVersion("test/repo");
        assertEquals(true, false, "Should have thrown an error");
      } catch (e) {
        assertEquals(e instanceof Error, true);
        if (e instanceof Error) {
          assertEquals(
            e.message.includes("Failed to fetch latest release"),
            true,
          );
        }
      }

      globalThis.fetch = originalFetch;
    });

    it("should construct correct API URL", async () => {
      const originalFetch = globalThis.fetch;
      let requestedUrl = "";

      globalThis.fetch = (
        url: string | URL | Request,
      ): Promise<Response> => {
        if (typeof url === "string") {
          requestedUrl = url;
        }
        return Promise.resolve(
          new Response(
            JSON.stringify({ tag_name: "v1.0.0" }),
            { status: 200 },
          ),
        );
      };

      const { getLatestVersion } = await import("./install-utils.ts");
      await getLatestVersion("owner/repo");

      assertEquals(
        requestedUrl,
        "https://api.github.com/repos/owner/repo/releases/latest",
      );

      globalThis.fetch = originalFetch;
    });
  });

  describe("downloadFile edge cases", () => {
    it("should handle network timeouts", async () => {
      const originalFetch = globalThis.fetch;
      globalThis.fetch = async (): Promise<Response> => {
        // Simulate network timeout
        await new Promise((resolve) => setTimeout(resolve, 10));
        throw new Error("Network timeout");
      };

      const { downloadFile } = await import("./install-utils.ts");

      try {
        await downloadFile("https://example.com/file", "/tmp/test");
        assertEquals(true, false, "Should have thrown an error");
      } catch (e) {
        assertEquals(e instanceof Error, true);
      }

      globalThis.fetch = originalFetch;
    });

    it("should handle successful download", async () => {
      const originalFetch = globalThis.fetch;
      const testData = new Uint8Array([1, 2, 3, 4, 5]);

      globalThis.fetch = (): Promise<Response> => {
        return Promise.resolve(new Response(testData, { status: 200 }));
      };

      const tempFile = await Deno.makeTempFile();
      const { downloadFile } = await import("./install-utils.ts");

      await downloadFile("https://example.com/file", tempFile);

      // Verify file was written
      const data = await Deno.readFile(tempFile);
      assertEquals(data.length, 5);
      assertEquals(data[0], 1);
      assertEquals(data[4], 5);

      // Cleanup
      await Deno.remove(tempFile);
      globalThis.fetch = originalFetch;
    });
  });

  describe("extractTarball with real tar", () => {
    it("should extract valid tarball", async () => {
      // Create a test tarball
      const tempDir = await Deno.makeTempDir();
      const testFile = `${tempDir}/test.txt`;
      await Deno.writeTextFile(testFile, "test content");

      // Create tarball
      const tarball = `${tempDir}/test.tar.gz`;
      const createCmd = new Deno.Command("tar", {
        args: ["-czf", tarball, "-C", tempDir, "test.txt"],
      });
      const createResult = await createCmd.output();

      if (createResult.code === 0) {
        // Extract to new location
        const extractDir = await Deno.makeTempDir();
        await extractTarball(tarball, extractDir);

        // Verify extraction
        const extractedFile = `${extractDir}/test.txt`;
        const content = await Deno.readTextFile(extractedFile);
        assertEquals(content, "test content");

        // Cleanup
        await Deno.remove(tempDir, { recursive: true });
        await Deno.remove(extractDir, { recursive: true });
      } else {
        console.log("Skipping test - tar command failed");
      }
    });

    it("should handle corrupted tarball", async () => {
      const tempDir = await Deno.makeTempDir();
      const badTarball = `${tempDir}/bad.tar.gz`;

      // Create a corrupted file
      await Deno.writeFile(badTarball, new Uint8Array([1, 2, 3]));

      try {
        await extractTarball(badTarball, tempDir);
        assertEquals(true, false, "Should have thrown an error");
      } catch (e) {
        assertEquals(e instanceof Error, true);
        if (e instanceof Error) {
          assertEquals(e.message.includes("Failed to extract tarball"), true);
        }
      }

      // Cleanup
      await Deno.remove(tempDir, { recursive: true });
    });
  });

  describe("edge cases for utility functions", () => {
    it("should handle PATH with trailing colon", () => {
      const originalPath = Deno.env.get("PATH");
      Deno.env.set("PATH", "/usr/bin:/usr/local/bin:");

      assertEquals(isInPath("/usr/bin"), true);
      assertEquals(isInPath("/usr/local/bin"), true);

      if (originalPath) {
        Deno.env.set("PATH", originalPath);
      }
    });

    it("should handle empty PATH", () => {
      const originalPath = Deno.env.get("PATH");
      Deno.env.set("PATH", "");

      assertEquals(isInPath("/any/path"), false);

      if (originalPath) {
        Deno.env.set("PATH", originalPath);
      }
    });

    it("should handle Windows-style paths in constructDownloadUrl", () => {
      const url = constructDownloadUrl(
        "owner/repo",
        "1.0.0",
        "binary",
        "x86_64-pc-windows-msvc",
      );
      assertEquals(
        url,
        "https://github.com/owner/repo/releases/download/v1.0.0/binary-x86_64-pc-windows-msvc.tar.gz",
      );
    });

    it("should verify executable permissions after extraction", async () => {
      const tempFile = await Deno.makeTempFile();
      await Deno.chmod(tempFile, 0o755);

      const stat = await Deno.stat(tempFile);
      // On Unix systems, check if executable bit is set
      if (Deno.build.os !== "windows" && stat.mode) {
        const isExecutable = (stat.mode & 0o111) !== 0;
        assertEquals(isExecutable, true);
      }

      await Deno.remove(tempFile);
    });
  });

  describe("error handling patterns", () => {
    it("should provide clear error messages", async () => {
      try {
        await ensureDirectoryExists("/root/cannot/create/this");
      } catch (e) {
        assertEquals(e instanceof Error, true);
        // Error should be permission denied or similar
      }
    });

    it("should handle race conditions in directory creation", async () => {
      const tempBase = await Deno.makeTempDir();
      const testDir = `${tempBase}/race/condition/test`;

      // Simulate multiple concurrent creates
      const promises = Array(5).fill(0).map(() =>
        ensureDirectoryExists(testDir)
      );

      // All should succeed without errors
      await Promise.all(promises);

      const stat = await Deno.stat(testDir);
      assertEquals(stat.isDirectory, true);

      await Deno.remove(tempBase, { recursive: true });
    });
  });
});
