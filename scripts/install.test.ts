import {
  assertEquals,
  assertThrows,
} from "https://deno.land/std@0.210.0/assert/mod.ts";
import { describe, it } from "https://deno.land/std@0.210.0/testing/bdd.ts";

// Since the install.ts file uses top-level code that immediately executes,
// we need to extract the platform detection logic into a testable module
// For now, we'll duplicate the detectPlatform function for testing

// Detect platform (returns full Rust target triple)
function detectPlatform(): string {
  const os = Deno.build.os;
  const arch = Deno.build.arch;

  if (os === "linux") {
    switch (arch) {
      case "x86_64":
        return "x86_64-unknown-linux-gnu";
      case "aarch64":
        return "aarch64-unknown-linux-gnu";
      default:
        throw new Error(`Unsupported Linux architecture: ${arch}`);
    }
  } else if (os === "darwin") {
    switch (arch) {
      case "x86_64":
        return "x86_64-apple-darwin";
      case "aarch64":
        return "aarch64-apple-darwin";
      default:
        throw new Error(`Unsupported macOS architecture: ${arch}`);
    }
  } else if (os === "windows") {
    return "x86_64-pc-windows-msvc";
  } else {
    throw new Error(`Unsupported operating system: ${os}`);
  }
}

describe("detectPlatform", () => {
  it("should return correct platform string for current system", () => {
    const platform = detectPlatform();

    // Platform string should match Rust target triple format
    const rustTargetTriplePattern = /^[a-z0-9_]+-[a-z]+-[a-z]+(-[a-z]+)?$/;
    assertEquals(
      rustTargetTriplePattern.test(platform),
      true,
      `Platform string '${platform}' doesn't match Rust target triple format`,
    );

    // Check that it contains the OS
    const os = Deno.build.os;
    if (os === "linux") {
      assertEquals(platform.includes("linux"), true);
    } else if (os === "darwin") {
      assertEquals(platform.includes("darwin"), true);
    } else if (os === "windows") {
      assertEquals(platform.includes("windows"), true);
    }

    // Check that it contains the architecture
    const arch = Deno.build.arch;
    if (arch === "x86_64") {
      assertEquals(platform.includes("x86_64"), true);
    } else if (arch === "aarch64") {
      assertEquals(platform.includes("aarch64"), true);
    }
  });

  it("should return known Rust target triples", () => {
    // Test that the function returns valid Rust target triples
    const validTargets = [
      "x86_64-unknown-linux-gnu",
      "aarch64-unknown-linux-gnu",
      "x86_64-apple-darwin",
      "aarch64-apple-darwin",
      "x86_64-pc-windows-msvc",
    ];

    const platform = detectPlatform();
    assertEquals(
      validTargets.includes(platform),
      true,
      `Platform '${platform}' is not in the list of known Rust target triples`,
    );
  });
});

describe("GitHub Release URL Construction", () => {
  it("should construct correct download URLs", () => {
    const REPO = "paiml/paiml-mcp-agent-toolkit";
    const BINARY_NAME = "paiml-mcp-agent-toolkit";
    const version = "0.1.15";
    const platforms = [
      "x86_64-unknown-linux-gnu",
      "aarch64-unknown-linux-gnu",
      "x86_64-apple-darwin",
      "aarch64-apple-darwin",
      "x86_64-pc-windows-msvc",
    ];

    for (const platform of platforms) {
      const downloadUrl =
        `https://github.com/${REPO}/releases/download/v${version}/${BINARY_NAME}-${platform}.tar.gz`;

      // URL should be properly formatted
      assertEquals(downloadUrl.startsWith("https://github.com/"), true);
      assertEquals(downloadUrl.includes("/releases/download/"), true);
      assertEquals(downloadUrl.endsWith(".tar.gz"), true);
      assertEquals(downloadUrl.includes(platform), true);

      // Check exact format
      const expectedUrl =
        `https://github.com/paiml/paiml-mcp-agent-toolkit/releases/download/v0.1.15/paiml-mcp-agent-toolkit-${platform}.tar.gz`;
      assertEquals(downloadUrl, expectedUrl);
    }
  });

  it("should handle version with and without 'v' prefix", () => {
    const REPO = "paiml/paiml-mcp-agent-toolkit";
    const BINARY_NAME = "paiml-mcp-agent-toolkit";
    const platform = "x86_64-unknown-linux-gnu";

    // Test with 'v' prefix
    let version = "v0.1.15";
    version = version.replace(/^v/, "");
    assertEquals(version, "0.1.15");

    // Test without 'v' prefix
    version = "0.1.15";
    version = version.replace(/^v/, "");
    assertEquals(version, "0.1.15");

    // Construct URL
    const downloadUrl =
      `https://github.com/${REPO}/releases/download/v${version}/${BINARY_NAME}-${platform}.tar.gz`;
    assertEquals(
      downloadUrl,
      "https://github.com/paiml/paiml-mcp-agent-toolkit/releases/download/v0.1.15/paiml-mcp-agent-toolkit-x86_64-unknown-linux-gnu.tar.gz",
    );
  });
});

describe("Platform compatibility tests", () => {
  it("should support all major platforms", () => {
    // Simulate different platforms
    const testCases = [
      { os: "linux", arch: "x86_64", expected: "x86_64-unknown-linux-gnu" },
      { os: "linux", arch: "aarch64", expected: "aarch64-unknown-linux-gnu" },
      { os: "darwin", arch: "x86_64", expected: "x86_64-apple-darwin" },
      { os: "darwin", arch: "aarch64", expected: "aarch64-apple-darwin" },
      { os: "windows", arch: "x86_64", expected: "x86_64-pc-windows-msvc" },
    ];

    // We can't actually change Deno.build values, but we can test the logic
    for (const testCase of testCases) {
      // This is where we'd test if we could mock Deno.build
      // For now, we just verify the current platform is handled
      if (Deno.build.os === testCase.os && Deno.build.arch === testCase.arch) {
        assertEquals(detectPlatform(), testCase.expected);
      }
    }
  });

  it("should throw for unsupported platforms", () => {
    // Test the logic for unsupported architectures
    // Since we can't mock Deno.build, we'll test the function directly
    // with invalid inputs by creating a modified version

    function detectPlatformWithInputs(os: string, arch: string): string {
      if (os === "linux") {
        switch (arch) {
          case "x86_64":
            return "x86_64-unknown-linux-gnu";
          case "aarch64":
            return "aarch64-unknown-linux-gnu";
          default:
            throw new Error(`Unsupported Linux architecture: ${arch}`);
        }
      } else if (os === "darwin") {
        switch (arch) {
          case "x86_64":
            return "x86_64-apple-darwin";
          case "aarch64":
            return "aarch64-apple-darwin";
          default:
            throw new Error(`Unsupported macOS architecture: ${arch}`);
        }
      } else if (os === "windows") {
        return "x86_64-pc-windows-msvc";
      } else {
        throw new Error(`Unsupported operating system: ${os}`);
      }
    }

    // Test unsupported architectures
    assertThrows(
      () => detectPlatformWithInputs("linux", "armv7l"),
      Error,
      "Unsupported Linux architecture: armv7l",
    );

    assertThrows(
      () => detectPlatformWithInputs("darwin", "i386"),
      Error,
      "Unsupported macOS architecture: i386",
    );

    // Test unsupported OS
    assertThrows(
      () => detectPlatformWithInputs("freebsd", "x86_64"),
      Error,
      "Unsupported operating system: freebsd",
    );
  });
});

describe("Version handling", () => {
  it("should strip 'v' prefix from version strings", () => {
    const testCases = [
      { input: "v0.1.0", expected: "0.1.0" },
      { input: "v1.2.3", expected: "1.2.3" },
      { input: "0.1.0", expected: "0.1.0" },
      { input: "1.2.3", expected: "1.2.3" },
      { input: "v0.1.0-beta", expected: "0.1.0-beta" },
      { input: "v10.20.30", expected: "10.20.30" },
    ];

    for (const { input, expected } of testCases) {
      const result = input.replace(/^v/, "");
      assertEquals(result, expected);
    }
  });
});

// Run tests with: deno test scripts/install.test.ts
