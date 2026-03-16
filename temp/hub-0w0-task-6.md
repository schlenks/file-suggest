## Task 6: Add fuzzy matching fallback

**Purpose:** Catch abbreviation-style queries like `bksvc` or `fsc` that FTS5, trigram, and LIKE all miss. This is the last-resort search layer.

**Gotchas:** Fuzzy matching scans all ~10k file paths in memory. This takes ~5-10ms, so it only fires when all other methods return empty. Must load the file list from the DB once, not per-character.

## Files
- Modify: `Cargo.toml` (add fuzzy-matcher dependency)
- Create: `src/fuzzy.rs`
- Modify: `src/search.rs` (add fuzzy fallback at end of search chain)

## Steps

**Step 1: Add dependency** to Cargo.toml: `fuzzy-matcher = "0.3"`

**Step 2: Write fuzzy.rs**
- `fuzzy_search(query, paths, limit) -> Vec<String>` using SkimMatcherV2
- Add `pub mod fuzzy;` to `src/lib.rs`

**Step 3: Add fuzzy fallback** to search chain after LIKE returns empty
- Add `load_all_paths` helper to load all paths from file_scores

**Step 4: Test**
Test: `echo '{"query": "bksvc"}' | CLAUDE_PROJECT_DIR=~/Developer/work/hub file-suggest`

**Step 5: Commit**
