#!/usr/bin/env -S deno run --allow-run --allow-read --allow-write

/**
 * Swap Configuration Script for Kubuntu
 * Increases swap space for Rust compilation
 *
 * Usage: deno run --allow-run --allow-read --allow-write swap-config.ts
 */

import { parse } from "https://deno.land/std@0.208.0/flags/mod.ts";

const args = parse(Deno.args, {
    default: {
        size: "8G",
        swappiness: 10,
        path: "/swapfile",
    },
    string: ["size", "path"],
    boolean: ["help", "check-only"],
});

if (args.help) {
    console.log(`
Swap Configuration Script for Kubuntu

Usage: deno run --allow-run --allow-read --allow-write swap-config.ts [options]

Options:
  --size <size>        Swap file size (default: 8G)
  --swappiness <num>   Swappiness value 0-100 (default: 10)
  --path <path>        Swap file path (default: /swapfile)
  --check-only         Only check current swap status
  --help               Show this help
`);
    Deno.exit(0);
}

async function runCommand(cmd: string[]): Promise<{ success: boolean; output: string }> {
    const process = new Deno.Command(cmd[0], {
        args: cmd.slice(1),
        stdout: "piped",
        stderr: "piped",
    });

    const { code, stdout, stderr } = await process.output();
    const output = new TextDecoder().decode(stdout) + new TextDecoder().decode(stderr);
    return { success: code === 0, output };
}

async function checkSwapStatus() {
    console.log("🔍 Checking current swap status...\n");

    const swapInfo = await runCommand(["swapon", "--show"]);
    console.log("Current swap devices:");
    console.log(swapInfo.output || "No swap devices found");

    const memInfo = await runCommand(["free", "-h"]);
    console.log("\nMemory information:");
    console.log(memInfo.output);
}

async function createSwapFile(path: string, size: string) {
    console.log(`\n📝 Creating swap file at ${path} with size ${size}...`);

    // Check if swap file already exists
    try {
        await Deno.stat(path);
        const response = prompt(`Swap file ${path} already exists. Remove it? (y/N): `);
        if (response?.toLowerCase() !== 'y') {
            console.log("Aborted.");
            return false;
        }

        // Disable existing swap
        console.log("Disabling existing swap...");
        await runCommand(["sudo", "swapoff", path]);

        // Remove existing file
        await runCommand(["sudo", "rm", path]);
    } catch {
        // File doesn't exist, continue
    }

    // Create new swap file
    console.log("Creating new swap file...");
    const { success, output: _output } = await runCommand([
        "sudo", "fallocate", "-l", size, path
    ]);

    if (!success) {
        console.log("fallocate failed, trying dd method...");
        const sizeInGB = parseInt(size);
        const ddResult = await runCommand([
            "sudo", "dd", "if=/dev/zero", `of=${path}`, "bs=1G", `count=${sizeInGB}`, "status=progress"
        ]);
        if (!ddResult.success) {
            console.error("Failed to create swap file:", ddResult.output);
            return false;
        }
    }

    // Set permissions
    console.log("Setting permissions...");
    await runCommand(["sudo", "chmod", "600", path]);

    // Make swap
    console.log("Formatting as swap...");
    await runCommand(["sudo", "mkswap", path]);

    // Enable swap
    console.log("Enabling swap...");
    const enableResult = await runCommand(["sudo", "swapon", path]);
    if (!enableResult.success) {
        console.error("Failed to enable swap:", enableResult.output);
        return false;
    }

    return true;
}

async function makePermanent(path: string) {
    console.log("\n📌 Making swap permanent...");

    // Check if already in fstab
    try {
        const fstab = await Deno.readTextFile("/etc/fstab");
        if (fstab.includes(path)) {
            console.log("Swap file already in /etc/fstab");
            return;
        }
    } catch {
        console.error("Could not read /etc/fstab");
        return;
    }

    // Add to fstab
    const fstabEntry = `\n${path} none swap sw 0 0\n`;
    await runCommand(["sudo", "sh", "-c", `echo '${fstabEntry}' >> /etc/fstab`]);
    console.log("Added to /etc/fstab");
}

async function setSwappiness(value: number) {
    console.log(`\n⚙️  Setting swappiness to ${value}...`);

    // Set current swappiness
    await runCommand(["sudo", "sysctl", `vm.swappiness=${value}`]);

    // Make permanent
    const sysctlPath = "/etc/sysctl.conf";
    try {
        const content = await Deno.readTextFile(sysctlPath);
        if (!content.includes("vm.swappiness")) {
            await runCommand([
                "sudo", "sh", "-c",
                `echo 'vm.swappiness=${value}' >> ${sysctlPath}`
            ]);
            console.log("Made swappiness permanent");
        } else {
            console.log("Swappiness already configured in sysctl.conf");
            console.log("You may need to manually edit /etc/sysctl.conf to update the value");
        }
    } catch {
        console.error("Could not update sysctl.conf");
    }
}

async function main() {
    console.log("🔧 Swap Configuration Script for Rust Development\n");

    // Check if running with sudo capabilities
    const { output } = await runCommand(["id", "-u"]);
    if (output.trim() === "0") {
        console.error("⚠️  Please don't run this script as root. It will ask for sudo when needed.");
        Deno.exit(1);
    }

    // Check current status
    await checkSwapStatus();

    if (args["check-only"]) {
        Deno.exit(0);
    }

    // Confirm action
    console.log(`\n📋 Configuration:`);
    console.log(`  - Swap file: ${args.path}`);
    console.log(`  - Size: ${args.size}`);
    console.log(`  - Swappiness: ${args.swappiness}`);

    const confirm = prompt("\nProceed with configuration? (y/N): ");
    if (confirm?.toLowerCase() !== 'y') {
        console.log("Aborted.");
        Deno.exit(0);
    }

    // Create swap file
    const success = await createSwapFile(args.path, args.size);
    if (!success) {
        console.error("❌ Failed to create swap file");
        Deno.exit(1);
    }

    // Make permanent
    await makePermanent(args.path);

    // Set swappiness
    await setSwappiness(Number(args.swappiness));

    // Show final status
    console.log("\n✅ Configuration complete!\n");
    await checkSwapStatus();

    console.log("\n💡 Rust compilation tips:");
    console.log("  - Set CARGO_BUILD_JOBS=2 to limit parallel compilation");
    console.log("  - Consider using 'cargo check' before 'cargo build'");
    console.log("  - Use 'cargo build --release' only when necessary");
}

// Run main
if (import.meta.main) {
    main().catch((err) => {
        console.error("❌ Error:", err);
        Deno.exit(1);
    });
}