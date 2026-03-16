[CODE-REVIEW-2/3] hub-5ym.1 wave-1

---

### Changed Files Manifest

| File | Lines Changed | Read in Full |
|------|--------------|-------------|
| `src/search.rs` | +2 / -2 | Yes |
| `tests/search_test.rs` | +12 / -0 | Yes |

---

### Rules Consulted

(none — no repo rules found at `.claude/rules/`)

---

### Requirement Mapping

| Requirement | Implementing Code | Status |
|-------------|------------------|--------|
| Write failing test `space_separated_query_finds_results` | `tests/search_test.rs:162-171` | Implemented |
| Add whitespace to `build_fts_query` split predicate | `src/search.rs:209` | Implemented |
| Add whitespace to `find_matching_dirs` split predicate | `src/search.rs:145` | Implemented |
| Test verifies `results[0] == "apps/admin/jest.config.ts"` | `tests/search_test.rs:170` | Implemented |
| Full test suite passes (29 tests) | Verified by `cargo test` | Implemented |
| Commit created | `aefb8b1` | Implemented |

---

### Uncovered Paths

1. **Space query + FTS5 empty → fallback branches** — The new test exercises the happy path (FTS5 returns results). No test covers a space-separated query that falls through to trigram or LIKE fallback. This is a minor gap; the fallback paths are pre-existing and were not touched by this diff.

2. **Leading/trailing whitespace in query** — `"  admin jest.config  "` would produce tokens `["admin", "jest", "config"]` after splitting on whitespace, which is correct behavior. The `.filter(|t| !t.is_empty())` guard handles this. Not a risk.

3. **Multiple consecutive spaces** — `"admin  jest.config"` is handled correctly by `filter(|t| !t.is_empty())`. Not a risk.

---

### Not Checked

- Runtime behavior against a real FTS5 database (only unit test environment with in-memory SQLite via `build_test_index` was verified).
- Whether the `search_trigram` fallback handles space-containing queries correctly when FTS5 returns nothing — this is pre-existing behavior, not introduced by this diff.

---

### Findings

**No Critical findings.**

**No Important findings.**

**No Minor findings.**

The two changed lines in `src/search.rs` are minimal and symmetric — both split predicates receive the identical whitespace extension. The `.filter(|t| !t.is_empty())` guard (already present in both functions) handles the edge cases introduced by whitespace splitting (leading/trailing/consecutive spaces). The new test is behaviorally meaningful: it exercises the full search pipeline (FTS5 + directory boost), asserts non-empty results AND a specific ranking, and uses the exact fixture from the spec.

---

### Assessment

**Ready to merge?** Yes

**Reasoning:** Both split predicates are correctly updated, the fix is consistent, and the new test validates the exact failure scenario described in the requirements. The implementation is minimal with no scope creep and 29/29 tests pass.
