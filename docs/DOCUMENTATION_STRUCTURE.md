# PMAT Documentation Structure

*Consolidated documentation structure aligned with SPECIFICATION.md*  
*Last Updated: 2025-08-22*

## Source of Truth

**Primary**: `/docs/SPECIFICATION.md` - Complete system specification v2.8.0

## Documentation Hierarchy

### 1. Core Documentation (Keep & Update)

#### System Documentation
- `docs/SPECIFICATION.md` ✅ - Complete system specification (SOURCE OF TRUTH)
- `docs/architecture/ARCHITECTURE.md` - High-level architecture overview
- `README.md` - Project entry point
- `CONTRIBUTING.md` - Contribution guidelines
- `CHANGELOG.md` - Release history

#### Execution & Planning
- `docs/execution/roadmap.md` ✅ - Sprint planning aligned with spec
- `docs/execution/quality-gates.md` ✅ - Quality enforcement documentation
- `docs/execution/velocity.json` ✅ - Sprint metrics tracking

#### API & Integration
- `docs/cli-reference.md` - CLI command reference (align with Section 23)
- `docs/api-guide.md` - HTTP API guide (align with Section 22)
- `docs/mcp-methods.md` - MCP tool documentation (align with Section 4)

#### Features (Active)
- `docs/features/refactor-engine.md` - Refactoring system (Section 16-18)
- `docs/features/satd-detection.md` - SATD detection (Section 8)
- `docs/features/deep-context-analysis.md` - Context generation (Section 10)
- `docs/features/provability-analysis.md` - Provability system (Section 15)

### 2. Archive (Move to archive/)

#### Outdated Release Notes
- `docs/release_notes/RELEASE_NOTES_v0.*.md` → `docs/archive/release_notes/`
- Keep only latest major version notes in main

#### Old Specifications
- `docs/archive/SPEC.md` - Superseded by SPECIFICATION.md
- `docs/todo/archive/*.md` - Completed specifications

#### Historical Documentation
- `docs/archive/2025/01/kaizen/` - Kaizen reports (historical value)
- `docs/archive/achievement-reports/` - Past achievements

### 3. Consolidate & Merge

#### Bugs → Issues
- Merge `docs/bugs/` into GitHub Issues with label 'bug'
- Archive resolved bugs to `docs/archive/bugs/`

#### Todo → Roadmap
- Active todos from `docs/todo/` → `docs/execution/roadmap.md`
- Completed todos → `docs/archive/todo/`

#### Features → Specification
- Feature descriptions in `docs/features/` should reference relevant SPECIFICATION.md sections
- Remove duplicate content, keep only implementation details

### 4. Delete (Redundant/Obsolete)

#### Duplicate Content
- Files that duplicate SPECIFICATION.md content without adding value
- Old implementation summaries superseded by current code

#### Development Artifacts
- `docs/7-2-2025-bugs.md` - Specific date bugs (move to issues)
- Temporary analysis files (`*_report.json`)

### 5. New Structure (To Create)

#### Missing from Specification
- `docs/testing/property-based.md` - Property testing guide (Section 28)
- `docs/testing/integration.md` - Integration testing guide (Section 29)
- `docs/testing/performance.md` - Performance testing guide (Section 30)
- `docs/quality/standards.md` - Quality standards (Sections 31-33)
- `docs/operations/error-handling.md` - Error handling guide (Section 34)
- `docs/operations/telemetry.md` - Logging & telemetry (Section 35)
- `docs/operations/configuration.md` - Configuration guide (Section 36)

## Documentation Rules

### 1. Single Source of Truth
- SPECIFICATION.md is the canonical reference
- All other docs must align with or reference the spec
- No duplicate definitions or conflicting information

### 2. Cross-References
- Use section references: "See SPECIFICATION.md Section X"
- Link between related documents
- Maintain consistency in terminology

### 3. Versioning
- Documentation version matches software version
- Update docs with code changes (enforced by pre-commit)
- Archive old versions, don't delete

### 4. Quality Standards
- Clear, concise technical writing
- Code examples must compile/run
- Diagrams use Mermaid format
- All commands tested and verified

## Migration Plan

### Phase 1: Archive (Immediate)
1. Create archive directories
2. Move outdated documentation
3. Update references

### Phase 2: Consolidate (Day 1)
1. Merge duplicate content
2. Update cross-references
3. Align with SPECIFICATION.md

### Phase 3: Create (Day 2)
1. Write missing documentation
2. Update existing docs to reference spec
3. Verify all links and references

### Phase 4: Validate
1. Check all cross-references
2. Ensure no broken links
3. Verify alignment with SPECIFICATION.md
4. Run documentation tests

## Maintenance

### Pre-Commit Checks
- Documentation updated with code changes
- No broken links
- Consistent formatting
- Version alignment

### Regular Reviews
- Monthly documentation audit
- Quarterly archive cleanup
- Annual structure review

## Tools

### Documentation Generation
```bash
# Generate API docs from code
cargo doc --no-deps

# Check for broken links
linkchecker docs/

# Format markdown
prettier --write docs/**/*.md
```

### Quality Checks
```bash
# Check documentation coverage
pmat analyze docs --coverage

# Verify examples compile
pmat verify-examples docs/

# Check cross-references
pmat check-refs docs/
```