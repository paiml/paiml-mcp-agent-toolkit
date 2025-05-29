#!/usr/bin/env -S deno run --allow-run --allow-read --allow-write --allow-env

import { ensureDir, exists } from "https://deno.land/std@0.208.0/fs/mod.ts";
import { join } from "https://deno.land/std@0.208.0/path/mod.ts";
import { parse } from "https://deno.land/std@0.208.0/flags/mod.ts";
import {
  bold,
  green,
  red,
  yellow,
} from "https://deno.land/std@0.208.0/fmt/colors.ts";

interface FuzzerConfig {
  name: string;
  duration: number;
  maxLen: number;
  description: string;
}

const FUZZERS: FuzzerConfig[] = [
  {
    name: "fuzz_mermaid_generation",
    duration: 300,
    maxLen: 100000,
    description: "Main Mermaid generation fuzzer",
  },
  {
    name: "fuzz_dag_builder",
    duration: 300,
    maxLen: 100000,
    description: "DAG builder fuzzer",
  },
  {
    name: "fuzz_mermaid_escaping",
    duration: 300,
    maxLen: 100000,
    description: "Special character escaping fuzzer",
  },
  {
    name: "fuzz_mermaid_performance",
    duration: 150,
    maxLen: 10000,
    description: "Performance regression fuzzer",
  },
];

async function checkCargoFuzz(): Promise<boolean> {
  try {
    const result = await new Deno.Command("cargo-fuzz", {
      args: ["--version"],
      stdout: "null",
      stderr: "null",
    }).output();
    return result.success;
  } catch {
    return false;
  }
}

async function installCargoFuzz() {
  console.log(yellow("cargo-fuzz not found. Installing..."));
  const result = await new Deno.Command("cargo", {
    args: ["install", "cargo-fuzz"],
    stdout: "inherit",
    stderr: "inherit",
  }).output();

  if (!result.success) {
    throw new Error("Failed to install cargo-fuzz");
  }
}

async function generateCorpusIfNeeded() {
  const corpusDir = "fuzz/corpus/fuzz_mermaid_generation";

  if (!await exists(corpusDir)) {
    await ensureDir(corpusDir);
  }

  const entries = [];
  for await (const entry of Deno.readDir(corpusDir)) {
    entries.push(entry);
  }

  if (entries.length === 0) {
    console.log(yellow("Generating fuzzing corpus..."));
    const result = await new Deno.Command("deno", {
      args: [
        "run",
        "--allow-write",
        "--allow-read",
        "scripts/generate-fuzz-corpus.ts",
      ],
      stdout: "inherit",
      stderr: "inherit",
    }).output();

    if (!result.success) {
      throw new Error("Failed to generate corpus");
    }
  }
}

async function runFuzzer(
  config: FuzzerConfig,
  duration?: number,
  maxLen?: number,
) {
  const actualDuration = duration ?? config.duration;
  const actualMaxLen = maxLen ?? config.maxLen;

  console.log(
    `\n${green(bold(`Running ${config.name} for ${actualDuration}s...`))}`,
  );
  console.log(`Description: ${config.description}`);

  // Create output directory
  const artifactDir = join("fuzz", "artifacts", config.name);
  await ensureDir(artifactDir);

  // Build fuzzer command
  const args = [
    "+nightly",
    "fuzz",
    "run",
    config.name,
    "--",
    `-max_total_time=${actualDuration}`,
    "-print_final_stats=1",
    `-max_len=${actualMaxLen}`,
    "-len_control=0",
    "-reduce_inputs=1",
    "-use_value_profile=1",
    `-artifact_prefix=${artifactDir}/`,
  ];

  // Run the fuzzer
  const logFile = join(artifactDir, "fuzzing.log");
  const logWriter = await Deno.open(logFile, {
    write: true,
    create: true,
    truncate: true,
  });

  const process = new Deno.Command("cargo", {
    args,
    stdout: "piped",
    stderr: "piped",
  });

  const child = process.spawn();

  // Stream output to both console and log file
  const decoder = new TextDecoder();
  const encoder = new TextEncoder();

  // Handle stdout
  (async () => {
    for await (const chunk of child.stdout) {
      const text = decoder.decode(chunk);
      await Deno.stdout.write(chunk);
      await logWriter.write(encoder.encode(text));
    }
  })();

  // Handle stderr
  (async () => {
    for await (const chunk of child.stderr) {
      const text = decoder.decode(chunk);
      await Deno.stderr.write(chunk);
      await logWriter.write(encoder.encode(text));
    }
  })();

  const status = await child.status;
  logWriter.close();

  // Check for crashes
  const crashes = [];
  try {
    for await (const entry of Deno.readDir(artifactDir)) {
      if (entry.name.startsWith("crash-")) {
        crashes.push(entry.name);
      }
    }
  } catch {
    // Directory might not exist if no crashes
  }

  if (crashes.length > 0) {
    console.log(
      red(`\n⚠️  ${crashes.length} crashes found in ${config.name}!`),
    );
    for (const crash of crashes) {
      console.log(red(`  - ${crash}`));
    }
  } else {
    console.log(green(`\n✓ No crashes found in ${config.name}`));
  }

  return { success: status.success, crashes: crashes.length };
}

async function generateCoverage(fuzzerName: string) {
  console.log(yellow("Generating coverage report..."));

  const result = await new Deno.Command("cargo", {
    args: ["+nightly", "fuzz", "coverage", fuzzerName],
    stdout: "inherit",
    stderr: "inherit",
  }).output();

  if (!result.success) {
    throw new Error("Failed to generate coverage");
  }

  // Check if lcov tools are available
  const hasLcov = await checkCommand("lcov");
  const hasGenhtml = await checkCommand("genhtml");

  if (hasLcov && hasGenhtml) {
    console.log("Generating HTML coverage report...");

    const coverageDir = `fuzz/coverage/${fuzzerName}`;
    const htmlDir = "fuzz/coverage/html";
    await ensureDir(htmlDir);

    // Extract coverage for server sources
    const lcovResult = await new Deno.Command("lcov", {
      args: [
        "--extract",
        `${coverageDir}/coverage.profdata`,
        "*/server/src/*",
        "-o",
        "fuzz/coverage/filtered.lcov",
      ],
      stdout: "inherit",
      stderr: "inherit",
    }).output();

    if (lcovResult.success) {
      // Generate HTML
      const genhtmlResult = await new Deno.Command("genhtml", {
        args: [
          "fuzz/coverage/filtered.lcov",
          "-o",
          htmlDir,
        ],
        stdout: "inherit",
        stderr: "inherit",
      }).output();

      if (genhtmlResult.success) {
        console.log(
          green(`Coverage report generated at: ${htmlDir}/index.html`),
        );
      }
    }
  }
}

async function checkCommand(cmd: string): Promise<boolean> {
  try {
    const result = await new Deno.Command(cmd, {
      args: ["--version"],
      stdout: "null",
      stderr: "null",
    }).output();
    return result.success;
  } catch {
    return false;
  }
}

async function countFiles(dir: string): Promise<number> {
  let count = 0;
  try {
    for await (const entry of Deno.readDir(dir)) {
      if (entry.isFile) count++;
      else if (entry.isDirectory) {
        count += await countFiles(join(dir, entry.name));
      }
    }
  } catch {
    // Directory might not exist
  }
  return count;
}

async function main() {
  const args = parse(Deno.args, {
    string: ["fuzzer", "duration", "max-len"],
    alias: {
      f: "fuzzer",
      d: "duration",
      m: "max-len",
    },
    default: {
      fuzzer: "all",
      duration: "300",
      "max-len": "100000",
    },
  });

  console.log(green(bold("PAIML MCP Agent Toolkit - Fuzzing Runner")));
  console.log("===========================================");

  // Check and install cargo-fuzz if needed
  if (!await checkCargoFuzz()) {
    await installCargoFuzz();
  }

  // Generate corpus if needed
  await generateCorpusIfNeeded();

  const fuzzerName = args.fuzzer as string;
  const duration = parseInt(args.duration as string);
  const maxLen = parseInt(args["max-len"] as string);

  if (fuzzerName === "all") {
    console.log("Running all fuzzers...");
    const results = [];

    for (const config of FUZZERS) {
      const result = await runFuzzer(config);
      results.push({ name: config.name, ...result });
    }

    // Summary
    console.log(`\n${yellow(bold("Summary:"))}`);
    for (const result of results) {
      const status = result.crashes > 0 ? red("FAILED") : green("PASSED");
      console.log(`- ${result.name}: ${status} (${result.crashes} crashes)`);
    }
  } else if (fuzzerName === "coverage") {
    await generateCoverage("fuzz_mermaid_generation");
  } else {
    // Run specific fuzzer
    const config = FUZZERS.find((f) => f.name === fuzzerName);
    if (!config) {
      console.error(red(`Unknown fuzzer: ${fuzzerName}`));
      console.log("Available fuzzers:");
      for (const f of FUZZERS) {
        console.log(`  - ${f.name}: ${f.description}`);
      }
      Deno.exit(1);
    }

    await runFuzzer(config, duration, maxLen);
  }

  // Final summary
  console.log(`\n${yellow(bold("Statistics:"))}`);
  console.log(`- Corpus files: ${await countFiles("fuzz/corpus")}`);
  console.log(`- Total crashes: ${await countFiles("fuzz/artifacts")}`);
  console.log(`- Logs saved in: fuzz/artifacts/`);

  console.log(`\n${green("Fuzzing session complete!")}`);
}

if (import.meta.main) {
  try {
    await main();
  } catch (error) {
    console.error(red(`Error: ${error.message}`));
    Deno.exit(1);
  }
}
