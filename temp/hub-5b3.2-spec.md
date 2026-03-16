[SPEC-REVIEW] hub-5b3.2 wave-1

## Findings

### Specification Compliance: PASS

All requirements from the specification have been implemented correctly:

1. **FETCH_RESULTS constant** (src/search.rs:7)
   - Correctly set to 30 as required
   - Comment explains purpose: "Fetch extra candidates from FTS5 so directory-boost re-ranking has enough candidates"

2. **find_matching_dirs() function** (src/search.rs:142-192)
   - Correctly queries distinct app/package prefixes from file_scores
   - Uses subquery pattern (lines 157-170) to avoid SQLite "alias in WHERE" limitation
   - Properly tokenizes query by '/', '.', '_', '-' (lines 146-149)
   - Filters to only match query tokens case-insensitively (lines 185-186)
   - Correctly guards against edge cases: filters out 'apps//' and 'packages//' (line 169)

3. **apply_directory_boost() function** (src/search.rs:200-213)
   - Uses partition to separate matching-dir vs non-matching files
   - Preserves relative BM25 order within each group (partition maintains insertion order)
   - Moves boosted results to front without overriding strong BM25 matches (rank-position approach)
   - Correctly truncates to MAX_RESULTS (15)

4. **Integration into search pipeline** (src/search.rs:21-26)
   - Calls find_matching_dirs() after FTS5 query
   - Calls apply_directory_boost() before returning results
   - Only applies when FTS5 returns non-empty results
   - Correctly falls through to trigram/LIKE/fuzzy for other result types

5. **search_fts() modification** (src/search.rs:71-91)
   - LIMIT changed to use FETCH_RESULTS constant (line 85)
   - Fetches 30 instead of 15 to provide re-ranking candidates

6. **Test: directory_context_boosts_files_in_matching_dir** (tests/search_test.rs:122-147)
   - Correct scenario: temporal-worker query with competing infra/app files
   - Indexes all four required files: docker/temporal-worker, apps/temporal-worker/*, .idea/*
   - Asserts results[0].starts_with("apps/temporal-worker/")
   - Test passes (verified: cargo test directory_context → PASS)

7. **Scope adherence**
   - No arbitrary directory boosting (only apps/X/ and packages/X/)
   - Files modified match spec exactly: src/search.rs + tests/search_test.rs
   - No extra features or over-engineering

### Test Results
- directory_context test: PASS
- All 27 tests: PASS (verified exit 0)

### Code Quality
- SQL subquery pattern correctly avoids SQLite limitation
- Edge cases properly handled (empty tokens, root-level files, no matching dirs)
- Comments explain non-obvious logic (alias limitation, boost strategy)
- Rust idiomatic code (partition, iteration, string operations)

## Conclusion
Spec compliant: No issues found. Implementation matches all requirements exactly.
