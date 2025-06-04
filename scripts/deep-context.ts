#!/usr/bin/env -S deno run --allow-read --allow-write --allow-run --allow-env

/**
 * @fileoverview Deep context generator for paiml-mcp-agent-toolkit project
 * Generates comprehensive project analysis including:
 * - File tree structure
 * - AST analysis for Rust files
 * - AST analysis for TypeScript files
 * - Makefile content and targets
 * - README content
 */

import { parse } from "https://deno.land/std@0.208.0/flags/mod.ts";
import { walk } from "https://deno.land/std@0.208.0/fs/walk.ts";
import {
  basename,
  join,
  relative,
} from "https://deno.land/std@0.208.0/path/mod.ts";
import {
  bold,
  cyan,
  gray,
  green,
  red,
  yellow,
} from "https://deno.land/std@0.208.0/fmt/colors.ts";
import * as ts from "https://esm.sh/typescript@5.3.3";

// Constants
const VERSION = "1.0.0";
const DEFAULT_OUTPUT_FILE = "deep_context";
const DEFAULT_IGNORE_DIRS = [
  ".git",
  "node_modules",
  "target",
  "dist",
  "build",
  "coverage",
  ".cargo",
  "__pycache__",
  ".venv",
  ".idea",
  ".vscode",
];

// Type definitions
type LogLevel = "info" | "success" | "warn" | "error" | "verbose";

interface ParsedArgs {
  output: string;
  verbose: boolean;
  help: boolean;
  version: boolean;
  _: string[];
}

interface RustFunction {
  name: string;
  visibility: string;
  async: boolean;
  generics: string[];
  params: string[];
  returnType: string;
  lineNumber: number;
}

interface RustStruct {
  name: string;
  visibility: string;
  generics: string[];
  derives: string[];
  fields: Array<{
    name: string;
    type: string;
    visibility: string;
  }>;
  lineNumber: number;
}

interface RustEnum {
  name: string;
  visibility: string;
  generics: string[];
  derives: string[];
  variants: string[];
  lineNumber: number;
}

interface RustImpl {
  structName: string;
  traitName?: string;
  methods: RustFunction[];
  lineNumber: number;
}

interface RustMod {
  name: string;
  visibility: string;
  isFile: boolean;
  lineNumber: number;
}

interface RustUse {
  path: string;
  alias?: string;
  lineNumber: number;
}

interface RustFileAST {
  filePath: string;
  uses: RustUse[];
  mods: RustMod[];
  structs: RustStruct[];
  enums: RustEnum[];
  functions: RustFunction[];
  impls: RustImpl[];
  traits: Array<{
    name: string;
    visibility: string;
    lineNumber: number;
  }>;
  constants: Array<{
    name: string;
    type: string;
    lineNumber: number;
  }>;
}

interface TypeScriptFunction {
  name: string;
  async: boolean;
  exported: boolean;
  params: Array<{
    name: string;
    type?: string;
  }>;
  returnType?: string;
  lineNumber: number;
}

interface TypeScriptInterface {
  name: string;
  exported: boolean;
  properties: Array<{
    name: string;
    type: string;
    optional: boolean;
  }>;
  lineNumber: number;
}

interface TypeScriptFileAST {
  filePath: string;
  imports: Array<{
    source: string;
    specifiers: string[];
    lineNumber: number;
  }>;
  exports: Array<{
    name: string;
    isDefault: boolean;
    lineNumber: number;
  }>;
  functions: TypeScriptFunction[];
  interfaces: TypeScriptInterface[];
  classes: Array<{
    name: string;
    exported: boolean;
    lineNumber: number;
  }>;
  constants: Array<{
    name: string;
    type?: string;
    exported: boolean;
    lineNumber: number;
  }>;
}

interface MakefileTarget {
  name: string;
  description?: string;
  dependencies: string[];
  commands: string[];
}

interface ProjectContext {
  rootPath: string;
  tree: string;
  rustFiles: RustFileAST[];
  tsFiles: TypeScriptFileAST[];
  makefiles: Array<{
    path: string;
    targets: MakefileTarget[];
    variables: Record<string, string>;
  }>;
  readmes: Array<{
    path: string;
    content: string;
  }>;
}

// Logging
let globalArgs: ParsedArgs;

function log(level: LogLevel, message: string): void {
  if (level === "verbose" && !globalArgs?.verbose) return;

  const prefixMap: Record<LogLevel, string> = {
    info: "ℹ️",
    success: "✅",
    warn: "⚠️",
    error: "❌",
    verbose: "⚙️",
  };

  const colorMap: Record<LogLevel, (str: string) => string> = {
    info: cyan,
    success: green,
    warn: yellow,
    error: red,
    verbose: gray,
  };

  const logFn = level === "error" || level === "warn"
    ? console.error
    : console.log;
  logFn(`${prefixMap[level]} ${colorMap[level](message)}`);
}

// File tree generation
async function generateProjectTree(
  root: string,
  ignoreDirs: string[],
): Promise<string> {
  try {
    // Try using tree command first
    const cmd = new Deno.Command("tree", {
      args: [
        "-a",
        "-L",
        "4",
        ...ignoreDirs.flatMap((d) => ["-I", d]),
        "--noreport",
      ],
      cwd: root,
      stdout: "piped",
      stderr: "piped",
    });

    const output = await cmd.output();
    if (output.success) {
      const tree = new TextDecoder().decode(output.stdout);
      return `${basename(root)}\n${tree}`;
    }
  } catch {
    // Fallback if tree command not available
  }

  // Manual tree generation
  let tree = "";
  const ignoreSet = new Set(ignoreDirs);

  async function buildTree(dir: string, prefix: string = ""): Promise<void> {
    const entries = [];
    for await (const entry of Deno.readDir(dir)) {
      if (!ignoreSet.has(entry.name)) {
        entries.push(entry);
      }
    }

    entries.sort((a, b) => {
      if (a.isDirectory !== b.isDirectory) {
        return a.isDirectory ? -1 : 1;
      }
      return a.name.localeCompare(b.name);
    });

    for (let i = 0; i < entries.length; i++) {
      const entry = entries[i];
      const isLastEntry = i === entries.length - 1;
      const connector = isLastEntry ? "└── " : "├── ";
      const extension = isLastEntry ? "    " : "│   ";

      tree += `${prefix}${connector}${entry.name}${
        entry.isDirectory ? "/" : ""
      }\n`;

      if (entry.isDirectory && prefix.length < 12) { // Limit depth
        const subDir = join(dir, entry.name);
        await buildTree(subDir, prefix + extension);
      }
    }
  }

  tree = `${basename(root)}/\n`;
  await buildTree(root);
  return tree;
}

// Rust AST parsing
async function parseRustFile(filePath: string): Promise<RustFileAST | null> {
  const relativePath = relative(Deno.cwd(), filePath).replace(/\\/g, "/");
  log("verbose", `Parsing Rust file: ${relativePath}`);

  try {
    const content = await Deno.readTextFile(filePath);
    const lines = content.split("\n");

    const ast: RustFileAST = {
      filePath: relativePath,
      uses: [],
      mods: [],
      structs: [],
      enums: [],
      functions: [],
      impls: [],
      traits: [],
      constants: [],
    };

    // Parse use statements
    const useRegex = /^\s*(?:pub\s+)?use\s+([^;]+)(?:\s+as\s+(\w+))?;/;
    lines.forEach((line, idx) => {
      const match = line.match(useRegex);
      if (match) {
        ast.uses.push({
          path: match[1].trim(),
          alias: match[2],
          lineNumber: idx + 1,
        });
      }
    });

    // Parse modules
    const modRegex = /^\s*(pub(?:\([\w\)]+\))?\s+)?mod\s+(\w+)\s*[{;]/;
    lines.forEach((line, idx) => {
      const match = line.match(modRegex);
      if (match) {
        ast.mods.push({
          name: match[2],
          visibility: match[1]?.trim() || "private",
          isFile: line.includes(";"),
          lineNumber: idx + 1,
        });
      }
    });

    // Parse structs
    const structRegex = /^\s*(pub(?:\([\w\)]+\))?\s+)?struct\s+(\w+)(<[^>]+>)?/;
    const deriveRegex = /#\[derive\(([^)]+)\)\]/;

    for (let i = 0; i < lines.length; i++) {
      const line = lines[i];
      const structMatch = line.match(structRegex);
      if (structMatch) {
        const derives: string[] = [];
        // Look for derives in previous lines
        for (let j = i - 1; j >= 0 && j >= i - 5; j--) {
          const deriveMatch = lines[j].match(deriveRegex);
          if (deriveMatch) {
            derives.push(...deriveMatch[1].split(",").map((d) => d.trim()));
          }
        }

        ast.structs.push({
          name: structMatch[2],
          visibility: structMatch[1]?.trim() || "private",
          generics: structMatch[3] ? [structMatch[3]] : [],
          derives,
          fields: [], // Would need more complex parsing for fields
          lineNumber: i + 1,
        });
      }
    }

    // Parse enums
    const enumRegex = /^\s*(pub(?:\([\w\)]+\))?\s+)?enum\s+(\w+)(<[^>]+>)?/;
    lines.forEach((line, idx) => {
      const match = line.match(enumRegex);
      if (match) {
        ast.enums.push({
          name: match[2],
          visibility: match[1]?.trim() || "private",
          generics: match[3] ? [match[3]] : [],
          derives: [], // Would need to look at previous lines
          variants: [], // Would need to parse enum body
          lineNumber: idx + 1,
        });
      }
    });

    // Parse functions
    const fnRegex =
      /^\s*(pub(?:\([\w\)]+\))?\s+)?(async\s+)?fn\s+(\w+)(<[^>]+>)?\s*\([^)]*\)(?:\s*->\s*([^{]+))?/;
    lines.forEach((line, idx) => {
      const match = line.match(fnRegex);
      if (match) {
        ast.functions.push({
          name: match[3],
          visibility: match[1]?.trim() || "private",
          async: !!match[2],
          generics: match[4] ? [match[4]] : [],
          params: [], // Would need more complex parsing
          returnType: match[5]?.trim() || "()",
          lineNumber: idx + 1,
        });
      }
    });

    // Parse impls
    const implRegex = /^\s*impl(?:<[^>]+>)?\s+(?:(\w+)\s+for\s+)?(\w+)/;
    lines.forEach((line, idx) => {
      const match = line.match(implRegex);
      if (match) {
        ast.impls.push({
          structName: match[2],
          traitName: match[1],
          methods: [], // Would need to parse impl body
          lineNumber: idx + 1,
        });
      }
    });

    // Parse traits
    const traitRegex = /^\s*(pub(?:\([\w\)]+\))?\s+)?trait\s+(\w+)/;
    lines.forEach((line, idx) => {
      const match = line.match(traitRegex);
      if (match) {
        ast.traits.push({
          name: match[2],
          visibility: match[1]?.trim() || "private",
          lineNumber: idx + 1,
        });
      }
    });

    // Parse constants
    const constRegex = /^\s*(pub(?:\([\w\)]+\))?\s+)?const\s+(\w+):\s*([^=]+)/;
    lines.forEach((line, idx) => {
      const match = line.match(constRegex);
      if (match) {
        ast.constants.push({
          name: match[2],
          type: match[3].trim(),
          lineNumber: idx + 1,
        });
      }
    });

    return ast;
  } catch (error) {
    log("error", `Failed to parse Rust file ${relativePath}: ${error}`);
    return null;
  }
}

// TypeScript AST parsing helper
function visitTypeScriptNode(
  node: ts.Node,
  ast: TypeScriptFileAST,
  sourceFile: ts.SourceFile,
): void {
  // Parse imports
  if (
    ts.isImportDeclaration(node) && node.moduleSpecifier &&
    ts.isStringLiteral(node.moduleSpecifier)
  ) {
    const specifiers: string[] = [];
    if (node.importClause) {
      if (node.importClause.name) {
        specifiers.push(node.importClause.name.text);
      }
      if (node.importClause.namedBindings) {
        if (ts.isNamedImports(node.importClause.namedBindings)) {
          node.importClause.namedBindings.elements.forEach((element) => {
            specifiers.push(element.name.text);
          });
        } else if (ts.isNamespaceImport(node.importClause.namedBindings)) {
          specifiers.push(`* as ${node.importClause.namedBindings.name.text}`);
        }
      }
    }

    ast.imports.push({
      source: node.moduleSpecifier.text,
      specifiers,
      lineNumber:
        sourceFile.getLineAndCharacterOfPosition(node.getStart()).line + 1,
    });
  }

  // Parse exports
  if (ts.isExportDeclaration(node)) {
    if (node.exportClause && ts.isNamedExports(node.exportClause)) {
      node.exportClause.elements.forEach((element) => {
        ast.exports.push({
          name: element.name.text,
          isDefault: false,
          lineNumber:
            sourceFile.getLineAndCharacterOfPosition(node.getStart()).line + 1,
        });
      });
    }
  } else if (ts.isExportAssignment(node)) {
    ast.exports.push({
      name: "default",
      isDefault: true,
      lineNumber:
        sourceFile.getLineAndCharacterOfPosition(node.getStart()).line + 1,
    });
  }

  // Parse functions
  if (ts.isFunctionDeclaration(node) && node.name) {
    const params: Array<{ name: string; type?: string }> = [];
    node.parameters.forEach((param) => {
      if (ts.isIdentifier(param.name)) {
        params.push({
          name: param.name.text,
          type: param.type ? param.type.getText(sourceFile) : undefined,
        });
      }
    });

    ast.functions.push({
      name: node.name.text,
      async: !!(node.modifiers?.some((m) =>
        m.kind === ts.SyntaxKind.AsyncKeyword
      )),
      exported: !!(node.modifiers?.some((m) =>
        m.kind === ts.SyntaxKind.ExportKeyword
      )),
      params,
      returnType: node.type ? node.type.getText(sourceFile) : undefined,
      lineNumber:
        sourceFile.getLineAndCharacterOfPosition(node.getStart()).line + 1,
    });
  }

  // Parse interfaces
  if (ts.isInterfaceDeclaration(node)) {
    const properties: Array<{ name: string; type: string; optional: boolean }> =
      [];

    node.members.forEach((member) => {
      if (
        ts.isPropertySignature(member) && member.name &&
        ts.isIdentifier(member.name)
      ) {
        properties.push({
          name: member.name.text,
          type: member.type ? member.type.getText(sourceFile) : "any",
          optional: !!member.questionToken,
        });
      }
    });

    ast.interfaces.push({
      name: node.name.text,
      exported: !!(node.modifiers?.some((m) =>
        m.kind === ts.SyntaxKind.ExportKeyword
      )),
      properties,
      lineNumber:
        sourceFile.getLineAndCharacterOfPosition(node.getStart()).line + 1,
    });
  }

  // Parse classes
  if (ts.isClassDeclaration(node) && node.name) {
    ast.classes.push({
      name: node.name.text,
      exported: !!(node.modifiers?.some((m) =>
        m.kind === ts.SyntaxKind.ExportKeyword
      )),
      lineNumber:
        sourceFile.getLineAndCharacterOfPosition(node.getStart()).line + 1,
    });
  }

  // Parse constants
  if (ts.isVariableStatement(node)) {
    const isExported = !!(node.modifiers?.some((m) =>
      m.kind === ts.SyntaxKind.ExportKeyword
    ));
    node.declarationList.declarations.forEach((decl) => {
      if (ts.isIdentifier(decl.name)) {
        ast.constants.push({
          name: decl.name.text,
          type: decl.type ? decl.type.getText(sourceFile) : undefined,
          exported: isExported,
          lineNumber:
            sourceFile.getLineAndCharacterOfPosition(decl.getStart()).line + 1,
        });
      }
    });
  }

  ts.forEachChild(node, (child) => visitTypeScriptNode(child, ast, sourceFile));
}

// TypeScript AST parsing
function parseTypeScriptFile(
  filePath: string,
  content: string,
): TypeScriptFileAST | null {
  const relativePath = relative(Deno.cwd(), filePath).replace(/\\/g, "/");
  log("verbose", `Parsing TypeScript file: ${relativePath}`);

  try {
    const sourceFile = ts.createSourceFile(
      filePath,
      content,
      ts.ScriptTarget.Latest,
      true,
    );

    const ast: TypeScriptFileAST = {
      filePath: relativePath,
      imports: [],
      exports: [],
      functions: [],
      interfaces: [],
      classes: [],
      constants: [],
    };

    visitTypeScriptNode(sourceFile, ast, sourceFile);
    return ast;
  } catch (error) {
    log("error", `Failed to parse TypeScript file ${relativePath}: ${error}`);
    return null;
  }
}

// Makefile parsing
async function parseMakefile(
  filePath: string,
): Promise<
  | {
    path: string;
    targets: MakefileTarget[];
    variables: Record<string, string>;
  }
  | null
> {
  const relativePath = relative(Deno.cwd(), filePath).replace(/\\/g, "/");
  log("verbose", `Parsing Makefile: ${relativePath}`);

  try {
    const content = await Deno.readTextFile(filePath);
    const lines = content.split("\n");

    const targets: MakefileTarget[] = [];
    const variables: Record<string, string> = {};

    let currentTarget: MakefileTarget | null = null;

    for (let i = 0; i < lines.length; i++) {
      const line = lines[i];

      // Variable definition
      const varMatch = line.match(/^([A-Z_]+)\s*[:?]?=\s*(.+)$/);
      if (varMatch) {
        variables[varMatch[1]] = varMatch[2].trim();
        continue;
      }

      // Target definition
      const targetMatch = line.match(/^([a-zA-Z0-9_-]+):\s*(.*)$/);
      if (targetMatch && !line.startsWith("\t") && !line.startsWith(" ")) {
        if (currentTarget) {
          targets.push(currentTarget);
        }

        currentTarget = {
          name: targetMatch[1],
          dependencies: targetMatch[2].split(/\s+/).filter((d) => d.length > 0),
          commands: [],
        };

        // Check for description in previous line
        if (i > 0 && lines[i - 1].startsWith("#")) {
          currentTarget.description = lines[i - 1].substring(1).trim();
        }
      }

      // Command for current target
      if (currentTarget && (line.startsWith("\t") || line.startsWith("  "))) {
        currentTarget.commands.push(line.trim());
      }
    }

    if (currentTarget) {
      targets.push(currentTarget);
    }

    return {
      path: relativePath,
      targets,
      variables,
    };
  } catch (error) {
    log("error", `Failed to parse Makefile ${relativePath}: ${error}`);
    return null;
  }
}

// Find all project files
async function findProjectFiles(root: string, ignoreDirs: string[]): Promise<{
  rustFiles: string[];
  tsFiles: string[];
  makefiles: string[];
  readmes: string[];
}> {
  log("verbose", "Finding project files...");

  const rustFiles: string[] = [];
  const tsFiles: string[] = [];
  const makefiles: string[] = [];
  const readmes: string[] = [];

  const ignoreSet = new Set(ignoreDirs);

  for await (const entry of walk(root, { includeDirs: false })) {
    const relativePath = relative(root, entry.path);
    const pathParts = relativePath.split("/");

    // Skip ignored directories
    if (pathParts.some((part) => ignoreSet.has(part))) continue;

    const filename = basename(entry.path).toLowerCase();

    if (entry.path.endsWith(".rs")) {
      rustFiles.push(entry.path);
    } else if (entry.path.endsWith(".ts") || entry.path.endsWith(".tsx")) {
      tsFiles.push(entry.path);
    } else if (filename === "makefile") {
      makefiles.push(entry.path);
    } else if (filename === "readme.md" || filename === "readme") {
      readmes.push(entry.path);
    }
  }

  log(
    "verbose",
    `Found ${rustFiles.length} Rust files, ${tsFiles.length} TypeScript files, ${makefiles.length} Makefiles, ${readmes.length} READMEs`,
  );

  return { rustFiles, tsFiles, makefiles, readmes };
}

// Generate output - main orchestrator
function generateOutput(context: ProjectContext): string {
  let output = generateHeader(context);
  output += generateProjectStructure(context);
  output += generateRustFilesSection(context);
  output += generateTypeScriptFilesSection(context);
  output += generateMakefilesSection(context);
  output += generateReadmeSection(context);
  output += generateFooter();

  return output;
}

// Generate document header
function generateHeader(context: ProjectContext): string {
  let output = `# Deep Context: ${basename(context.rootPath)}\n\n`;
  output += `Generated: ${new Date().toISOString()}\n`;
  output += `Version: ${VERSION}\n\n`;
  return output;
}

// Generate project structure section
function generateProjectStructure(context: ProjectContext): string {
  let output = `## Project Structure\n\n`;
  output += "```\n";
  output += context.tree;
  output += "```\n\n";
  return output;
}

// Generate Rust files documentation
function generateRustFilesSection(context: ProjectContext): string {
  if (context.rustFiles.length === 0) return "";

  let output = `## Rust Files (${context.rustFiles.length})\n\n`;
  
  for (const rustFile of context.rustFiles) {
    output += generateRustFileDetails(rustFile);
  }
  
  return output;
}

// Generate details for a single Rust file
function generateRustFileDetails(rustFile: RustFileAST): string {
  let output = `### ${rustFile.filePath}\n\n`;
  
  output += generateRustImports(rustFile.uses);
  output += generateRustModules(rustFile.mods);
  output += generateRustStructs(rustFile.structs);
  output += generateRustEnums(rustFile.enums);
  output += generateRustFunctions(rustFile.functions);
  output += generateRustImplementations(rustFile.impls);
  output += generateRustTraits(rustFile.traits);
  
  return output;
}

// Generate Rust imports section
function generateRustImports(uses: RustUse[]): string {
  if (uses.length === 0) return "";
  
  let output = `**Imports:**\n`;
  for (const use of uses) {
    output += `- \`${use.path}\`${use.alias ? ` as ${use.alias}` : ""} (line ${use.lineNumber})\n`;
  }
  output += "\n";
  return output;
}

// Generate Rust modules section
function generateRustModules(mods: RustMod[]): string {
  if (mods.length === 0) return "";
  
  let output = `**Modules:**\n`;
  for (const mod of mods) {
    output += `- \`${mod.visibility} mod ${mod.name}\` ${mod.isFile ? "(file)" : "(inline)"} (line ${mod.lineNumber})\n`;
  }
  output += "\n";
  return output;
}

// Generate Rust structs section
function generateRustStructs(structs: RustStruct[]): string {
  if (structs.length === 0) return "";
  
  let output = `**Structs:**\n`;
  for (const struct of structs) {
    output += `- \`${struct.visibility} struct ${struct.name}\``;
    if (struct.derives.length > 0) {
      output += ` [derives: ${struct.derives.join(", ")}]`;
    }
    output += ` (line ${struct.lineNumber})\n`;
  }
  output += "\n";
  return output;
}

// Generate Rust enums section
function generateRustEnums(enums: RustEnum[]): string {
  if (enums.length === 0) return "";
  
  let output = `**Enums:**\n`;
  for (const enum_ of enums) {
    output += `- \`${enum_.visibility} enum ${enum_.name}\` (line ${enum_.lineNumber})\n`;
  }
  output += "\n";
  return output;
}

// Generate Rust functions section
function generateRustFunctions(functions: RustFunction[]): string {
  if (functions.length === 0) return "";
  
  let output = `**Functions:**\n`;
  for (const fn of functions) {
    output += `- \`${fn.visibility} ${fn.async ? "async " : ""}fn ${fn.name}(...) -> ${fn.returnType}\` (line ${fn.lineNumber})\n`;
  }
  output += "\n";
  return output;
}

// Generate Rust implementations section
function generateRustImplementations(impls: RustImpl[]): string {
  if (impls.length === 0) return "";
  
  let output = `**Implementations:**\n`;
  for (const impl of impls) {
    output += `- \`impl ${impl.traitName ? `${impl.traitName} for ` : ""}${impl.structName}\` (line ${impl.lineNumber})\n`;
  }
  output += "\n";
  return output;
}

// Generate Rust traits section
function generateRustTraits(traits: Array<{name: string; visibility: string; lineNumber: number}>): string {
  if (traits.length === 0) return "";
  
  let output = `**Traits:**\n`;
  for (const trait of traits) {
    output += `- \`${trait.visibility} trait ${trait.name}\` (line ${trait.lineNumber})\n`;
  }
  output += "\n";
  return output;
}

// Generate TypeScript files documentation
function generateTypeScriptFilesSection(context: ProjectContext): string {
  if (context.tsFiles.length === 0) return "";

  let output = `## TypeScript Files (${context.tsFiles.length})\n\n`;
  
  for (const tsFile of context.tsFiles) {
    output += generateTypeScriptFileDetails(tsFile);
  }
  
  return output;
}

// Generate details for a single TypeScript file
function generateTypeScriptFileDetails(tsFile: TypeScriptFileAST): string {
  let output = `### ${tsFile.filePath}\n\n`;
  
  output += generateTypeScriptImports(tsFile.imports);
  output += generateTypeScriptExports(tsFile.exports);
  output += generateTypeScriptInterfaces(tsFile.interfaces);
  output += generateTypeScriptFunctions(tsFile.functions);
  output += generateTypeScriptClasses(tsFile.classes);
  
  return output;
}

// Generate TypeScript imports section
function generateTypeScriptImports(imports: Array<{source: string; specifiers: string[]; lineNumber: number}>): string {
  if (imports.length === 0) return "";
  
  let output = `**Imports:**\n`;
  for (const imp of imports) {
    output += `- from \`${imp.source}\`: ${imp.specifiers.join(", ") || "*"} (line ${imp.lineNumber})\n`;
  }
  output += "\n";
  return output;
}

// Generate TypeScript exports section
function generateTypeScriptExports(exports: Array<{name: string; isDefault: boolean; lineNumber: number}>): string {
  if (exports.length === 0) return "";
  
  let output = `**Exports:**\n`;
  for (const exp of exports) {
    output += `- \`${exp.isDefault ? "default " : ""}${exp.name}\` (line ${exp.lineNumber})\n`;
  }
  output += "\n";
  return output;
}

// Generate TypeScript interfaces section
function generateTypeScriptInterfaces(interfaces: TypeScriptInterface[]): string {
  if (interfaces.length === 0) return "";
  
  let output = `**Interfaces:**\n`;
  for (const iface of interfaces) {
    output += `- \`${iface.exported ? "export " : ""}interface ${iface.name}\` with ${iface.properties.length} properties (line ${iface.lineNumber})\n`;
  }
  output += "\n";
  return output;
}

// Generate TypeScript functions section
function generateTypeScriptFunctions(functions: TypeScriptFunction[]): string {
  if (functions.length === 0) return "";
  
  let output = `**Functions:**\n`;
  for (const fn of functions) {
    output += `- \`${fn.exported ? "export " : ""}${fn.async ? "async " : ""}function ${fn.name}(${fn.params.map((p) => p.name).join(", ")})${fn.returnType ? `: ${fn.returnType}` : ""}\` (line ${fn.lineNumber})\n`;
  }
  output += "\n";
  return output;
}

// Generate TypeScript classes section
function generateTypeScriptClasses(classes: Array<{name: string; exported: boolean; lineNumber: number}>): string {
  if (classes.length === 0) return "";
  
  let output = `**Classes:**\n`;
  for (const cls of classes) {
    output += `- \`${cls.exported ? "export " : ""}class ${cls.name}\` (line ${cls.lineNumber})\n`;
  }
  output += "\n";
  return output;
}

// Generate Makefiles documentation
function generateMakefilesSection(context: ProjectContext): string {
  if (context.makefiles.length === 0) return "";

  let output = `## Makefiles (${context.makefiles.length})\n\n`;
  
  for (const makefile of context.makefiles) {
    output += generateMakefileDetails(makefile);
  }
  
  return output;
}

// Generate details for a single Makefile
function generateMakefileDetails(makefile: {path: string; targets: MakefileTarget[]; variables: Record<string, string>}): string {
  let output = `### ${makefile.path}\n\n`;
  
  output += generateMakefileVariables(makefile.variables);
  output += generateMakefileTargets(makefile.targets);
  
  return output;
}

// Generate Makefile variables section
function generateMakefileVariables(variables: Record<string, string>): string {
  if (Object.keys(variables).length === 0) return "";
  
  let output = `**Variables:**\n`;
  for (const [name, value] of Object.entries(variables)) {
    output += `- \`${name} = ${value}\`\n`;
  }
  output += "\n";
  return output;
}

// Generate Makefile targets section
function generateMakefileTargets(targets: MakefileTarget[]): string {
  if (targets.length === 0) return "";
  
  let output = `**Targets:**\n\n`;
  output += "| Target | Description | Dependencies | Commands |\n";
  output += "|--------|-------------|--------------|----------|\n";

  for (const target of targets) {
    const commands = target.commands.length > 2
      ? `${target.commands.slice(0, 2).join("; ")}... (${target.commands.length} total)`
      : target.commands.join("; ");

    output += `| \`${target.name}\` | ${target.description || "-"} | ${target.dependencies.join(", ") || "-"} | ${commands || "-"} |\n`;
  }
  output += "\n";
  return output;
}

// Generate README files documentation
function generateReadmeSection(context: ProjectContext): string {
  if (context.readmes.length === 0) return "";

  let output = `## README Files (${context.readmes.length})\n\n`;

  for (const readme of context.readmes) {
    output += `### ${readme.path}\n\n`;
    output += "```markdown\n";
    output += readme.content;
    output += "\n```\n\n";
  }
  
  return output;
}

// Generate document footer
function generateFooter(): string {
  let output = "---\n";
  output += `Generated by deep-context v${VERSION}\n`;
  return output;
}

// Main function
async function main() {
  const args = parse(Deno.args, {
    string: ["output"],
    boolean: ["verbose", "help", "version"],
    default: {
      output: join(Deno.cwd(), DEFAULT_OUTPUT_FILE),
      verbose: false,
    },
    alias: {
      o: "output",
      v: "verbose",
      h: "help",
    },
  });

  globalArgs = {
    output: args.output as string,
    verbose: args.verbose as boolean,
    help: args.help as boolean,
    version: args.version as boolean,
    _: args._.map(String),
  };

  if (globalArgs.help) {
    console.log(`
${bold("Deep Context Generator")} v${VERSION}

${bold("Usage:")}
  deno run --allow-read --allow-write --allow-run scripts/deep-context.ts [options]

${bold("Options:")}
  -o, --output <path>    Output file path (default: ./deep_context)
  -v, --verbose          Enable verbose logging
  -h, --help             Show this help
  --version              Show version

${bold("Example:")}
  deno run --allow-read --allow-write --allow-run scripts/deep-context.ts -o context.md -v
`);
    return;
  }

  if (globalArgs.version) {
    console.log(`deep-context v${VERSION}`);
    return;
  }

  log("info", `${bold("Deep Context Generator")} - v${VERSION}`);
  log("info", "=====================================");

  const outputPath = `${globalArgs.output}.md`;
  const rootPath = Deno.cwd();

  log("info", `Project Root: ${rootPath}`);
  log("info", `Output Path: ${outputPath}`);

  const context: ProjectContext = {
    rootPath,
    tree: "",
    rustFiles: [],
    tsFiles: [],
    makefiles: [],
    readmes: [],
  };

  try {
    // Generate project tree
    log("info", "Generating project tree...");
    context.tree = await generateProjectTree(rootPath, DEFAULT_IGNORE_DIRS);

    // Find all project files
    const files = await findProjectFiles(rootPath, DEFAULT_IGNORE_DIRS);

    // Parse Rust files
    log("info", `Parsing ${files.rustFiles.length} Rust files...`);
    for (const file of files.rustFiles) {
      const ast = await parseRustFile(file);
      if (ast) {
        context.rustFiles.push(ast);
      }
    }

    // Parse TypeScript files
    log("info", `Parsing ${files.tsFiles.length} TypeScript files...`);
    for (const file of files.tsFiles) {
      try {
        const content = await Deno.readTextFile(file);
        const ast = parseTypeScriptFile(file, content);
        if (ast) {
          context.tsFiles.push(ast);
        }
      } catch (error) {
        log("error", `Failed to read ${file}: ${error}`);
      }
    }

    // Parse Makefiles
    log("info", `Parsing ${files.makefiles.length} Makefiles...`);
    for (const file of files.makefiles) {
      const parsed = await parseMakefile(file);
      if (parsed) {
        context.makefiles.push(parsed);
      }
    }

    // Read READMEs
    log("info", `Reading ${files.readmes.length} README files...`);
    for (const file of files.readmes) {
      try {
        const content = await Deno.readTextFile(file);
        context.readmes.push({
          path: relative(rootPath, file),
          content,
        });
      } catch (error) {
        log("error", `Failed to read ${file}: ${error}`);
      }
    }

    // Generate output
    log("info", "Generating output...");
    const output = generateOutput(context);

    // Write output
    await Deno.writeTextFile(outputPath, output);
    log("success", `Deep context written to ${outputPath}`);

    // Print summary
    console.log(bold("\n--- Analysis Complete ---"));
    console.log(`Rust Files:       ${context.rustFiles.length}`);
    console.log(`TypeScript Files: ${context.tsFiles.length}`);
    console.log(`Makefiles:        ${context.makefiles.length}`);
    console.log(`README Files:     ${context.readmes.length}`);

    const totalFunctions = context.rustFiles.reduce((sum, f) =>
      sum + f.functions.length, 0) +
      context.tsFiles.reduce((sum, f) => sum + f.functions.length, 0);
    const totalStructs = context.rustFiles.reduce((sum, f) =>
      sum + f.structs.length + f.enums.length, 0) +
      context.tsFiles.reduce((sum, f) =>
        sum + f.interfaces.length + f.classes.length, 0);

    console.log(`Total Functions:  ${totalFunctions}`);
    console.log(`Total Types:      ${totalStructs}`);
  } catch (error) {
    log("error", `Fatal error: ${error}`);
    Deno.exit(1);
  }
}

// Run main
if (import.meta.main) {
  await main();
}
