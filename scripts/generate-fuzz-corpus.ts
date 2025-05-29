#!/usr/bin/env -S deno run --allow-write --allow-read

import { ensureDir } from "https://deno.land/std@0.208.0/fs/mod.ts";
import { join } from "https://deno.land/std@0.208.0/path/mod.ts";

interface FuzzNode {
  id: string;
  label: string;
  node_type: "Function" | "Class" | "Module" | "Trait" | "Interface";
  complexity: number;
  file_path: string;
  line_number: number;
}

interface FuzzEdge {
  from_idx: number;
  to_idx: number;
  edge_type: "Calls" | "Imports" | "Inherits" | "Implements" | "Uses";
}

interface FuzzInput {
  nodes: FuzzNode[];
  edges: FuzzEdge[];
  options: {
    show_complexity: boolean;
    max_depth?: number;
    filter_external: boolean;
    group_by_module: boolean;
  };
}

async function generateCorpus() {
  const corpusDir = "fuzz/corpus/fuzz_mermaid_generation";
  await ensureDir(corpusDir);

  const testCases: FuzzInput[] = [
    // Minimal valid graph
    {
      nodes: [{
        id: "a",
        label: "A",
        node_type: "Function",
        complexity: 1,
        file_path: "a.rs",
        line_number: 1,
      }],
      edges: [],
      options: {
        show_complexity: false,
        filter_external: false,
        group_by_module: false,
      },
    },

    // Special characters in labels
    {
      nodes: [
        {
          id: "sql",
          label: "SELECT * FROM users WHERE id = 'test'",
          node_type: "Function",
          complexity: 5,
          file_path: "db.rs",
          line_number: 10,
        },
        {
          id: "json",
          label: '{"key": "value"}',
          node_type: "Function",
          complexity: 3,
          file_path: "json.rs",
          line_number: 20,
        },
        {
          id: "regex",
          label: "[\\w\\s]+|[\\d]{3}",
          node_type: "Function",
          complexity: 7,
          file_path: "regex.rs",
          line_number: 30,
        },
        {
          id: "pipe",
          label: "process | data | transform",
          node_type: "Function",
          complexity: 4,
          file_path: "pipe.rs",
          line_number: 40,
        },
      ],
      edges: [
        { from_idx: 0, to_idx: 1, edge_type: "Calls" },
        { from_idx: 1, to_idx: 2, edge_type: "Calls" },
      ],
      options: {
        show_complexity: true,
        filter_external: false,
        group_by_module: false,
      },
    },

    // Complex graph with all node types
    {
      nodes: [
        {
          id: "mod1",
          label: "module1",
          node_type: "Module",
          complexity: 1,
          file_path: "mod.rs",
          line_number: 1,
        },
        {
          id: "trait1",
          label: "MyTrait",
          node_type: "Trait",
          complexity: 2,
          file_path: "traits.rs",
          line_number: 10,
        },
        {
          id: "struct1",
          label: "MyStruct",
          node_type: "Class",
          complexity: 5,
          file_path: "structs.rs",
          line_number: 20,
        },
        {
          id: "fn1",
          label: "process_data",
          node_type: "Function",
          complexity: 8,
          file_path: "lib.rs",
          line_number: 30,
        },
        {
          id: "fn2",
          label: "validate_input",
          node_type: "Function",
          complexity: 12,
          file_path: "lib.rs",
          line_number: 50,
        },
        {
          id: "interface1",
          label: "DataProcessor",
          node_type: "Interface",
          complexity: 3,
          file_path: "interfaces.rs",
          line_number: 60,
        },
      ],
      edges: [
        { from_idx: 2, to_idx: 1, edge_type: "Inherits" },
        { from_idx: 3, to_idx: 4, edge_type: "Calls" },
        { from_idx: 0, to_idx: 2, edge_type: "Imports" },
        { from_idx: 2, to_idx: 5, edge_type: "Implements" },
      ],
      options: {
        show_complexity: true,
        max_depth: 5,
        filter_external: true,
        group_by_module: true,
      },
    },

    // Pathological naming
    {
      nodes: [
        {
          id: "",
          label: "Empty ID",
          node_type: "Function",
          complexity: 1,
          file_path: "empty.rs",
          line_number: 1,
        },
        {
          id: "a".repeat(200),
          label: "Long ID",
          node_type: "Function",
          complexity: 2,
          file_path: "long.rs",
          line_number: 10,
        },
        {
          id: "special!@#$%",
          label: "Special chars",
          node_type: "Function",
          complexity: 3,
          file_path: "special.rs",
          line_number: 20,
        },
        {
          id: "🦀",
          label: "Unicode",
          node_type: "Function",
          complexity: 4,
          file_path: "unicode.rs",
          line_number: 30,
        },
        {
          id: "with spaces",
          label: "Spaced ID",
          node_type: "Function",
          complexity: 5,
          file_path: "spaces.rs",
          line_number: 40,
        },
      ],
      edges: [],
      options: {
        show_complexity: true,
        filter_external: false,
        group_by_module: false,
      },
    },

    // Deep nesting (chain of calls)
    generateDeepGraph(20),

    // Wide graph (many siblings)
    generateWideGraph(50),

    // Disconnected components
    {
      nodes: [
        {
          id: "a1",
          label: "ComponentA1",
          node_type: "Function",
          complexity: 2,
          file_path: "a.rs",
          line_number: 1,
        },
        {
          id: "a2",
          label: "ComponentA2",
          node_type: "Function",
          complexity: 3,
          file_path: "a.rs",
          line_number: 10,
        },
        {
          id: "b1",
          label: "ComponentB1",
          node_type: "Function",
          complexity: 4,
          file_path: "b.rs",
          line_number: 1,
        },
        {
          id: "b2",
          label: "ComponentB2",
          node_type: "Function",
          complexity: 5,
          file_path: "b.rs",
          line_number: 10,
        },
      ],
      edges: [
        { from_idx: 0, to_idx: 1, edge_type: "Calls" },
        { from_idx: 2, to_idx: 3, edge_type: "Calls" },
      ],
      options: {
        show_complexity: false,
        filter_external: false,
        group_by_module: false,
      },
    },

    // Cyclic dependencies
    {
      nodes: [
        {
          id: "fn_a",
          label: "function_a",
          node_type: "Function",
          complexity: 6,
          file_path: "cycle.rs",
          line_number: 1,
        },
        {
          id: "fn_b",
          label: "function_b",
          node_type: "Function",
          complexity: 7,
          file_path: "cycle.rs",
          line_number: 10,
        },
        {
          id: "fn_c",
          label: "function_c",
          node_type: "Function",
          complexity: 8,
          file_path: "cycle.rs",
          line_number: 20,
        },
      ],
      edges: [
        { from_idx: 0, to_idx: 1, edge_type: "Calls" },
        { from_idx: 1, to_idx: 2, edge_type: "Calls" },
        { from_idx: 2, to_idx: 0, edge_type: "Calls" },
      ],
      options: {
        show_complexity: true,
        filter_external: false,
        group_by_module: false,
      },
    },

    // All edge types
    {
      nodes: [
        {
          id: "base",
          label: "BaseClass",
          node_type: "Class",
          complexity: 10,
          file_path: "base.rs",
          line_number: 1,
        },
        {
          id: "derived",
          label: "DerivedClass",
          node_type: "Class",
          complexity: 15,
          file_path: "derived.rs",
          line_number: 1,
        },
        {
          id: "trait",
          label: "MyTrait",
          node_type: "Trait",
          complexity: 2,
          file_path: "traits.rs",
          line_number: 1,
        },
        {
          id: "fn",
          label: "helper",
          node_type: "Function",
          complexity: 5,
          file_path: "helpers.rs",
          line_number: 1,
        },
        {
          id: "mod",
          label: "utils",
          node_type: "Module",
          complexity: 1,
          file_path: "utils.rs",
          line_number: 1,
        },
      ],
      edges: [
        { from_idx: 1, to_idx: 0, edge_type: "Inherits" },
        { from_idx: 1, to_idx: 2, edge_type: "Implements" },
        { from_idx: 0, to_idx: 3, edge_type: "Calls" },
        { from_idx: 4, to_idx: 3, edge_type: "Imports" },
        { from_idx: 0, to_idx: 2, edge_type: "Uses" },
      ],
      options: {
        show_complexity: true,
        filter_external: false,
        group_by_module: false,
      },
    },

    // High complexity nodes
    {
      nodes: [
        {
          id: "low",
          label: "simple_function",
          node_type: "Function",
          complexity: 1,
          file_path: "complex.rs",
          line_number: 1,
        },
        {
          id: "medium",
          label: "moderate_function",
          node_type: "Function",
          complexity: 10,
          file_path: "complex.rs",
          line_number: 10,
        },
        {
          id: "high",
          label: "complex_function",
          node_type: "Function",
          complexity: 20,
          file_path: "complex.rs",
          line_number: 20,
        },
        {
          id: "extreme",
          label: "extremely_complex",
          node_type: "Function",
          complexity: 255,
          file_path: "complex.rs",
          line_number: 30,
        },
      ],
      edges: [],
      options: {
        show_complexity: true,
        filter_external: false,
        group_by_module: false,
      },
    },
  ];

  // Write corpus files
  for (let i = 0; i < testCases.length; i++) {
    const data = serializeInput(testCases[i]);
    await Deno.writeFile(
      join(corpusDir, `seed_${i.toString().padStart(3, "0")}`),
      data,
    );
  }

  console.log(`Generated ${testCases.length} corpus files in ${corpusDir}`);
}

function generateDeepGraph(depth: number): FuzzInput {
  const nodes: FuzzNode[] = [];
  const edges: FuzzEdge[] = [];

  for (let i = 0; i < depth; i++) {
    nodes.push({
      id: `level_${i}`,
      label: `Function at level ${i}`,
      node_type: "Function",
      complexity: i + 1,
      file_path: `level${i}.rs`,
      line_number: i * 10,
    });

    if (i > 0) {
      edges.push({
        from_idx: i - 1,
        to_idx: i,
        edge_type: "Calls",
      });
    }
  }

  return {
    nodes,
    edges,
    options: {
      show_complexity: true,
      filter_external: false,
      group_by_module: false,
    },
  };
}

function generateWideGraph(width: number): FuzzInput {
  const nodes: FuzzNode[] = [];
  const edges: FuzzEdge[] = [];

  // Root node
  nodes.push({
    id: "root",
    label: "Root Function",
    node_type: "Function",
    complexity: 10,
    file_path: "root.rs",
    line_number: 1,
  });

  // Children
  for (let i = 0; i < width; i++) {
    nodes.push({
      id: `child_${i}`,
      label: `Child ${i}`,
      node_type: i % 2 === 0 ? "Function" : "Class",
      complexity: (i % 10) + 1,
      file_path: `child${i}.rs`,
      line_number: i * 5,
    });

    edges.push({
      from_idx: 0,
      to_idx: i + 1,
      edge_type: i % 3 === 0 ? "Calls" : i % 3 === 1 ? "Imports" : "Uses",
    });
  }

  return {
    nodes,
    edges,
    options: {
      show_complexity: false,
      filter_external: false,
      group_by_module: false,
    },
  };
}

function serializeInput(input: FuzzInput): Uint8Array {
  // Create a binary representation that libfuzzer can parse
  const encoder = new TextEncoder();

  // Simple serialization format for fuzzing
  const binary: number[] = [];

  // Number of nodes (1 byte)
  binary.push(Math.min(input.nodes.length, 255));

  // Nodes
  for (const node of input.nodes.slice(0, 255)) {
    // ID length and data
    const idBytes = encoder.encode(node.id);
    binary.push(Math.min(idBytes.length, 255));
    binary.push(...Array.from(idBytes.slice(0, 255)));

    // Label length and data
    const labelBytes = encoder.encode(node.label);
    binary.push(Math.min(labelBytes.length, 255));
    binary.push(...Array.from(labelBytes.slice(0, 255)));

    // Node type (1 byte)
    const typeMap = {
      "Function": 0,
      "Class": 1,
      "Module": 2,
      "Trait": 3,
      "Interface": 4,
    };
    binary.push(typeMap[node.node_type] || 0);

    // Complexity (1 byte)
    binary.push(Math.min(node.complexity, 255));

    // File path length and data
    const pathBytes = encoder.encode(node.file_path);
    binary.push(Math.min(pathBytes.length, 255));
    binary.push(...Array.from(pathBytes.slice(0, 255)));

    // Line number (2 bytes)
    binary.push((node.line_number >> 8) & 0xFF);
    binary.push(node.line_number & 0xFF);
  }

  // Number of edges (1 byte)
  binary.push(Math.min(input.edges.length, 255));

  // Edges
  for (const edge of input.edges.slice(0, 255)) {
    binary.push(edge.from_idx);
    binary.push(edge.to_idx);

    // Edge type (1 byte)
    const typeMap = {
      "Calls": 0,
      "Imports": 1,
      "Inherits": 2,
      "Implements": 3,
      "Uses": 4,
    };
    binary.push(typeMap[edge.edge_type] || 0);
  }

  // Options (4 bytes)
  binary.push(input.options.show_complexity ? 1 : 0);
  binary.push(input.options.max_depth || 0);
  binary.push(input.options.filter_external ? 1 : 0);
  binary.push(input.options.group_by_module ? 1 : 0);

  return new Uint8Array(binary);
}

// Run the generator
if (import.meta.main) {
  await generateCorpus();
}
