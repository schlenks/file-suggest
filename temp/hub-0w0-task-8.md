## Task 8: Incremental index updates

**Purpose:** Reduce git hook build time from ~100ms (full rebuild) to ~10ms (delta update).

**Gotchas:**
- FTS5 DELETE requires knowing the exact row content. Use `DELETE FROM files_fts WHERE path = ?`.
- Frecency scores become stale with incremental updates. `build --full` refreshes them.
- Check `output.status.success()` on git diff, fall back to full rebuild on failure.
- Also run `git ls-files --others --exclude-standard` to catch untracked files.

## Files
- Modify: `src/git.rs` (add get_changed_files, get_head_hash functions)
- Modify: `src/index.rs` (add incremental_build function)
- Modify: `src/db.rs` (add metadata table for HEAD hash, delete support)
- Modify: `src/main.rs` (default `build` to incremental, add `build --full` flag)
- Create: `tests/index_test.rs`

## Steps

**Step 1: Write failing tests** in `tests/index_test.rs`
- `incremental_build_adds_new_files`
- `incremental_build_removes_deleted_files`
- `incremental_falls_back_to_full_when_no_hash`

**Step 2: Add get_changed_files and get_head_hash** to git.rs

**Step 3: Add metadata table** to db.rs schema, store HEAD hash

**Step 4: Add incremental_build** to index.rs
- Check stored HEAD hash vs current
- If delta > 500 files, fall back to full
- DELETE removed, INSERT added with scoring

**Step 5: Update main.rs** build command
- Default to incremental, --full flag for full rebuild

**Step 6: Store HEAD hash** in full build via insert_metadata

**Step 7: Test and verify**

**Step 8: Commit**
