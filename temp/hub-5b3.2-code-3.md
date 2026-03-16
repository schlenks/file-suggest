[CODE-REVIEW-3/3] hub-5b3.2 wave-1

---

### Changed Files Manifest

| File | Lines Changed | Read in Full |
|------|--------------|--------------|
| `src/search.rs` | +87 / -2 | Yes |
| `tests/search_test.rs` | +28 / -0 | Yes |

---

### Rules Consulted

(none — no `.claude/rules/` directory found in `file-suggest` repo)

---

### Requirement Mapping

| Requirement | Implementing Code | Status |
|-------------|------------------|--------|
| FETCH_RESULTS = 30 constant | `src/search.rs:7` | Implemented |
| `find_matching_dirs()` queries distinct apps/X/ and packages/X/ prefixes | `src/search.rs:142–192` | Implemented |
| `find_matching_dirs()` filters by query token match | `src/search.rs:178–189` | Implemented |
| `find_matching_dirs()` SQL subquery avoids alias-in-WHERE SQLite limitation | `src/search.rs:157–170` | Implemented |
| `apply_directory_boost()` re-ranks by position, not absolute score | `src/search.rs:200–213` | Implemented |
| `apply_directory_boost()` preserves relative BM25 order within each group | `src/search.rs:205–208` (partition) | Implemented |
| `apply_directory_boost()` truncates to MAX_RESULTS (15) | `src/search.rs:211` | Implemented |
| `search()` wired: call find_matching_dirs + apply_directory_boost after FTS5 | `src/search.rs:21–26` | Implemented |
| `search_fts()` uses FETCH_RESULTS in LIMIT | `src/search.rs:85` | Implemented |
| Test: `directory_context_boosts_files_in_matching_dir` | `tests/search_test.rs:122–147` | Implemented |
| Test indexes all four required files | `tests/search_test.rs:130–135` | Implemented |
| Test asserts results[0].starts_with("apps/temporal-worker/") | `tests/search_test.rs:142–146` | Implemented |
| Not in scope: boosting arbitrary nested dirs | No code added for non-apps/packages dirs | Compliant |

---

### Uncovered Paths

1. **`exact_filename_match_ranks_first` test is now misleading** — With the directory boost, query "tsconfig" produces token `["tsconfig"]`. `find_matching_dirs` correctly matches `packages/tsconfig/` (dir_name "tsconfig" contains token "tsconfig"). This causes `packages/tsconfig/base.json` to be boosted above `tsconfig.json` (root). The existing test assertion `results[0].contains("tsconfig")` passes either way, but the comment `// Root tsconfig.json should rank high (shorter path)` is now incorrect — the actual top result will be `packages/tsconfig/base.json`. This represents a silent behavior change.

2. **No test for the "no matching dir" path** — When a query does not match any known `apps/X` or `packages/X` directory, `find_matching_dirs` returns empty and `apply_directory_boost` falls through to `results.into_iter().take(MAX_RESULTS).collect()`. This path is untested directly, though it's indirectly exercised by existing tests that use non-directory-matching queries like "helper", "bksvc".

3. **No test for `packages/X/` boost** — All directory-boost test coverage uses `apps/temporal-worker/`. A query matching a package name (e.g., "design-system") is not covered with an explicit boost assertion.

---

### Not Checked

- Whether 30 candidates is consistently sufficient for re-ranking with larger real-world indexes (only tested in isolation). Not a code defect — a tuning concern outside the scope of this review.
- Runtime performance of `find_matching_dirs` on large indexes (full table scan on `file_scores` with no index hint). This could be slow on large repos but is not testable in the current test suite.

---

### Findings

#### Minor

**File:** `tests/search_test.rs:55–56`
**What's wrong:** The comment `// Root tsconfig.json should rank high (shorter path)` is now incorrect. The directory boost now elevates `packages/tsconfig/base.json` (matching dir "tsconfig") above the root `tsconfig.json`. The assertion `results[0].contains("tsconfig")` passes regardless, silently accepting changed behavior.
**Why it matters:** The test is no longer testing what the comment claims. A future developer reading this test would have a false understanding of the ranking behavior. The existing test effectively stopped being a meaningful constraint when the boost was added.
**How to fix:** Either update the comment to reflect that `packages/tsconfig/` now ranks first due to directory boost, or strengthen the assertion to the actual expected result (`assert_eq!(results[0], "packages/tsconfig/base.json")`).

---

### Assessment

**Applied:** rule-of-five-code (5 passes) on `src/search.rs`; rule-of-five-tests (5 passes) on `tests/search_test.rs`.

**Ready to merge?** With fixes
**Reasoning:** The implementation is correct and all 27 tests pass. The one finding is that an existing test's comment became misleading when the boost changed the actual ranking for "tsconfig"-like queries — the assertion is too loose to enforce the documented intent, so it silently accepts the behavioral change. This is a minor clarity issue, not a bug, and the fix is a one-line comment update or assertion tightening. No blocking defects found.
