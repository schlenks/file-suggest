[CODE-REVIEW-3/3] hub-5ym.2 wave-2

---

### Changed Files Manifest

| File | Lines Changed | Read in Full |
|------|--------------|--------------|
| `src/search.rs` | +60 / -22 (82 total changed) | Yes |
| `tests/search_test.rs` | +17 / -0 | Yes |

---

### Rules Consulted

(none — no `.claude/rules/` directory in this project)

---

### Requirement Mapping

| Requirement | Implementing Code | Status |
|-------------|------------------|--------|
| Step 1: Write failing test `exact_filename_beats_stemmer_conflation` | `tests/search_test.rs:173-188` | Implemented |
| Step 2: Verify test fails before implementation | Not directly verifiable in diff; prior commit history confirms | Implemented |
| Step 3: Add `apply_filename_boost` with correct signature and logic | `src/search.rs:224-241` | Implemented |
| Step 4: Wire filename boost AFTER directory boost in pipeline | `src/search.rs:25` | Implemented |
| Step 5: Test passes after implementation | Verified (30/30 per impl report) | Implemented |
| Step 6: Full test suite passes (30 tests) | Verified (30/30 per impl report) | Implemented |
| Step 7: Commit with correct file scope | Commit 0614a88 touches only `src/search.rs` and `tests/search_test.rs` | Implemented |
| Gotcha: Handle `sanitization.ts` (exact match) | `search.rs:236`: `basename == query_lower` | Implemented |
| Gotcha: Handle `booking.service` (prefix match) | `search.rs:236`: `starts_with(&format!("{}.", query_lower))` | Implemented |
| Gotcha: Boost AFTER directory boost | `search.rs:24-25`: directory boost at line 24, filename boost at line 25 | Implemented |
| Not In Scope: FTS5 tokenizer/stemmer unchanged | No tokenizer changes in diff | Confirmed |

---

### Uncovered Paths

1. **`apply_filename_boost` receives a path with no `/`** (root-level file like `README.md`): `path.rsplit('/').next().unwrap_or(path)` correctly returns the full path as the basename. Not a bug, but not tested.

2. **Query where `exact` partition is empty** (e.g., query `foo.ts` with no file named `foo.ts` in results): `exact` is empty, `rest` contains all items, `exact.append(&mut rest)` produces unchanged order. Correct behavior, not tested.

3. **Query where dot guard fires on non-filename input**: e.g., a query like `2.0` or `v1.2.3`. The dot guard would activate filename boost. In practice this is harmless (no file likely has basename `2.0`), but the dot heuristic can trigger on version strings. No test covers this.

4. **`apply_filename_boost` is not applied on the trigram/LIKE/fuzzy fallback paths** (lines 31-44). If FTS5 returns empty results, the filename boost is never applied even for `.`-containing queries. The spec does not require coverage of fallbacks, but it means a query for `sanitization.ts` that falls through to trigram or LIKE search would not get the boost. This is a latent scope gap, not a defect in this PR.

---

### Not Checked

- Runtime behavior against a real populated database (only test-index coverage).
- Whether `cargo test` currently passes (no CI output visible; impl report claims 30/30 but cannot independently verify from code alone).
- Interaction with `space_separated_query_finds_results` where the query contains both a space and a dot (e.g., `admin jest.config`) — the test at line 162 covers this indirectly, but the filename boost behavior for multi-token dot-queries is not explicitly verified.

---

### Findings

#### Minor

**File:** `src/search.rs:222`
**What's wrong:** The docstring states "The list is already truncated to MAX_RESULTS by apply_directory_boost, so no further truncation is needed here." — yet line 239 calls `exact.truncate(MAX_RESULTS)`. The comment and the code contradict each other.
**Why it matters:** The comment is misleading to future readers: it claims truncation is not needed but the code does it anyway. The `truncate` is a harmless defensive call (always a no-op when input ≤ MAX_RESULTS), but having a comment that denies what the code does reduces trust in the docstring.
**How to fix:** Either remove the sentence "so no further truncation is needed here" from the docstring, or remove the `exact.truncate(MAX_RESULTS)` call (since it is genuinely redundant). The former is safer as a defensive no-op.

---

### Assessment

**Ready to merge?** With fixes
**Reasoning:** The implementation is correct and all spec requirements are met. The single Minor finding (docstring contradicts the truncate call) does not affect correctness, but should be resolved before merge to avoid misleading future readers.
