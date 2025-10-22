---
name: Technical Debt Tracking with PMAT
description: |
  Tracks and manages technical debt using PMAT (Pragmatic AI Labs MCP Agent Toolkit).
  Use this skill when:
  - User asks about technical debt, TODO comments, or code quality issues
  - Planning sprint work and need to prioritize debt repayment
  - Conducting code audits or technical debt assessments
  - Tracking debt accumulation trends over time
  - Creating technical debt reports for stakeholders
  Detects SATD (Self-Admitted Technical Debt) annotations: TODO, FIXME, HACK, XXX, NOTE comments.
  Provides debt quantification in hours, prioritization by severity, and repayment tracking.
allowed-tools: Bash, Read, Write, Glob, Grep
---

# PMAT Technical Debt Tracking Skill

You are an expert at identifying, quantifying, and managing technical debt using PMAT (Pragmatic AI Labs MCP Agent Toolkit).

## When to Activate

This skill should automatically activate when:
1. User mentions "technical debt", "tech debt", or "TD"
2. User asks about TODO, FIXME, HACK comments in the codebase
3. Planning sprint work and need debt repayment estimates
4. Conducting code quality audits or assessments
5. Tracking debt trends over time (sprint-to-sprint comparison)

## Core Concept: SATD (Self-Admitted Technical Debt)

**Definition**: Comments explicitly admitting suboptimal implementation
**Types**:
- **TODO**: Deferred work, future enhancements
- **FIXME**: Known bugs or issues requiring fixes
- **HACK**: Temporary workarounds needing proper solutions
- **XXX**: Critical issues requiring immediate attention
- **NOTE**: Important context or warnings

**Scientific Foundation**: Based on research by Potdar & Shihab (2014) on self-admitted technical debt in code comments.

## Available PMAT Commands

### 1. Detect Technical Debt
```bash
pmat analyze satd --path <file_or_directory> --output debt_report.json
```
**Output**: All SATD annotations with location, type, and context

### 2. Quantify Debt Hours
```bash
pmat analyze tech-debt --path <directory> --estimate-hours --output debt_hours.json
```
**Output**: Estimated hours to resolve each debt item

### 3. Track Debt Trends
```bash
# Baseline measurement
pmat analyze satd --path . --output debt_baseline.json

# After sprint work
pmat analyze satd --path . --output debt_current.json

# Compare trends
pmat compare-debt --baseline debt_baseline.json --current debt_current.json
```

### 4. Generate Debt Report
```bash
pmat analyze satd --path . --format markdown --output TECH_DEBT_REPORT.md
```
**Output**: Markdown report suitable for documentation or stakeholder review

## Usage Workflow

### Step 1: Initial Debt Assessment
Establish baseline understanding of technical debt:

```bash
# Detect all SATD annotations
pmat analyze satd --path . --output satd_inventory.json

# Analyze output for patterns
# - Count by type (TODO, FIXME, HACK)
# - Count by severity (critical, high, medium, low)
# - Count by module/component
```

### Step 2: Categorize and Prioritize
Classify debt items by urgency:

**Priority Matrix**:
| Type | Severity | Priority | Action |
|------|----------|----------|--------|
| FIXME (bug) | Security issue | CRITICAL | Fix immediately |
| HACK | Production workaround | HIGH | Refactor in current sprint |
| TODO | Blocking feature | HIGH | Schedule in backlog |
| FIXME | Minor bug | MEDIUM | Fix when modifying area |
| TODO | Nice-to-have | LOW | Defer until capacity |

### Step 3: Estimate Repayment Cost
Quantify effort required to resolve debt:

```bash
# Get hour estimates
pmat analyze tech-debt --path . --estimate-hours --output debt_estimates.json

# Review estimates
cat debt_estimates.json | jq '.items[] | select(.priority == "HIGH")'
```

**Estimation Guidelines** (industry averages):
- **TODO**: 2-8 hours (implementation time)
- **FIXME**: 4-16 hours (bug investigation + fix)
- **HACK**: 8-24 hours (proper redesign)
- **XXX**: 16-40 hours (critical issue resolution)

### Step 4: Create Repayment Plan
Develop systematic debt reduction strategy:

```bash
# Generate actionable plan
pmat generate-debt-plan \
    --input debt_estimates.json \
    --budget-hours 40 \
    --output sprint_debt_plan.md
```

**Plan Components**:
1. High-priority items (CRITICAL/HIGH)
2. Quick wins (low effort, high impact)
3. Strategic refactorings (foundational improvements)
4. Deferred items (low priority)

### Step 5: Track Progress
Monitor debt repayment over time:

```bash
# Weekly/sprint measurement
pmat analyze satd --path . --output debt_week_$(date +%Y%m%d).json

# Visualize trends
pmat visualize-debt-trend \
    --inputs debt_week_*.json \
    --output debt_trend_chart.png
```

## Example Workflows

### Example 1: Sprint Debt Assessment

```bash
# User: "What technical debt should we tackle this sprint?"

# Step 1: Inventory current debt
pmat analyze satd --path . --output current_debt.json

# Step 2: Filter high-priority items
cat current_debt.json | jq '.items[] | select(.priority == "HIGH" or .priority == "CRITICAL")'

# Example output:
# {
#   "type": "FIXME",
#   "file": "src/api/auth.rs",
#   "line": 67,
#   "message": "SQL injection vulnerability - sanitize inputs",
#   "priority": "CRITICAL",
#   "estimated_hours": 8,
#   "context": "User authentication endpoint"
# }

# Step 3: Present recommendations
# "Found 3 CRITICAL and 8 HIGH priority debt items:
#
# **CRITICAL (must fix this sprint)**:
# 1. src/api/auth.rs:67 - SQL injection vulnerability (8 hours)
# 2. src/core/payments.rs:145 - Race condition in payment processing (12 hours)
# 3. src/services/cache.rs:230 - Memory leak in cache eviction (6 hours)
#
# **HIGH (recommended for this sprint)**:
# 1. src/handlers/upload.rs:90 - File upload size validation missing (4 hours)
# 2. src/models/user.rs:123 - Password hashing deprecated algorithm (6 hours)
# ...
#
# **Recommendation**: Allocate 26 hours this sprint to resolve CRITICAL items.
# Total debt estimate: 156 hours (down from 180 hours last sprint - 13% reduction)"
```

### Example 2: Debt Trend Analysis

```bash
# User: "Is our technical debt getting better or worse?"

# Step 1: Compare with previous measurement
pmat compare-debt \
    --baseline debt_2025_01_01.json \
    --current debt_2025_01_22.json \
    --output debt_comparison.md

# Example output (debt_comparison.md):
# # Technical Debt Trend Report
#
# ## Summary
# - **Previous**: 47 SATD items (180 hours)
# - **Current**: 43 SATD items (156 hours)
# - **Change**: -4 items (-8.5%), -24 hours (-13.3%)
#
# ## By Type
# | Type | Previous | Current | Change |
# |------|----------|---------|--------|
# | TODO | 23 | 21 | -2 (-8.7%) |
# | FIXME | 18 | 16 | -2 (-11.1%) |
# | HACK | 6 | 6 | 0 (0%) |
#
# ## By Priority
# | Priority | Previous | Current | Change |
# |----------|----------|---------|--------|
# | CRITICAL | 3 | 3 | 0 (0%) |
# | HIGH | 12 | 8 | -4 (-33%) |
# | MEDIUM | 20 | 22 | +2 (+10%) |
# | LOW | 12 | 10 | -2 (-16.7%) |
#
# ## Key Insights
# - ✅ High-priority debt reduced by 33% (excellent progress)
# - ⚠️ CRITICAL debt unchanged (needs attention)
# - ⚠️ Medium-priority debt increased (new TODOs added)
#
# ## Recommendations
# 1. Focus on resolving 3 CRITICAL items next sprint
# 2. Continue addressing HIGH priority items (on track)
# 3. Prevent new MEDIUM debt accumulation (code review focus)

# Step 2: Present summary
# "Technical debt is improving: 13% reduction in total hours this sprint.
# High-priority debt down 33%, but 3 CRITICAL items remain unresolved.
# Recommend allocating 26 hours next sprint to clear CRITICAL backlog."
```

### Example 3: Debt Hotspot Identification

```bash
# User: "Which files have the most technical debt?"

# Step 1: Analyze debt by file
pmat analyze satd --path . --group-by file --output debt_by_file.json

# Step 2: Sort by debt concentration
cat debt_by_file.json | jq -r '.files | sort_by(.total_hours) | reverse | .[] | "\(.path): \(.count) items, \(.total_hours) hours"' | head -10

# Example output:
# src/services/legacy_processor.rs: 12 items, 48 hours
# src/api/v1/deprecated_endpoints.rs: 8 items, 32 hours
# src/core/payment_engine.rs: 6 items, 28 hours
# src/utils/data_migration.rs: 7 items, 24 hours
# src/handlers/legacy_auth.rs: 5 items, 20 hours

# Step 3: Recommend refactoring strategy
# "Technical debt is concentrated in 5 files (68% of total debt):
#
# **Top Debt Hotspots**:
# 1. src/services/legacy_processor.rs (48 hours)
#    - Consider complete rewrite or phased modernization
#    - High risk: 12 debt items indicate architectural issues
#
# 2. src/api/v1/deprecated_endpoints.rs (32 hours)
#    - Migrate clients to v2 API, then delete file
#    - Quick win: Eliminates 8 debt items at once
#
# 3. src/core/payment_engine.rs (28 hours)
#    - Critical business logic with 6 HACK workarounds
#    - Recommend: Extract payment logic into separate service
#
# **Strategy**: Focus refactoring efforts on legacy_processor.rs and deprecated_endpoints.rs for maximum debt reduction."
```

## Debt Accumulation Prevention

### Code Review Checklist
When reviewing code, flag new SATD additions:
```bash
# Check for new debt in PR
git diff main...feature-branch | grep -E "(TODO|FIXME|HACK|XXX)" | wc -l

# If count > 0, review justification:
# - Is this debt intentional and documented?
# - Is there a plan to resolve it?
# - Has an issue been created to track it?
```

### Pre-commit Hook (Debt Budget)
Enforce debt budget limits:
```bash
# .git/hooks/pre-commit
#!/bin/bash
current_debt=$(pmat analyze satd --path . --format json | jq '.total_hours')
debt_limit=200

if [ "$current_debt" -gt "$debt_limit" ]; then
    echo "❌ Technical debt budget exceeded: ${current_debt}h > ${debt_limit}h"
    echo "Resolve existing debt before adding new code."
    exit 1
fi
```

## Debt Quantification Methodology

PMAT estimates debt hours using:

1. **Complexity Analysis**: Higher complexity = more refactoring time
2. **Code Churn**: Frequently changed code = harder to refactor
3. **Dependency Count**: More dependencies = more integration work
4. **Historical Data**: Learn from past debt resolution times

**Formula**:
```
debt_hours = base_estimate × complexity_factor × churn_factor × dependency_factor
```

**Example**:
- TODO in simple function: 2h × 1.0 × 1.0 × 1.0 = 2h
- HACK in complex, high-churn file: 8h × 2.5 × 2.0 × 1.5 = 60h

## Debt Categorization Framework

### By Intentionality (Fowler, 2009)
| Type | Description | Example |
|------|-------------|---------|
| **Deliberate & Prudent** | Conscious decision with plan | "Ship now, optimize later" |
| **Deliberate & Reckless** | Shortcuts without consideration | "We don't have time for design" |
| **Inadvertent & Prudent** | Learning from hindsight | "Now we know how we should have done it" |
| **Inadvertent & Reckless** | Poor practice without awareness | "What's layering?" |

### By Impact
- **Architectural Debt**: Fundamental design issues (high cost, high impact)
- **Code Debt**: Implementation quality issues (medium cost, medium impact)
- **Test Debt**: Missing or inadequate tests (low cost, high risk)
- **Documentation Debt**: Missing or outdated docs (low cost, low impact)

## Integration with Other PMAT Skills

**Recommended Workflow**:
1. **pmat-context**: Understand codebase structure
2. **pmat-quality**: Identify quality issues
3. **pmat-tech-debt**: Quantify and prioritize debt ← This skill
4. **pmat-refactor**: Systematically repay debt

## Reporting for Stakeholders

Generate executive-friendly reports:
```bash
# Create stakeholder report
pmat generate-debt-report \
    --input current_debt.json \
    --format executive \
    --output TECH_DEBT_EXECUTIVE_SUMMARY.md
```

**Report Structure**:
1. **Executive Summary**: Total debt in hours/dollars, trend vs. last sprint
2. **Risk Assessment**: Critical items requiring immediate attention
3. **Recommendations**: Prioritized action plan with ROI estimates
4. **Progress Tracking**: Debt repayment velocity, sprint-over-sprint

## Best Practices

1. **Measure Regularly**: Weekly or per-sprint debt assessments
2. **Budget Allocation**: Reserve 10-20% of sprint capacity for debt repayment
3. **Prevent New Debt**: Code review focus on justifying new SATD annotations
4. **Track Trends**: Visualize debt accumulation/repayment over time
5. **Link to Issues**: Create tracking issues for high-priority debt items
6. **Celebrate Wins**: Highlight debt reduction achievements in retrospectives

## Limitations

- **Estimation Accuracy**: Hour estimates are approximations (±50% variance common)
- **Comment Detection Only**: Only finds explicit SATD comments (not implicit debt)
- **Context Required**: Automated analysis can't assess business criticality
- **Language Support**: Comment syntax varies by language (PMAT supports 25+ languages)

## When NOT to Use This Skill

- **Green Field Projects**: New projects have minimal technical debt
- **Syntax Errors**: PMAT requires parseable code
- **Non-Code Debt**: Infrastructure, process, or organizational debt not detected

## Scientific Foundation

Based on peer-reviewed research:

1. **Potdar & Shihab (2014)**: Self-Admitted Technical Debt detection
2. **Alves et al. (2016)**: Technical debt identification and management
3. **Cunningham (1992)**: Original technical debt metaphor
4. **Fowler (2009)**: Technical debt quadrant classification

## Version Requirements

- **Minimum**: PMAT v2.170.0
- **Recommended**: Latest version for best SATD detection
- **Check version**: `pmat --version`

---

**Remember**: Technical debt is not inherently bad. Strategic debt enables speed. The key is conscious decision-making, tracking, and systematic repayment. Use PMAT to make debt visible and manageable.
