---
name: No muda on hook edits
description: Stop iterating on pre-push/pre-commit hooks — if it takes more than 1 edit, it's waste
type: feedback
---

Do not repeatedly edit git hooks. If a hook needs more than one edit, stop — it's muda (waste).
**Why:** User explicitly called out excessive hook editing as muda. Time spent on hooks is time not spent on real code improvements.
**How to apply:** Fix hooks once or skip them. Focus on actual code defects, features, and stack improvements instead.
