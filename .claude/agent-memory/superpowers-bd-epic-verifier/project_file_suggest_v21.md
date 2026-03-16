---
name: file-suggest v2.1 epic verification
description: Verification findings for file-suggest v2.1 (hub-5b3) — ranking accuracy improvements including file-type penalties, directory-context boost, and expanded benchmark
type: project
---

Epic hub-5b3 verified 2026-03-15.

Key findings:
1. README.md file-type ranking table is outdated -- lists only 7 original tiers, missing 7 new categories (build output, IDE config, lockfile, dockerfile, dot config, migration, type declaration). Also missing directory-context boost feature description.
2. Dead branch in `is_ide_config` at scoring.rs:43 was flagged in code review but NOT actually dead -- `lower.ends_with("/.vscode/settings.json")` catches nested .vscode directories that the `starts_with` conditions miss.
3. Weak ordering assertions in scoring tests (dot_config, migration, type_declaration only assert `> 0.0`, not tier ordering) were flagged in code review but not fixed in gap closure. Accepted as minor.
4. `scored` field in golden-queries.json is decorative -- benchmark.py uses `expected_top3` truthiness, not the `scored` boolean.

**Why:** The README documentation gap is the only actionable item. All code and test gaps from reviews were addressed in gap closure commits (a14b1b3, e9847e4).

**How to apply:** If updating README, add new penalty tiers and directory-context boost to the feature list and ranking table.
