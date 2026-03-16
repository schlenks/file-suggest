---
name: file-suggest v2 epic verification
description: Verification findings for file-suggest v2 (hub-0w0) — Rust CLI for Claude Code file suggestions
type: project
---

Epic hub-0w0 verified 2026-03-15.

Key finding: fuzzy fallback (Task 6) was implemented as a module (`src/fuzzy.rs`) but never wired into the search pipeline (`src/search.rs`). The `search()` function chain ends at `search_like_fallback` with no call to `fuzzy::fuzzy_search`. This means abbreviation-style queries like "bksvc" only work if tested directly, not via the actual search entry point. The search_test covers fuzzy via direct call, masking this gap.

**Why:** The implementation diverged from the plan which specified adding fuzzy as the last-resort fallback after LIKE returns empty. The `load_all_paths` helper was also never implemented.

**How to apply:** If this epic is reopened, the fuzzy fallback needs to be wired into `search::search()` after `search_like_fallback`. This would also require implementing `load_all_paths`.

Also: `src/incremental.rs` was created as a separate module, whereas the plan specified adding `incremental_build` to `src/index.rs`. This is a reasonable structural deviation.

Top-3 accuracy: 86.7% vs 87% target (0.3% miss). User was aware and accepted this.
