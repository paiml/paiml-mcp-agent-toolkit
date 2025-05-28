#!/usr/bin/env -S deno run --allow-net --allow-read --allow-write --allow-env --allow-run

/**
 * Test script to verify curl installation would work
 */

const GREEN = "\x1b[32m";
const RED = "\x1b[31m";
const NC = "\x1b[0m";

async function test(): Promise<boolean> {
  console.log("Testing curl installation process...");
  console.log("==================================");

  let allPassed = true;

  // Test 1: Can we fetch the install script?
  console.log("\n1. Testing install script fetch from GitHub...");
  try {
    const response = await fetch(
      "https://raw.githubusercontent.com/paiml/paiml-mcp-agent-toolkit/master/scripts/install.ts",
    );
    if (response.ok) {
      const text = await response.text();
      await Deno.writeTextFile("/tmp/test-install.ts", text);
      console.log(`${GREEN}✅ Successfully fetched install script${NC}`);
    } else {
      console.log(
        `${RED}❌ Failed to fetch install script: ${response.status}${NC}`,
      );
      allPassed = false;
    }
  } catch (e) {
    console.log(`${RED}❌ Failed to fetch install script: ${e}${NC}`);
    allPassed = false;
  }

  // Test 2: Is the script valid TypeScript?
  console.log("\n2. Validating install script...");
  try {
    const cmd = new Deno.Command("deno", {
      args: ["check", "/tmp/test-install.ts"],
    });
    const { code } = await cmd.output();
    if (code === 0) {
      console.log(`${GREEN}✅ Install script syntax is valid${NC}`);
    } else {
      console.log(`${RED}❌ Install script has syntax errors${NC}`);
      allPassed = false;
    }
  } catch (e) {
    console.log(`${RED}❌ Could not validate script: ${e}${NC}`);
    allPassed = false;
  }

  // Test 3: Check script permissions and shebang
  console.log("\n3. Checking script header...");
  try {
    const content = await Deno.readTextFile("/tmp/test-install.ts");
    if (content.startsWith("#!/usr/bin/env")) {
      console.log(`${GREEN}✅ Script has proper shebang${NC}`);
    } else {
      console.log(`${RED}❌ Script missing proper shebang${NC}`);
      allPassed = false;
    }
  } catch (e) {
    console.log(`${RED}❌ Could not read script: ${e}${NC}`);
    allPassed = false;
  }

  // Test 4: Check OS/arch detection
  console.log("\n4. Testing OS/arch detection...");
  const os = Deno.build.os;
  const arch = Deno.build.arch;

  const osMap: Record<string, string> = {
    "linux": "linux",
    "darwin": "macos",
    "windows": "windows",
  };

  const archMap: Record<string, string> = {
    "x86_64": "x86_64",
    "aarch64": "aarch64",
  };

  const mappedOs = osMap[os] || "unknown";
  const mappedArch = archMap[arch] || "unknown";

  console.log(`   Detected OS: ${mappedOs}`);
  console.log(`   Detected Arch: ${mappedArch}`);

  if (mappedOs !== "unknown" && mappedArch !== "unknown") {
    console.log(`${GREEN}✅ OS/arch detection would work${NC}`);
  } else {
    console.log(`${RED}❌ OS/arch detection failed${NC}`);
    allPassed = false;
  }

  // Test 5: Check GitHub API access
  console.log("\n5. Testing GitHub API access...");
  try {
    const response = await fetch(
      "https://api.github.com/repos/paiml/paiml-mcp-agent-toolkit/releases",
    );
    if (response.ok) {
      console.log(`${GREEN}✅ Can access GitHub API${NC}`);
    } else {
      console.log(`${RED}❌ Cannot access GitHub API: ${response.status}${NC}`);
      allPassed = false;
    }
  } catch (e) {
    console.log(`${RED}❌ Cannot access GitHub API: ${e}${NC}`);
    allPassed = false;
  }

  // Test 6: Show what URL would be constructed
  console.log("\n6. Installation URL that would be used:");
  const version = "0.1.4";
  console.log(
    `   https://github.com/paiml/paiml-mcp-agent-toolkit/releases/download/v${version}/paiml-mcp-agent-toolkit-${mappedOs}-${mappedArch}.tar.gz`,
  );

  // Cleanup
  try {
    await Deno.remove("/tmp/test-install.ts");
  } catch {
    // Ignore cleanup errors
  }

  return allPassed;
}

// Main
async function main(): Promise<void> {
  const passed = await test();

  console.log("\n==================================");
  if (passed) {
    console.log(`${GREEN}✅ All pre-installation checks passed!${NC}`);
    console.log(
      "\nNote: Actual installation will fail until a GitHub release is created with the binary artifacts.",
    );
    console.log("\nTo test locally, you can:");
    console.log("1. Build the binary: make server-build-binary");
    console.log(
      "2. Create a test tarball: tar -czf paiml-mcp-agent-toolkit-linux-x86_64.tar.gz -C server/target/release paiml-mcp-agent-toolkit",
    );
    console.log("3. Upload as a GitHub release asset");
  } else {
    console.log(`${RED}❌ Some checks failed!${NC}`);
    Deno.exit(1);
  }
}

if (import.meta.main) {
  main();
}
