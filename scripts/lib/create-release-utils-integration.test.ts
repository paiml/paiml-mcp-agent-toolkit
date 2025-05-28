import {
  assertRejects,
} from "https://deno.land/std@0.210.0/testing/asserts.ts";
import { describe, it } from "https://deno.land/std@0.210.0/testing/bdd.ts";
import {
  buildReleaseBinary,
  createGitHubRelease,
  getVersion,
} from "./create-release-utils.ts";

describe("create-release-utils integration tests", () => {
  describe("getVersion error case", () => {
    it("should throw error when Cargo.toml is missing version", async () => {
      // Create a temp directory with a Cargo.toml without version
      const tempDir = await Deno.makeTempDir();
      const originalCwd = Deno.cwd();

      try {
        await Deno.mkdir(`${tempDir}/server`, { recursive: true });
        await Deno.writeTextFile(
          `${tempDir}/server/Cargo.toml`,
          '[package]\nname = "test"\n',
        );

        Deno.chdir(tempDir);

        await assertRejects(
          async () => await getVersion(),
          Error,
          "Could not find version in Cargo.toml",
        );
      } finally {
        Deno.chdir(originalCwd);
        await Deno.remove(tempDir, { recursive: true });
      }
    });
  });

  describe("buildReleaseBinary integration", () => {
    it("should fail when not in project root", async () => {
      const tempDir = await Deno.makeTempDir();
      const originalCwd = Deno.cwd();

      try {
        Deno.chdir(tempDir);

        // This should fail because make server-build-binary won't work
        await assertRejects(
          async () => await buildReleaseBinary(),
          Error,
        );
      } finally {
        Deno.chdir(originalCwd);
        await Deno.remove(tempDir, { recursive: true });
      }
    });
  });

  describe("createGitHubRelease integration", () => {
    it("should fail when gh is not authenticated", async () => {
      // Save original GH_TOKEN
      const originalToken = Deno.env.get("GH_TOKEN");
      const originalGithubToken = Deno.env.get("GITHUB_TOKEN");

      try {
        // Clear tokens to ensure gh is not authenticated
        Deno.env.delete("GH_TOKEN");
        Deno.env.delete("GITHUB_TOKEN");

        // This should fail due to lack of authentication
        await assertRejects(
          async () =>
            await createGitHubRelease(
              "test-version",
              "test notes",
              "test.tar.gz",
            ),
          Error,
        );
      } finally {
        // Restore tokens
        if (originalToken) Deno.env.set("GH_TOKEN", originalToken);
        if (originalGithubToken) {
          Deno.env.set("GITHUB_TOKEN", originalGithubToken);
        }
      }
    });
  });
});
