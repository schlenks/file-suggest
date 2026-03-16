[CODE-REVIEW-2/3] hub-5b3.2 wave-1

---

### Changed Files Manifest

| File | Lines Changed | Read in Full |
|------|--------------|--------------|
| `src/search.rs` | +85 / -2 (87 net) | Yes |
| `tests/search_test.rs` | +28 / 0 (28 net) | Yes |

---

### Rules Consulted

(none — no repo rules found; `.claude/rules/` not present in file-suggest)

---

### Requirement Mapping

| Requirement | Implementing Code | Status |
|-------------|------------------|--------|
| Fetch 30 results from FTS5 (not 15) | `src/search.rs:7` — `FETCH_RESULTS = 30`; `src/search.rs:85` — `LIMIT {FETCH_RESULTS}` | Implemented |
| `find_matching_dirs()` — distinct apps/X/ and packages/X/ prefixes | `src/search.rs:142-192` | Implemented |
| `find_matching_dirs()` — filter by query token match | `src/search.rs:178-189` | Implemented |
| SQL subquery to avoid alias-in-WHERE SQLite limitation | `src/search.rs:157-170` | Implemented |
| `apply_directory_boost()` — rank-position re-ranking, not absolute | `src/search.rs:200-213` — partition approach | Implemented |
| Must not override strong BM25 matches | `src/search.rs:205-208` — partition preserves BM25 order within each group | Implemented |
| Wire into search pipeline after FTS5 | `src/search.rs:23-25` | Implemented |
| Test: `directory_context_boosts_files_in_matching_dir` | `tests/search_test.rs:122-147` | Implemented |
| Test: index four required files (docker, apps/temporal-worker/*, .idea) | `tests/search_test.rs:131-135` | Implemented |
| Test: assert results[0].starts_with("apps/temporal-worker/") | `tests/search_test.rs:142-146` | Implemented |
| Boost only apps/X and packages/X (not arbitrary dirs) | `src/search.rs:161-165` — CASE WHEN limits to these two prefixes | Implemented |

---

### Uncovered Paths

1. **`packages/X/` boost not tested** — `find_matching_dirs` includes the `packages/` CASE branch (`src/search.rs:163-165`) but no test exercises it. A query like `data` where `packages/data/` exists in the index would exercise this path.

2. **Token containment false-positive not tested** — If a query token is a substring of a dir name (e.g., query `an` matches dir `angular`), the dir gets a boost. This is a behavior consequence of the containment check at `src/search.rs:186`. No test documents this as intentional or catches unexpected results from it.

3. **`apply_directory_boost` with empty `matching_dirs` not directly tested** — The `if matching_dirs.is_empty()` early return at line 201 takes the path only when `find_matching_dirs` returns `[]`. This is reachable (e.g., query "bksvc" with no matching apps/ or packages/ dirs) but not covered by a dedicated test case.

---

### Not Checked

- **Runtime behavior against real file corpus**: The SQL correctness for extracting `apps/X/` and `packages/X/` was verified by code trace only. A live DB with actual file paths was not queried.
- **Performance with large file_scores table**: `find_matching_dirs` performs a full table scan on `file_scores` with no index on the `LIKE 'apps/%'` condition. Not benchmarked. Not a concern for typical workspace sizes (~10k files) but not verified.
- **Cargo test results**: Tests were not run in this review session; test suite pass was asserted by the implementer in the IMPL-REPORT comment.

---

### Findings

#### Minor

**Finding M1: `packages/X/` boost path has no test coverage**
- **File:line**: `tests/search_test.rs` — missing test; `src/search.rs:163-165` — the uncovered branch
- **What's wrong**: The spec explicitly requires "Only `apps/X` and `packages/X`" but only `apps/` is exercised by the new test. The `packages/` CASE branch in the SQL subquery is untested.
- **Why it matters**: A regression in the `packages/` branch (e.g., off-by-one in `substr(path, 10, ...)`) would go undetected. The spec calls out both prefixes as first-class supported paths.
- **How to fix**: Add a second test — e.g., index `packages/data/src/user.repository.ts` + a competing `scripts/data-export.sh`, query "data", assert `results[0].starts_with("packages/data/")`.

**Finding M2: Token containment check is a substring match, not whole-token equality**
- **File:line**: `src/search.rs:186` — `dir_name.to_lowercase().contains(&tok.to_lowercase())`
- **What's wrong**: A query token `api` will match directory `apis`, `rapid`, `therapist`. A query `an` matches `angular`, `plans`. This is wider than a whole-token match.
- **Why it matters**: Extra directories get boosted, potentially pushing correct results down. The spec says boost fires when "query tokens match a well-known directory name" — "match" implies whole-name equivalence, not substring containment.
- **How to fix** (if desired): Change `contains` to equality: `dir_name.to_lowercase() == tok.to_lowercase()`. The existing test still passes because "temporal" is a whole-name match for `temporal-worker` only after the split... actually `temporal-worker` as a dir name with token `temporal` would fail equality. A word-boundary approach is needed: check if the dir name equals any token, or contains the full token as a hyphen/underscore-delimited segment. This requires splitting the dir name on the same delimiters and checking for set intersection. This may have been a deliberate choice — not flagged as Important because the spec wording is ambiguous and the boost is non-destructive.

---

### Assessment

**Ready to merge?** With fixes
**Reasoning:** The core implementation is correct and matches all spec requirements. The two minor findings (missing `packages/` test, substring-vs-whole-token matching) do not break the feature but leave a gap in the test coverage for one of two explicitly specified directory prefixes, and introduce a latent behavioral ambiguity that could cause unexpected boosts for short query tokens.
