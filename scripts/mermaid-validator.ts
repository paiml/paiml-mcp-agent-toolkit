#!/usr/bin/env -S deno run --allow-read --allow-net

/**
 * Mermaid Specification Compliance Validator
 *
 * Validates Mermaid diagram syntax using basic parsing and validation rules
 * Returns detailed error information for invalid diagrams
 */

import { parse } from "https://deno.land/std@0.208.0/flags/mod.ts";
import { walk } from "https://deno.land/std@0.208.0/fs/walk.ts";

interface ValidationResult {
  valid: boolean;
  diagram_type?: string;
  nodes?: number;
  edges?: number;
  error?: ValidationError;
  file?: string;
  syntax_tree?: unknown;
}

interface ValidationError {
  message: string;
  line: number | null;
  detail: string;
  position?: unknown;
}

interface BatchValidationResult {
  total: number;
  valid: number;
  invalid: number;
  results: ValidationResult[];
}

/**
 * Validates Mermaid diagram syntax using rule-based validation
 * Since we can't easily import mermaid.js in Deno, we use comprehensive syntax checking
 */
class MermaidValidator {
  private readonly validDiagramTypes = [
    "graph",
    "flowchart",
    "sequenceDiagram",
    "classDiagram",
    "stateDiagram",
    "erDiagram",
    "journey",
    "gantt",
    "pie",
    "gitGraph",
    "mindmap",
    "timeline",
  ];

  private readonly validDirections = ["TD", "TB", "BT", "RL", "LR"];

  private readonly arrowPatterns = [
    "->>",
    "-->>",
    "-->",
    "-.->",
    "==>",
    "---",
    "--.",
    "-.",
    "=",
    "..",
    "~~",
  ];

  validateDiagram(mmdContent: string): ValidationResult {
    try {
      // Check for extremely malformed content that could cause issues
      if (mmdContent.includes("\x00")) {
        return {
          valid: false,
          error: {
            message: "Invalid characters in diagram",
            line: null,
            detail: "Diagram contains null bytes or invalid characters",
          },
        };
      }

      // Basic syntax validation
      const syntaxCheck = this.validateSyntax(mmdContent);
      if (!syntaxCheck.valid) {
        return syntaxCheck;
      }

      // Detailed validation
      const detailedValidation = this.performDetailedValidation(mmdContent);
      if (!detailedValidation.valid) {
        return detailedValidation;
      }

      return {
        valid: true,
        diagram_type: this.detectDiagramType(mmdContent),
        nodes: this.countNodes(mmdContent),
        edges: this.countEdges(mmdContent),
        syntax_tree: null, // Would be populated by real mermaid parser
      };
    } catch (error) {
      const errorMessage = error instanceof Error
        ? error.message
        : String(error);
      return {
        valid: false,
        error: {
          message: errorMessage,
          line: null,
          detail: `Validation error: ${errorMessage}`,
        },
      };
    }
  }

  private validateSyntax(content: string): ValidationResult {
    const trimmed = content.trim();
    if (!trimmed) {
      return {
        valid: false,
        error: {
          message: "Empty diagram content",
          line: 1,
          detail: "Diagram cannot be empty",
        },
      };
    }

    const lines = trimmed.split("\n");
    const firstLine = lines[0].trim().toLowerCase();

    // Check for valid diagram type
    const hasValidType = this.validDiagramTypes.some((type) =>
      firstLine.startsWith(type.toLowerCase())
    );

    if (!hasValidType) {
      return {
        valid: false,
        error: {
          message: "Invalid diagram type",
          line: 1,
          detail: `First line must start with a valid diagram type: ${
            this.validDiagramTypes.join(", ")
          }`,
        },
      };
    }

    // Validate direction for graph/flowchart
    if (firstLine.startsWith("graph") || firstLine.startsWith("flowchart")) {
      const parts = firstLine.split(/\s+/);
      if (parts.length > 1) {
        const direction = parts[1].toUpperCase();
        if (!this.validDirections.includes(direction)) {
          return {
            valid: false,
            error: {
              message: "Invalid direction",
              line: 1,
              detail: `Direction must be one of: ${
                this.validDirections.join(", ")
              }`,
            },
          };
        }
      }
    }

    return { valid: true };
  }

  private performDetailedValidation(content: string): ValidationResult {
    const lines = content.split("\n");
    const diagramType = this.detectDiagramType(content);

    for (let i = 0; i < lines.length; i++) {
      const line = lines[i].trim();
      if (!line || line.startsWith("%")) continue; // Skip empty lines and comments

      const lineValidation = this.validateLine(line, i + 1, diagramType);
      if (!lineValidation.valid) {
        return lineValidation;
      }
    }

    return { valid: true };
  }

  private validateLine(
    line: string,
    lineNumber: number,
    diagramType?: string,
  ): ValidationResult {
    // Skip style lines, participant declarations, and other special syntax
    if (
      line.toLowerCase().includes("style") ||
      line.includes("fill:") ||
      line.toLowerCase().includes("participant")
    ) {
      return { valid: true };
    }

    // Check for unmatched quotes FIRST (before any other processing)
    const quotes = ['"', "'"];
    for (const quote of quotes) {
      const quoteCount = (line.match(new RegExp(quote, "g")) || []).length;
      if (quoteCount % 2 !== 0) {
        return {
          valid: false,
          error: {
            message: "Unmatched quotes",
            line: lineNumber,
            detail: `Line has unmatched ${quote} quotes`,
          },
        };
      }
    }

    // Now remove quoted content to avoid false positives in bracket checking
    let cleanLine = line;
    cleanLine = cleanLine.replace(/"[^"]*"/g, "");
    cleanLine = cleanLine.replace(/'[^']*'/g, "");

    // Then remove arrow patterns
    for (const pattern of this.arrowPatterns) {
      const escapedPattern = pattern.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
      cleanLine = cleanLine.replace(new RegExp(escapedPattern, "g"), " ");
    }

    // For mixed bracket validation, handle Mermaid special node shapes
    // Remove known Mermaid node shape patterns before bracket checking
    cleanLine = cleanLine.replace(/>\s*\w+\]/g, ""); // Remove >Flag] type patterns
    cleanLine = cleanLine.replace(/\w+\s*</g, ""); // Remove Flag< type patterns

    // Check for unmatched brackets on the cleaned line
    const brackets = ["[]", "()", "{}", "<>"];
    for (const bracket of brackets) {
      const open = bracket[0];
      const close = bracket[1];
      const openCount =
        (cleanLine.match(new RegExp(`\\${open}`, "g")) || []).length;
      const closeCount =
        (cleanLine.match(new RegExp(`\\${close}`, "g")) || []).length;

      if (openCount !== closeCount) {
        return {
          valid: false,
          error: {
            message: `Unmatched ${bracket} brackets`,
            line: lineNumber,
            detail:
              `Line has ${openCount} opening and ${closeCount} closing ${bracket} brackets`,
          },
        };
      }
    }

    // Skip node ID validation for sequence diagrams (use different syntax)
    if (diagramType !== "sequence") {
      // Validate node IDs (must start with letter or underscore)
      const nodeIdPattern = /^[a-zA-Z_][a-zA-Z0-9_]*$/;
      const arrowIndex = this.findFirstArrow(line);

      if (arrowIndex !== -1) {
        const beforeArrow = line.substring(0, arrowIndex).trim();
        const afterArrowFull = line.substring(
          arrowIndex + this.getArrowLength(line, arrowIndex),
        ).trim();

        // Handle edge labels like -->|label| by finding the target after the label
        const afterArrow = this.removeEdgeLabel(afterArrowFull);

        // Extract node ID from before arrow (handle node definitions with brackets)
        const sourceId = this.extractNodeId(beforeArrow);
        const targetId = this.extractNodeId(afterArrow);

        if (sourceId && !nodeIdPattern.test(sourceId)) {
          return {
            valid: false,
            error: {
              message: "Invalid node ID",
              line: lineNumber,
              detail:
                `Node ID '${sourceId}' must start with letter or underscore and contain only alphanumeric characters`,
            },
          };
        }

        if (targetId && !nodeIdPattern.test(targetId)) {
          return {
            valid: false,
            error: {
              message: "Invalid node ID",
              line: lineNumber,
              detail:
                `Node ID '${targetId}' must start with letter or underscore and contain only alphanumeric characters`,
            },
          };
        }
      }
    }

    return { valid: true };
  }

  private findFirstArrow(line: string): number {
    let minIndex = Infinity;

    for (const pattern of this.arrowPatterns) {
      const index = line.indexOf(pattern);
      if (index !== -1 && index < minIndex) {
        minIndex = index;
      }
    }

    return minIndex === Infinity ? -1 : minIndex;
  }

  private getArrowLength(line: string, startIndex: number): number {
    for (const pattern of this.arrowPatterns) {
      if (line.substring(startIndex).startsWith(pattern)) {
        return pattern.length;
      }
    }
    return 0;
  }

  private removeEdgeLabel(text: string): string {
    // Remove edge labels like |label| from after arrows
    return text.replace(/^\|[^|]*\|\s*/, "").trim();
  }

  private extractNodeId(text: string): string | null {
    if (!text) return null;

    // Handle special Mermaid node shapes more carefully
    // First, check if text starts with a valid node ID followed by bracket content
    const nodeWithBracketMatch = text.match(
      /^([a-zA-Z_][a-zA-Z0-9_]*)\s*[\[\(\{<>]/,
    );
    if (nodeWithBracketMatch) {
      return nodeWithBracketMatch[1];
    }

    // Check if it's just a standalone node ID (no brackets)
    const standaloneNodeMatch = text.match(/^([a-zA-Z_][a-zA-Z0-9_]*)$/);
    if (standaloneNodeMatch) {
      return standaloneNodeMatch[1];
    }

    // Fallback: Remove brackets and their contents to get just the ID
    const withoutBrackets = text.replace(
      /\[[^\]]*\]|\([^)]*\)|\{[^}]*\}|<[^>]*>/g,
      "",
    );
    const trimmed = withoutBrackets.trim();

    // Return the first word (node ID) - this preserves invalid IDs for validation
    const words = trimmed.split(/\s+/);
    return words[0] || null;
  }

  private detectDiagramType(content: string): string {
    const firstLine = content.trim().split("\n")[0].trim().toLowerCase();

    if (firstLine.startsWith("graph")) return "flowchart";
    if (firstLine.startsWith("flowchart")) return "flowchart";
    if (firstLine.startsWith("sequencediagram")) return "sequence";
    if (firstLine.startsWith("classdiagram")) return "class";
    if (firstLine.startsWith("statediagram")) return "state";
    if (firstLine.startsWith("erdiagram")) return "er";
    if (firstLine.startsWith("gantt")) return "gantt";
    if (firstLine.startsWith("pie")) return "pie";

    return "unknown";
  }

  private countNodes(content: string): number {
    const lines = content.split("\n");
    const nodeIds = new Set<string>();

    for (const line of lines) {
      const trimmed = line.trim();
      if (
        !trimmed || trimmed.startsWith("%") ||
        trimmed.toLowerCase().startsWith("graph") ||
        trimmed.toLowerCase().startsWith("flowchart")
      ) {
        continue;
      }

      // Extract node IDs from this line
      const arrowIndex = this.findFirstArrow(trimmed);
      if (arrowIndex !== -1) {
        const beforeArrow = trimmed.substring(0, arrowIndex).trim();
        const afterArrowFull = trimmed.substring(
          arrowIndex + this.getArrowLength(trimmed, arrowIndex),
        ).trim();

        // Handle edge labels like -->|label| by finding the target after the label
        const afterArrow = this.removeEdgeLabel(afterArrowFull);

        const sourceId = this.extractNodeId(beforeArrow);
        const targetId = this.extractNodeId(afterArrow);

        if (sourceId) nodeIds.add(sourceId);
        if (targetId) nodeIds.add(targetId);
      } else {
        // Standalone node definition
        const nodeId = this.extractNodeId(trimmed);
        if (nodeId) nodeIds.add(nodeId);
      }
    }

    return nodeIds.size;
  }

  private countEdges(content: string): number {
    let edgeCount = 0;
    const lines = content.split("\n");

    for (const line of lines) {
      const trimmed = line.trim();
      if (
        !trimmed || trimmed.startsWith("%") ||
        trimmed.toLowerCase().startsWith("graph") ||
        trimmed.toLowerCase().startsWith("flowchart") ||
        trimmed.toLowerCase().startsWith("style")
      ) {
        continue;
      }

      // Count edges on this line (avoid counting patterns in style definitions)
      const arrowIndex = this.findFirstArrow(trimmed);
      if (arrowIndex !== -1) {
        edgeCount++;
      }
    }

    return edgeCount;
  }

  async validateFile(filePath: string): Promise<ValidationResult> {
    try {
      const content = await Deno.readTextFile(filePath);
      const result = this.validateDiagram(content);
      result.file = filePath;
      return result;
    } catch (error) {
      const errorMessage = error instanceof Error
        ? error.message
        : String(error);
      return {
        valid: false,
        file: filePath,
        error: {
          message: `Failed to read file: ${errorMessage}`,
          line: null,
          detail: errorMessage,
        },
      };
    }
  }

  async batchValidate(directory: string): Promise<BatchValidationResult> {
    const files: string[] = [];

    try {
      for await (
        const entry of walk(directory, {
          exts: [".mmd", ".mermaid"],
          skip: [
            /node_modules/,
            /target/,
            /\.git/,
            /dist/,
            /build/,
          ],
        })
      ) {
        if (entry.isFile) {
          files.push(entry.path);
        }
      }
    } catch (error) {
      const errorMessage = error instanceof Error
        ? error.message
        : String(error);
      console.warn(
        `Warning: Could not read directory ${directory}: ${errorMessage}`,
      );
    }

    const results: ValidationResult[] = [];
    for (const file of files) {
      results.push(await this.validateFile(file));
    }

    return {
      total: files.length,
      valid: results.filter((r) => r.valid).length,
      invalid: results.filter((r) => !r.valid).length,
      results,
    };
  }
}

// CLI interface
if (import.meta.main) {
  const flags = parse(Deno.args, {
    string: ["target"],
    boolean: ["help"],
    alias: { h: "help" },
  });

  if (flags.help || Deno.args.length === 0) {
    console.log(`
Usage: deno run --allow-read scripts/mermaid-validator.ts <file.mmd|directory>

Validates Mermaid diagram syntax and returns JSON results.

Options:
  -h, --help    Show this help message

Examples:
  deno run --allow-read scripts/mermaid-validator.ts diagram.mmd
  deno run --allow-read scripts/mermaid-validator.ts artifacts/mermaid/
`);
    Deno.exit(0);
  }

  const validator = new MermaidValidator();
  const target = Deno.args[0];

  try {
    const stats = await Deno.stat(target);

    let result: ValidationResult | BatchValidationResult;
    if (stats.isDirectory) {
      result = await validator.batchValidate(target);
    } else {
      result = await validator.validateFile(target);
    }

    console.log(JSON.stringify(result, null, 2));

    // Exit with appropriate code
    if ("valid" in result) {
      Deno.exit(result.valid ? 0 : 1);
    } else {
      Deno.exit((result as BatchValidationResult).invalid > 0 ? 1 : 0);
    }
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    console.error(JSON.stringify(
      {
        valid: false,
        error: {
          message: errorMessage,
          line: null,
          detail: "Validator execution failed",
        },
      },
      null,
      2,
    ));
    Deno.exit(1);
  }
}

export { type BatchValidationResult, MermaidValidator, type ValidationResult };
