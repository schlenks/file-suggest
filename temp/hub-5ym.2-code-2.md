[CODE-REVIEW-2/3] hub-5ym.2 wave-2

---

### Changed Files Manifest

| File | Lines Changed | Read in Full |
|------|--------------|-------------|
| `src/search.rs` | +60 / -22 (82 total) | Yes |
| `tests/search_test.rs` | +17 / -0 | Yes |

---

### Rules Consulted

(none — no `.claude/rules/` directory found in `file-suggest` repo)

---

### Requirement Mapping

| Requirement | Implementing Code | Status |
|-------------|------------------|--------|
| Step 1: Write failing test `exact_filename_beats_stemmer_conflation` | `tests/search_test.rs:173-188` | Implemented |
| Step 3: Add `apply_filename_boost` function with correct signature | `src/search.rs:224-241` | Implemented |
| Step 3: Handle exact match (`sanitization.ts`) | `src/search.rs:236` — `basename == query_lower` | Implemented |
| Step 3: Handle prefix match (`booking.service` → `booking.service.ts`) | `src/search.rs:236` — `basename.starts_with(&format!("{}.", query_lower))` | Implemented |
| Step 4: Wire after directory boost | `src/search.rs:25` — after line 24 | Implemented |
| Step 6: All 30 tests pass | Verified via IMPL-REPORT; test count actually 31 per grep (minor discrepancy in self-report, not a code issue) | Implemented |
| Step 7: Only allowed files modified (`src/search.rs`, `tests/search_test.rs`) | Confirmed via `git diff --stat` | Implemented |
| Gotcha: boost must come AFTER directory boost | Pipeline order at lines 24-25 confirmed | Implemented |
| Gotcha: queries with and without extensions handled | Both cases handled in partition logic at line 236 | Implemented |
| Not In Scope: FTS5 tokenizer/stemmer changes | No tokenizer/stemmer changes in diff | Clean |

---

### Uncovered Paths

1. **Prefix-match path untested** (`booking.service` → `booking.service.ts`): The spec explicitly calls this out as a Gotcha but the test suite only covers the exact-match case (`sanitization.ts`). The `starts_with(&format!("{}.", query_lower))` branch has no test exercising it.

2. **Filename boost not applied to trigram/LIKE/fuzzy fallback paths** (`src/search.rs:31-44`): If FTS5 returns empty results and the query falls through to trigram, LIKE, or fuzzy, `apply_filename_boost` is never called. For a query like `sanitization.ts` that has FTS5 results, this is fine. However, for a filename-extension query that happens to miss FTS5 (edge case), the boost silently doesn't apply. The spec does not address fallback paths, so this is noted as a gap rather than a violation.

3. **Dot-only or trailing-dot queries** (`query = "."` or `"booking."`): Guard passes (`contains('.')`), boost attempts to match, no files match, returns original order. Harmless but untested.

---

### Not Checked

- Runtime test execution was not run in this review environment; test results taken from IMPL-REPORT at face value.
- Interaction with the actual FTS5 porter stemmer in a real index was not independently verified — trust placed in the existing test infrastructure.

---

### Findings

#### Minor

**M1 — `src/search.rs:239` — Redundant truncation after comment says it's unnecessary**

`exact.truncate(MAX_RESULTS)` is called in `apply_filename_boost` (line 239). The doc comment on lines 221-223 explicitly states: "The list is already truncated to MAX_RESULTS by apply_directory_boost, so no further truncation is needed here." The call is harmless but contradicts the documentation. Either remove the call and trust `apply_directory_boost`, or remove the "no further truncation is needed" sentence from the doc comment.

Fix: Remove the `exact.truncate(MAX_RESULTS);` line (line 239) since the invariant is guaranteed by `apply_directory_boost`, OR update the doc comment to acknowledge it as a defensive guard.

**M2 — `tests/search_test.rs` — Prefix-match gotcha from spec is untested**

The spec explicitly documents `booking.service` as a case that must work (prefix match against `booking.service.ts`). Only the exact-match case is tested. If the `starts_with` branch were accidentally broken, no test would catch it.

Fix: Add a test `filename_prefix_beats_unrelated_files` using files like `("apps/api/src/booking.service.ts", 0.0)` and querying with `"booking.service"`.

---

### Assessment

**Ready to merge?** With fixes

**Reasoning:** All spec requirements are implemented correctly, the primary motivating test passes, and the dot-guard enhancement is well-reasoned. Two minor issues: a doc comment that contradicts the code (redundant truncation call), and the spec-documented prefix-match case having no test coverage.
