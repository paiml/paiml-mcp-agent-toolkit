import { assertEquals } from "https://deno.land/std@0.210.0/assert/mod.ts";
import { describe, it } from "https://deno.land/std@0.210.0/testing/bdd.ts";

const REPO = "paiml/paiml-mcp-agent-toolkit";

describe("GitHub Release Integration Tests", () => {
  it("should verify release assets exist with correct naming", async () => {
    // Fetch the latest release info
    const response = await fetch(
      `https://api.github.com/repos/${REPO}/releases/latest`,
    );
    assertEquals(response.ok, true, "Failed to fetch latest release");

    const release = await response.json();
    const assets = release.assets || [];

    // Expected platforms
    const expectedPlatforms = [
      "x86_64-unknown-linux-gnu",
      "aarch64-unknown-linux-gnu",
      "x86_64-apple-darwin",
      "aarch64-apple-darwin",
      "x86_64-pc-windows-msvc",
    ];

    // Check that we have assets
    assertEquals(assets.length > 0, true, "No assets found in latest release");

    // Extract asset names
    const assetNames = assets.map((asset: { name: string }) => asset.name);

    // Check that at least some expected platforms have releases
    const foundPlatforms = expectedPlatforms.filter((platform) =>
      assetNames.some((name: string) => name.includes(platform))
    );

    assertEquals(
      foundPlatforms.length > 0,
      true,
      "No release assets found for any expected platform",
    );

    // Verify asset naming convention
    for (const assetName of assetNames) {
      if (assetName.endsWith(".tar.gz")) {
        // Should match pattern: paiml-mcp-agent-toolkit-{platform}.tar.gz
        const pattern =
          /^paiml-mcp-agent-toolkit-[a-z0-9_]+-[a-z]+-[a-z]+(-[a-z]+)?\.tar\.gz$/;
        assertEquals(
          pattern.test(assetName),
          true,
          `Asset name '${assetName}' doesn't match expected pattern`,
        );
      }
    }

    console.log(
      `✓ Found ${assets.length} assets in release ${release.tag_name}`,
    );
    console.log(`✓ Platforms with releases: ${foundPlatforms.join(", ")}`);
  });

  it("should verify download URLs are accessible", async () => {
    // Get latest release
    const response = await fetch(
      `https://api.github.com/repos/${REPO}/releases/latest`,
    );
    const release = await response.json();
    const version = release.tag_name.replace(/^v/, "");

    // Test constructing a download URL
    const platform = "x86_64-unknown-linux-gnu";
    const downloadUrl =
      `https://github.com/${REPO}/releases/download/v${version}/paiml-mcp-agent-toolkit-${platform}.tar.gz`;

    // Check if URL is accessible (HEAD request to avoid downloading)
    const headResponse = await fetch(downloadUrl, { method: "HEAD" });

    // GitHub redirects to S3, so we accept 302 as success
    const isAccessible = headResponse.status === 200 ||
      headResponse.status === 302;
    assertEquals(
      isAccessible,
      true,
      `Download URL not accessible: ${downloadUrl} (status: ${headResponse.status})`,
    );

    console.log(`✓ Download URL verified: ${downloadUrl}`);
  });

  it("should match installer script URL construction", async () => {
    // This test verifies that our URL construction in the installer
    // matches the actual release asset URLs

    const response = await fetch(
      `https://api.github.com/repos/${REPO}/releases/latest`,
    );
    const release = await response.json();
    const version = release.tag_name.replace(/^v/, "");
    const assets = release.assets || [];

    // Find a Linux x86_64 asset (most common)
    const linuxAsset = assets.find((
      asset: { name: string; browser_download_url: string },
    ) => asset.name.includes("x86_64-unknown-linux-gnu"));

    if (linuxAsset) {
      // Construct URL as the installer would
      const BINARY_NAME = "paiml-mcp-agent-toolkit";
      const platform = "x86_64-unknown-linux-gnu";
      const constructedUrl =
        `https://github.com/${REPO}/releases/download/v${version}/${BINARY_NAME}-${platform}.tar.gz`;

      // The browser_download_url should match our constructed URL
      assertEquals(
        linuxAsset.browser_download_url,
        constructedUrl,
        "Installer URL construction doesn't match actual release URL",
      );

      console.log(`✓ URL construction verified for ${platform}`);
    }
  });
});

// Run with: deno test --allow-net scripts/install.integration.test.ts
