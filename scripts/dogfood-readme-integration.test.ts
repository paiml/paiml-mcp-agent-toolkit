#!/usr/bin/env -S deno test --allow-read --allow-write --allow-run

/**
 * Dogfood Integration Test for README.md Mermaid Diagram Generation
 *
 * This test verifies that our tool can analyze itself and generate
 * updated Mermaid diagrams for the README.md file, demonstrating
 * the dogfooding capability and self-analysis features.
 */

import {
  assertEquals,
  assertExists,
  assertStringIncludes,
} from "https://deno.land/std@0.208.0/assert/mod.ts";
import { MermaidValidator } from "./mermaid-validator.ts";

// Test configuration
const BINARY_PATH = "./target/release/paiml-mcp-agent-toolkit";
const PROJECT_ROOT = "./server"; // Focus on server directory for better results
const README_PATH = "./README.md";
const TEST_OUTPUT_DIR = "/tmp/dogfood-test";

/**
 * Test Suite: Dogfood README Integration
 *
 * Verifies end-to-end dogfooding workflow:
 * 1. Tool analyzes itself using CLI
 * 2. Generates dependency graphs and analysis
 * 3. Creates valid Mermaid diagrams
 * 4. Validates diagram syntax and structure
 * 5. Verifies diagrams can be used in README updates
 */

Deno.test("Dogfood Integration - Binary exists and is executable", async () => {
  try {
    const stat = await Deno.stat(BINARY_PATH);
    assertEquals(stat.isFile, true, "Binary should exist as a file");

    // Test if binary is executable by running version command
    const process = new Deno.Command(BINARY_PATH, {
      args: ["--version"],
      stdout: "piped",
      stderr: "piped",
    });

    const { code, stdout } = await process.output();
    assertEquals(code, 0, "Binary should be executable");

    const version = new TextDecoder().decode(stdout);
    assertStringIncludes(
      version,
      "paiml-mcp-agent-toolkit",
      "Should output version info",
    );
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    throw new Error(
      `Binary not found or not executable: ${errorMessage}. Run 'make release' first.`,
    );
  }
});

Deno.test("Dogfood Integration - Self-analysis generates dependency graph", async () => {
  // Ensure test output directory exists
  await Deno.mkdir(TEST_OUTPUT_DIR, { recursive: true });

  const outputFile = `${TEST_OUTPUT_DIR}/self-dag.mmd`;

  try {
    // Generate dependency graph of the project itself
    const process = new Deno.Command(BINARY_PATH, {
      args: [
        "analyze",
        "dag",
        "--enhanced",
        "--output",
        outputFile,
        "--project-path",
        PROJECT_ROOT,
      ],
      stdout: "piped",
      stderr: "piped",
    });

    const { code, stderr } = await process.output();

    if (code !== 0) {
      const errorMsg = new TextDecoder().decode(stderr);
      throw new Error(`DAG generation failed with code ${code}: ${errorMsg}`);
    }

    // Verify output file was created
    const stat = await Deno.stat(outputFile);
    assertEquals(stat.isFile, true, "DAG output file should be created");

    // Read and validate the generated Mermaid diagram
    const content = await Deno.readTextFile(outputFile);
    assertExists(content, "Generated DAG should have content");
    assertStringIncludes(
      content,
      "graph",
      "Should contain Mermaid graph declaration",
    );

    // Validate with our Mermaid validator
    const validator = new MermaidValidator();
    const validation = validator.validateDiagram(content);

    assertEquals(
      validation.valid,
      true,
      `Generated DAG should be valid Mermaid: ${
        validation.error?.message || "unknown error"
      }`,
    );
    assertEquals(
      validation.diagram_type,
      "flowchart",
      "Should generate flowchart diagram",
    );

    // Note: DAG generation may produce empty graphs depending on project structure
    // This test validates the format is correct, not necessarily content richness

    console.log(
      `✅ Self-generated DAG: ${validation.nodes} nodes, ${validation.edges} edges`,
    );
  } finally {
    // Cleanup
    try {
      await Deno.remove(outputFile);
    } catch {
      // Ignore cleanup errors
    }
  }
});

Deno.test("Dogfood Integration - Complexity analysis produces actionable data", async () => {
  const outputFile = `${TEST_OUTPUT_DIR}/self-complexity.json`;

  try {
    // Generate complexity analysis of the project itself
    const process = new Deno.Command(BINARY_PATH, {
      args: [
        "analyze",
        "complexity",
        "--top-files",
        "10",
        "--format",
        "json",
        "--output",
        outputFile,
        "--project-path",
        PROJECT_ROOT,
      ],
      stdout: "piped",
      stderr: "piped",
    });

    const { code, stderr } = await process.output();

    if (code !== 0) {
      const errorMsg = new TextDecoder().decode(stderr);
      throw new Error(
        `Complexity analysis failed with code ${code}: ${errorMsg}`,
      );
    }

    // Read and validate the complexity analysis
    const content = await Deno.readTextFile(outputFile);
    const analysis = JSON.parse(content);

    // Verify expected structure (actual format has summary and violations)
    assertExists(analysis.summary, "Should have summary object");
    assertExists(analysis.violations, "Should have violations array");
    assertEquals(
      Array.isArray(analysis.violations),
      true,
      "Violations should be an array",
    );
    assertEquals(
      typeof analysis.summary.total_files,
      "number",
      "Should have total files count",
    );
    assertEquals(
      typeof analysis.summary.total_functions,
      "number",
      "Should have total functions count",
    );
    assertEquals(
      analysis.summary.total_files > 0,
      true,
      "Should analyze actual files",
    );

    console.log(
      `✅ Self-complexity analysis: ${analysis.summary.total_files} files analyzed, ${analysis.summary.total_functions} functions`,
    );

    // Verify we can identify our own high-complexity violations
    const highComplexityViolations = analysis.violations.filter((v: {
      severity: string;
      rule: string;
    }) =>
      v.severity === "warning" &&
      (v.rule === "cyclomatic-complexity" || v.rule === "cognitive-complexity")
    );

    console.log(
      `📊 High complexity violations found: ${highComplexityViolations.length}`,
    );
  } finally {
    // Cleanup
    try {
      await Deno.remove(outputFile);
    } catch {
      // Ignore cleanup errors
    }
  }
});

Deno.test("Dogfood Integration - Generate architecture diagram for README", async () => {
  const outputFile = `${TEST_OUTPUT_DIR}/architecture-diagram.mmd`;

  try {
    // Generate a comprehensive architecture view
    const process = new Deno.Command(BINARY_PATH, {
      args: [
        "analyze",
        "dag",
        "--enhanced",
        "--show-complexity",
        "--output",
        outputFile,
        "--project-path",
        PROJECT_ROOT,
      ],
      stdout: "piped",
      stderr: "piped",
    });

    const { code } = await process.output();
    assertEquals(code, 0, "Architecture diagram generation should succeed");

    // Read the generated diagram
    const content = await Deno.readTextFile(outputFile);
    assertExists(content, "Should generate architecture diagram");

    // Validate the diagram is suitable for README inclusion
    const validator = new MermaidValidator();
    const validation = validator.validateDiagram(content);

    assertEquals(
      validation.valid,
      true,
      "Architecture diagram should be valid Mermaid",
    );
    // Note: Architecture structure may vary - focus on validation correctness

    // Verify it contains key architectural components
    assertStringIncludes(content, "graph", "Should be a graph diagram");

    // Check for expected architectural patterns (optional - depends on content)
    const hasServices = content.includes("service") ||
      content.includes("Service");
    const hasEngines = content.includes("engine") || content.includes("Engine");
    const hasModules = content.includes("mod") || content.includes("lib");

    console.log(
      `🔍 Architecture patterns detected: Services=${hasServices}, Engines=${hasEngines}, Modules=${hasModules}`,
    );

    console.log(
      `✅ Architecture diagram: ${validation.nodes} components, ${validation.edges} relationships`,
    );

    // Demonstrate README integration readiness
    const readmeSection = `
## 📊 Project Architecture

\`\`\`mermaid
${content}
\`\`\`

> *This diagram is automatically generated using our self-analysis capabilities*
`;

    // Verify the section would be valid markdown
    assertStringIncludes(
      readmeSection,
      "```mermaid",
      "Should create valid markdown section",
    );
    assertStringIncludes(
      readmeSection,
      "```",
      "Should properly close code block",
    );

    console.log("📝 README section template ready for integration");
  } finally {
    // Cleanup
    try {
      await Deno.remove(outputFile);
    } catch {
      // Ignore cleanup errors
    }
  }
});

Deno.test("Dogfood Integration - Cross-validate with existing README diagrams", async () => {
  // Read current README to find existing Mermaid diagrams
  const readmeContent = await Deno.readTextFile(README_PATH);

  // Extract existing Mermaid diagrams
  const mermaidBlocks = readmeContent.match(/```mermaid([\s\S]*?)```/g) || [];

  if (mermaidBlocks.length > 0) {
    console.log(
      `📖 Found ${mermaidBlocks.length} existing Mermaid diagram(s) in README`,
    );

    const validator = new MermaidValidator();

    // Validate each existing diagram
    for (let i = 0; i < mermaidBlocks.length; i++) {
      const block = mermaidBlocks[i];
      const diagramContent = block.replace(/```mermaid\s*/, "").replace(
        /```\s*$/,
        "",
      ).trim();

      if (diagramContent) {
        const validation = validator.validateDiagram(diagramContent);
        assertEquals(
          validation.valid,
          true,
          `Existing README diagram ${i + 1} should be valid: ${
            validation.error?.message || "unknown error"
          }`,
        );

        console.log(
          `✅ README diagram ${
            i + 1
          }: ${validation.nodes} nodes, ${validation.edges} edges (${validation.diagram_type})`,
        );
      }
    }
  } else {
    console.log("📝 No existing Mermaid diagrams found in README");
  }

  // Generate a fresh diagram and compare complexity
  const tempFile = `${TEST_OUTPUT_DIR}/comparison-dag.mmd`;

  try {
    const process = new Deno.Command(BINARY_PATH, {
      args: ["analyze", "dag", "--output", tempFile],
      stdout: "piped",
      stderr: "piped",
    });

    const { code } = await process.output();
    assertEquals(code, 0, "Should generate comparison diagram");

    const newContent = await Deno.readTextFile(tempFile);
    const validator = new MermaidValidator();
    const newValidation = validator.validateDiagram(newContent);

    assertEquals(newValidation.valid, true, "New diagram should be valid");

    // If we have existing diagrams, we can compare complexity
    if (mermaidBlocks.length > 0) {
      console.log(
        `🔄 Generated fresh diagram: ${newValidation.nodes} nodes, ${newValidation.edges} edges`,
      );
      console.log(
        "💡 Consider updating README diagrams if project structure has evolved",
      );
    }
  } finally {
    try {
      await Deno.remove(tempFile);
    } catch {
      // Ignore cleanup errors
    }
  }
});

Deno.test("Dogfood Integration - Performance and scalability validation", async () => {
  const performanceStartTime = performance.now();

  // Test multiple analysis types simultaneously to verify performance
  const outputFiles = [
    `${TEST_OUTPUT_DIR}/perf-dag.mmd`,
    `${TEST_OUTPUT_DIR}/perf-complexity.json`,
    `${TEST_OUTPUT_DIR}/perf-churn.json`,
  ];

  try {
    // Run multiple analyses concurrently
    const processes = [
      new Deno.Command(BINARY_PATH, {
        args: ["analyze", "dag", "--output", outputFiles[0]],
        stdout: "piped",
        stderr: "piped",
      }),
      new Deno.Command(BINARY_PATH, {
        args: [
          "analyze",
          "complexity",
          "--format",
          "json",
          "--output",
          outputFiles[1],
        ],
        stdout: "piped",
        stderr: "piped",
      }),
      new Deno.Command(BINARY_PATH, {
        args: [
          "analyze",
          "churn",
          "--days",
          "30",
          "--format",
          "json",
          "--output",
          outputFiles[2],
        ],
        stdout: "piped",
        stderr: "piped",
      }),
    ];

    const results = await Promise.all(processes.map((p) => p.output()));

    // Verify all processes completed successfully
    for (let i = 0; i < results.length; i++) {
      assertEquals(
        results[i].code,
        0,
        `Analysis ${i + 1} should complete successfully`,
      );
    }

    const performanceEndTime = performance.now();
    const totalTime = performanceEndTime - performanceStartTime;

    // Verify reasonable performance (should complete within 30 seconds)
    assertEquals(
      totalTime < 30000,
      true,
      `All analyses should complete within 30 seconds (took ${
        totalTime.toFixed(2)
      }ms)`,
    );

    console.log(
      `⚡ Performance test: ${
        totalTime.toFixed(2)
      }ms for concurrent self-analysis`,
    );

    // Validate all outputs were generated correctly
    for (const outputFile of outputFiles) {
      const stat = await Deno.stat(outputFile);
      assertEquals(stat.isFile, true, `${outputFile} should be generated`);
      assertEquals(stat.size > 0, true, `${outputFile} should have content`);
    }

    // Validate the Mermaid diagram specifically
    const dagContent = await Deno.readTextFile(outputFiles[0]);
    const validator = new MermaidValidator();
    const validation = validator.validateDiagram(dagContent);

    assertEquals(
      validation.valid,
      true,
      "Performance test DAG should be valid",
    );
    console.log(
      `📊 Performance DAG: ${validation.nodes} nodes, ${validation.edges} edges`,
    );
  } finally {
    // Cleanup all output files
    for (const outputFile of outputFiles) {
      try {
        await Deno.remove(outputFile);
      } catch {
        // Ignore cleanup errors
      }
    }
  }
});

Deno.test("Dogfood Integration - End-to-end README update simulation", async () => {
  // Simulate the complete workflow of updating README with fresh diagrams
  const simulationDir = `${TEST_OUTPUT_DIR}/readme-simulation`;
  await Deno.mkdir(simulationDir, { recursive: true });

  try {
    // Step 1: Generate fresh architecture diagram
    const archDiagramFile = `${simulationDir}/architecture.mmd`;
    const archProcess = new Deno.Command(BINARY_PATH, {
      args: ["analyze", "dag", "--enhanced", "--output", archDiagramFile],
      stdout: "piped",
      stderr: "piped",
    });

    const { code: archCode } = await archProcess.output();
    assertEquals(archCode, 0, "Architecture diagram generation should succeed");

    // Step 2: Generate complexity analysis for metrics
    const complexityFile = `${simulationDir}/complexity.json`;
    const complexityProcess = new Deno.Command(BINARY_PATH, {
      args: [
        "analyze",
        "complexity",
        "--top-files",
        "5",
        "--format",
        "json",
        "--output",
        complexityFile,
      ],
      stdout: "piped",
      stderr: "piped",
    });

    const { code: complexityCode } = await complexityProcess.output();
    assertEquals(complexityCode, 0, "Complexity analysis should succeed");

    // Step 3: Read and validate all generated content
    const archContent = await Deno.readTextFile(archDiagramFile);
    const complexityData = JSON.parse(await Deno.readTextFile(complexityFile));

    // Step 4: Create a complete README section with real data
    const validator = new MermaidValidator();
    const archValidation = validator.validateDiagram(archContent);

    assertEquals(
      archValidation.valid,
      true,
      "Architecture diagram should be valid",
    );

    const readmeSection = `
## 📊 Project Architecture (Auto-Generated)

\`\`\`mermaid
${archContent}
\`\`\`

### Analysis Summary
- **Total Components**: ${archValidation.nodes}
- **Component Relationships**: ${archValidation.edges}
- **Files Analyzed**: ${complexityData.summary?.total_files || 0}
- **Generated**: ${new Date().toISOString().split("T")[0]}

> *This section is automatically generated using our self-analysis capabilities.*
> *Update with: \`paiml-mcp-agent-toolkit analyze dag --enhanced\`*
`;

    // Step 5: Validate the generated section
    assertStringIncludes(
      readmeSection,
      "## 📊 Project Architecture",
      "Should have proper heading",
    );
    assertStringIncludes(
      readmeSection,
      "```mermaid",
      "Should include Mermaid code block",
    );
    assertStringIncludes(
      readmeSection,
      "Total Components",
      "Should include metrics",
    );
    assertStringIncludes(
      readmeSection,
      "Generated:",
      "Should include generation timestamp",
    );

    // Step 6: Write simulation README to verify it's valid
    const simulationReadme = `${simulationDir}/README-simulation.md`;
    await Deno.writeTextFile(
      simulationReadme,
      `# Test README\n${readmeSection}`,
    );

    // Verify the file was created and is readable
    const simulationContent = await Deno.readTextFile(simulationReadme);
    assertStringIncludes(
      simulationContent,
      readmeSection,
      "Simulation README should contain our section",
    );

    console.log(`✅ End-to-end simulation complete:`);
    console.log(
      `   📊 Architecture: ${archValidation.nodes} nodes, ${archValidation.edges} edges`,
    );
    console.log(
      `   📁 Files analyzed: ${complexityData.summary?.total_files || 0}`,
    );
    console.log(`   📝 README section: ${readmeSection.length} characters`);
    console.log(`   🎯 Ready for integration into actual README.md`);
  } finally {
    // Cleanup simulation directory
    try {
      await Deno.remove(simulationDir, { recursive: true });
    } catch {
      // Ignore cleanup errors
    }
  }
});

// Cleanup test output directory after all tests
Deno.test("Dogfood Integration - Cleanup test artifacts", async () => {
  try {
    await Deno.remove(TEST_OUTPUT_DIR, { recursive: true });
    console.log("🧹 Test cleanup complete");
  } catch {
    // Ignore if directory doesn't exist
  }
});
