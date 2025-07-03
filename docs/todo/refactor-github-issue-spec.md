# Spec: `pmat refactor auto --github-issue`

**Status:** Proposed
**Champion:** Gemini
**Date:** 2025-07-02

## 1. Overview

This document outlines the specification for a new feature: `pmat refactor auto --github-issue <URL>`. This enhancement will allow `pmat` to use a GitHub issue as the primary driver for its automated refactoring process. By integrating with GitHub, `pmat` can move from a purely metric-driven approach to a context-aware, intent-driven one, focusing its powerful refactoring capabilities on problems that have been explicitly identified by human developers.

This feature combines two key concepts:
1.  **Issue-Based Scoping:** The GitHub issue's content will determine *which files* to refactor.
2.  **Issue-Driven Prioritization:** The issue's description will influence *which problems* to prioritize within those files.

## 2. User-Facing Changes

The primary change will be the addition of a new flag to the `pmat refactor auto` command.

### New Flag

`--github-issue <URL>`

-   **Description:** Specifies the URL of a GitHub issue to guide the refactoring process.
-   **Example:** `pmat refactor auto --github-issue https://github.com/paiml/mcp-agent-toolkit/issues/123`

When this flag is used, `pmat` will ignore its default file-discovery and prioritization logic and instead focus exclusively on the context provided by the issue.

## 3. Architecture & Implementation

The new workflow will be integrated into the existing `refactor_auto_handlers.rs` and will consist of the following steps:

### 3.1. Workflow Diagram

```mermaid
graph TD
    A[User executes `pmat refactor auto --github-issue <URL>`] --> B{Fetch and Parse GitHub Issue};
    B --> C{Extract File Paths & Keywords};
    C --> D{Identify Target Files};
    D --> E{Run Standard Analysis (Lint, Complexity, etc.)};
    E --> F{Dynamically Score Violations};
    F --> G{Select Highest-Priority File};
    G --> H{Generate AI Refactoring Request};
    H --> I[Output Request for AI Agent];

    subgraph "Issue Parsing"
        B; C;
    end

    subgraph "Targeted Analysis"
        D; E;
    end

    subgraph "Prioritization"
        F; G;
    end

    subgraph "Refactoring"
        H; I;
    end
```

### 3.2. Implementation Details

#### Step 1: Fetch and Parse GitHub Issue

-   A new function, `fetch_github_issue(url: &str) -> Result<Issue>`, will be created.
-   This function will use a library like `reqwest` to make a GET request to the GitHub API.
-   The function will parse the JSON response into a simple `Issue` struct containing the `title` and `body`.

#### Step 2: Extract File Paths and Keywords

-   **File Path Extraction:** A regular expression will be used to find all potential file paths within the issue's title and body (e.g., `src/server/main.rs`, `server/src/cli/handlers.rs`).
-   **Keyword Analysis:** The issue's text will be scanned for keywords that indicate the nature of the problem. A `HashMap` will map keywords to violation categories:
    -   `"performance"`, `"slow"`, `"optimize"` -> `Complexity`, `Performance`
    -   `"bug"`, `"error"`, `"fix"` -> `Correctness`, `Bug`
    -   `"unreadable"`, `"confusing"`, `"cleanup"` -> `Readability`, `Complexity`
    -   `"security"`, `"vulnerability"` -> `Security`

#### Step 3: Dynamic Violation Scoring

-   The existing `calculate_file_severity_score` function will be enhanced.
-   It will accept the list of keywords extracted from the issue.
-   If a violation's category (e.g., a complexity-related lint) matches a keyword found in the issue, its severity score will be multiplied by a configurable factor (e.g., `3.0`). This will heavily prioritize fixing the problems the user is actually concerned about.

#### Step 4: Targeted Refactoring

-   The list of files extracted from the issue will be used to populate the `target_files` list in `handle_refactor_auto`.
-   The refactoring loop will iterate *only* over these files.
-   When selecting the next file to refactor, the system will use the dynamically adjusted severity scores, ensuring that it works on the most relevant problem first.

#### Step 5: Context-Aware AI Prompting

-   The final AI refactoring request (the JSON output) will be augmented with a new field, `issue_context`.
-   This field will contain the title and a summary of the body of the GitHub issue. This gives the AI agent crucial context about the *intent* behind the refactoring request, leading to more intelligent and relevant code suggestions.

```json
{
  "file_path": "src/server/billing.rs",
  "issue_context": {
    "title": "Billing logic is too complex and hard to test",
    "summary": "The `calculate_final_price` function has a cyclomatic complexity of 25, making it difficult to understand and test. We need to break it down into smaller, more manageable functions."
  },
  "violations": [
    // ... existing violation data
  ],
  // ...
}
```

## 4. Example Usage

1.  **A developer creates a GitHub issue:**
    -   **Title:** "High complexity in `services/complexity.rs`"
    -   **Body:** "The `ComplexityVisitor` in `services/complexity.rs` is becoming too difficult to maintain. We need to refactor it to improve readability and make it easier to add new rules."

2.  **Another developer runs the command:**
    ```bash
    pmat refactor auto --github-issue <URL_TO_ISSUE>
    ```

3.  **`pmat` performs the following actions:**
    -   Fetches the issue and extracts the file path `services/complexity.rs` and the keywords "complexity" and "readability".
    -   Runs its standard analysis on that file.
    -   When scoring violations, it gives a higher weight to any complexity-related lints.
    -   It generates a refactoring request for `services/complexity.rs`, including the issue's title and body as context for the AI.
