#!/usr/bin/env -S deno test --allow-read --allow-write

/**
 * Comprehensive test suite for MermaidValidator
 * Tests all validation functionality, edge cases, and error handling
 */

import {
  assertEquals,
  assertExists,
  assertStringIncludes,
} from "https://deno.land/std@0.208.0/assert/mod.ts";
import { MermaidValidator } from "./mermaid-validator.ts";

Deno.test("MermaidValidator - Basic instantiation", () => {
  const validator = new MermaidValidator();
  assertExists(validator);
});

Deno.test("MermaidValidator - Valid flowchart diagram", () => {
  const validator = new MermaidValidator();
  const validDiagram = `graph TD
    A[Start] --> B{Decision}
    B -->|Yes| C[Action]
    B -->|No| D[End]
    C --> D`;

  const result = validator.validateDiagram(validDiagram);
  assertEquals(result.valid, true);
  assertEquals(result.diagram_type, "flowchart");
  assertEquals(result.nodes, 4);
  assertEquals(result.edges, 4);
});

Deno.test("MermaidValidator - Valid graph with LR direction", () => {
  const validator = new MermaidValidator();
  const validDiagram = `graph LR
    A --> B
    B --> C`;

  const result = validator.validateDiagram(validDiagram);
  assertEquals(result.valid, true);
  assertEquals(result.diagram_type, "flowchart");
  assertEquals(result.nodes, 3);
  assertEquals(result.edges, 2);
});

Deno.test("MermaidValidator - Valid sequence diagram", () => {
  const validator = new MermaidValidator();
  const validDiagram = `sequenceDiagram
    participant A
    participant B
    A->>B: Hello
    B-->>A: Hi`;

  const result = validator.validateDiagram(validDiagram);
  assertEquals(result.valid, true);
  assertEquals(result.diagram_type, "sequence");
});

Deno.test("MermaidValidator - Empty diagram should fail", () => {
  const validator = new MermaidValidator();

  const result = validator.validateDiagram("");
  assertEquals(result.valid, false);
  assertExists(result.error);
  assertEquals(result.error!.message, "Empty diagram content");
  assertEquals(result.error!.line, 1);
});

Deno.test("MermaidValidator - Whitespace-only diagram should fail", () => {
  const validator = new MermaidValidator();

  const result = validator.validateDiagram("   \n  \t  \n   ");
  assertEquals(result.valid, false);
  assertExists(result.error);
  assertEquals(result.error!.message, "Empty diagram content");
});

Deno.test("MermaidValidator - Invalid diagram type should fail", () => {
  const validator = new MermaidValidator();
  const invalidDiagram = `invalidType TD
    A --> B`;

  const result = validator.validateDiagram(invalidDiagram);
  assertEquals(result.valid, false);
  assertExists(result.error);
  assertEquals(result.error!.message, "Invalid diagram type");
  assertEquals(result.error!.line, 1);
  assertStringIncludes(
    result.error!.detail,
    "graph, flowchart, sequenceDiagram",
  );
});

Deno.test("MermaidValidator - Invalid direction should fail", () => {
  const validator = new MermaidValidator();
  const invalidDiagram = `graph INVALID
    A --> B`;

  const result = validator.validateDiagram(invalidDiagram);
  assertEquals(result.valid, false);
  assertExists(result.error);
  assertEquals(result.error!.message, "Invalid direction");
  assertEquals(result.error!.line, 1);
  assertStringIncludes(result.error!.detail, "TD, TB, BT, RL, LR");
});

Deno.test("MermaidValidator - Unmatched square brackets should fail", () => {
  const validator = new MermaidValidator();
  const invalidDiagram = `graph TD
    A[Start --> B[End]`;

  const result = validator.validateDiagram(invalidDiagram);
  assertEquals(result.valid, false);
  assertExists(result.error);
  assertEquals(result.error!.message, "Unmatched [] brackets");
  assertEquals(result.error!.line, 2);
});

Deno.test("MermaidValidator - Unmatched parentheses should fail", () => {
  const validator = new MermaidValidator();
  const invalidDiagram = `graph TD
    A(Start --> B(End)`;

  const result = validator.validateDiagram(invalidDiagram);
  assertEquals(result.valid, false);
  assertExists(result.error);
  assertEquals(result.error!.message, "Unmatched () brackets");
  assertEquals(result.error!.line, 2);
});

Deno.test("MermaidValidator - Unmatched curly braces should fail", () => {
  const validator = new MermaidValidator();
  const invalidDiagram = `graph TD
    A{Decision --> B{End}`;

  const result = validator.validateDiagram(invalidDiagram);
  assertEquals(result.valid, false);
  assertExists(result.error);
  assertEquals(result.error!.message, "Unmatched {} brackets");
  assertEquals(result.error!.line, 2);
});

Deno.test("MermaidValidator - Unmatched angle brackets should fail", () => {
  const validator = new MermaidValidator();
  const invalidDiagram = `graph TD
    A<Start> --> B<End`;

  const result = validator.validateDiagram(invalidDiagram);
  assertEquals(result.valid, false);
  assertExists(result.error);
  assertEquals(result.error!.message, "Unmatched <> brackets");
  assertEquals(result.error!.line, 2);
});

Deno.test("MermaidValidator - Unmatched double quotes should fail", () => {
  const validator = new MermaidValidator();
  const invalidDiagram = `graph TD
    A["Start] --> B["End"]`;

  const result = validator.validateDiagram(invalidDiagram);
  assertEquals(result.valid, false);
  assertExists(result.error);
  assertEquals(result.error!.message, "Unmatched quotes");
  assertEquals(result.error!.line, 2);
});

Deno.test("MermaidValidator - Unmatched single quotes should fail", () => {
  const validator = new MermaidValidator();
  const invalidDiagram = `graph TD
    A['Start'] --> B['End`;

  const result = validator.validateDiagram(invalidDiagram);
  assertEquals(result.valid, false);
  assertExists(result.error);
  assertEquals(result.error!.message, "Unmatched quotes");
  assertEquals(result.error!.line, 2);
});

Deno.test("MermaidValidator - Invalid node ID starting with number should fail", () => {
  const validator = new MermaidValidator();
  const invalidDiagram = `graph TD
    1A --> B`;

  const result = validator.validateDiagram(invalidDiagram);
  assertEquals(result.valid, false);
  assertExists(result.error);
  assertEquals(result.error!.message, "Invalid node ID");
  assertEquals(result.error!.line, 2);
  assertStringIncludes(result.error!.detail, "1A");
});

Deno.test("MermaidValidator - Invalid node ID with special characters should fail", () => {
  const validator = new MermaidValidator();
  const invalidDiagram = `graph TD
    A-B --> C`;

  const result = validator.validateDiagram(invalidDiagram);
  assertEquals(result.valid, false);
  assertExists(result.error);
  assertEquals(result.error!.message, "Invalid node ID");
  assertEquals(result.error!.line, 2);
  assertStringIncludes(result.error!.detail, "A-B");
});

Deno.test("MermaidValidator - Valid node IDs with underscores", () => {
  const validator = new MermaidValidator();
  const validDiagram = `graph TD
    _start --> my_node
    my_node --> end_node`;

  const result = validator.validateDiagram(validDiagram);
  assertEquals(result.valid, true);
  assertEquals(result.nodes, 3);
});

Deno.test("MermaidValidator - Complex valid diagram with multiple arrow types", () => {
  const validator = new MermaidValidator();
  const validDiagram = `graph TD
    A --> B
    B -.-> C
    C ==> D
    D --- E`;

  const result = validator.validateDiagram(validDiagram);
  assertEquals(result.valid, true);
  assertEquals(result.nodes, 5);
  assertEquals(result.edges, 4);
});

Deno.test("MermaidValidator - Comments should be ignored", () => {
  const validator = new MermaidValidator();
  const validDiagram = `graph TD
    %% This is a comment
    A --> B
    %% Another comment
    B --> C`;

  const result = validator.validateDiagram(validDiagram);
  assertEquals(result.valid, true);
  assertEquals(result.nodes, 3);
  assertEquals(result.edges, 2);
});

Deno.test("MermaidValidator - Node definitions with labels", () => {
  const validator = new MermaidValidator();
  const validDiagram = `graph TD
    A[Start Process]
    B{Is Valid?}
    C[Process Data]
    A --> B
    B -->|Yes| C`;

  const result = validator.validateDiagram(validDiagram);
  assertEquals(result.valid, true);
  assertEquals(result.nodes, 3);
  assertEquals(result.edges, 2);
});

Deno.test("MermaidValidator - Diagram type detection", () => {
  const validator = new MermaidValidator();
  const testCases = [
    { input: "graph TD", expected: "flowchart" },
    { input: "flowchart LR", expected: "flowchart" },
    { input: "sequenceDiagram", expected: "sequence" },
    { input: "classDiagram", expected: "class" },
    { input: "stateDiagram", expected: "state" },
    { input: "erDiagram", expected: "er" },
    { input: "gantt", expected: "gantt" },
    { input: "pie", expected: "pie" },
    { input: "unknown", expected: "unknown" },
  ];

  for (const testCase of testCases) {
    const diagram = `${testCase.input}\n    A --> B`;
    const result = validator.validateDiagram(diagram);

    if (testCase.expected === "unknown") {
      assertEquals(result.valid, false);
    } else {
      assertEquals(result.diagram_type, testCase.expected);
    }
  }
});

Deno.test("MermaidValidator - Edge counting accuracy", () => {
  const validator = new MermaidValidator();
  const diagram = `graph TD
    A --> B
    B -.-> C
    C ==> D
    D --- E
    E --> A`;

  const result = validator.validateDiagram(diagram);
  assertEquals(result.valid, true);
  assertEquals(result.edges, 5);
});

Deno.test("MermaidValidator - Node counting with reused nodes", () => {
  const validator = new MermaidValidator();
  const diagram = `graph TD
    A --> B
    B --> C
    C --> A
    A --> D`;

  const result = validator.validateDiagram(diagram);
  assertEquals(result.valid, true);
  assertEquals(result.nodes, 4); // A, B, C, D (A is reused)
});

Deno.test("MermaidValidator - Exception handling", () => {
  const validator = new MermaidValidator();

  // Mock a scenario that might cause an exception
  // by testing with extremely malformed content
  const malformedDiagram = "graph TD\n" + "\x00".repeat(1000);

  const result = validator.validateDiagram(malformedDiagram);
  // Should handle gracefully and return error result
  assertEquals(result.valid, false);
  assertExists(result.error);
});

// File I/O tests with temporary files
Deno.test("MermaidValidator - validateFile with valid file", async () => {
  const validator = new MermaidValidator();
  const tempFile = await Deno.makeTempFile({ suffix: ".mmd" });

  try {
    const validContent = `graph TD
      A --> B
      B --> C`;

    await Deno.writeTextFile(tempFile, validContent);

    const result = await validator.validateFile(tempFile);
    assertEquals(result.valid, true);
    assertEquals(result.file, tempFile);
    assertEquals(result.nodes, 3);
  } finally {
    await Deno.remove(tempFile);
  }
});

Deno.test("MermaidValidator - validateFile with invalid file", async () => {
  const validator = new MermaidValidator();
  const tempFile = await Deno.makeTempFile({ suffix: ".mmd" });

  try {
    const invalidContent = `invalidType TD
      A --> B`;

    await Deno.writeTextFile(tempFile, invalidContent);

    const result = await validator.validateFile(tempFile);
    assertEquals(result.valid, false);
    assertEquals(result.file, tempFile);
    assertExists(result.error);
  } finally {
    await Deno.remove(tempFile);
  }
});

Deno.test("MermaidValidator - validateFile with nonexistent file", async () => {
  const validator = new MermaidValidator();
  const nonexistentFile = "/tmp/nonexistent-file.mmd";

  const result = await validator.validateFile(nonexistentFile);
  assertEquals(result.valid, false);
  assertEquals(result.file, nonexistentFile);
  assertExists(result.error);
  assertStringIncludes(result.error!.message, "Failed to read file");
});

Deno.test("MermaidValidator - batchValidate with directory", async () => {
  const validator = new MermaidValidator();
  const tempDir = await Deno.makeTempDir();

  try {
    // Create valid file
    const validFile = `${tempDir}/valid.mmd`;
    await Deno.writeTextFile(
      validFile,
      `graph TD
      A --> B`,
    );

    // Create invalid file
    const invalidFile = `${tempDir}/invalid.mmd`;
    await Deno.writeTextFile(
      invalidFile,
      `invalidType TD
      A --> B`,
    );

    // Create non-mermaid file (should be ignored)
    const textFile = `${tempDir}/readme.txt`;
    await Deno.writeTextFile(textFile, "This is not a mermaid file");

    const result = await validator.batchValidate(tempDir);
    assertEquals(result.total, 2); // Only .mmd files
    assertEquals(result.valid, 1);
    assertEquals(result.invalid, 1);
    assertEquals(result.results.length, 2);

    // Check individual results
    const validResult = result.results.find((r) =>
      r.file?.includes("valid.mmd")
    );
    const invalidResult = result.results.find((r) =>
      r.file?.includes("invalid.mmd")
    );

    assertExists(validResult);
    assertExists(invalidResult);
    assertEquals(validResult!.valid, true);
    assertEquals(invalidResult!.valid, false);
  } finally {
    await Deno.remove(tempDir, { recursive: true });
  }
});

Deno.test("MermaidValidator - batchValidate with nonexistent directory", async () => {
  const validator = new MermaidValidator();
  const nonexistentDir = "/tmp/nonexistent-directory";

  const result = await validator.batchValidate(nonexistentDir);
  assertEquals(result.total, 0);
  assertEquals(result.valid, 0);
  assertEquals(result.invalid, 0);
  assertEquals(result.results.length, 0);
});

Deno.test("MermaidValidator - batchValidate with nested directories", async () => {
  const validator = new MermaidValidator();
  const tempDir = await Deno.makeTempDir();

  try {
    // Create nested structure
    const nestedDir = `${tempDir}/nested`;
    await Deno.mkdir(nestedDir);

    // Create files in root
    await Deno.writeTextFile(
      `${tempDir}/root.mmd`,
      `graph TD
      A --> B`,
    );

    // Create files in nested directory
    await Deno.writeTextFile(
      `${nestedDir}/nested.mermaid`,
      `flowchart LR
      X --> Y`,
    );

    const result = await validator.batchValidate(tempDir);
    assertEquals(result.total, 2);
    assertEquals(result.valid, 2);
    assertEquals(result.invalid, 0);
  } finally {
    await Deno.remove(tempDir, { recursive: true });
  }
});

Deno.test("MermaidValidator - Complex diagram validation", () => {
  const validator = new MermaidValidator();
  const complexDiagram = `graph TB
    Start([Start]) --> Input[/User Input/]
    Input --> Validate{Valid?}
    Validate -->|Yes| Process[Process Data]
    Validate -->|No| Error[Show Error]
    Process --> Save[(Save to DB)]
    Save --> Success[/Display Success/]
    Error --> Input
    Success --> End([End])`;

  const result = validator.validateDiagram(complexDiagram);
  assertEquals(result.valid, true);
  assertEquals(result.diagram_type, "flowchart");
  // Should correctly count unique nodes despite complex syntax
  assertEquals(result.nodes, 8);
});

Deno.test("MermaidValidator - Mixed bracket types in single line", () => {
  const validator = new MermaidValidator();
  const diagram = `graph TD
    A[Square] --> B((Circle))
    B --> C{Diamond}
    C --> D>Flag]`;

  const result = validator.validateDiagram(diagram);
  assertEquals(result.valid, true);
  assertEquals(result.nodes, 4);
});

Deno.test("MermaidValidator - Edge labels with special syntax", () => {
  const validator = new MermaidValidator();
  const diagram = `graph TD
    A -->|"Edge Label"| B
    B -.->|Edge 2| C
    C ==>|Edge 3| D`;

  const result = validator.validateDiagram(diagram);
  assertEquals(result.valid, true);
  assertEquals(result.nodes, 4);
  assertEquals(result.edges, 3);
});

// Performance test with large diagram
Deno.test("MermaidValidator - Performance with large diagram", () => {
  const validator = new MermaidValidator();

  // Generate a large but valid diagram
  let largeDiagram = "graph TD\n";
  for (let i = 0; i < 100; i++) {
    largeDiagram += `    Node${i} --> Node${i + 1}\n`;
  }

  const startTime = performance.now();
  const result = validator.validateDiagram(largeDiagram);
  const endTime = performance.now();

  assertEquals(result.valid, true);
  assertEquals(result.nodes, 101);
  assertEquals(result.edges, 100);

  // Should complete in reasonable time (less than 1 second)
  const duration = endTime - startTime;
  console.log(`Large diagram validation took ${duration.toFixed(2)}ms`);
  assertEquals(
    duration < 1000,
    true,
    "Validation should complete within 1 second",
  );
});
