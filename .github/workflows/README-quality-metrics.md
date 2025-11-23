# Quality Metrics Tracking Workflow

**Phase 3.4 O(1) Quality Gates - CI/CD Integration**

## Overview

This workflow automatically tracks quality metrics (lint, test, coverage, build time) on every push/PR and provides trend analysis to detect regressions early.

## Features

- **Automatic Metric Recording**: Records lint and test-fast durations automatically
- **Trend Analysis**: Analyzes 30-day trends to detect regressions
- **PR Warnings**: Posts warnings to PRs when metrics are trending toward threshold breaches
- **Artifact Storage**: Keeps metrics data for 90 days
- **Reports**: Generates detailed metric reports for each CI run

## How It Works

### 1. Metric Recording

The workflow runs `make lint` and `make test-fast`, measures their duration, and records them:

```bash
START=$(date +%s%3N)
make lint
END=$(date +%s%3N)
DURATION=$((END - START))
pmat record-metric lint $DURATION
```

### 2. Trend Analysis

After recording, `pmat show-metrics --trend` analyzes the last 30 days:

```
📊 Quality Metrics Trends (30 days)

lint
  Direction: ↑ Regressing
  Mean: 23390.50
  Std Dev: 2156.30
  Slope: 235.46/day
  Recommendations:
    • ⚠️ WARNING: Approaching threshold in ~15 days
    • Remove unused dependencies (saves ~2-3s)
```

### 3. Regression Detection (PRs only)

For pull requests, the workflow predicts threshold breaches:

```bash
pmat predict-quality --all --failures-only --format json > regressions.json
```

If regressions are detected, it posts a comment to the PR with:
- Which metrics are regressing
- Predicted days until threshold breach
- Specific recommendations to fix the issue

### 4. Artifacts

All metrics data is uploaded as artifacts:
- **quality-metrics**: Raw `.pmat-metrics/` data
- **metrics-report**: Markdown report with trends and analysis

## Manual Usage

You can also record metrics manually in CI:

```yaml
- name: Record custom metric
  run: |
    pmat record-metric my-metric 42.5
```

Or with a custom timestamp:

```bash
pmat record-metric lint 25000 --timestamp 1763906533
```

## View Trends Locally

```bash
# Show all metric trends
pmat show-metrics --trend

# Show specific metric
pmat show-metrics --trend --metric lint

# JSON output
pmat show-metrics --trend --format json

# Show only regressing metrics
pmat show-metrics --trend --failures-only
```

## Thresholds

Default thresholds from `.pmat-metrics.toml`:
- **lint**: ≤30s (30,000ms)
- **test-fast**: ≤5min (300,000ms)
- **coverage**: ≤10min (600,000ms)
- **build-release**: ≤50MB (50,000,000 bytes)

## Architecture

```
GitHub Actions
     ↓
  measure duration
     ↓
  pmat record-metric
     ↓
  .pmat-metrics/trends/
     ↓
  pmat show-metrics --trend
     ↓
  PageRank hot metrics + ML predictions
     ↓
  PR comment (if regressing)
```

## Toyota Way Principles

- **Jidoka** (Built-in Quality): Automated regression detection
- **Andon Cord**: Stop-the-line PR warnings when quality degrades
- **Kaizen**: Continuous improvement via trend tracking
- **Genchi Genbutsu**: Direct measurement of actual build/test performance
- **Muda** (Waste Elimination): Fast trend analysis without re-running tests

## See Also

- Phase 3.2: PageRank hot metrics
- Phase 4.1: Predictive threshold breach detection
- `docs/specifications/quick-test-build-O(1)-checking.md`
