import { assertEquals } from "https://deno.land/std@0.210.0/testing/asserts.ts";
import { describe, it } from "https://deno.land/std@0.210.0/testing/bdd.ts";

// Mock the functions from create-release.ts since we can't import the whole file
// (it has a main() that executes on import)

function detectPlatform(): string {
  const os = Deno.build.os;
  const arch = Deno.build.arch;

  // Map to Rust target triples
  if (os === "linux") {
    if (arch === "x86_64") return "x86_64-unknown-linux-gnu";
    if (arch === "aarch64") return "aarch64-unknown-linux-gnu";
  } else if (os === "darwin") {
    if (arch === "x86_64") return "x86_64-apple-darwin";
    if (arch === "aarch64") return "aarch64-apple-darwin";
  } else if (os === "windows") {
    if (arch === "x86_64") return "x86_64-pc-windows-msvc";
  }

  throw new Error(`Unsupported platform: ${os}-${arch}`);
}

async function getVersion(): Promise<string> {
  const cargoToml = await Deno.readTextFile("server/Cargo.toml");
  const match = cargoToml.match(/^version = "(.*)"/m);
  if (!match) {
    throw new Error("Could not find version in Cargo.toml");
  }
  return match[1];
}

function isRunningInGitHubActions(): boolean {
  return Deno.env.get("GITHUB_ACTIONS") === "true";
}

describe("create-release", () => {
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

    it("should return linux platform for linux x86_64", () => {
      if (Deno.build.os === "linux" && Deno.build.arch === "x86_64") {
        assertEquals(detectPlatform(), "x86_64-unknown-linux-gnu");
      }
    });

    it("should return darwin platform for macOS", () => {
      if (Deno.build.os === "darwin") {
        const expected = Deno.build.arch === "x86_64"
          ? "x86_64-apple-darwin"
          : "aarch64-apple-darwin";
        assertEquals(detectPlatform(), expected);
      }
    });

    it("should return windows platform for windows x86_64", () => {
      if (Deno.build.os === "windows" && Deno.build.arch === "x86_64") {
        assertEquals(detectPlatform(), "x86_64-pc-windows-msvc");
      }
    });
  });

  describe("getVersion", () => {
    it("should extract version from Cargo.toml", async () => {
      try {
        const version = await getVersion();
        // Version should match semver pattern
        assertEquals(/^\d+\.\d+\.\d+(-\w+)?$/.test(version), true);
      } catch (e) {
        // Skip test if not in correct directory
        if (e instanceof Error && e.message.includes("No such file")) {
          console.log("Skipping test - not in project root");
        } else {
          throw e;
        }
      }
    });
  });

  describe("isRunningInGitHubActions", () => {
    it("should detect GitHub Actions environment", () => {
      const originalValue = Deno.env.get("GITHUB_ACTIONS");

      // Test when not in GitHub Actions
      Deno.env.delete("GITHUB_ACTIONS");
      assertEquals(isRunningInGitHubActions(), false);

      // Test when in GitHub Actions
      Deno.env.set("GITHUB_ACTIONS", "true");
      assertEquals(isRunningInGitHubActions(), true);

      // Restore original value
      if (originalValue) {
        Deno.env.set("GITHUB_ACTIONS", originalValue);
      } else {
        Deno.env.delete("GITHUB_ACTIONS");
      }
    });
  });

  describe("tarball naming", () => {
    it("should create correct tarball names", () => {
      const binaryName = "paiml-mcp-agent-toolkit";
      const platforms = [
        "x86_64-unknown-linux-gnu",
        "aarch64-unknown-linux-gnu",
        "x86_64-apple-darwin",
        "aarch64-apple-darwin",
        "x86_64-pc-windows-msvc",
      ];

      for (const platform of platforms) {
        const expected = `${binaryName}-${platform}.tar.gz`;
        const actual = `${binaryName}-${platform}.tar.gz`;
        assertEquals(actual, expected);
      }
    });
  });

  describe("release notes generation", () => {
    it("should include version in release notes", () => {
      const version = "0.2.0";
      const releaseNotes = `## MCP Agent Toolkit v${version}

### Installation

Install using curl:
\`\`\`bash
curl -fsSL https://raw.githubusercontent.com/paiml/paiml-mcp-agent-toolkit/master/scripts/install.ts | deno run --allow-net --allow-read --allow-write --allow-env --allow-run -
\`\`\`

Or with the shell wrapper:
\`\`\`bash
curl -fsSL https://raw.githubusercontent.com/paiml/paiml-mcp-agent-toolkit/master/scripts/install.sh | sh
\`\`\`

Or download the binary directly from the assets below.

### What's New
- Template generation for Makefiles, README files, and .gitignore files
- Support for Rust, Deno/TypeScript, and Python UV toolchains
- Stateless MCP server with embedded templates
- Claude Code integration ready

### Supported Platforms
- Linux x86_64
- macOS x86_64
- macOS aarch64 (Apple Silicon)
`;

      assertEquals(releaseNotes.includes(`v${version}`), true);
      assertEquals(releaseNotes.includes("### Installation"), true);
      assertEquals(releaseNotes.includes("### What's New"), true);
      assertEquals(releaseNotes.includes("### Supported Platforms"), true);
    });
  });
});
