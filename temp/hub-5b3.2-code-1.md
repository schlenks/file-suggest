[CODE-REVIEW-1/3] hub-5b3.2 wave-1

---

### Changed Files Manifest

| File | Lines Changed | Read in Full |
|------|--------------|--------------|
| `src/search.rs` | +87 / -2 | Yes |
| `tests/search_test.rs` | +28 / -0 | Yes |

---

### Rules Consulted

(none — no repo rules found at `.claude/rules/`)

---

### Requirement Mapping

| Requirement | Implementing Code | Status |
|-------------|------------------|--------|
| Fetch 30 results from FTS5 (not 15) for re-ranking | `src/search.rs:7` `FETCH_RESULTS = 30`; `src/search.rs:85` LIMIT uses `FETCH_RESULTS` | Implemented |
| `find_matching_dirs()` — queries distinct `apps/X/` and `packages/X/` prefixes, filters by query token match | `src/search.rs:142–192` | Implemented |
| SQL subquery to avoid alias-in-WHERE SQLite limitation | `src/search.rs:157–170` | Implemented |
| `apply_directory_boost()` — re-ranks by position, not absolute score; moves matching-dir files to front; truncates to MAX_RESULTS | `src/search.rs:200–213` | Implemented |
| Wire `find_matching_dirs` + `apply_directory_boost` into `search()` after FTS5 results | `src/search.rs:22–26` | Implemented |
| Boost must not override strong BM25 matches — use rank-position-based scoring | `apply_directory_boost` uses partition (preserves relative BM25 order within each group) | Implemented |
| Only `apps/X` and `packages/X` — no arbitrary nested dirs | SQL CASE expression limits to those two prefixes | Implemented |
| Test: `directory_context_boosts_files_in_matching_dir` with all four required files | `tests/search_test.rs:122–147` | Implemented |
| Test indexes: `docker/temporal-worker`, `apps/temporal-worker/src/workers/emailWorker.ts`, `apps/temporal-worker/package.json`, `.idea/runConfigurations/temporal_worker_dev.xml` | `tests/search_test.rs:131–136` | Implemented |
| Assert `results[0].starts_with("apps/temporal-worker/")` | `tests/search_test.rs:142–146` | Implemented |

---

### Uncovered Paths

1. **`packages/X/` boost path has no test coverage.** The spec says "Only `apps/X` and `packages/X`" and the SQL implements both branches, but the single new test only exercises the `apps/X` case. A query that matches a `packages/` directory has no test. The `packages/` branch of the SQL CASE expression (`substr(path, 10, ...)`) is untested.

2. **`apply_directory_boost` early-return path (empty `matching_dirs`) truncates via `take(MAX_RESULTS)` rather than going through the same truncation as the boosted path.** Both produce the same result (truncate to 15), but this path is exercised by all existing tests — it's not a bug, but the behavior is implicitly relied upon by every non-boosted query.

3. **Query with space characters** — `build_fts_query` splits on `/._-` but NOT on spaces. A query like `"temporal worker"` (with space) would produce a single token `["temporal worker"]` in `find_matching_dirs`, which would never match any dir name. The FTS5 query builder also doesn't split on spaces. This is a pre-existing behavior, not introduced in this diff, but `find_matching_dirs` inherits the same tokenizer split chars. Not a regression.

---

### Not Checked

- Runtime behavior on a real large repository (hundreds of `apps/X` and `packages/X` entries) — the SQL fetches ALL distinct prefixes and filters in Rust; scalability assumption is reasonable but untested at scale.
- Whether `cargo test` is currently green (test results from the impl report are taken at face value; I did not run the test suite independently).

---

### Findings

#### Important

**Finding 1 — Missing test for `packages/X/` directory boost**

- **File:line:** `tests/search_test.rs` (no line — test is absent)
- **What's wrong:** The spec explicitly requires boosting for both `apps/X` and `packages/X`. The SQL has a `WHEN path LIKE 'packages/%' THEN ...` branch at `src/search.rs:163–164`. This branch is entirely untested. A typo in the `substr(path, 10, ...)` offset (e.g., off-by-one) would go undetected.
- **Why it matters:** Half the specified boost surface area has no regression protection. If the `packages/` SQL branch silently produces wrong results (e.g., empty dir names yielding `packages//` that passes the filter due to a different edge), no test would catch it.
- **How to fix:** Add a test analogous to `directory_context_boosts_files_in_matching_dir` using a `packages/X/` path. Example scenario: query `"data"`, index files `["packages/data/src/user.repository.ts", "some-root-data-file.ts"]`, assert `results[0].starts_with("packages/data/")`.

---

#### Minor

**Finding 2 — `apply_directory_boost` inconsistency: `take` vs `truncate`**

- **File:line:** `src/search.rs:202`
- **What's wrong:** The empty-`matching_dirs` early-return path uses `results.into_iter().take(MAX_RESULTS).collect()` while the main path uses `boosted.truncate(MAX_RESULTS)`. Both are correct, but the inconsistency is a mild clarity issue for future maintainers who may not immediately recognize they're equivalent.
- **Why it matters:** Minor readability concern only; no behavioral difference.
- **Fix:** Change the early return to `{ let mut r = results; r.truncate(MAX_RESULTS); r }` to match the style of the main path, or leave as-is (acceptable).

---

### Assessment

**Ready to merge?** With fixes

**Reasoning:** All spec requirements are implemented correctly. One Important gap: the `packages/X/` SQL branch is untested despite being explicitly in scope per the spec. Adding one test to cover that path removes the only meaningful regression risk before merge.
