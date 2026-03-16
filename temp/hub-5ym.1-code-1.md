[CODE-REVIEW-1/3] hub-5ym.1 wave-1

### Changed Files Manifest

| File | Lines Changed | Read in Full |
|------|--------------|--------------|
| `src/search.rs` | +2 / -2 | Yes |
| `tests/search_test.rs` | +12 / -0 | Yes |

### Rules Consulted

(none — no repo rules found at `.claude/rules/`)

### Requirement Mapping

| Requirement | Implementing Code | Status |
|-------------|------------------|--------|
| Write failing test `space_separated_query_finds_results` | `tests/search_test.rs:162-171` | Implemented |
| Add whitespace to `build_fts_query` split predicate | `src/search.rs:209` | Implemented |
| Add whitespace to `find_matching_dirs` split predicate | `src/search.rs:145` | Implemented |
| Test verifies non-empty results and correct first result | `tests/search_test.rs:169-170` | Implemented |
| Full test suite passes (29 tests) | Confirmed via `cargo test` — 29/29 pass | Implemented |

### Uncovered Paths

1. **Query containing only whitespace** — `build_fts_query("   ")` will produce an empty `tokens` vec and return `""`, which is handled by the `if fts_query.is_empty()` guard (line 73-75). This path is correct but untested. Not a finding since `search_empty` would likely be called first anyway if the input is just spaces — actually it would NOT because `query.is_empty()` check on line 13 only matches truly empty string, not whitespace-only. A whitespace-only query routes to `search_fts`, `build_fts_query` returns `""`, the guard returns `Ok(vec![])`, then trigram (`query.len() < 3` may pass), LIKE fallback `%   %`, and finally fuzzy. The empty-guard in `build_fts_query` prevents a malformed MATCH but users would get poor results. Pre-existing behavior, not introduced by this diff.

2. **Tab/newline characters in query** — `is_whitespace()` matches `\t`, `\n`, `\r`, etc. A query like `"admin\tjest.config"` would tokenize identically to `"admin jest.config"`. No test covers non-space whitespace characters. This is a correct extension of behavior (no harm), but is untested.

### Not Checked

- Whether the FTS5 tokenizer used at index time (`tokenize_path` splits on `/`, `.`, `-`, `_` but NOT whitespace) creates any impedance mismatch with query tokens. Confirmed: real file paths in the index don't contain spaces, so query-time whitespace splitting adds no new token types — matching is sound.
- Integration behavior on a real filesystem index with space-containing queries (only tested via unit test with synthetic index).

### Findings

No Critical or Important findings.

**Minor (1):**

- **File:** `tests/search_test.rs:162-171`
- **What's wrong:** The new test does not assert `results.len() >= 2` before accessing `results[0]`. The `assert!(!results.is_empty())` guard on line 169 ensures `results[0]` is safe, but the test then asserts that `results[0]` is specifically `apps/admin/jest.config.ts` without verifying the result set has at least 2 entries (for confidence the directory boost is discriminating correctly). Compare with `source_file_ranks_above_test_file` which does `assert!(results.len() >= 2)` before asserting rank order.
- **Why it matters:** If the search returns only one result (the admin file is the only one indexed), the test passes trivially without proving the boost correctly ranked `admin` above `partners` and `marketplace`. However, since all three files are indexed at identical frecency (0.0) and only one has `apps/admin/`, the FTS query `"admin"* AND "jest"* AND "config"*` matches all three; the directory boost should elevate `apps/admin/jest.config.ts`. Adding `assert!(results.len() >= 2)` would confirm disambiguation is actually tested.
- **How to fix:**
  ```rust
  let results = search::search("admin jest.config", &db_path).unwrap();
  assert!(results.len() >= 2, "all three files should match");
  assert_eq!(results[0], "apps/admin/jest.config.ts");
  ```

### Assessment

**Ready to merge?** Yes (with optional minor fix)
**Reasoning:** Both split predicates are correctly updated, the fix directly addresses the reported failures, and 29/29 tests pass with no regressions. The single minor finding is a test assertion robustness concern that does not affect correctness of the production code.
