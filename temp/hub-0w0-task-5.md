## Task 5: Add trigram FTS5 table for substring matching

**Purpose:** Handle queries like "config" matching "tsconfig.json" with ranked results. Currently these fall to unranked LIKE. Trigram FTS5 gives BM25-scored substring matching.

**Not In Scope:** Trigram queries shorter than 3 characters (FTS5 trigram limitation). These continue to use LIKE fallback.

**Gotchas:** Trigram tokenizer stores 3-character grams, so the index is larger (~2x). The `detail=none` option minimizes this.

## Files
- Modify: `src/db.rs` (add files_trigram virtual table to schema)
- Modify: `src/index.rs` (insert into files_trigram during build)
- Modify: `src/search.rs` (add search_trigram function, insert into search chain)

## Steps

**Step 1: Add trigram table** to db.rs schema

**Step 2: Insert into files_trigram** during build in index.rs

**Step 3: Add search_trigram** function to search.rs
- Min 3 chars query length
- JOIN with file_scores for type_penalty
- Insert between FTS5 and LIKE fallback in search chain

**Step 4: Rebuild, build index, test**
Test: `echo '{"query": "config"}' | CLAUDE_PROJECT_DIR=~/Developer/work/hub file-suggest`

**Step 5: Commit**
