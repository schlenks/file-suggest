[IMPL-REPORT] hub-5b3.2 wave-1
### Evidence
- Commit: 806e950 | Files: 2 changed (113 insertions/2 deletions) | Tests: 27/27 pass, exit 0
### Summary
- Implemented directory-context boost for `apps/X/` and `packages/X/` prefixes: FTS5 now fetches 30 candidates (FETCH_RESULTS constant), `find_matching_dirs()` queries distinct well-known dir prefixes and filters by query token match, `apply_directory_boost()` partitions results so matching-dir files lead the list before truncating to 15. Fixes the temporal-worker pattern where short infra paths beat app source files.
- Files modified: `src/search.rs`, `tests/search_test.rs` — matches allowed list exactly.
- Self-review: rule-of-five-code applied (5 passes), rule-of-five-tests applied (5 passes). No scope violations. SQL subquery avoids alias-in-WHERE SQLite limitation. Edge cases handled: empty tokens, no matching dirs, all-in-matching-dir, files at root of apps/X (guarded by `apps//` filter). Test crafted to confirm the failure case (infra path `docker/temporal-worker` wins without boost) then passes with boost.
