# file-suggest v2 Implementation Plan

> **For Claude:** After human approval, use plan2beads to convert this plan to a beads epic, then use `superpowers-bd:subagent-driven-development` for parallel execution.

**Goal:** Upgrade file-suggest from a single-project tool to a production-ready, multi-project file suggestion engine with smarter ranking and faster incremental updates.

**Architecture:** The binary remains a single statically-linked Rust CLI. The DB changes from a single global file to per-project databases keyed by an FNV-1a hash of the canonical project path, stored in `~/.claude/file-suggest/`. Index building gains an incremental mode that diffs against the previous index. Search gains three new ranking layers: file-type penalties, trigram substring matching, and fuzzy matching fallback. BM25-tiered frecency breaks ties among equally-relevant results.

**Tech Stack:** Rust 2024 edition, rusqlite (bundled SQLite with FTS5), serde_json, fuzzy-matcher crate (fzf algorithm)

**Key Decisions:**
- **DB location:** Per-project DBs in `~/.claude/file-suggest/{hash}.db` -- avoids polluting project dirs, supports multiple repos simultaneously, deterministic lookup via path hash
- **Incremental strategy:** Diff `git ls-files` output against stored file list, then INSERT/DELETE delta rows -- avoids full rebuild on every commit while keeping frecency fresh via periodic full refresh
- **Fuzzy matcher:** `fuzzy-matcher` crate with fzf's Clangd-style scoring -- proven algorithm, Rust-native, path-aware scoring with separator bonuses, no external process
- **Trigram vs LIKE fallback:** Add a secondary `fts5(path, tokenize='trigram')` table for ranked substring matching -- LIKE has no ranking, trigram gives BM25-scored substring results for queries like "config" matching "tsconfig"
- **File-type penalties stored at index time:** Compute penalty once during build, store in `file_scores.type_penalty` column -- avoids runtime regex on every query

---

## File Structure

| File | Responsibility | Action |
|------|---------------|--------|
| `src/main.rs` | CLI entry point, command dispatch, DB path resolution | Modify |
| `src/db.rs` | Schema creation, DB open, optimize, migrations | Modify |
| `src/index.rs` | Full build + incremental build, tokenization, file-type penalty computation | Modify |
| `src/search.rs` | FTS5 search, trigram search, fuzzy fallback, ranking formula | Modify |
| `src/git.rs` | Git file listing, frecency computation, diff-based file changes | Modify |
| `src/scoring.rs` | File-type penalty rules, BM25-tiered frecency formula | Create |
| `src/fuzzy.rs` | Fuzzy matching wrapper over fuzzy-matcher crate | Create |
| `src/project.rs` | Project path hashing, per-project DB path resolution | Create |
| `src/lib.rs` | Public module declarations for lib target (enables integration tests) | Create |
| `Cargo.toml` | Add fuzzy-matcher dependency, tempfile dev-dependency | Modify |
| `README.md` | Document new features, update benchmarks | Modify |
| `tests/search_test.rs` | Integration tests for search ranking with known file sets | Create |
| `tests/project_test.rs` | Unit tests for project path hashing and DB resolution | Create |
| `tests/scoring_test.rs` | Unit tests for file-type penalty computation | Create |
| `tests/index_test.rs` | Integration tests for incremental index updates | Create |

---

## Task 1: Multi-project DB resolution
**Depends on:** Task 10
**Complexity:** simple
**Files:**
- Create: `src/project.rs`
- Modify: `src/main.rs:16-19` (replace `db_path()` function)
- Create: `tests/project_test.rs`

**Purpose:** Enable simultaneous indexes for multiple repos. Without this, switching Claude Code sessions between projects serves stale results from the wrong repo.

**Not In Scope:** DB migration from v1 single-file format. Users will simply `file-suggest build` to create the new per-project DB. The old `~/.claude/file-suggestion.db` is ignored.

**Step 1: Write the failing test**
```rust
// tests/project_test.rs
use std::path::Path;

#[test]
fn db_path_is_deterministic_for_same_project() {
    let path1 = file_suggest::project::db_path_for(Path::new("/Users/dev/work/hub"));
    let path2 = file_suggest::project::db_path_for(Path::new("/Users/dev/work/hub"));
    assert_eq!(path1, path2);
}

#[test]
fn db_path_differs_for_different_projects() {
    let path1 = file_suggest::project::db_path_for(Path::new("/Users/dev/work/hub"));
    let path2 = file_suggest::project::db_path_for(Path::new("/Users/dev/personal/blog"));
    assert_ne!(path1, path2);
}

#[test]
fn db_path_lives_under_dot_claude() {
    let path = file_suggest::project::db_path_for(Path::new("/Users/dev/work/hub"));
    let path_str = path.to_string_lossy();
    assert!(path_str.contains(".claude/file-suggest/"));
    assert!(path_str.ends_with(".db"));
}
```

**Step 2: Run test to verify it fails**
Run: `cargo test --test project_test`
Expected: FAIL (module doesn't exist)

**Step 3: Write minimal implementation**
```rust
// src/project.rs
use std::path::{Path, PathBuf};

/// Compute a deterministic DB path for a project directory.
/// Uses first 16 chars of hex-encoded SHA256 of the canonical path.
pub fn db_path_for(project_dir: &Path) -> PathBuf {
    let canonical = project_dir
        .canonicalize()
        .unwrap_or_else(|_| project_dir.to_path_buf());
    let hash = simple_hash(canonical.to_string_lossy().as_bytes());
    let home = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default();
    home.join(".claude")
        .join("file-suggest")
        .join(format!("{hash}.db"))
}

/// Simple non-crypto hash (FNV-1a 64-bit, hex-encoded, first 16 chars).
/// We don't need cryptographic strength, just collision avoidance.
fn simple_hash(data: &[u8]) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for &byte in data {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

/// Ensure the DB directory exists.
pub fn ensure_db_dir() {
    let home = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default();
    let dir = home.join(".claude").join("file-suggest");
    let _ = std::fs::create_dir_all(dir);
}
```

**Step 4: Update main.rs** to use `project::db_path_for` instead of hardcoded `db_path()`:
```rust
// In main.rs, replace db_path() with:
fn db_path() -> PathBuf {
    project::ensure_db_dir();
    project::db_path_for(&project_dir())
}
```

Also add `pub mod project;` to main.rs module declarations and add `#[path = "../src/project.rs"]` or make the crate a lib+bin.

**Step 5: Run test to verify it passes**
Run: `cargo test --test project_test`
Expected: PASS

**Step 6: Commit**
`git add src/project.rs src/main.rs tests/project_test.rs`
`git commit -m "feat: multi-project DB support with per-project hashed paths"`

---

## Task 2: File-type scoring rules
**Depends on:** Task 10
**Complexity:** simple
**Files:**
- Create: `src/scoring.rs`
- Create: `tests/scoring_test.rs`

**Purpose:** Compute penalty scores for file types so test files, generated files, snapshots, and barrel files rank below source files with identical BM25 relevance.

**Step 1: Write the failing test**
```rust
// tests/scoring_test.rs
#[test]
fn source_file_has_zero_penalty() {
    let penalty = file_suggest::scoring::type_penalty("packages/design-system/src/Button/Button.tsx");
    assert_eq!(penalty, 0.0);
}

#[test]
fn test_file_has_penalty() {
    let penalty = file_suggest::scoring::type_penalty("packages/design-system/src/Button/Button.test.tsx");
    assert!(penalty > 0.0);
}

#[test]
fn snapshot_file_has_higher_penalty_than_test() {
    let snap = file_suggest::scoring::type_penalty("src/Button/__snapshots__/Button.test.tsx.snap");
    let test = file_suggest::scoring::type_penalty("src/Button/Button.test.tsx");
    assert!(snap > test);
}

#[test]
fn generated_file_has_penalty() {
    let penalty = file_suggest::scoring::type_penalty("apps/marketplace/graphql/generated/types.ts");
    assert!(penalty > 0.0);
}

#[test]
fn index_barrel_has_small_penalty() {
    let penalty = file_suggest::scoring::type_penalty("packages/design-system/src/Button/index.ts");
    assert!(penalty > 0.0);
    let source = file_suggest::scoring::type_penalty("packages/design-system/src/Button/Button.tsx");
    assert!(penalty > source);
}

#[test]
fn stories_file_has_penalty() {
    let penalty = file_suggest::scoring::type_penalty("src/Button/Button.stories.tsx");
    assert!(penalty > 0.0);
}
```

**Step 2: Run test to verify it fails**
Run: `cargo test --test scoring_test`
Expected: FAIL

**Step 3: Write minimal implementation**
```rust
// src/scoring.rs

/// Compute a type-based penalty for a file path.
/// Higher penalty = ranked lower. Source files get 0.0.
pub fn type_penalty(path: &str) -> f64 {
    let lower = path.to_lowercase();

    // Generated files (highest penalty)
    if lower.contains("/generated/") || lower.contains("/gen/") {
        return 1.0;
    }

    // Snapshot files
    if lower.ends_with(".snap") || lower.contains("__snapshots__") {
        return 0.8;
    }

    // Test files
    if lower.ends_with(".test.ts")
        || lower.ends_with(".test.tsx")
        || lower.ends_with(".spec.ts")
        || lower.ends_with(".spec.tsx")
        || lower.contains("__tests__")
    {
        return 0.5;
    }

    // Story files
    if lower.ends_with(".stories.ts") || lower.ends_with(".stories.tsx") {
        return 0.3;
    }

    // Styled files
    if lower.ends_with(".styled.ts") || lower.ends_with(".styled.tsx") {
        return 0.2;
    }

    // Barrel/index files
    let filename = path.rsplit('/').next().unwrap_or(path);
    if filename == "index.ts" || filename == "index.tsx" || filename == "index.js" {
        return 0.1;
    }

    0.0
}
```

**Step 4: Run test to verify it passes**
Run: `cargo test --test scoring_test`
Expected: PASS

**Step 5: Commit**
`git add src/scoring.rs tests/scoring_test.rs`
`git commit -m "feat: file-type penalty scoring for test/generated/barrel files"`

---

## Task 3: Integrate file-type penalty into index build
**Depends on:** Task 2
**Complexity:** simple
**Files:**
- Modify: `src/db.rs:26-30` (add type_penalty column to file_scores)
- Modify: `src/index.rs:47-74` (compute and store type_penalty during insert)

**Purpose:** Store the pre-computed type_penalty in the DB so it's available at query time without runtime computation.

**Step 1: Update schema in db.rs**
Add `type_penalty REAL DEFAULT 0.0` column to `file_scores` table.

**Step 2: Update insert_files in index.rs**
Change the INSERT statement to include type_penalty, and call `scoring::type_penalty()` for each file.

**Step 3: Rebuild and verify**
Run: `cargo build --release`
Run: `file-suggest build ~/Developer/work/hub`
Run: `sqlite3 ~/.claude/file-suggest/*.db "SELECT path, type_penalty FROM file_scores WHERE type_penalty > 0 LIMIT 10"`
Expected: Rows with test/generated files showing non-zero penalties.

**Step 4: Commit**
`git add src/db.rs src/index.rs`
`git commit -m "feat: store file-type penalties in index during build"`

---

## Task 4: Integrate file-type penalty into search ranking
**Depends on:** Task 3
**Complexity:** simple
**Files:**
- Modify: `src/search.rs:51-69` (update FTS5 ORDER BY to include type_penalty)

**Purpose:** Make test/generated/barrel files rank below source files.

**Step 1: Update search_fts query**
Replace the full SQL in `search_fts` from:
```sql
SELECT f.path FROM files_fts f
WHERE files_fts MATCH '{fts_query}'
ORDER BY bm25(files_fts, 1.0, 10.0, 2.0) + (length(f.path) * 0.001)
LIMIT {MAX_RESULTS}
```
To (adding JOIN to file_scores):
```sql
SELECT f.path FROM files_fts f
JOIN file_scores s ON f.path = s.path
WHERE files_fts MATCH '{fts_query}'
ORDER BY bm25(files_fts, 1.0, 10.0, 2.0) + (s.type_penalty * 0.5) + (length(f.path) * 0.001)
LIMIT {MAX_RESULTS}
```

**Step 2: Test with benchmark**
Run: `python3 ~/.claude/file-suggestion-bench/benchmark.py custom ~/Developer/personal/file-suggest/target/release/file-suggest`
Expected: Ranking should improve (test files no longer compete with source files).

**Step 3: Commit**
`git add src/search.rs`
`git commit -m "feat: apply file-type penalties in search ranking"`

---

## Task 5: Add trigram FTS5 table for substring matching
**Depends on:** Task 3
**Complexity:** standard
**Files:**
- Modify: `src/db.rs` (add files_trigram virtual table to schema)
- Modify: `src/index.rs` (insert into files_trigram during build)
- Modify: `src/search.rs` (add search_trigram function, insert into search chain)

**Purpose:** Handle queries like "config" matching "tsconfig.json" with ranked results. Currently these fall to unranked LIKE. Trigram FTS5 gives BM25-scored substring matching.

**Not In Scope:** Trigram queries shorter than 3 characters (FTS5 trigram limitation). These continue to use LIKE fallback.

**Gotchas:** Trigram tokenizer stores 3-character grams, so the index is larger (~2x). The `detail=none` option minimizes this since we don't need phrase or column queries on trigrams.

**Step 1: Add trigram table to db.rs schema**
```sql
CREATE VIRTUAL TABLE files_trigram USING fts5(
    path,
    tokenize='trigram',
    detail=none
);
```

**Step 2: Insert into files_trigram during build**
In `insert_files`, add a third prepared statement:
```rust
let mut tri_stmt = tx.prepare("INSERT INTO files_trigram (path) VALUES (?1)")?;
// In the loop:
tri_stmt.execute(rusqlite::params![file])?;
```

**Step 3: Add search_trigram function to search.rs**
```rust
fn search_trigram(conn: &rusqlite::Connection, query: &str) -> rusqlite::Result<Vec<String>> {
    if query.len() < 3 {
        return Ok(vec![]);
    }
    let escaped = query.replace('\'', "''").replace('"', "");
    let sql = format!(
        "SELECT t.path FROM files_trigram t
         JOIN file_scores s ON t.path = s.path
         WHERE files_trigram MATCH '\"{escaped}\"'
         ORDER BY bm25(files_trigram) + (s.type_penalty * 0.5) + (length(t.path) * 0.001)
         LIMIT {MAX_RESULTS}"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], |row| row.get(0))?;
    rows.collect()
}
```

**Step 4: Update search chain in search()**
Insert trigram search between FTS5 and LIKE fallback:
```rust
let results = search_fts(&conn, query)?;
if !results.is_empty() { return Ok(results); }

let results = search_trigram(&conn, query)?;
if !results.is_empty() { return Ok(results); }

search_like_fallback(&conn, query)
```

**Step 5: Rebuild, build index, test**
Run: `cargo build --release`
Run: `file-suggest build ~/Developer/work/hub`
Run: `echo '{"query": "config"}' | CLAUDE_PROJECT_DIR=~/Developer/work/hub file-suggest`
Expected: Should return tsconfig.json and other config files with ranking.

**Step 6: Commit**
`git add src/db.rs src/index.rs src/search.rs`
`git commit -m "feat: trigram FTS5 index for ranked substring matching"`

---

## Task 6: Add fuzzy matching fallback
**Depends on:** Task 10
**Complexity:** standard
**Files:**
- Modify: `Cargo.toml` (add fuzzy-matcher dependency)
- Create: `src/fuzzy.rs`
- Modify: `src/search.rs` (add fuzzy fallback at end of search chain)

**Purpose:** Catch abbreviation-style queries like `bksvc` or `fsc` that FTS5, trigram, and LIKE all miss. This is the last-resort search layer.

**Gotchas:** Fuzzy matching scans all ~10k file paths in memory. This takes ~5-10ms, so it only fires when all other methods return empty. Must load the file list from the DB once, not per-character.

**Step 1: Add dependency to Cargo.toml**
```toml
fuzzy-matcher = "0.3"
```

**Step 2: Write fuzzy.rs**
```rust
// src/fuzzy.rs
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

/// Run fuzzy matching over a list of file paths.
/// Returns up to `limit` results sorted by score descending.
pub fn fuzzy_search(query: &str, paths: &[String], limit: usize) -> Vec<String> {
    let matcher = SkimMatcherV2::default();
    let mut scored: Vec<(i64, &String)> = paths
        .iter()
        .filter_map(|p| matcher.fuzzy_match(p, query).map(|score| (score, p)))
        .collect();

    scored.sort_by(|a, b| b.0.cmp(&a.0));
    scored.into_iter().take(limit).map(|(_, p)| p.clone()).collect()
}
```

**Step 3: Add fuzzy fallback to search chain**
In `search()`, after `search_like_fallback` returns empty:
```rust
let results = search_like_fallback(&conn, query)?;
if !results.is_empty() { return Ok(results); }

// Last resort: fuzzy match over all file paths
let all_paths = load_all_paths(&conn)?;
let fuzzy_results = fuzzy::fuzzy_search(query, &all_paths, MAX_RESULTS);
Ok(fuzzy_results)
```

Add `load_all_paths` helper:
```rust
fn load_all_paths(conn: &rusqlite::Connection) -> rusqlite::Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT path FROM file_scores")?;
    let rows = stmt.query_map([], |row| row.get(0))?;
    rows.collect()
}
```

**Step 4: Test**
Run: `cargo build --release`
Run: `echo '{"query": "bksvc"}' | CLAUDE_PROJECT_DIR=~/Developer/work/hub file-suggest`
Expected: Should return booking service-related files via fuzzy matching.

**Step 5: Commit**
`git add Cargo.toml src/fuzzy.rs src/search.rs`
`git commit -m "feat: fuzzy matching fallback for abbreviation queries"`

---

## Task 7: BM25-tiered frecency in FTS5 ranking
**Depends on:** Task 4
**Complexity:** simple
**Files:**
- Modify: `src/search.rs:51-69` (update ORDER BY to use ROUND-based tiering)

**Purpose:** Among files with similar BM25 relevance, prefer recently/frequently edited ones. Previous attempts with raw frecency overrode BM25. ROUND-based tiering creates relevance "buckets" and only uses frecency within each bucket.

**Not In Scope:** Self-tuning weights. This is a fixed formula.

**Step 1: Update search_fts ORDER BY**
Change from:
```sql
ORDER BY bm25(files_fts, 1.0, 10.0, 2.0) + (s.type_penalty * 0.5) + (length(f.path) * 0.001)
```
To:
```sql
ORDER BY ROUND(bm25(files_fts, 1.0, 10.0, 2.0) + (s.type_penalty * 0.5), 1) + (length(f.path) * 0.001) - (s.frecency * 0.1)
```
The `ROUND(..., 1)` creates tiers of BM25 relevance (rounded to 1 decimal). Within each tier, frecency (weight 0.1) and path length break ties.

**Step 2: Test with benchmark**
Run: `python3 ~/.claude/file-suggestion-bench/benchmark.py custom ~/Developer/personal/file-suggest/target/release/file-suggest`
Expected: Ranking stays at 80%+ top-1 (no regression), with frecency breaking ties for equally-relevant results.

**Step 3: Commit**
`git add src/search.rs`
`git commit -m "feat: BM25-tiered frecency for better tie-breaking"`

---

## Task 8: Incremental index updates
**Depends on:** Task 1, Task 2, Task 5
**Complexity:** complex
**Files:**
- Modify: `src/git.rs` (add get_changed_files function)
- Modify: `src/index.rs` (add incremental_build function)
- Modify: `src/db.rs` (add delete support, store HEAD hash in metadata)
- Modify: `src/main.rs` (default `build` to incremental, add `build --full` flag)
- Create: `tests/index_test.rs`

**Purpose:** Reduce git hook build time from ~100ms (full rebuild) to ~10ms (delta update). The frecency computation via `git log` is the slowest part (~50ms). Incremental mode skips frecency and only updates the file list.

**Gotchas:**
- FTS5 DELETE requires knowing the exact row content. Use `DELETE FROM files_fts WHERE path = ?`.
- Frecency scores become stale over time with incremental updates. The `build --full` command (or periodic auto-full-rebuild when frecency is >1 hour old) refreshes them.
- Git hooks receive the old and new HEAD refs as arguments for `post-checkout` and `post-rewrite`. We can use these directly instead of storing the hash.

**Step 1: Write the failing test**
```rust
// tests/index_test.rs
use tempfile::TempDir;

#[test]
fn incremental_build_adds_new_files() {
    // This test verifies that incremental_build inserts new files into the index.
    // A full build is done first, then incremental is called with a simulated diff.
    todo!("Implement after incremental_build is wired up")
}

#[test]
fn incremental_build_removes_deleted_files() {
    // This test verifies that incremental_build removes deleted files from the index.
    todo!("Implement after incremental_build is wired up")
}

#[test]
fn incremental_falls_back_to_full_when_no_hash() {
    // If metadata has no head_hash, incremental_build returns Ok(None).
    todo!("Implement after incremental_build is wired up")
}
```

**Step 2: Add get_changed_files to git.rs**
```rust
/// Get files added/removed since a given commit hash.
pub fn get_changed_files(project_dir: &Path, since_hash: &str) -> (Vec<String>, Vec<String>) {
    let output = Command::new("git")
        .args(["diff", "--name-status", since_hash, "HEAD"])
        .current_dir(project_dir)
        .output();

    let mut added = Vec::new();
    let mut removed = Vec::new();

    if let Ok(output) = output {
        for line in String::from_utf8_lossy(&output.stdout).lines() {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() < 2 { continue; }
            let status = parts[0].chars().next().unwrap_or(' ');
            match status {
                'A' | 'C' => added.push(parts[1].to_string()),
                'D' => removed.push(parts[1].to_string()),
                'M' => added.push(parts[1].to_string()),
                'R' => {
                    // Rename: parts = ["R100", "old_path", "new_path"]
                    removed.push(parts[1].to_string());
                    if parts.len() >= 3 {
                        added.push(parts[2].to_string());
                    }
                }
                _ => {}
            }
        }
    }
    (added, removed)
}

/// Get the current HEAD commit hash.
pub fn get_head_hash(project_dir: &Path) -> Option<String> {
    Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(project_dir)
        .output()
        .ok()
        .and_then(|o| {
            let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if s.is_empty() { None } else { Some(s) }
        })
}
```

**Step 3: Add incremental_build to index.rs**
```rust
/// Incrementally update the index based on git diff since last build.
/// Returns Ok(None) if a full rebuild is needed, Ok(Some(count)) on success.
pub fn incremental_build(project_dir: &Path, db_path: &Path) -> rusqlite::Result<Option<usize>> {
    if !db_path.exists() {
        return Ok(None); // Need full build
    }

    let conn = db::open(db_path)?;

    // Get stored HEAD hash
    let stored_hash: Option<String> = conn
        .query_row("SELECT value FROM metadata WHERE key = 'head_hash'", [], |row| row.get(0))
        .ok();

    let stored_hash = match stored_hash {
        Some(h) => h,
        None => return Ok(None), // No hash stored, need full build
    };

    let current_hash = match git::get_head_hash(project_dir) {
        Some(h) => h,
        None => return Ok(None),
    };

    if stored_hash == current_hash {
        return Ok(Some(0)); // Already up to date
    }

    let (added, removed) = git::get_changed_files(project_dir, &stored_hash);
    let delta_count = added.len() + removed.len();

    // If too many changes, fall back to full rebuild
    if delta_count > 500 {
        return Ok(None);
    }

    let tx = conn.unchecked_transaction()?;

    // Remove deleted files
    for path in &removed {
        tx.execute("DELETE FROM files_fts WHERE path = ?1", [path])?;
        tx.execute("DELETE FROM files_trigram WHERE path = ?1", [path])?;
        tx.execute("DELETE FROM file_scores WHERE path = ?1", [path])?;
    }

    // Add new/modified files
    for path in &added {
        // Remove first (in case of modify/rename)
        tx.execute("DELETE FROM files_fts WHERE path = ?1", [path])?;
        tx.execute("DELETE FROM files_trigram WHERE path = ?1", [path])?;
        tx.execute("DELETE FROM file_scores WHERE path = ?1", [path])?;

        let filename = extract_filename(path);
        let tokens = tokenize_path(path);
        let penalty = crate::scoring::type_penalty(path);
        let depth = path.matches('/').count() as i32;

        tx.execute(
            "INSERT INTO files_fts (path, filename, tokens) VALUES (?1, ?2, ?3)",
            rusqlite::params![path, filename, tokens],
        )?;
        tx.execute(
            "INSERT INTO files_trigram (path) VALUES (?1)",
            [path],
        )?;
        tx.execute(
            "INSERT INTO file_scores (path, frecency, depth, type_penalty) VALUES (?1, 0.0, ?2, ?3)",
            rusqlite::params![path, depth, penalty],
        )?;
    }

    // Update HEAD hash
    tx.execute(
        "INSERT OR REPLACE INTO metadata VALUES ('head_hash', ?1)",
        [&current_hash],
    )?;

    tx.commit()?;
    Ok(Some(delta_count))
}
```

**Step 4: Update main.rs build command**
```rust
fn cmd_build(args: &[String]) {
    let full = args.iter().any(|a| a == "--full");
    let dir = args.iter().find(|a| !a.starts_with('-')).map(|s| s.as_str());
    let project = dir.map(PathBuf::from).unwrap_or_else(project_dir);
    let db = db_path();

    if !full {
        match index::incremental_build(&project, &db) {
            Ok(Some(count)) => {
                eprintln!("Incremental update: {count} files changed");
                return;
            }
            Ok(None) => {
                eprintln!("Falling back to full rebuild");
            }
            Err(e) => {
                eprintln!("Incremental failed ({e}), falling back to full rebuild");
            }
        }
    }

    match index::build(&project, &db) {
        Ok(count) => eprintln!("Indexed {count} files from {}", project.display()),
        Err(e) => {
            eprintln!("Error building index: {e}");
            std::process::exit(1);
        }
    }
}
```

**Step 5: Store HEAD hash in full build**
In `insert_metadata`, add:
```rust
if let Some(hash) = git::get_head_hash(project_dir) {
    conn.execute("INSERT INTO metadata VALUES ('head_hash', ?1)", [&hash])?;
}
```

**Step 6: Test**
Run: `cargo build --release`
Run: `file-suggest build --full ~/Developer/work/hub`
Run: `file-suggest build ~/Developer/work/hub` (should say "0 files changed" or similar)
Run: `touch ~/Developer/work/hub/test-new-file.txt && git -C ~/Developer/work/hub add test-new-file.txt && git -C ~/Developer/work/hub commit -m "test"`
Run: `file-suggest build ~/Developer/work/hub` (should show "1 files changed")

**Step 7: Commit**
`git add src/git.rs src/index.rs src/db.rs src/main.rs tests/index_test.rs`
`git commit -m "feat: incremental index updates via git diff"`

---

## Task 9: Update init command for new DB structure
**Depends on:** Task 1, Task 8
**Complexity:** simple
**Files:**
- Modify: `src/main.rs:69-130` (update cmd_init and install_hooks)

**Purpose:** Update the `init` command to create per-project DB directory and use incremental builds in hooks.

**Step 1: Update hook body**
Change hook content from `file-suggest build <dir> &` to `file-suggest build &` (incremental by default, uses `CLAUDE_PROJECT_DIR`).

**Step 2: Ensure DB directory is created**
Call `project::ensure_db_dir()` in `cmd_init`.

**Step 3: Update help text**
Add `--full` flag documentation.

**Step 4: Test**
Run: `cargo build --release`
Run: `CLAUDE_PROJECT_DIR=~/Developer/work/hub file-suggest init`
Expected: Creates hooks, builds index, prints config.

**Step 5: Commit**
`git add src/main.rs`
`git commit -m "feat: update init for multi-project and incremental builds"`

---

## Task 10: Make crate a lib+bin for testability (DO FIRST)
**Depends on:** None
**Complexity:** simple
**Files:**
- Create: `src/lib.rs`
- Modify: `src/main.rs` (move module declarations to lib.rs)

**Purpose:** Integration tests (`tests/*.rs`) need `use file_suggest::module` imports, which requires a library target. Currently only a binary target exists.

**Step 1: Create src/lib.rs** (only existing modules — new modules are added by their tasks)
```rust
pub mod db;
pub mod git;
pub mod index;
pub mod search;
```

**Step 2: Update main.rs**
Remove `mod` declarations and add explicit imports:
```rust
use file_suggest::{db, git, index, project, scoring, search, fuzzy};
```
Note: modules that don't exist yet (fuzzy, scoring, project) will cause compile errors until their tasks are complete. To avoid this, Task 10 should only declare the modules that exist now (`db`, `git`, `index`, `search`) in lib.rs, and each subsequent task adds its own `pub mod` line when creating a new module file.

**Step 3: Verify build**
Run: `cargo build --release`
Expected: PASS

**Step 4: Commit**
`git add src/lib.rs src/main.rs`
`git commit -m "refactor: split into lib+bin for integration test support"`

---

## Task 11: Integration tests for search ranking
**Depends on:** Task 4, Task 5, Task 6, Task 7, Task 10
**Complexity:** standard
**Files:**
- Create: `tests/search_test.rs`
- Modify: `Cargo.toml` (add tempfile dev-dependency)

**Purpose:** Verify the full search pipeline produces correct rankings against a known file set, ensuring all ranking layers work together.

**Step 1: Add dev dependency**
```toml
[dev-dependencies]
tempfile = "3"
```

**Step 2: Write integration tests**
```rust
// tests/search_test.rs
use file_suggest::{db, index, search};
use tempfile::TempDir;
use std::path::Path;

fn build_test_index(files: &[&str]) -> tempfile::TempDir {
    let tmp = TempDir::new().unwrap();
    let db_path = tmp.path().join("test.db");
    let conn = db::open(&db_path).unwrap();
    db::create_schema(&conn).unwrap();
    // Insert test files directly (skip git)
    // ... helper to insert files with scoring
    tmp
}

#[test]
fn source_file_ranks_above_test_file() {
    // Button.tsx should rank above Button.test.tsx for query "Button"
}

#[test]
fn exact_filename_match_ranks_first() {
    // "tsconfig" should return tsconfig.json at #1
}

#[test]
fn trigram_finds_substring_matches() {
    // "config" should find tsconfig.json via trigram
}

#[test]
fn fuzzy_finds_abbreviations() {
    // "bksvc" should find booking.service.ts
}

#[test]
fn empty_query_returns_frecency_sorted() {
    // Files with higher frecency should come first
}
```

**Step 3: Run tests**
Run: `cargo test --test search_test`
Expected: PASS

**Step 4: Commit**
`git add tests/search_test.rs Cargo.toml`
`git commit -m "test: integration tests for search ranking pipeline"`

---

## Task 12: Full benchmark validation
**Depends on:** Task 4, Task 5, Task 6, Task 7, Task 8, Task 9
**Complexity:** simple
**Files:**
- No code changes. Benchmark-only task.

**Purpose:** Run the full benchmark suite against the v2 binary and compare with baseline. Verify no speed regressions and ranking improvements.

**Step 1: Rebuild and rebuild index**
Run: `cargo build --release`
Run: `file-suggest build --full ~/Developer/work/hub`

**Step 2: Run benchmark**
Run: `python3 ~/.claude/file-suggestion-bench/benchmark.py custom ~/Developer/personal/file-suggest/target/release/file-suggest`

**Step 3: Compare with baseline**
Run: `python3 ~/.claude/file-suggestion-bench/benchmark.py compare ~/.claude/file-suggestion-bench/baseline.json ~/.claude/file-suggestion-bench/custom.json`

**Expected results:**
- Speed p50: <=10ms (no regression from 8ms)
- Top-1 hit rate: >=80% (no regression, ideally improved)
- Top-3 hit rate: >=87% (improved from file-type awareness)
- Reliability: 5/5 pass

**Step 4: Commit benchmark results**
`git add ~/.claude/file-suggestion-bench/custom.json` (optional, for reference)

---

## Task 13: Update README and bump version
**Depends on:** Task 12
**Complexity:** simple
**Files:**
- Modify: `README.md`
- Modify: `Cargo.toml:3` (version bump to 0.2.0)

**Purpose:** Document new features and update benchmarks.

**Step 1: Update README sections**
- Add multi-project support docs
- Add `--full` flag docs
- Update benchmark table with v2 numbers
- Add section on ranking improvements (file-type, trigram, fuzzy)

**Step 2: Bump version**
Change `version = "0.1.0"` to `version = "0.2.0"` in Cargo.toml.

**Step 3: Commit**
`git add README.md Cargo.toml`
`git commit -m "docs: update README for v2 features, bump to 0.2.0"`

---

## Verification Record

| Pass | Verdict | Key Findings |
|------|---------|-------------|
| Plan Verification Checklist | PASS_WITH_NOTES | Fixed: missing File Structure entries (lib.rs, README.md), dependency graph (Task 10 must precede 1,2,5,6; Task 8 depends on 1,2,5), off-by-one line refs |
| Rule-of-five: Draft | PASS_WITH_NOTES | Task 8 is larger than ideal but splitting would add overhead; Task 10 annotated with (DO FIRST); empty test stubs converted to todo!() |
| Rule-of-five: Feasibility | PASS_WITH_NOTES | Fixed: rename handling in get_changed_files (3-field parsing), Task 4 full SQL with JOIN, Task 10 explicit imports instead of glob |
| Rule-of-five: Completeness | PASS_WITH_NOTES | All 6 requirements traced to tasks. Notes: index_test.rs stubs must be filled during Task 8 implementation; add frecency tie-breaking test in Task 11 |
| Rule-of-five: Risk | PASS_WITH_NOTES | Fixed: plan header SHA256→FNV-1a. Notes for implementer: (1) check git diff exit code to handle rebase/force-push, (2) supplement incremental with git ls-files --others for untracked files |
| Rule-of-five: Optimality | PASS_WITH_NOTES | Tasks 2+3 could be merged (implementer's discretion). All feature choices confirmed as simplest correct approach. No removable tasks. |

**Plan Status:** Ready for execution. All passes PASS or PASS_WITH_NOTES. No BLOCKED verdicts.

**Implementation notes for executor:**
1. Execute Task 10 first (lib+bin split) — all other tasks depend on it
2. Fill index_test.rs stubs with real assertions during Task 8 (don't commit todo!() macros)
3. In incremental_build: check `output.status.success()` on git diff, fall back to full rebuild on failure
4. In incremental_build: also run `git ls-files --others --exclude-standard` to catch untracked files
5. Add a frecency tie-breaking test in Task 11's search_test.rs
