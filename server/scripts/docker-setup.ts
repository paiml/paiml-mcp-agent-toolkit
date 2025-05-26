#!/usr/bin/env -S deno run --allow-run --allow-read --allow-write --allow-sys --allow-env

/**
 * Docker Installation and Repair Script
 *
 * This script handles Docker installation and common permission issues on Linux systems.
 * It can install Docker if not present and fix permission problems for the current user.
 */

import { existsSync } from "https://deno.land/std@0.208.0/fs/mod.ts";
import { parse } from "https://deno.land/std@0.208.0/flags/mod.ts";

interface DockerStatus {
  installed: boolean;
  running: boolean;
  userInGroup: boolean;
  socketPermissions: boolean;
  version?: string;
}

async function runCommand(
  cmd: string[],
): Promise<{ success: boolean; output: string; error: string }> {
  try {
    const process = new Deno.Command(cmd[0], {
      args: cmd.slice(1),
      stdout: "piped",
      stderr: "piped",
    });

    const { code, stdout, stderr } = await process.output();

    return {
      success: code === 0,
      output: new TextDecoder().decode(stdout),
      error: new TextDecoder().decode(stderr),
    };
  } catch (error) {
    return {
      success: false,
      output: "",
      error: error instanceof Error ? error.message : String(error),
    };
  }
}

async function checkDockerStatus(): Promise<DockerStatus> {
  const status: DockerStatus = {
    installed: false,
    running: false,
    userInGroup: false,
    socketPermissions: false,
  };

  // Check if Docker is installed
  const dockerCheck = await runCommand(["which", "docker"]);
  status.installed = dockerCheck.success;

  if (status.installed) {
    // Check Docker version
    const versionCheck = await runCommand(["docker", "--version"]);
    if (versionCheck.success) {
      status.version = versionCheck.output.trim();
    }

    // Check if Docker daemon is running
    const psCheck = await runCommand(["docker", "ps"]);
    status.running = psCheck.success;

    // Check if user is in docker group
    const username = Deno.env.get("USER") || "";
    const groupsCheck = await runCommand(["groups", username]);
    if (groupsCheck.success) {
      status.userInGroup = groupsCheck.output.includes("docker");
    }

    // Check socket permissions
    if (existsSync("/var/run/docker.sock")) {
      const statCheck = await runCommand([
        "stat",
        "-c",
        "%a",
        "/var/run/docker.sock",
      ]);
      if (statCheck.success) {
        const perms = statCheck.output.trim();
        status.socketPermissions = perms === "666" || perms === "660";
      }
    }
  }

  return status;
}

async function installDocker(): Promise<boolean> {
  console.log("🐳 Installing Docker...");

  // Detect distribution
  const osReleaseCheck = await runCommand(["cat", "/etc/os-release"]);
  if (!osReleaseCheck.success) {
    console.error("❌ Could not detect Linux distribution");
    return false;
  }

  const osInfo = osReleaseCheck.output.toLowerCase();

  if (osInfo.includes("ubuntu") || osInfo.includes("debian")) {
    console.log("📦 Detected Ubuntu/Debian system");

    // Update package index
    console.log("📋 Updating package index...");
    const updateResult = await runCommand(["sudo", "apt-get", "update"]);
    if (!updateResult.success) {
      console.error("❌ Failed to update package index");
      return false;
    }

    // Install prerequisites
    console.log("📦 Installing prerequisites...");
    const prereqResult = await runCommand([
      "sudo",
      "apt-get",
      "install",
      "-y",
      "ca-certificates",
      "curl",
      "gnupg",
      "lsb-release",
    ]);
    if (!prereqResult.success) {
      console.error("❌ Failed to install prerequisites");
      return false;
    }

    // Add Docker's official GPG key
    console.log("🔑 Adding Docker GPG key...");
    await runCommand(["sudo", "mkdir", "-p", "/etc/apt/keyrings"]);
    const gpgResult = await runCommand([
      "bash",
      "-c",
      "curl -fsSL https://download.docker.com/linux/ubuntu/gpg | sudo gpg --dearmor -o /etc/apt/keyrings/docker.gpg",
    ]);
    if (!gpgResult.success) {
      console.error("❌ Failed to add Docker GPG key");
      return false;
    }

    // Set up repository
    console.log("📚 Setting up Docker repository...");
    const repoResult = await runCommand([
      "bash",
      "-c",
      `echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/ubuntu $(lsb_release -cs) stable" | sudo tee /etc/apt/sources.list.d/docker.list > /dev/null`,
    ]);
    if (!repoResult.success) {
      console.error("❌ Failed to set up Docker repository");
      return false;
    }

    // Update package index again
    await runCommand(["sudo", "apt-get", "update"]);

    // Install Docker
    console.log("🐳 Installing Docker Engine...");
    const installResult = await runCommand([
      "sudo",
      "apt-get",
      "install",
      "-y",
      "docker-ce",
      "docker-ce-cli",
      "containerd.io",
      "docker-buildx-plugin",
      "docker-compose-plugin",
    ]);
    if (!installResult.success) {
      console.error("❌ Failed to install Docker");
      return false;
    }

    // Start Docker service
    console.log("🚀 Starting Docker service...");
    await runCommand(["sudo", "systemctl", "start", "docker"]);
    await runCommand(["sudo", "systemctl", "enable", "docker"]);

    return true;
  } else {
    console.error(
      "❌ Unsupported distribution. Please install Docker manually.",
    );
    console.log("📖 Visit: https://docs.docker.com/engine/install/");
    return false;
  }
}

async function fixDockerPermissions(): Promise<boolean> {
  console.log("🔧 Fixing Docker permissions...");

  const username = Deno.env.get("USER") || "";
  if (!username) {
    console.error("❌ Could not determine current user");
    return false;
  }

  // Create docker group if it doesn't exist
  console.log("👥 Ensuring docker group exists...");
  await runCommand(["sudo", "groupadd", "-f", "docker"]);

  // Add user to docker group
  console.log(`👤 Adding ${username} to docker group...`);
  const addUserResult = await runCommand([
    "sudo",
    "usermod",
    "-aG",
    "docker",
    username,
  ]);
  if (!addUserResult.success) {
    console.error("❌ Failed to add user to docker group");
    return false;
  }

  // Fix socket permissions
  console.log("🔒 Fixing socket permissions...");
  await runCommand(["sudo", "chmod", "666", "/var/run/docker.sock"]);

  // Restart Docker service
  console.log("🔄 Restarting Docker service...");
  await runCommand(["sudo", "systemctl", "restart", "docker"]);

  return true;
}

async function main() {
  const args = parse(Deno.args, {
    boolean: ["help", "install", "repair", "status"],
    alias: { h: "help", i: "install", r: "repair", s: "status" },
  });

  if (args.help || (!args.install && !args.repair && !args.status)) {
    console.log(`Docker Setup Script

Usage: deno run --allow-all docker-setup.ts [options]

Options:
  -h, --help     Show this help message
  -s, --status   Check Docker installation status
  -i, --install  Install Docker if not present
  -r, --repair   Fix Docker permission issues
  
Examples:
  # Check Docker status
  deno run --allow-all docker-setup.ts --status
  
  # Install Docker
  deno run --allow-all docker-setup.ts --install
  
  # Fix permissions
  deno run --allow-all docker-setup.ts --repair
  
  # Install and fix permissions
  deno run --allow-all docker-setup.ts --install --repair
`);
    Deno.exit(0);
  }

  console.log("🐳 Docker Setup Script");
  console.log("=====================\n");

  // Always check status first
  const status = await checkDockerStatus();

  if (args.status || (!args.install && !args.repair)) {
    console.log("📊 Docker Status:");
    console.log(`  Installed: ${status.installed ? "✅" : "❌"}`);
    if (status.version) {
      console.log(`  Version: ${status.version}`);
    }
    console.log(`  Running: ${status.running ? "✅" : "❌"}`);
    console.log(`  User in docker group: ${status.userInGroup ? "✅" : "❌"}`);
    console.log(
      `  Socket permissions OK: ${status.socketPermissions ? "✅" : "❌"}`,
    );

    if (!status.installed) {
      console.log("\n💡 Run with --install to install Docker");
    } else if (!status.userInGroup || !status.socketPermissions) {
      console.log("\n💡 Run with --repair to fix permission issues");
    }
  }

  // Install Docker if requested and not installed
  if (args.install && !status.installed) {
    const installSuccess = await installDocker();
    if (!installSuccess) {
      console.error("\n❌ Docker installation failed");
      Deno.exit(1);
    }
    console.log("\n✅ Docker installed successfully");
  }

  // Repair permissions if requested
  if (args.repair) {
    if (!status.installed) {
      console.error("\n❌ Docker is not installed. Run with --install first.");
      Deno.exit(1);
    }

    const repairSuccess = await fixDockerPermissions();
    if (!repairSuccess) {
      console.error("\n❌ Failed to fix Docker permissions");
      Deno.exit(1);
    }

    console.log("\n✅ Docker permissions fixed");
    console.log(
      "⚠️  You need to log out and back in for group membership to take effect",
    );
    console.log("💡 Or run: newgrp docker");
  }

  // Final status check
  if (args.install || args.repair) {
    console.log("\n📊 Final Docker Status:");
    const finalStatus = await checkDockerStatus();
    console.log(`  Installed: ${finalStatus.installed ? "✅" : "❌"}`);
    console.log(`  Running: ${finalStatus.running ? "✅" : "❌"}`);
    console.log(
      `  User in docker group: ${finalStatus.userInGroup ? "✅" : "❌"}`,
    );
    console.log(
      `  Socket permissions OK: ${finalStatus.socketPermissions ? "✅" : "❌"}`,
    );
  }
}

if (import.meta.main) {
  main();
}
