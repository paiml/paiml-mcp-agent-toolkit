#!/usr/bin/env -S deno run --allow-run --allow-read --allow-write

/**
 * Swap Configuration Script
 * Increases swap from 512MB to 8GB for better system performance
 */

interface SwapConfig {
  size: string;
  swappiness: number;
}

const SWAP_CONFIG: SwapConfig = {
  size: "8G", // Increase from 512MB to 8GB
  swappiness: 10, // Good for systems with lots of RAM
};

class SwapManager {
  private swapfile = "/swapfile";

  constructor(private config: SwapConfig) {}

  async configure(): Promise<void> {
    try {
      console.log("🔧 Starting swap configuration...\n");

      // Show current status
      await this.showCurrentStatus();

      // Disable current swap
      console.log("📴 Disabling current swap...");
      await this.exec("swapoff /swapfile");

      // Remove old swapfile
      console.log("🗑️  Removing old swapfile...");
      await this.exec("rm -f /swapfile");

      // Create new swapfile
      console.log(`📦 Creating new ${this.config.size} swapfile...`);
      await this.exec(`fallocate -l ${this.config.size} /swapfile`);

      // Set permissions
      console.log("🔒 Setting permissions...");
      await this.exec("chmod 600 /swapfile");

      // Make swap
      console.log("🔨 Formatting as swap...");
      await this.exec("mkswap /swapfile");

      // Enable swap
      console.log("✅ Enabling swap...");
      await this.exec("swapon /swapfile");

      // Update fstab
      await this.updateFstab();

      // Set swappiness
      await this.setSwappiness();

      // Show new status
      console.log("\n✨ New swap configuration:");
      await this.showCurrentStatus();

      console.log("\n🎉 Swap configuration completed successfully!");
    } catch (error) {
      console.error("❌ Error configuring swap:", error);
      Deno.exit(1);
    }
  }

  private async exec(command: string): Promise<string> {
    try {
      const cmd = new Deno.Command("sudo", {
        args: ["sh", "-c", command],
        stdout: "piped",
        stderr: "piped",
      });

      const { code, stdout, stderr } = await cmd.output();

      if (code !== 0) {
        const errorText = new TextDecoder().decode(stderr);
        throw new Error(`Command failed: ${command}\n${errorText}`);
      }

      return new TextDecoder().decode(stdout);
    } catch (error) {
      throw new Error(`Failed to execute: ${command}\n${error}`);
    }
  }

  private async showCurrentStatus(): Promise<void> {
    console.log("Current memory status:");
    const freeOutput = await this.runCommand("free", ["-h"]);
    console.log(freeOutput);

    console.log("Current swap details:");
    try {
      const swapOutput = await this.runCommand("swapon", ["--show"]);
      console.log(swapOutput);
    } catch {
      console.log("No swap currently active");
    }
  }

  private async runCommand(
    command: string,
    args: string[],
  ): Promise<string> {
    const cmd = new Deno.Command(command, {
      args,
      stdout: "piped",
      stderr: "piped",
    });

    const { code, stdout } = await cmd.output();
    if (code !== 0) {
      throw new Error(`Command ${command} failed`);
    }

    return new TextDecoder().decode(stdout);
  }

  private async updateFstab(): Promise<void> {
    console.log("📝 Updating /etc/fstab...");

    const fstabContent = await Deno.readTextFile("/etc/fstab");
    const swapEntry = "/swapfile none swap sw 0 0";

    if (!fstabContent.includes("/swapfile")) {
      console.log("  Adding swapfile entry to fstab...");
      await this.exec(`echo '${swapEntry}' >> /etc/fstab`);
    } else {
      console.log("  Swapfile entry already exists in fstab");
    }
  }

  private async setSwappiness(): Promise<void> {
    console.log(`⚙️  Setting swappiness to ${this.config.swappiness}...`);

    // Set current swappiness
    await this.exec(`sysctl vm.swappiness=${this.config.swappiness}`);

    // Make persistent
    const sysctlFile = "/etc/sysctl.conf";
    const sysctlContent = await Deno.readTextFile(sysctlFile);

    if (!sysctlContent.includes("vm.swappiness")) {
      console.log("  Adding swappiness to sysctl.conf...");
      await this.exec(
        `echo 'vm.swappiness=${this.config.swappiness}' >> ${sysctlFile}`,
      );
    } else {
      console.log("  Updating swappiness in sysctl.conf...");
      const updatedContent = sysctlContent.replace(
        /vm\.swappiness=\d+/,
        `vm.swappiness=${this.config.swappiness}`,
      );
      await Deno.writeTextFile("/tmp/sysctl.conf", updatedContent);
      await this.exec("mv /tmp/sysctl.conf /etc/sysctl.conf");
    }
  }
}

// Main execution
if (import.meta.main) {
  const manager = new SwapManager(SWAP_CONFIG);

  console.log("================================");
  console.log("Swap Configuration Script");
  console.log("================================");
  console.log(`Target swap size: ${SWAP_CONFIG.size}`);
  console.log(`Target swappiness: ${SWAP_CONFIG.swappiness}`);
  console.log("================================\n");

  console.log("⚠️  This script will:");
  console.log("1. Disable current swap");
  console.log("2. Delete old swapfile");
  console.log("3. Create new 8GB swapfile");
  console.log("4. Enable and configure new swap");
  console.log("5. Make changes permanent\n");

  console.log(
    "Please run with: sudo deno run --allow-run --allow-read --allow-write config-swap.ts\n",
  );

  // Check if running as root
  if (Deno.uid() !== 0) {
    console.error("❌ This script must be run as root (use sudo)");
    Deno.exit(1);
  }

  await manager.configure();
}
