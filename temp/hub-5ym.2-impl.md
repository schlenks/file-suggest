[IMPL-REPORT] hub-5ym.2 wave-2
### Evidence
- Commit: 0614a88 | Files: 2 changed (77 insertions, 22 deletions) | Tests: 30/30 pass, exit 0
### Summary
- Added `apply_filename_boost` post-processing function that promotes files whose basename exactly matches the query (exact or prefix-with-dot), guarded by a `.`-in-query check to avoid conflicting with directory boost for directory-style queries like `temporal-worker`.
- Files modified: `src/search.rs`, `tests/search_test.rs` (matches allowed list exactly).
- Self-review: All 30 tests pass including the new `exact_filename_beats_stemmer_conflation` test and all pre-existing tests. The linter flagged `find_matching_dirs` as too long (lizard miscount due to closures), fixed by extracting `extract_top_level_dir` and `dir_matches_tokens` helpers. Scope clean — only allowed files touched.
