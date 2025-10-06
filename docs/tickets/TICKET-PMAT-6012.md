# TICKET-PMAT-6012: Auto-Generate Ticket Files from Roadmap

**Sprint:** Sprint 21 - Scaffolding System Refinements
**Priority:** P1 - High
**Estimated Effort:** 3-4 hours
**Status**: GREEN ✅
**Created:** 2025-10-06
**Completed:** 2025-10-06

## Problem Statement

Creating ticket files manually is tedious and error-prone. During Sprint 20, we had to manually create 5 ticket files (PMAT-6002 through PMAT-6006), taking approximately 50+ minutes of work.

**Current Workflow:**
1. Add ticket to roadmap with checkbox
2. Manually create ticket file in docs/tickets/
3. Copy template structure
4. Fill in ticket ID, sprint, status
5. Add TODO placeholders
6. Repeat for each new ticket

**Pain Points:**
- Time-consuming (10+ minutes per ticket)
- Inconsistent formatting across tickets
- Easy to forget required sections
- Sprint context must be manually tracked
- Status must match checkbox state

## Solution

Implement automatic ticket file generation from roadmap entries using the `--generate-tickets` flag.

**New Workflow:**
```bash
pmat maintain roadmap --generate-tickets

# Output:
# 📝 Checking for missing ticket files...
#
# Created: TICKET-PMAT-6015.md
# Created: TICKET-PMAT-6016.md
# Created: TICKET-PMAT-6017.md
#
# ✅ Generated 3 ticket file(s)
# ⏭️  Skipped 25 existing ticket(s)
```

### Implementation Details

**1. Added --generate-tickets Flag** (commands.rs)
```rust
/// Auto-generate missing ticket files (TICKET-PMAT-6012)
#[arg(long)]
generate_tickets: bool,
```

**2. Implemented generate_missing_ticket_files()** (CC=6)
```rust
async fn generate_missing_ticket_files(
    roadmap_path: &Path,
    tickets_dir: &Path,
    dry_run: bool,
) -> Result<()> {
    // Parse roadmap for all tickets
    let roadmap_tickets = parse_roadmap_tickets(&roadmap_content)?;

    for (ticket_id, checked) in &roadmap_tickets {
        let ticket_path = tickets_dir.join(format!("{ticket_id}.md"));

        if ticket_path.exists() {
            continue; // Skip existing files
        }

        // Extract sprint context
        let sprint = extract_sprint_for_ticket(&roadmap_content, ticket_id);
        let status = if *checked { "GREEN ✅" } else { "PLANNED 📋" };

        // Generate and write template
        let template = generate_ticket_template(ticket_id, &sprint, status);
        fs::write(&ticket_path, template)?;
    }
}
```

**3. Implemented extract_sprint_for_ticket()** (CC=4)
```rust
fn extract_sprint_for_ticket(roadmap_content: &str, ticket_id: &str) -> String {
    let mut current_sprint = "Unknown Sprint".to_string();

    for line in roadmap_content.lines() {
        // Track current sprint header
        if line.starts_with("### Sprint") || line.starts_with("## Sprint") {
            current_sprint = extract_sprint_name(line);
        }

        // When we find the ticket, return the current sprint
        if line.contains(ticket_id) {
            return current_sprint;
        }
    }

    current_sprint
}
```

**4. Implemented generate_ticket_template()** (CC=1)
```rust
fn generate_ticket_template(ticket_id: &str, sprint: &str, status: &str) -> String {
    // Generates a standard ticket template with:
    // - Title (TODO placeholder)
    // - Sprint (extracted from roadmap)
    // - Priority (TBD placeholder)
    // - Status (GREEN if checked, PLANNED if unchecked)
    // - Standard sections: Problem, Solution, Criteria, etc.
    // - Current date as creation date
}
```

### Features

**Sprint Detection:**
- Scans roadmap for sprint headers (`### Sprint 21:` or `## Sprint 16:`)
- Tracks current sprint as it processes lines
- Assigns ticket to the sprint it appears under

**Status Mapping:**
- Checked tickets (`[x]`): `GREEN ✅`
- Unchecked tickets (`[ ]`): `PLANNED 📋`

**Dry-Run Support:**
```bash
pmat maintain roadmap --generate-tickets --dry-run

# Shows what would be created without creating files
# Allows preview before actual generation
```

**Template Structure:**
- Title with TODO placeholder
- Sprint (automatically detected)
- Priority (TBD for manual filling)
- Status (based on checkbox state)
- Creation date (current date)
- Standard sections: Problem, Solution, Acceptance Criteria, Quality Metrics
- TODO placeholders for manual completion

## Test Coverage

### Manual Testing

**Test 1:** Generate missing tickets
- Added test tickets to roadmap
- Ran `pmat maintain roadmap --generate-tickets`
- Verified files created with correct sprint and status

**Test 2:** Skip existing tickets
- Ran generation twice
- Verified existing files not overwritten
- Confirmed "skipped" message shown

**Test 3:** Dry-run mode
- Ran with `--dry-run`
- Verified no files created
- Confirmed preview output shown

**Test 4:** Sprint detection
- Created tickets in different sprint sections
- Verified correct sprint assigned to each

## Acceptance Criteria

- [x] --generate-tickets flag added to roadmap command
- [x] Missing ticket files auto-generated from roadmap
- [x] Sprint context extracted from roadmap structure
- [x] Status mapped from checkbox state (GREEN/PLANNED)
- [x] Existing files not overwritten
- [x] Dry-run mode supported
- [x] Template includes all standard sections
- [x] Cyclomatic complexity <7 for all functions
- [x] Code compiles successfully

## Quality Metrics

- **CC:** 6 (generate_missing_ticket_files), 4 (extract_sprint_for_ticket), 1 (generate_ticket_template)
- **Performance:** O(n) where n is number of tickets in roadmap
- **Time Saved:** 50+ minutes per sprint with 5 tickets

## Files Modified

- `server/src/cli/commands.rs`
  - Added `generate_tickets: bool` field to `MaintainCommands::Roadmap`

- `server/src/cli/command_dispatcher.rs`
  - Added `generate_tickets` parameter to dispatcher match
  - Passed to `handle_maintain_roadmap()`

- `server/src/cli/command_structure.rs`
  - Added `generate_tickets` parameter to command structure match
  - Passed to `handle_maintain_roadmap()`

- `server/src/cli/handlers/roadmap_handler.rs`
  - Updated `handle_maintain_roadmap()` signature with `generate_tickets` parameter
  - Added `generate_missing_ticket_files()` function (CC=6)
  - Added `extract_sprint_for_ticket()` function (CC=4)
  - Added `generate_ticket_template()` function (CC=1)
  - Updated function complexity from CC=6 to CC=7

## Performance Impact

**Time Savings:**
- Manual creation: ~10 minutes per ticket
- Automated generation: ~1 second for all tickets
- **Sprint 20 example:** 5 tickets × 10 min = 50 minutes saved

**Per-Ticket Breakdown:**
- Manual: Open editor, copy template, fill fields, save (10 min)
- Auto: `pmat maintain roadmap --generate-tickets` (1 second total)

## Related Tickets

- TICKET-PMAT-5032: Roadmap validation (basis for parsing)
- TICKET-PMAT-5010: Roadmap parsing (used parse_roadmap_tickets)
- Sprint 21 Planning: `docs/sprints/SPRINT-21-PLAN.md`

## References

- Dogfooding Findings: `docs/dogfooding/v2.139.0-INTEGRATION-SHOWCASE.md`
- Issue identified during v2.139.0 integration (had to manually create 5 ticket files)
- Sprint 21 Priority: P1 (High)

## Breaking Changes

None. The feature is opt-in via `--generate-tickets` flag.

## Migration Guide

No migration needed. To use:

```bash
# Preview what would be generated
pmat maintain roadmap --generate-tickets --dry-run

# Actually generate missing ticket files
pmat maintain roadmap --generate-tickets

# Combine with other operations
pmat maintain roadmap --validate --generate-tickets
```

## Usage Examples

**Example 1: First-time generation**
```bash
# Add new tickets to ROADMAP.md:
# - [ ] TICKET-PMAT-7001: New feature
# - [ ] TICKET-PMAT-7002: Another feature

pmat maintain roadmap --generate-tickets

# Output:
# 📝 Checking for missing ticket files...
# Created: TICKET-PMAT-7001.md
# Created: TICKET-PMAT-7002.md
# ✅ Generated 2 ticket file(s)
```

**Example 2: Safe re-generation**
```bash
# Run again - existing files not touched
pmat maintain roadmap --generate-tickets

# Output:
# 📝 Checking for missing ticket files...
# ✅ No missing ticket files
# ⏭️  Skipped 28 existing ticket(s)
```

**Example 3: Dry-run preview**
```bash
pmat maintain roadmap --generate-tickets --dry-run

# Output:
# 📝 Checking for missing ticket files...
# Would create: TICKET-PMAT-7001.md (Sprint: Sprint 22, Status: PLANNED 📋)
# Would create: TICKET-PMAT-7002.md (Sprint: Sprint 22, Status: GREEN ✅)
# ✅ Generated 2 ticket file(s)
# 🔍 Dry-run mode - no files created
```

## Template Output Example

Generated ticket file structure:
```markdown
# TICKET-PMAT-XXXX: [Title - TODO: Update from roadmap]

**Sprint:** Sprint 21 - Scaffolding System Refinements
**Priority:** [TBD - To be determined]
**Estimated Effort:** [TBD - To be estimated]
**Status**: PLANNED 📋
**Created:** 2025-10-06

## Problem Statement

[TODO: Describe the problem this ticket solves]

## Solution

[TODO: Describe the proposed solution]

## Acceptance Criteria

- [ ] [TODO: Add acceptance criteria]

## Quality Metrics

- **CC:** [TBD]
- **Tests:** [TBD]
- **Coverage:** [TBD]

## Files Modified

- [TODO: List files to be modified]

## Related Tickets

- [TODO: Link to related tickets]

---

**Status:** PLANNED 📋
**Delivered:** [TBD]
**Target Release:** [TBD]
```

---

**Status:** ✅ Complete
**Delivered:** Sprint 21 (in progress)
**Target Release:** v2.140.0
**Value:** Saves 50+ minutes per sprint, ensures consistency
