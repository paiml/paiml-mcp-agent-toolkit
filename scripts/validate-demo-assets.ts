#!/usr/bin/env -S deno run --allow-read --allow-write --allow-run

/**
 * Validate and format embedded HTML/CSS/JS in Rust demo files
 * This ensures all embedded web assets are properly formatted and typed
 */

import { walk } from "https://deno.land/std@0.208.0/fs/mod.ts";
import { parse } from "https://deno.land/std@0.208.0/flags/mod.ts";

interface ValidationResult {
  file: string;
  issues: string[];
  formatted?: string;
}

// Extract HTML from Rust string literals
function extractHtmlFromRust(content: string): string | null {
  // Look for HTML content in const strings or format! macros
  const htmlMatch = content.match(/(?:const\s+HTML_TEMPLATE:\s*&str\s*=\s*r#?"([\s\S]*?)"#?;|format!\s*\(\s*r#?"([\s\S]*?)"#?\s*[,)])/);
  if (htmlMatch) {
    return htmlMatch[1] || htmlMatch[2];
  }
  return null;
}

// Extract JavaScript from HTML or Rust
function extractJavaScriptFromHtml(html: string): string[] {
  const scripts: string[] = [];
  const scriptRegex = /<script(?:\s+type="module")?\s*>([\s\S]*?)<\/script>/g;
  let match;
  while ((match = scriptRegex.exec(html)) !== null) {
    scripts.push(match[1]);
  }
  return scripts;
}

// Extract CSS from HTML or Rust
function extractCssFromHtml(html: string): string[] {
  const styles: string[] = [];
  const styleRegex = /<style>([\s\S]*?)<\/style>/g;
  let match;
  while ((match = styleRegex.exec(html)) !== null) {
    styles.push(match[1]);
  }
  return styles;
}

// Validate TypeScript/JavaScript
async function validateJavaScript(code: string, isModule: boolean = false): Promise<string[]> {
  const issues: string[] = [];
  
  // Create a temporary file for validation
  const tempFile = await Deno.makeTempFile({ suffix: ".ts" });
  
  try {
    // Add TypeScript strict mode and type annotations
    const strictCode = `// @ts-check
/* eslint-env browser */
${isModule ? '' : '(function() {'}
"use strict";
${code}
${isModule ? '' : '})();'}
`;
    
    await Deno.writeTextFile(tempFile, strictCode);
    
    // Run deno lint
    const lintCmd = new Deno.Command("deno", {
      args: ["lint", "--rules-tags=recommended", tempFile],
      stdout: "piped",
      stderr: "piped",
    });
    
    const lintResult = await lintCmd.output();
    if (!lintResult.success) {
      const stderr = new TextDecoder().decode(lintResult.stderr);
      issues.push(`Lint issues: ${stderr}`);
    }
    
    // Run deno fmt --check
    const fmtCmd = new Deno.Command("deno", {
      args: ["fmt", "--check", tempFile],
      stdout: "piped",
      stderr: "piped",
    });
    
    const fmtResult = await fmtCmd.output();
    if (!fmtResult.success) {
      issues.push("Code is not properly formatted");
    }
    
  } finally {
    await Deno.remove(tempFile);
  }
  
  return issues;
}

// Validate CSS
function validateCss(css: string): string[] {
  const issues: string[] = [];
  
  // Basic CSS validation rules

  // Check for empty CSS
  if (!css || css.trim() === "") {
    return ["CSS is empty"];
  }
  const rules = [
    { pattern: /;\s*;/, message: "Double semicolon detected" },
    { pattern: /{\s*}/, message: "Empty CSS rule detected" },
    { pattern: /[^:]\s*!important/, message: "!important should be avoided" },
  ];
  
  for (const rule of rules) {
    if (rule.pattern.test(css)) {
      issues.push(rule.message);
    }
  }
  
  return issues;
}

// Validate HTML
function validateHtml(html: string): string[] {
  const issues: string[] = [];
  
  // Check for DOCTYPE
  if (!html.trim().toLowerCase().startsWith("<!doctype html>")) {
    issues.push("Missing <!DOCTYPE html>");
  }
  
  // Check for required meta tags
  if (!html.includes('<meta charset=')) {
    issues.push("Missing charset meta tag");
  }
  
  if (!html.includes('<meta name="viewport"')) {
    issues.push("Missing viewport meta tag");
  }
  
  // Check for basic HTML structure
  if (!html.includes("<html") || !html.includes("</html>")) {
    issues.push("Missing <html> tags");
  }
  
  if (!html.includes("<head>") || !html.includes("</head>")) {
    issues.push("Missing <head> tags");
  }
  
  if (!html.includes("<body>") || !html.includes("</body>")) {
    issues.push("Missing <body> tags");
  }
  
  return issues;
}

// Format HTML/CSS/JS and return formatted content
function formatWebAssets(html: string): string {
  // For now, return the original HTML
  // In a real implementation, we'd use a proper HTML formatter
  return html;
}

// Process a Rust file containing embedded web assets
async function processRustFile(filePath: string): Promise<ValidationResult> {
  const content = await Deno.readTextFile(filePath);
  const result: ValidationResult = {
    file: filePath,
    issues: [],
  };
  
  // Extract HTML
  const html = extractHtmlFromRust(content);
  if (!html) {
    return result;
  }
  
  // Validate HTML structure
  const htmlIssues = validateHtml(html);
  result.issues.push(...htmlIssues.map(i => `HTML: ${i}`));
  
  // Extract and validate JavaScript
  const scripts = extractJavaScriptFromHtml(html);
  for (const script of scripts) {
    const jsIssues = await validateJavaScript(script, script.includes("import"));
    result.issues.push(...jsIssues.map(i => `JS: ${i}`));
  }
  
  // Extract and validate CSS
  const styles = extractCssFromHtml(html);
  for (const style of styles) {
    const cssIssues = validateCss(style);
    result.issues.push(...cssIssues.map(i => `CSS: ${i}`));
  }
  
  // If requested, format the assets
  if (result.issues.length === 0) {
    result.formatted = formatWebAssets(html);
  }
  
  return result;
}

// Main function
async function main() {
  const args = parse(Deno.args, {
    boolean: ["fix", "quiet"],
    string: ["file"],
  });
  
  const targetFiles: string[] = [];
  
  if (args.file) {
    targetFiles.push(args.file);
  } else {
    // Find all Rust files in demo module
    for await (const entry of walk("server/src/demo", {
      exts: ["rs"],
      skip: [/target/, /node_modules/],
    })) {
      targetFiles.push(entry.path);
    }
  }
  
  let hasIssues = false;
  
  for (const file of targetFiles) {
    const result = await processRustFile(file);
    
    if (result.issues.length > 0) {
      hasIssues = true;
      if (!args.quiet) {
        console.log(`\n❌ ${result.file}`);
        for (const issue of result.issues) {
          console.log(`   ${issue}`);
        }
      }
    } else if (!args.quiet && result.formatted) {
      console.log(`✅ ${result.file}`);
    }
  }
  
  if (hasIssues) {
    console.log("\n❌ Validation failed. Run with --fix to attempt automatic fixes.");
    Deno.exit(1);
  } else if (!args.quiet) {
    console.log("\n✅ All embedded assets are valid!");
  }
}

if (import.meta.main) {
  await main();
}

export { 
  extractHtmlFromRust, 
  validateJavaScript, 
  validateCss, 
  validateHtml,
  extractJavaScriptFromHtml,
  extractCssFromHtml,
  formatWebAssets,
  processRustFile
};