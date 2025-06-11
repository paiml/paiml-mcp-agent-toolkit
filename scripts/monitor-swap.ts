#!/usr/bin/env -S deno run --allow-run=free

/**
 * Simple swap monitor that warns when swap usage is high
 */

import {
  bold,
  green,
  red,
  yellow,
} from "https://deno.land/std@0.220.0/fmt/colors.ts";

async function getSwapInfo() {
  const cmd = new Deno.Command("free", {
    args: ["-m"],
    stdout: "piped",
  });

  const output = await cmd.output();
  const text = new TextDecoder().decode(output.stdout);
  const lines = text.split("\n");

  const memLine = lines.find((l) => l.startsWith("Mem:"));
  const swapLine = lines.find((l) => l.startsWith("Swap:"));

  if (!swapLine || !memLine) return null;

  const memParts = memLine.split(/\s+/);
  const swapParts = swapLine.split(/\s+/);

  return {
    swap: {
      total: parseInt(swapParts[1]),
      used: parseInt(swapParts[2]),
      free: parseInt(swapParts[3]),
    },
    mem: {
      total: parseInt(memParts[1]),
      used: parseInt(memParts[2]),
      free: parseInt(memParts[3]),
      available: parseInt(memParts[6] || memParts[3]),
    },
  };
}

async function checkSwapStatus() {
  const info = await getSwapInfo();
  if (!info) {
    console.error(red("Failed to get system memory info"));
    return;
  }

  const swapPercent = info.swap.total > 0
    ? Math.round((info.swap.used / info.swap.total) * 100)
    : 0;

  console.log(bold("\n=== Swap Monitor ==="));
  console.log(
    `Memory: ${info.mem.used}MB used / ${info.mem.total}MB total (${info.mem.available}MB available)`,
  );
  console.log(
    `Swap: ${info.swap.used}MB used / ${info.swap.total}MB total (${swapPercent}%)`,
  );

  if (swapPercent > 90) {
    console.log(red(bold("\n⚠️  CRITICAL: Swap usage is very high!")));
    console.log(red("This may cause system crashes during testing."));

    if (info.mem.available > info.swap.used * 2) {
      console.log(yellow("\nYou have enough RAM to clear swap safely:"));
      console.log(green("  sudo swapoff -a && sudo swapon -a"));
    } else {
      console.log(
        yellow("\nWarning: Not enough free RAM to safely clear swap."),
      );
      console.log("Consider closing some applications first.");
    }

    console.log(yellow("\nTo optimize swap settings for development:"));
    console.log(green("  ./scripts/configure-swap.ts"));
  } else if (swapPercent > 50) {
    console.log(yellow("\n⚠️  Warning: Swap usage is moderate."));
    console.log("Consider monitoring system performance.");
  } else {
    console.log(green("\n✅ Swap usage is normal."));
  }
}

// Check if running in watch mode
const watch = Deno.args.includes("--watch") || Deno.args.includes("-w");

if (watch) {
  console.log("Running in watch mode. Press Ctrl+C to exit.");
  while (true) {
    await checkSwapStatus();
    await new Promise((resolve) => setTimeout(resolve, 5000)); // Check every 5 seconds
  }
} else {
  await checkSwapStatus();
}
