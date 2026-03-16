## Task 7: BM25-tiered frecency in FTS5 ranking

**Purpose:** Among files with similar BM25 relevance, prefer recently/frequently edited ones. ROUND-based tiering creates relevance "buckets" and only uses frecency within each bucket.

**Not In Scope:** Self-tuning weights. This is a fixed formula.

## Files
- Modify: `src/search.rs` (update ORDER BY to use ROUND-based tiering)

## Steps

**Step 1: Update search_fts ORDER BY**
Change to:
```sql
ORDER BY ROUND(bm25(files_fts, 1.0, 10.0, 2.0) + (s.type_penalty * 0.5), 1) + (length(f.path) * 0.001) - (s.frecency * 0.1)
```
The `ROUND(..., 1)` creates tiers of BM25 relevance. Within each tier, frecency (weight 0.1) and path length break ties.

**Step 2: Test with benchmark**

**Step 3: Commit**
