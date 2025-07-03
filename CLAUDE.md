# Claude Agent Guide: paiml-mcp-agent-toolkit (pmat)

This guide provides the essential operational instructions for working on the `pmat` codebase, grounded in the principles of the Toyota Way.

## The Toyota Way: Our Guiding Philosophy

-   **Kaizen (改善): Continuous, Incremental Improvement.** We improve the codebase one file at a time. This ensures that every change is small, verifiable, and moves us toward our quality goals. Avoid large, sweeping changes.
-   **Genchi Genbutsu (現地現物): Go and See.** We don't guess where problems are. We use `pmat`'s analysis tools to find the *actual* root cause of quality issues, such as complexity hotspots or technical debt.
-   **Jidoka (自働化): Automation with a Human Touch.** We use `pmat refactor auto` to automate the creation of a refactoring plan, but an intelligent agent (you) must verify and apply the changes, ensuring correctness.

## Absolute Rules

1.  **NEVER `cd server`:** All commands **MUST** be run from the project root (`/home/noah/src/paiml-mcp-agent-toolkit`). The `Makefile` is configured to correctly handle the workspace structure.
2.  **ALWAYS Use Workspace Commands:** If you must run `cargo` commands directly, use workspace flags to target the `server` package (e.g., `cargo check --package pmat`). The `make` commands handle this for you.

## The Kaizen Refactoring Loop (The "Kata")

This is the core workflow for improving the codebase. Treat it as a repeatable practice (a kata) to drive quality.

### Step 1: Find the Target (Genchi Genbutsu)

First, "go and see" the problems. Use `pmat`'s analysis tools to identify the most critical area for improvement. Choose **one** of the following commands to find a target file:

-   **For General Quality Issues:**
    ```bash
    pmat analyze lint-hotspot --top-files 5
    ```
-   **For High Complexity:**
    ```bash
    pmat analyze complexity --top-files 5
    ```
-   **For Technical Debt:**
    ```bash
    pmat analyze satd
    ```
-   **For Unused Code:**
    ```bash
    pmat analyze dead-code
    ```

### Step 2: Create the Refactoring Plan (Jidoka)

Once you have identified a target file, use `pmat refactor auto` to generate an automated, AI-driven refactoring plan.

```bash
# Generate a refactoring plan for the chosen file
pmat refactor auto --file <path/to/target/file.rs>
```

### Step 3: Verify the Improvement

After applying the refactoring, you **MUST** verify that the change improved quality and did not introduce regressions.

1.  **Run Quality Gate:** Ensure the specific file now meets our zero-tolerance standards.
    ```bash
    pmat quality-gate --file <path/to/target/file.rs>
    ```
2.  **Run Fast Tests:** Confirm that the changes have not broken any existing functionality.
    ```bash
    make test-fast
    ```

Once both checks pass, commit the changes and return to Step 1 to find the next target.

## Mandatory Checks Before Committing

After making **any** code changes, you **MUST** run the following commands from the project root. A commit will not be accepted otherwise.

```bash
make lint
make test-fast
```

## Quality Standards (Zero Tolerance)

-   **Complexity:** Maximum cyclomatic complexity of **20** per function (target: 5).
-   **Test Coverage:** Minimum of **80%** per file.
-   **Technical Debt:** **Zero** self-admitted technical debt (no `TODO`, `FIXME`, `HACK` comments).
-   **Linting:** Must pass all `clippy::pedantic` and `clippy::nursery` lints.