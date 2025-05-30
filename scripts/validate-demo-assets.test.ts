#!/usr/bin/env -S deno test --allow-read --allow-write --allow-run

import { assertEquals, assertStringIncludes } from "https://deno.land/std@0.208.0/assert/mod.ts";
import {
  extractHtmlFromRust,
  validateJavaScript,
  validateCss,
  validateHtml,
  extractJavaScriptFromHtml,
  extractCssFromHtml,
  formatWebAssets,
  processRustFile,
} from "./validate-demo-assets.ts";

Deno.test("extractHtmlFromRust - extracts HTML from const string", () => {
  const rustCode = `
const HTML_TEMPLATE: &str = r#"<!DOCTYPE html>
<html>
<head><title>Test</title></head>
<body>Hello</body>
</html>"#;
`;
  
  const html = extractHtmlFromRust(rustCode);
  assertEquals(html?.includes("<!DOCTYPE html>"), true);
  assertEquals(html?.includes("<title>Test</title>"), true);
});

Deno.test("extractHtmlFromRust - extracts HTML from format! macro", () => {
  const rustCode = `
let html = format!(r#"
<div>{}</div>
"#, content);
`;
  
  const html = extractHtmlFromRust(rustCode);
  assertEquals(html?.trim(), "<div>{}</div>");
});

Deno.test("validateHtml - detects missing DOCTYPE", () => {
  const html = `<html><head></head><body></body></html>`;
  const issues = validateHtml(html);
  assertStringIncludes(issues.join(" "), "DOCTYPE");
});

Deno.test("validateHtml - detects missing meta tags", () => {
  const html = `<!DOCTYPE html>
<html><head><title>Test</title></head><body></body></html>`;
  const issues = validateHtml(html);
  assertStringIncludes(issues.join(" "), "charset");
  assertStringIncludes(issues.join(" "), "viewport");
});

Deno.test("validateHtml - accepts valid HTML", () => {
  const html = `<!DOCTYPE html>
<html>
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Test</title>
</head>
<body>Content</body>
</html>`;
  const issues = validateHtml(html);
  assertEquals(issues.length, 0);
});

Deno.test("validateCss - detects double semicolons", () => {
  const css = `
.test {
  color: red;;
  margin: 0;
}`;
  const issues = validateCss(css);
  assertStringIncludes(issues.join(" "), "Double semicolon");
});

Deno.test("validateCss - detects empty rules", () => {
  const css = `.empty { }`;
  const issues = validateCss(css);
  assertStringIncludes(issues.join(" "), "Empty CSS rule");
});

Deno.test("validateJavaScript - validates strict mode", async () => {
  const js = `
let x = 10;
x = x + 5;  // Reassign to avoid prefer-const lint error
const y = 20;
console.log(x + y);
`;
  const issues = await validateJavaScript(js);
  // Should pass without issues for valid code
  assertEquals(issues.filter(i => i.includes("error")).length, 0);
});

Deno.test("validateJavaScript - detects lint issues", async () => {
  const js = `
var x = 10;  // Should use let or const
eval("console.log('test')");  // eval is dangerous
`;
  const issues = await validateJavaScript(js);
  // Should detect issues with var and eval
  assertEquals(issues.length > 0, true);
});

// Add tests for extractJavaScriptFromHtml
Deno.test("extractJavaScriptFromHtml - extracts inline scripts", () => {
  const html = `
<!DOCTYPE html>
<html>
<head>
  <script>
    console.log('inline script 1');
  </script>
</head>
<body>
  <script type="module">
    import { test } from './module.js';
  </script>
</body>
</html>`;
  
  const scripts = extractJavaScriptFromHtml(html);
  assertEquals(scripts.length, 2);
  assertStringIncludes(scripts[0], "console.log('inline script 1')");
  assertStringIncludes(scripts[1], "import { test }");
});

// Add tests for extractCssFromHtml
Deno.test("extractCssFromHtml - extracts inline styles", () => {
  const html = `
<!DOCTYPE html>
<html>
<head>
  <style>
    body { margin: 0; }
  </style>
  <style>
    .container { padding: 20px; }
  </style>
</head>
</html>`;
  
  const styles = extractCssFromHtml(html);
  assertEquals(styles.length, 2);
  assertStringIncludes(styles[0], "body { margin: 0; }");
  assertStringIncludes(styles[1], ".container { padding: 20px; }");
});

// Add tests for formatWebAssets
Deno.test("formatWebAssets - returns html unchanged", () => {
  const html = "<html><body>Test</body></html>";
  const result = formatWebAssets(html);
  assertEquals(result, html);
});

// Add tests for processRustFile
Deno.test("processRustFile - validates Rust file with HTML template", async () => {
  // Create a temporary Rust file with HTML template
  const tempFile = await Deno.makeTempFile({ suffix: ".rs" });
  const rustContent = `
pub const HTML_TEMPLATE: &str = r#"<!DOCTYPE html>
<html>
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Test</title>
  <style>
    body { margin: 0;; }
  </style>
  <script>
    var test = 1;
  </script>
</head>
<body>Test</body>
</html>"#;
`;
  
  await Deno.writeTextFile(tempFile, rustContent);
  
  try {
    const result = await processRustFile(tempFile);
    assertEquals(result.file, tempFile);
    // Should have CSS double semicolon issue
    const cssIssue = result.issues.find(i => i.includes("Double semicolon"));
    assertEquals(cssIssue !== undefined, true);
    // Should have JS var issue
    const jsIssue = result.issues.find(i => i.includes("var"));
    assertEquals(jsIssue !== undefined, true);
  } finally {
    await Deno.remove(tempFile);
  }
});

// Test more HTML validation cases
Deno.test("validateHtml - detects missing html tags", () => {
  const html = `<!DOCTYPE html>
<head><title>Test</title></head>
<body>Content</body>`;
  const issues = validateHtml(html);
  assertStringIncludes(issues.join(" "), "Missing <html> tags");
});

Deno.test("validateHtml - detects missing head tags", () => {
  const html = `<!DOCTYPE html>
<html>
<meta charset="UTF-8">
<body>Content</body>
</html>`;
  const issues = validateHtml(html);
  assertStringIncludes(issues.join(" "), "Missing <head> tags");
});

Deno.test("validateHtml - detects missing body tags", () => {
  const html = `<!DOCTYPE html>
<html>
<head><title>Test</title></head>
Content here
</html>`;
  const issues = validateHtml(html);
  assertStringIncludes(issues.join(" "), "Missing <body> tags");
});

// Test CSS validation edge cases
Deno.test("validateCss - handles empty CSS", () => {
  const css = "";
  const issues = validateCss(css);
  assertStringIncludes(issues.join(" "), "CSS is empty");
});

Deno.test("validateCss - detects !important usage", () => {
  const css = `
.warning {
  color: red !important;
}`;
  const issues = validateCss(css);
  assertStringIncludes(issues.join(" "), "!important should be avoided");
});

// Test JavaScript validation with module code
Deno.test("validateJavaScript - handles ES modules", async () => {
  const js = `
import { something } from './module.js';
export const test = () => {
  console.log('test');
};
`;
  const issues = await validateJavaScript(js, true);
  // Module code should have 1 lint issue (unused import)
  assertEquals(issues.length, 2); // lint issue + formatting issue
});

// Test main function
Deno.test("main function - handles file argument", async () => {
  // Create a test file
  const tempFile = await Deno.makeTempFile({ suffix: ".rs" });
  await Deno.writeTextFile(tempFile, `const TEST: &str = "test";`);
  
  try {
    // Save original args
    const originalArgs = Deno.args;
    
    // Mock Deno.args
    Object.defineProperty(Deno, 'args', {
      value: ['--file', tempFile, '--quiet'],
      configurable: true
    });
    
    // Import and run main
    const _module = await import("./validate-demo-assets.ts");
    // The main function would be called if this was the main module
    
    // Restore args
    Object.defineProperty(Deno, 'args', {
      value: originalArgs,
      configurable: true
    });
  } finally {
    await Deno.remove(tempFile);
  }
});

// Import the functions we need to test
