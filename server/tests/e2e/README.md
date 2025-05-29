# E2E Tests with Deno Test Runner

This directory contains end-to-end tests using Deno's built-in test runner, which provides significant advantages over standalone test scripts.

## Benefits of Using Deno Test Runner

### 1. **Test Discovery & Organization**
```bash
# Run all E2E tests
deno test tests/e2e/

# Run specific test file
deno test tests/e2e/mcp_protocol.test.ts

# Run tests matching a pattern
deno test tests/e2e/ --filter "should handle"
```

### 2. **Better Test Reporting**
- Structured test output with timing information
- Clear pass/fail indicators
- Test suite summaries
- Integration with CI/CD tools

### 3. **Parallel Execution**
```bash
# Run tests in parallel (default)
deno test --parallel tests/e2e/

# Run tests serially
deno test --jobs=1 tests/e2e/
```

### 4. **Coverage Integration**
```bash
# Run with coverage
deno test --coverage=coverage tests/e2e/

# Generate coverage report
deno coverage coverage
```

### 5. **Watch Mode**
```bash
# Re-run tests on file changes
deno test --watch tests/e2e/
```

### 6. **BDD-Style Testing**
Using `describe` and `it` blocks for better test organization:

```typescript
describe("MCP Server E2E Tests", () => {
  describe("Protocol Handling", () => {
    it("should complete initialize handshake", async () => {
      // test implementation
    });
    
    it("should list available tools", async () => {
      // test implementation
    });
  });
});
```

### 7. **Test Isolation**
- Each test runs in isolation
- `beforeAll`, `afterEach` hooks for setup/teardown
- No global state pollution

### 8. **Better Error Handling**
- Stack traces point to exact test failures
- Async errors are properly caught
- Test timeouts are configurable

## Running E2E Tests

### From the server directory:
```bash
# Run all E2E tests
deno test --allow-all tests/e2e/

# Run with specific permissions
deno test --allow-read --allow-run --allow-write tests/e2e/

# Run with coverage
deno test --allow-all --coverage=coverage tests/e2e/
```

### From the Makefile:
```bash
# Run as part of the test suite
make test

# Run only E2E tests
cd server && deno test --allow-all tests/e2e/
```

## Test Structure

### `mcp_protocol.test.ts`
Tests the core MCP protocol functionality:
- Initialize handshake
- Resource listing
- Tool discovery and execution
- Template generation
- Error handling

### `installation.test.ts`
Tests the installation process:
- Binary discovery in standard paths
- Binary execution
- MCP protocol operations
- Installation simulation

## Writing New E2E Tests

1. Create a new test file in `tests/e2e/`
2. Import test utilities:
   ```typescript
   import { assertEquals, assertExists } from "https://deno.land/std/assert/mod.ts";
   import { describe, it, beforeAll, afterEach } from "https://deno.land/std/testing/bdd.ts";
   ```

3. Structure your tests using BDD style:
   ```typescript
   describe("Feature Name", () => {
     let client: TestClient;
     
     beforeAll(() => {
       // Setup
     });
     
     afterEach(async () => {
       // Cleanup
     });
     
     it("should do something", async () => {
       // Test implementation
     });
   });
   ```

## Migration from Standalone Scripts

The original standalone scripts (`test-mcp-e2e.ts`, `test-installation.ts`) have been converted to proper test suites. The benefits include:

1. **No manual process management** - Deno handles test lifecycle
2. **Consistent error reporting** - All tests report errors the same way
3. **Integration with test tools** - Works with test runners, coverage tools, CI/CD
4. **Easier debugging** - Can run individual tests, use debugger
5. **Performance** - Tests can run in parallel when appropriate

## CI/CD Integration

These tests integrate seamlessly with CI/CD pipelines:

```yaml
# GitHub Actions example
- name: Run E2E Tests
  run: |
    cd server
    deno test --allow-all tests/e2e/
```

The structured output makes it easy to identify failures in CI logs.