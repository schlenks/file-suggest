## Task 4: Integrate file-type penalty into search ranking

**Purpose:** Make test/generated/barrel files rank below source files.

## Files
- Modify: `src/search.rs` (update FTS5 ORDER BY to include type_penalty)

## Steps

**Step 1: Update search_fts query**
Add JOIN to file_scores and include type_penalty in ORDER BY:
```sql
SELECT f.path FROM files_fts f
JOIN file_scores s ON f.path = s.path
WHERE files_fts MATCH '{fts_query}'
ORDER BY bm25(files_fts, 1.0, 10.0, 2.0) + (s.type_penalty * 0.5) + (length(f.path) * 0.001)
LIMIT {MAX_RESULTS}
```

**Step 2: Test with benchmark**

**Step 3: Commit**
