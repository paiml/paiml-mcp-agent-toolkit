# Sprint 21 Priority Matrix

**Sprint:** Sprint 21 - Scaffolding System Refinements
**Date:** October 6, 2025
**Status:** Planning

## Priority Matrix

### Impact vs Effort

```
High Impact │
           │  PMAT-6010 ★     PMAT-6013 ★
           │  (Parallel)      (MCP)
           │
           │  PMAT-6011 ★     PMAT-6012 ★
           │  (Hook Fix)      (Auto-gen)
Medium     │
Impact     │  PMAT-6015       PMAT-6014
           │  (Diagnostics)   (Smart Cov)
           │
Low Impact │
           │  PMAT-6016
           │  (Trends)
           └─────────────────────────────────
              Low Effort       High Effort
              (1-3 hrs)        (4-6 hrs)
```

**Legend:**
- ★ = Recommended for Sprint 21
- Size indicates priority (P0 largest)

---

## Decision Matrix

### Priority 0 (Must Do)

| Ticket | Impact | Effort | ROI | Reason |
|--------|--------|--------|-----|--------|
| **PMAT-6010** Parallel Checks | High | Medium | ⭐⭐⭐⭐⭐ | 14-70% speed improvement, affects all users |
| **PMAT-6011** Hook Verification | High | Low | ⭐⭐⭐⭐⭐ | Fixes annoying bug, quick win |

**Total P0 Effort:** 4-6 hours
**Total P0 Value:** Critical bug fix + major performance improvement

---

### Priority 1 (Should Do)

| Ticket | Impact | Effort | ROI | Reason |
|--------|--------|--------|-----|--------|
| **PMAT-6012** Auto-gen Tickets | High | Medium | ⭐⭐⭐⭐ | Eliminates manual work, proven pain point |
| **PMAT-6013** MCP Scaffolding | High | High | ⭐⭐⭐⭐ | Enables agent integration, strategic |

**Total P1 Effort:** 7-10 hours
**Total P1 Value:** Automation + agent ecosystem expansion

**P0 + P1 Combined:** 11-16 hours (fits in 2-3 day sprint)

---

### Priority 2 (Nice to Have)

| Ticket | Impact | Effort | ROI | Reason |
|--------|--------|--------|-----|--------|
| **PMAT-6014** Smart Coverage | Medium | High | ⭐⭐⭐ | 10x faster for incremental, but not critical |
| **PMAT-6015** Hook Diagnostics | Medium | Medium | ⭐⭐⭐ | Helps debugging, but not frequent issue |

**Total P2 Effort:** 6-8 hours
**Total P2 Value:** Quality of life improvements

---

### Priority 3 (Future)

| Ticket | Impact | Effort | ROI | Reason |
|--------|--------|--------|-----|--------|
| **PMAT-6016** Health Trends | Low | Medium | ⭐⭐ | Interesting but not actionable yet |

**Total P3 Effort:** 3-4 hours
**Total P3 Value:** Nice to have, defer to Sprint 22+

---

## Recommended Scope

### Option 1: Minimum (P0 Only) - 4-6 hours ⚡

**Tickets:**
- PMAT-6010: Parallel health checks
- PMAT-6011: Fix hook verification

**Pros:**
- Quick wins
- Addresses real bugs
- Noticeable performance improvement
- Low risk

**Cons:**
- Misses strategic MCP integration
- Misses automation opportunity

**Recommendation:** Only if time-constrained

---

### Option 2: Recommended (P0 + P1) - 11-16 hours ⭐ RECOMMENDED

**Tickets:**
- PMAT-6010: Parallel health checks
- PMAT-6011: Fix hook verification
- PMAT-6012: Auto-generate tickets
- PMAT-6013: MCP scaffolding

**Pros:**
- Completes all high-impact items
- Enables agent ecosystem
- Eliminates manual work
- Fits in 2-3 day sprint
- Strategic + tactical wins

**Cons:**
- More testing needed
- MCP integration has learning curve

**Recommendation:** ⭐ **THIS IS THE RECOMMENDED SCOPE**

---

### Option 3: Full (All Tickets) - 20-28 hours

**All 7 tickets**

**Pros:**
- Addresses all findings
- Complete feature set

**Cons:**
- Too ambitious for 2-3 days
- Higher risk of incomplete work
- Some low-ROI items

**Recommendation:** Split P2-P3 into Sprint 22

---

## Value Proposition by Ticket

### PMAT-6010: Parallel Health Checks

**Value:**
```
Current: pmat maintain health --check-build --check-tests
  Build: 10s
  Tests: 60s
  Total: 70s (sequential)

After:
  Build + Tests: 60s (parallel)
  Savings: 10s (14% faster)

With 3 checks:
  Current: 100s (10+60+30 sequential)
  After: 60s (parallel)
  Savings: 40s (40% faster)
```

**ROI:** High - Used multiple times per day by every developer

---

### PMAT-6011: Hook Verification Fix

**Value:**
```
Current:
$ pmat hooks refresh
$ pmat hooks verify
⚠️ Hook content outdated  # Annoying!

After:
$ pmat hooks refresh
$ pmat hooks verify
✅ Hooks verified  # Clean!
```

**ROI:** High - Eliminates false warnings, builds trust in tooling

---

### PMAT-6012: Auto-generate Tickets

**Value:**
```
Current (Sprint 20):
  Manual creation: 5 tickets × 10 min = 50 minutes
  Risk: Inconsistent formatting
  Pain: Tedious copy/paste

After:
  pmat maintain roadmap --generate-tickets
  Time: ~5 seconds
  Savings: 49 minutes, 55 seconds per sprint
  Bonus: Consistent formatting
```

**ROI:** High - Proven pain point from Sprint 20

---

### PMAT-6013: MCP Scaffolding

**Value:**
```
Current:
  Agents: Can't use scaffolding
  Manual: Developer runs CLI commands
  Integration: None

After:
  Agents: Full scaffolding access via MCP
  Automation: AI can create projects
  Integration: Claude Code, Cline, etc.
```

**ROI:** High - Strategic, enables agent ecosystem

---

### PMAT-6014: Smart Coverage (P2)

**Value:**
```
Current:
  Full coverage: 100+ seconds every time

After (small changes):
  Smart coverage: ~10 seconds (10x faster)

After (large changes):
  Still runs full coverage (same as before)
```

**ROI:** Medium - Only helps for small changes, falls back otherwise

---

### PMAT-6015: Hook Diagnostics (P2)

**Value:**
```
Current (when hooks fail):
  Generic error message
  Manual debugging needed

After:
  Comprehensive diagnostics
  Actionable suggestions
  Quick resolution
```

**ROI:** Medium - Helps when issues occur, but rare

---

### PMAT-6016: Health Trends (P3)

**Value:**
```
Current:
  Snapshot: Current health only

After:
  Trends: Historical data
  Velocity: Track completion rate
  Insights: Identify patterns
```

**ROI:** Low - Interesting but not immediately actionable

---

## Timeline Estimates

### P0 + P1 (Recommended Scope)

**Day 1 (6-8 hours):**
- Morning: PMAT-6011 (Hook fix) - 1-2h
- Mid-day: PMAT-6010 (Parallel) - 3-4h
- Afternoon: Testing & documentation - 2h

**Day 2 (5-8 hours):**
- Morning: PMAT-6012 (Auto-gen tickets) - 3-4h
- Afternoon: Start PMAT-6013 (MCP) - 2-3h

**Day 3 (4-6 hours):**
- Morning: Complete PMAT-6013 (MCP) - 2-3h
- Afternoon: Integration testing, dogfooding - 2-3h
- Evening: Documentation, release prep - 1h

**Total:** 15-22 hours over 2-3 days ✅ Realistic

---

## Risk Assessment

### Low Risk (Quick Wins)
- ✅ PMAT-6011: Hook fix (simple logic change)
- ✅ PMAT-6012: Auto-gen tickets (template generation)

### Medium Risk (New Patterns)
- ⚠️ PMAT-6010: Parallel execution (race conditions possible)
- ⚠️ PMAT-6015: Diagnostics (comprehensive testing needed)

### Higher Risk (Complex Integration)
- ⚠️ PMAT-6013: MCP scaffolding (protocol integration)
- ⚠️ PMAT-6014: Smart coverage (git integration, edge cases)

**Mitigation:**
- Start with low-risk items
- Extensive testing for parallel execution
- MCP integration follows existing patterns
- Fall back to sequential if parallel fails

---

## Success Metrics

### Performance
- [ ] Parallel health checks: 14-40% faster
- [ ] Hook verification: Zero false positives
- [ ] Ticket generation: <5 seconds for 10 tickets

### Adoption
- [ ] 5 MCP tools exposed and documented
- [ ] Integration tests for all new features
- [ ] Dogfooding validation successful

### Quality
- [ ] All existing tests still passing
- [ ] New features CC <8
- [ ] Coverage >80%
- [ ] Zero regressions

---

## Recommendation

**Recommended Scope: P0 + P1 (4 tickets, 11-16 hours)**

**Rationale:**
1. **Addresses all critical issues** from dogfooding
2. **Fits in 2-3 day sprint** comfortably
3. **High impact, proven value** - all tickets solve real problems
4. **Strategic wins** - MCP integration enables future growth
5. **Quick wins** - Hook fix and parallel checks show immediate results

**Defer to Sprint 22:**
- PMAT-6014: Smart coverage (P2)
- PMAT-6015: Hook diagnostics (P2)
- PMAT-6016: Health trends (P3)

---

**Status:** Ready for team review
**Next Step:** Approve scope and begin Sprint 21 execution
**Target:** v2.140.0 release
