[CODE-REVIEW-1/3] hub-5ym.2 wave-2

---

### Changed Files Manifest

| File | Lines Changed | Read in Full |
|------|---------------|--------------|
| `src/search.rs` | +60 / -22 | Yes |
| `tests/search_test.rs` | +17 / -0 | Yes |

---

### Rules Consulted

(none — no `.claude/rules/` directory found in `file-suggest` repo)

---

### Requirement Mapping

| Requirement | Implementing Code | Status |
|-------------|------------------|--------|
| Step 1: Write failing test `exact_filename_beats_stemmer_conflation` | `tests/search_test.rs:173-188` | Implemented |
| Step 2: Verify test fails before implementation | Not verifiable post-commit; IMPL-REPORT attests | Implemented (attested) |
| Step 3: Add `apply_filename_boost` function with specified signature and logic | `src/search.rs:224-241` | Implemented |
| Step 4: Wire `apply_filename_boost` after `apply_directory_boost` in pipeline | `src/search.rs:25` | Implemented |
| Gotcha: Handle queries with and without extensions (`sanitization.ts` exact, `booking.service` prefix) | `src/search.rs:236` — `basename == query_lower \|\| basename.starts_with(&format!("{}.", query_lower))` | Implemented |
| Gotcha: Boost must come AFTER directory boost | `src/search.rs:24-25` — directory boost line 24, filename boost line 25 | Implemented |
| Step 6: Full test suite passes (30 tests) | Verified: `cargo test` exits 0, 30/30 pass | Implemented |
| Step 7: Commit with correct file scope | Commit `0614a88`, only `src/search.rs` and `tests/search_test.rs` modified | Implemented |
| Bonus (not in spec): Extract `extract_top_level_dir`, `dir_matches_tokens` helpers | `src/search.rs:138-160` | Enhancement (not in scope, not a violation) |

---

### Uncovered Paths

1. **Filename boost not applied in trigram/LIKE/fuzzy fallback branches** (`src/search.rs:31-44`): If FTS5 returns empty results for a dot-query, `apply_filename_boost` is never called. The fallback branches return results without filename boost. This is an existing gap, not introduced by this PR, and the spec explicitly scopes the boost to the FTS5 path. Documented as Not Checked since test coverage for this fallback + filename query scenario is absent.

2. **`exact.truncate(MAX_RESULTS)` is redundant but present** (`src/search.rs:239`): The docstring states "The list is already truncated to MAX_RESULTS by `apply_directory_boost`, so no further truncation is needed here." The code then unconditionally truncates anyway. Not a bug (results are already MAX_RESULTS or fewer), but the comment contradicts the code. Minor.

3. **`space_separated_query_finds_results` test** (`tests/search_test.rs:162-171`): Query is `"admin jest.config"`, which contains `.`. `apply_filename_boost` activates with `query_lower = "admin jest.config"`. Partition checks `basename == "admin jest.config"` (impossible) or `basename.starts_with("admin jest.config.")` (impossible). Result: boost is a no-op for this query and the test still passes correctly. Not a bug, but the interaction was worth tracing.

---

### Not Checked

- **Step 2 (failing test pre-implementation)**: Cannot be verified from the post-commit diff. Taken on faith from the IMPL-REPORT.
- **Runtime behavior against actual FTS5 stemmer**: The Porter stemmer conflation behavior is assumed from the spec. No manual verification of whether `sanitization`, `sanitize`, `sanitizer`, `sanitizers` all stem to the same token in the FTS5 engine used.
- **Multi-dot queries** (e.g., `foo.service.ts`): The query contains `.`, guard passes. Partition checks `basename == "foo.service.ts"` or `basename.starts_with("foo.service.ts.")` — the latter would match `foo.service.ts.bak` but not `foo.service.ts` itself (that is caught by exact equality). Behavior is correct for the main case. Not covered by a test.
- **Query IS just an extension** (e.g., `.ts`): Contains `.`, guard passes. All `.ts` files would match `starts_with(".ts.")` — only files literally named `.ts.something`. Exact match `basename == ".ts"` would match a file literally named `.ts`. Unlikely to matter in practice.

---

### Findings

#### Minor

**M1 — Misleading docstring on `apply_filename_boost`**
- **File:line**: `src/search.rs:222-223`
- **What's wrong**: The docstring says "The list is already truncated to MAX_RESULTS by `apply_directory_boost`, so no further truncation is needed here." — but the function body calls `exact.truncate(MAX_RESULTS)` on line 239. The comment and the code contradict each other.
- **Why it matters**: Future maintainers may remove the truncate call (trusting the comment) or add a second one (doubting it), causing subtle correctness issues if the pipeline changes.
- **How to fix**: Either remove the `exact.truncate(MAX_RESULTS)` call (since the list is at most MAX_RESULTS coming in) OR update the comment to say "As a defensive measure, we still truncate in case the pipeline changes." The truncate call is harmless and defensive; the comment should be corrected to match.

---

### Assessment

**Ready to merge?** Yes (with note)

**Reasoning:** All spec requirements are implemented correctly. The new test covers the primary failure case, all 30 tests pass with no regressions, and the dot-in-query guard correctly prevents the boost from conflicting with directory boost for bare directory queries. The one Minor finding (docstring/code contradiction on truncation) is cosmetic and does not affect correctness.
