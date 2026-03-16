## Task 3: Integrate file-type penalty into index build

**Purpose:** Store the pre-computed type_penalty in the DB so it's available at query time without runtime computation.

## Files
- Modify: `src/db.rs` (add type_penalty column to file_scores)
- Modify: `src/index.rs` (compute and store type_penalty during insert)

## Steps

**Step 1: Update schema** in db.rs
Add `type_penalty REAL DEFAULT 0.0` column to `file_scores` table.

**Step 2: Update insert_files** in index.rs
Change the INSERT statement to include type_penalty, and call `scoring::type_penalty()` for each file.

**Step 3: Rebuild and verify**
Run: `cargo build --release`
Run: `file-suggest build ~/Developer/work/hub`
Verify: test/generated files show non-zero penalties in DB.

**Step 4: Commit**
