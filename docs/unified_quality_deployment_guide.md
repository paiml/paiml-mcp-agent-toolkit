# PMAT Unified Quality System - Production Deployment Guide

## Executive Summary

This guide provides comprehensive instructions for deploying the PMAT Unified Quality Enforcement System in production environments. The system implements a dual-track approach delivering immediate value while building toward theoretical maximum quality automation.

**Key Features:**
- 🔄 **Real-time Quality Monitoring**: 5-10ms file analysis latency using incremental AST parsing
- ⚖️ **SRE-style Error Budgets**: Flexible enforcement with team-specific quality budgets
- 🧠 **Intelligent Assistance**: Context-aware refactoring suggestions with confidence scoring  
- 🤖 **Conservative Automation**: Safe transformations with Git rollback protection
- 📊 **Comprehensive Observability**: Prometheus metrics, Grafana dashboards, GitHub Actions integration

## Prerequisites

### System Requirements
- **OS**: Linux, macOS, or Windows with WSL2
- **Memory**: Minimum 4GB RAM, recommended 8GB for large codebases  
- **Storage**: 2GB free space for caches and logs
- **Network**: Outbound HTTPS for GitHub API, Prometheus, and dependency updates

### Software Dependencies
- **Rust**: 1.70.0 or later (`rustup install stable`)
- **Git**: 2.30.0 or later for safety net features
- **Optional**: Docker and docker-compose for containerized deployment

### Access Requirements
- **GitHub**: Personal access token with repo and workflow permissions
- **Prometheus**: Access to metrics collection endpoint (if using external Prometheus)
- **Team Permissions**: Admin access to repositories for webhook and Actions setup

## Phase 1: Foundation Setup (Week 1)

### 1.1 Installation

#### Option A: Direct Installation (Recommended)
```bash
# Install latest stable version
cargo install pmat --version latest --features unified-quality

# Verify installation
pmat --version
pmat unified-quality --help
```

#### Option B: Docker Deployment
```bash
# Clone the repository
git clone https://github.com/paiml/paiml-mcp-agent-toolkit
cd paiml-mcp-agent-toolkit

# Build Docker image
docker build -t pmat-unified-quality .

# Run container with mounted project
docker run -v $(pwd):/workspace pmat-unified-quality
```

### 1.2 Initial Configuration

Create the configuration file `pmat-quality.toml`:

```toml
[foundation]
# Real-time monitoring configuration
update_interval = "5s"
complexity_threshold = 20
watch_patterns = [
    "**/*.rs",
    "**/*.py", 
    "**/*.js",
    "**/*.ts"
]
debounce_interval = "500ms"
max_batch_size = 50
incremental_parsing = true
cache_ast = true

[enforcement]
# Error budget enforcement
enabled = true
default_complexity_budget = 100
default_satd_budget = 10
default_coverage_floor = 0.8
regeneration_rate = 10.0  # Points per day
grace_period_days = 14

[intelligence]
# AI-powered assistance
enabled = true
confidence_threshold = 0.75
max_suggestions = 5
include_examples = true
learning_mode = true

[automation] 
# Conservative automation
enabled = false  # Start disabled, enable after team training
safety_threshold = 0.95
git_safety_net = true
rollback_timeout = "10m"
max_automated_changes = 5

[performance]
# Performance monitoring
continuous_monitoring = true
benchmark_interval = "1h"
max_analysis_time_ms = 5000
max_memory_mb = 1024
regression_threshold_percent = 20.0

[observability]
# Metrics and monitoring
prometheus_enabled = true
prometheus_port = 9090
dashboard_enabled = true 
dashboard_port = 8080
github_actions_enabled = false  # Configure later
```

### 1.3 Team Registration

Register your teams with appropriate error budgets:

```bash
# Register development team
pmat unified-quality team register \
    --team-id "dev-team" \
    --complexity-budget 150 \
    --satd-budget 15 \
    --coverage-floor 0.75

# Register critical systems team  
pmat unified-quality team register \
    --team-id "platform-team" \
    --complexity-budget 75 \
    --satd-budget 5 \
    --coverage-floor 0.90

# Register new team (higher budget during ramp-up)
pmat unified-quality team register \
    --team-id "new-team" \
    --complexity-budget 200 \
    --satd-budget 25 \
    --coverage-floor 0.65 \
    --grace-period-days 30
```

### 1.4 Initial Monitoring Setup

Start monitoring your primary codebase:

```bash
# Initialize monitoring for main repository
cd /path/to/your/project
pmat unified-quality init

# Start monitoring daemon
pmat unified-quality monitor start \
    --config pmat-quality.toml \
    --daemon \
    --log-level info

# Verify monitoring is working
pmat unified-quality status
pmat unified-quality metrics summary
```

## Phase 2: Quality Intelligence (Week 2-3)

### 2.1 Enable Intelligent Assistance

```bash
# Enable intelligence features
pmat unified-quality intelligence enable \
    --confidence-threshold 0.8 \
    --learning-mode true

# Analyze current codebase for improvement opportunities
pmat unified-quality analyze hotspots \
    --top-files 20 \
    --output-format detailed

# Generate initial refactoring recommendations
pmat unified-quality suggest refactoring \
    --target-complexity 15 \
    --min-confidence 0.75
```

### 2.2 Set Up Web Dashboard

The dashboard provides real-time visibility into quality metrics:

```bash
# Start dashboard (runs on port 8080 by default)
pmat unified-quality dashboard start \
    --port 8080 \
    --update-interval 5s

# Access at http://localhost:8080
# Features available:
# - Real-time quality metrics
# - File-level analysis
# - Team budget status  
# - Refactoring suggestions
# - Performance insights
```

### 2.3 Configure Team Onboarding

Set up progressive onboarding for team adoption:

```bash
# Start onboarding for a team
pmat unified-quality onboarding start \
    --team-id "dev-team" \
    --learning-style practical \
    --experience-level intermediate

# Track onboarding progress
pmat unified-quality onboarding progress \
    --team-id "dev-team"

# Generate onboarding report
pmat unified-quality onboarding report \
    --team-id "dev-team" \
    --format markdown \
    --output team-progress.md
```

## Phase 3: Enforcement Integration (Week 3-4)

### 3.1 Pre-commit Hook Installation

Install quality gates in your Git workflow:

```bash
# Install pre-commit hook
pmat unified-quality hooks install \
    --pre-commit \
    --complexity-threshold 20 \
    --block-on-violation

# The hook will run:
# 1. Complexity analysis on changed files
# 2. SATD detection and limits
# 3. Error budget checks
# 4. Coverage impact analysis
```

### 3.2 GitHub Actions Integration

Create `.github/workflows/quality-gate.yml`:

```yaml
name: Quality Gate
on:
  pull_request:
    branches: [main, master]
  push:
    branches: [main, master]

jobs:
  quality-gate:
    runs-on: ubuntu-latest
    permissions:
      contents: read
      pull-requests: write
      checks: write
    
    steps:
    - uses: actions/checkout@v4
      with:
        fetch-depth: 0
    
    - name: Setup Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        profile: minimal
        override: true
    
    - name: Install PMAT
      run: cargo install pmat --version latest --features unified-quality
    
    - name: Run Quality Analysis
      id: quality
      env:
        GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
      run: |
        pmat unified-quality analyze pull-request \
          --pr-number ${{ github.event.number }} \
          --base-ref ${{ github.base_ref }} \
          --head-ref ${{ github.head_ref }} \
          --output-format github-actions
    
    - name: Update PR Comment
      if: github.event_name == 'pull_request'
      uses: actions/github-script@v7
      with:
        script: |
          const fs = require('fs');
          const comment = fs.readFileSync('quality-report.md', 'utf8');
          
          // Find existing comment
          const comments = await github.rest.issues.listComments({
            owner: context.repo.owner,
            repo: context.repo.repo,
            issue_number: context.issue.number,
          });
          
          const existingComment = comments.data.find(
            comment => comment.body.includes('📊 Code Quality Report')
          );
          
          if (existingComment) {
            // Update existing comment
            await github.rest.issues.updateComment({
              owner: context.repo.owner,
              repo: context.repo.repo,
              comment_id: existingComment.id,
              body: comment,
            });
          } else {
            // Create new comment
            await github.rest.issues.createComment({
              owner: context.repo.owner,
              repo: context.repo.repo,
              issue_number: context.issue.number,
              body: comment,
            });
          }
```

### 3.3 Configure Error Budget Policies

Set up team-specific quality enforcement policies:

```bash
# Configure strict enforcement for critical components
pmat unified-quality policy create \
    --name "critical-systems" \
    --max-complexity 10 \
    --max-satd 0 \
    --min-coverage 0.95 \
    --block-on-violation \
    --applies-to "src/core/**" "src/security/**"

# Configure relaxed policy for experimental code
pmat unified-quality policy create \
    --name "experimental" \
    --max-complexity 30 \
    --max-satd 20 \
    --min-coverage 0.60 \
    --warn-only \
    --applies-to "experiments/**" "prototypes/**"
```

## Phase 4: Automation & Optimization (Week 4+)

### 4.1 Enable Conservative Automation

**⚠️ IMPORTANT**: Only enable after team training and testing in non-critical branches.

```bash
# Enable automation with safety limits
pmat unified-quality automation enable \
    --safety-threshold 0.95 \
    --max-changes-per-day 10 \
    --git-safety-net \
    --rollback-timeout 30m \
    --dry-run  # Test first

# After testing, enable for real
pmat unified-quality automation enable \
    --production-mode \
    --notify-changes \
    --require-approval-above-threshold 0.85
```

### 4.2 Performance Optimization

Configure performance monitoring and optimization:

```bash
# Establish performance baseline
pmat unified-quality performance baseline \
    --name "production-v1" \
    --full-analysis

# Enable continuous performance monitoring
pmat unified-quality performance monitor \
    --continuous \
    --alert-on-regression 25% \
    --optimize-automatically false

# Run performance benchmarks
pmat unified-quality performance benchmark \
    --suite comprehensive \
    --output-format prometheus
```

### 4.3 Prometheus Integration

Configure metrics collection in `prometheus.yml`:

```yaml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'pmat-quality'
    static_configs:
      - targets: ['localhost:9090']
    scrape_interval: 30s
    metrics_path: /metrics
```

Example Grafana dashboard queries:

```promql
# Average complexity per team
avg(quality_pmat_complexity) by (team)

# Error budget consumption rate  
rate(quality_pmat_violations_total[5m]) * 3600

# Analysis performance
histogram_quantile(0.95, quality_pmat_analysis_duration_seconds_bucket)
```

## Monitoring and Maintenance

### Daily Operations

```bash
# Check system health
pmat unified-quality health check

# Review quality metrics
pmat unified-quality metrics dashboard

# Check error budget status
pmat unified-quality budget status --all-teams

# Review automation results
pmat unified-quality automation report --last-24h
```

### Weekly Maintenance

```bash
# Clean old data and optimize
pmat unified-quality maintenance weekly

# Update quality baselines
pmat unified-quality baseline update --all-projects

# Generate team reports
pmat unified-quality report generate \
    --teams all \
    --format pdf \
    --include-trends \
    --email-to stakeholders@company.com
```

### Monthly Reviews

```bash
# Comprehensive system analysis
pmat unified-quality analyze comprehensive \
    --include-trends \
    --benchmark-comparison \
    --team-progress

# Update quality policies based on data
pmat unified-quality policy review \
    --suggest-adjustments \
    --data-driven

# Plan next phase improvements
pmat unified-quality roadmap generate \
    --based-on-usage \
    --team-feedback
```

## Troubleshooting

### Common Issues

**High Memory Usage**
```bash
# Check cache size
pmat unified-quality cache stats

# Clean caches if needed
pmat unified-quality cache clean --older-than 7d

# Adjust cache limits in config
[foundation]
max_cache_size_mb = 512
cache_cleanup_interval = "1h"
```

**Slow Analysis Performance**
```bash
# Check performance stats
pmat unified-quality performance stats

# Enable optimization
pmat unified-quality performance optimize \
    --strategy cache-optimization \
    --strategy parallel-processing

# Reduce analysis scope if needed
[foundation]
watch_patterns = ["**/*.rs"]  # Focus on primary language
max_batch_size = 25          # Reduce batch size
```

**Error Budget Conflicts**
```bash
# Check budget status
pmat unified-quality budget status --team-id problematic-team

# Regenerate budgets if needed
pmat unified-quality budget regenerate --team-id problematic-team

# Adjust policies if too restrictive
pmat unified-quality policy adjust \
    --team-id problematic-team \
    --increase-budget 25%
```

### Log Analysis

Important log locations:
- **System logs**: `~/.pmat/logs/unified-quality.log`
- **Performance logs**: `~/.pmat/logs/performance.log`
- **Automation logs**: `~/.pmat/logs/automation.log`

```bash
# Monitor real-time logs
tail -f ~/.pmat/logs/unified-quality.log

# Check for errors in last hour
grep ERROR ~/.pmat/logs/*.log | grep "$(date -d '1 hour ago' '+%Y-%m-%d %H')"

# Export logs for analysis
pmat unified-quality logs export \
    --since "2024-01-01" \
    --level warn,error \
    --format json \
    --output quality-issues.json
```

## Security Considerations

### Access Control
- Use dedicated service accounts for CI/CD integration
- Rotate GitHub tokens monthly
- Limit network access to required endpoints only
- Enable audit logging for all configuration changes

### Data Protection
- Quality metrics may contain sensitive code patterns
- Configure data retention policies appropriately
- Use encrypted storage for cached AST data
- Implement data export controls for compliance

### Secrets Management
```bash
# Use environment variables for sensitive config
export PMAT_GITHUB_TOKEN="your-token-here"
export PMAT_PROMETHEUS_AUTH="user:password"

# Or use external secret management
pmat unified-quality config set \
    --secret github_token \
    --from-vault "secret/pmat/github-token"
```

## Scaling Considerations

### Horizontal Scaling
For large organizations (>1000 developers):

```bash
# Deploy multiple instances with load balancing
pmat unified-quality cluster init \
    --nodes 3 \
    --load-balancer nginx \
    --shared-storage redis

# Partition teams across instances
pmat unified-quality cluster assign-teams \
    --instance-1 "team-a,team-b,team-c" \
    --instance-2 "team-d,team-e,team-f"
```

### Performance Tuning
```toml
[performance]
# For large codebases (>1M LOC)
max_analysis_time_ms = 10000
max_memory_mb = 4096
parallel_analysis = true
worker_threads = 8

# Cache optimization for high-frequency changes
[foundation]
cache_size_mb = 2048
incremental_cache = true
persist_cache = true
```

## Success Metrics

Track these key metrics to measure deployment success:

### Technical Metrics
- **Analysis Latency**: < 5ms per file (95th percentile)
- **System Uptime**: > 99.5%
- **Memory Usage**: < 1GB peak
- **Cache Hit Ratio**: > 85%

### Quality Metrics  
- **Average Complexity**: Trending down over 12 weeks
- **SATD Debt**: < 10 issues per team per sprint
- **Coverage**: Maintaining > 80% overall
- **Quality Violations**: < 5 high-severity per week

### Adoption Metrics
- **Team Onboarding**: 100% teams through Phase 2 within 8 weeks
- **Automation Usage**: > 50% teams using conservative automation
- **Developer Satisfaction**: > 8/10 in quarterly surveys
- **Time to Quality**: < 2 weeks from violation to resolution

## Next Steps

After successful deployment:

1. **Advanced Features**: Enable experimental ML-driven suggestions
2. **Custom Policies**: Develop organization-specific quality rules  
3. **Integration Expansion**: Connect to additional tools (JIRA, Slack)
4. **Research Track**: Evaluate formal verification capabilities
5. **Community Contribution**: Share quality patterns with PMAT community

## Support Resources

- **Documentation**: https://docs.pmat.dev/unified-quality
- **Community**: https://github.com/paiml/paiml-mcp-agent-toolkit/discussions
- **Issue Tracking**: https://github.com/paiml/paiml-mcp-agent-toolkit/issues
- **Professional Support**: enterprise@paiml.com

---

**Remember**: The unified quality system implements a progressive adoption approach. Start with monitoring (Phase 1), add intelligence (Phase 2), enable enforcement (Phase 3), and finally activate automation (Phase 4). Each phase builds foundation for the next while delivering immediate value.

For questions or deployment assistance, reach out to the PMAT community or professional services team.