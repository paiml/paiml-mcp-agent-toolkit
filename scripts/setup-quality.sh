#!/bin/bash
# One-time setup for permanent PMAT quality enforcement
# Adapted from ruchy's setup-quality.sh

echo "🔧 Setting up PMAT quality enforcement (Toyota Way integration)..."

# Install pre-commit hook
echo "📝 Installing pre-commit hook..."
cp scripts/pre-commit .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
echo "✓ Pre-commit hook installed"

# Install PMAT if not already built
if [ ! -f "./target/debug/pmat" ]; then
    echo "🔨 Building PMAT binary for quality gates..."
    make build || echo "Note: Build failed, some quality checks will be skipped"
else
    echo "✓ PMAT binary found"
fi

# Create required documentation structure
echo "📂 Creating documentation structure..."
mkdir -p docs/execution docs/quality docs/architecture/decisions
echo "✓ Documentation directories created"

# Initialize roadmap template if not exists (should already exist from implementation)
if [ ! -s docs/execution/roadmap.md ]; then
    echo "📋 Initializing roadmap template..."
    cat > docs/execution/roadmap.md << 'EOF'
# PMAT Development Roadmap

## Current Sprint: Setup & Quality Gates ✅ IN PROGRESS

### In Progress 🚧
- [ ] PMAT-0001: Quality enforcement setup
- [ ] PMAT-0002: Documentation synchronization

### Completed ✅
- [x] PMAT-0000: Project quality gates setup

## Task Log
| ID | Description | Status | Complexity | Owner |
|----|-------------|--------|------------|-------|
| PMAT-0001 | Quality enforcement setup | 🚧 | Medium | - |
| PMAT-0002 | Documentation sync system | 🚧 | Medium | - |
EOF
    echo "✓ Roadmap template created"
else
    echo "✓ Roadmap already exists"
fi

# Initialize quality-gates.md if not exists (should already exist)
if [ ! -s docs/execution/quality-gates.md ]; then
    echo "📊 Initializing quality gates template..."
    cat > docs/execution/quality-gates.md << 'EOF'
# PMAT Quality Gates

## Enforcement Status
- Pre-commit hooks: ✅ Active
- CI/CD pipeline: ✅ Active  
- PMAT integration: ✅ Active
- Documentation sync: ✅ Required

## Quality Metrics (Toyota Way Standards)
- Cyclomatic complexity: ≤20
- Cognitive complexity: ≤15
- Test coverage: >80%
- SATD comments: 0
- Lint warnings: 0

## Last Sprint Report
Generated: $(date)
All quality gates configured and active
EOF
    echo "✓ Quality gates template created"
else
    echo "✓ Quality gates documentation already exists"
fi

# Initialize velocity.json if not exists (should already exist)
if [ ! -s docs/execution/velocity.json ]; then
    echo "📈 Initializing velocity tracking..."
    cat > docs/execution/velocity.json << 'EOF'
{
  "metadata": {
    "version": "1.0",
    "project": "PMAT (PAIML MCP Agent Toolkit)",
    "last_updated": "$(date -Iseconds)",
    "measurement_period": "daily"
  },
  "quality_metrics": {
    "technical_debt": {
      "satd_comments": 0,
      "target": 0,
      "clippy_warnings": 0,
      "status": "quality_gates_configured"
    }
  },
  "sprint_management": {
    "current_sprint": "setup",
    "documentation_sync": "active"
  }
}
EOF
    echo "✓ Velocity tracking initialized"
else
    echo "✓ Velocity tracking already exists"
fi

# Configure git
echo "🔧 Configuring git hooks..."
git config core.hooksPath .git/hooks
echo "✓ Git hooks configured"

# Create pmat.toml if it doesn't exist (should already exist)
if [ ! -f "pmat.toml" ]; then
    echo "⚙️  Creating pmat.toml configuration..."
    cat > pmat.toml << 'EOF'
[quality_gate]
max_cyclomatic_complexity = 30
max_cognitive_complexity = 25
max_satd_comments = 0
min_test_coverage = 80.0

[documentation]
required_updates = [
    "docs/execution/roadmap.md",
    "docs/execution/quality-gates.md", 
    "CHANGELOG.md"
]
task_id_pattern = "PMAT-[0-9]{4}"

[toyota_way]
enable_mcp_first_dogfooding = true
require_genchi_genbutsu_analysis = true
enforce_jidoka_automation = true
EOF
    echo "✓ pmat.toml configuration created"
else
    echo "✓ pmat.toml already exists"
fi

# Verify setup
echo ""
echo "🔍 Verifying setup..."

# Check if pre-commit hook is executable
if [ -x ".git/hooks/pre-commit" ]; then
    echo "✓ Pre-commit hook is executable"
else
    echo "❌ Pre-commit hook is not executable"
fi

# Check documentation structure
if [ -d "docs/execution" ] && [ -f "docs/execution/roadmap.md" ] && [ -f "docs/execution/quality-gates.md" ]; then
    echo "✓ Documentation structure is complete"
else
    echo "❌ Documentation structure is incomplete"
fi

# Check git configuration
if git config core.hooksPath | grep -q ".git/hooks"; then
    echo "✓ Git hooks are configured"
else
    echo "❌ Git hooks are not properly configured"
fi

echo ""
echo "✅ PMAT Quality enforcement configured!"
echo ""
echo "🎯 Toyota Way Quality Gates Active:"
echo "  📝 Documentation synchronization enforced"
echo "  🔧 Pre-commit quality checks enabled"  
echo "  📊 Quality metrics tracking configured"
echo "  🚀 MCP-first dogfooding approach active"
echo ""
echo "Usage:"
echo "  make dev           - Start development with quality checks"
echo "  make commit        - Create quality-enforced commit"
echo "  make sprint-close  - Verify sprint quality"
echo "  pmat quality-gate  - Run comprehensive quality analysis"
echo ""
echo "📋 Next Steps:"
echo "  1. Update docs/execution/roadmap.md with your current tasks"
echo "  2. Make your first commit - documentation synchronization will be enforced"
echo "  3. Use 'make validate' to verify all quality gates pass"
echo ""
echo "🎯 Remember: Documentation MUST be updated with every code change (Toyota Way)"