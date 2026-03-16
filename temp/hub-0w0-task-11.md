## Task 11: Integration tests for search ranking

**Purpose:** Verify the full search pipeline produces correct rankings against a known file set, ensuring all ranking layers work together.

## Files
- Create: `tests/search_test.rs`
- Modify: `Cargo.toml` (add tempfile dev-dependency)

## Steps

**Step 1: Add dev dependency** `tempfile = "3"` to Cargo.toml

**Step 2: Write integration tests** in `tests/search_test.rs`
- Helper: `build_test_index(files)` creates a temp DB with known files
- `source_file_ranks_above_test_file` -- Button.tsx above Button.test.tsx
- `exact_filename_match_ranks_first` -- tsconfig returns tsconfig.json at #1
- `trigram_finds_substring_matches` -- "config" finds tsconfig.json
- `fuzzy_finds_abbreviations` -- "bksvc" finds booking.service.ts
- `empty_query_returns_frecency_sorted` -- higher frecency first
- `frecency_tie_breaking_test` -- same BM25, different frecency

**Step 3: Run tests**

**Step 4: Commit**
