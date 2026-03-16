## Key Decisions

- **DB location:** Per-project DBs in `~/.claude/file-suggest/{hash}.db` -- avoids polluting project dirs, supports multiple repos simultaneously, deterministic lookup via FNV-1a path hash
- **Incremental strategy:** Diff `git ls-files` output against stored file list, then INSERT/DELETE delta rows -- avoids full rebuild on every commit while keeping frecency fresh via periodic full refresh
- **Fuzzy matcher:** `fuzzy-matcher` crate with fzf's Clangd-style scoring -- proven algorithm, Rust-native, path-aware scoring with separator bonuses, no external process
- **Trigram vs LIKE fallback:** Add a secondary `fts5(path, tokenize='trigram')` table for ranked substring matching -- LIKE has no ranking, trigram gives BM25-scored substring results
- **File-type penalties stored at index time:** Compute penalty once during build, store in `file_scores.type_penalty` column -- avoids runtime regex on every query

## Architecture

The binary remains a single statically-linked Rust CLI. The DB changes from a single global file to per-project databases keyed by an FNV-1a hash of the canonical project path, stored in `~/.claude/file-suggest/`. Index building gains an incremental mode that diffs against the previous index. Search gains three new ranking layers: file-type penalties, trigram substring matching, and fuzzy matching fallback. BM25-tiered frecency breaks ties among equally-relevant results.

## Tech Stack

Rust 2024 edition, rusqlite (bundled SQLite with FTS5), serde_json, fuzzy-matcher crate (fzf algorithm)

## Plan

Full plan at: `/Users/schlenks/Developer/personal/file-suggest/docs/plans/2026-03-15-file-suggest-v2.md`
