[CODE-REVIEW-1/1] hub-5b3.1 wave-1

## Changed Files Manifest

| File | Lines Changed | Read in Full |
|------|--------------|--------------|
| src/scoring.rs | +80 insertions | Yes |
| tests/scoring_test.rs | +46 insertions | Yes |

## Rules Consulted

(none — no `.claude/rules/` directory found in file-suggest repo)

## Requirement Mapping

| Requirement | Implementing Code | Status |
|-------------|------------------|--------|
| Add dockerfile penalty | is_dockerfile (scoring.rs:54-57), type_penalty tier 0.4 (scoring.rs:108-110), test (scoring_test.rs:41-44) | Implemented |
| Add IDE config high penalty | is_ide_config (scoring.rs:42-44), type_penalty tier 0.9 (scoring.rs:90-92), test (scoring_test.rs:47-52) | Implemented |
| Add lockfile high penalty | is_lockfile (scoring.rs:47-51), type_penalty tier 0.9 (scoring.rs:93-95), test (scoring_test.rs:55-60) | Implemented |
| Add build output max penalty | is_build_output (scoring.rs:37-39), type_penalty tier 1.0 (scoring.rs:82-84), test (scoring_test.rs:63-66) | Implemented |
| Add dot config small penalty | is_dot_config (scoring.rs:60-64), type_penalty tier 0.15 (scoring.rs:131-133), test (scoring_test.rs:69-72) | Implemented |
| Add migration penalty | is_migration (scoring.rs:67-69), type_penalty tier 0.2 (scoring.rs:121-123), test (scoring_test.rs:75-78) | Implemented |
| Add type declaration penalty | is_type_declaration (scoring.rs:72-74), type_penalty tier 0.2 (scoring.rs:118-120), test (scoring_test.rs:81-84) | Implemented |
| Reorganize type_penalty() into 10 tiers | scoring.rs:78-141 | Implemented |
| Tests for all new categories written first | Tests present in scoring_test.rs | Implemented |

## Uncovered Paths

1. Root-level migrations directory not matched: `is_migration` uses `lower.contains("/migrations/")` requiring a leading slash. A file at `migrations/001_init.sql` (root-level, no parent directory) will return false and receive 0.0 penalty instead of 0.2. Only paths like `apps/db/migrations/foo.sql` or the test's `migrations/migrations/foo.js` (which has an inner `/migrations/`) are matched.

2. `ide_config_has_high_penalty` test compares IDE penalty against `src/test.ts` which has 0.0 penalty (not a test file by the predicate's definition — `is_test` requires `.test.ts`). The test passes but for the wrong reason: it verifies `0.9 > 0.0` rather than `0.9 > 0.5` (test file). The intent was to show IDE config outranks a test file, but the file used is not actually a test file.

3. New tests for dot_config, migration, and type_declaration only assert `> 0.0`, not tier ordering. No test verifies these are ranked below test files (0.5), lockfiles (0.9), or relative to each other.

## Not Checked

- Whether `cargo test --test scoring_test` actually passes in the current state (tests were reported passing by the implementer; I cannot run the test suite).
- Whether other callers of `type_penalty()` beyond the test file exist and could be affected by tier reordering.

## Findings

### Important

**Finding 1: Root-level migration directory not matched**
- File: `src/scoring.rs:68`
- What's wrong: `is_migration` checks `lower.contains("/migrations/")` with a required leading slash. A file at the repository root like `migrations/0001_init.sql` has no leading slash before `migrations` and will not match, returning 0.0 instead of 0.2.
- Why it matters: This is the standard layout for Rust projects (e.g., `sqlx` migrations) and many other languages. The penalization goal is defeated for the most common case.
- How to fix:
  ```rust
  fn is_migration(lower: &str) -> bool {
      lower.starts_with("migrations/") || lower.contains("/migrations/")
  }
  ```

### Minor

**Finding 2: Dead/unreachable branch in `is_ide_config`**
- File: `src/scoring.rs:43`
- What's wrong: The third condition `lower.ends_with("/.vscode/settings.json")` is unreachable. `ends_with` checks the string's suffix; a path ending in `/.vscode/settings.json` would have to end with that 24-character suffix, but it would also contain `.vscode/` and be caught by `starts_with(".vscode/")` first. The `ends_with("/.vscode/settings.json")` condition can never trigger.
- Why it matters: Dead code; adds cognitive overhead and misleads readers about what the function actually covers.
- How to fix: Remove the third condition. The function is already correct with just the first two conditions.

**Finding 3: Semantically misleading test assertion in `ide_config_has_high_penalty`**
- File: `tests/scoring_test.rs:50-51`
- What's wrong: The test uses `src/test.ts` as the comparison baseline (named `test`), implying it's a test file. But `is_test` requires `.test.ts` extension — `src/test.ts` gets 0.0 penalty. The test effectively asserts `0.9 > 0.0` rather than verifying IDE config ranks above actual test files (0.5).
- Why it matters: The test passes but does not verify the intended ordering property. If IDE config were accidentally set to 0.3 (below test's 0.5), this test would still pass.
- How to fix:
  ```rust
  fn ide_config_has_high_penalty() {
      let vscode = file_suggest::scoring::type_penalty(".vscode/settings.json");
      let test = file_suggest::scoring::type_penalty("src/Button.test.ts");
      assert!(vscode > test); // 0.9 > 0.5
  }
  ```
  Same fix applies to `lockfile_has_high_penalty` test.

**Finding 4: Weak ordering assertions for new tier tests**
- File: `tests/scoring_test.rs:69-84`
- What's wrong: Tests for `dot_config`, `migration`, and `type_declaration` only assert `> 0.0`. They do not verify placement within the penalty tier hierarchy (e.g., that type_declaration at 0.2 ranks below test at 0.5, or that dot_config at 0.15 ranks below type_declaration at 0.2).
- Why it matters: A tier value error (e.g., accidentally 0.6 for dot_config, outranking test files) would not be caught.

## Assessment

**Ready to merge?** With fixes
**Reasoning:** The root-level migration path gap (Finding 1) is a functional correctness issue where a common file layout fails to receive the intended penalty. The dead branch in `is_ide_config` and the weak/misleading test assertions are minor but should be addressed before merge.
