use crate::{db, fuzzy};
use rusqlite::params;
use std::path::Path;

const MAX_RESULTS: usize = 15;
/// Fetch extra candidates from FTS5 so directory-boost re-ranking has enough candidates.
const FETCH_RESULTS: usize = 30;

/// Search the FTS5 index. Entry point for the fileSuggestion command.
pub fn search(query: &str, db_path: &Path) -> rusqlite::Result<Vec<String>> {
    let conn = db::open(db_path)?;

    if query.is_empty() {
        return search_empty(&conn);
    }

    if query.contains('/') {
        return search_path_prefix(&conn, query);
    }

    let results = search_fts(&conn, query)?;
    if !results.is_empty() {
        let matching_dirs = find_matching_dirs(&results, query);
        let boosted = apply_directory_boost(results, &matching_dirs);
        let boosted = apply_filename_boost(boosted, query);
        return Ok(boosted);
    }

    // Trigram may fail for some queries (phrase limitations with detail=none).
    // Treat errors as empty results and continue to next fallback.
    if let Ok(results) = search_trigram(&conn, query) {
        if !results.is_empty() {
            return Ok(results);
        }
    }

    let results = search_like_fallback(&conn, query)?;
    if !results.is_empty() {
        return Ok(results);
    }

    // Last resort: fuzzy match over all file paths
    let all_paths = load_all_paths(&conn)?;
    Ok(fuzzy::fuzzy_search(query, &all_paths, MAX_RESULTS))
}

/// Empty query: return files sorted by frecency (most recently edited first).
fn search_empty(conn: &rusqlite::Connection) -> rusqlite::Result<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT path FROM file_scores ORDER BY frecency DESC LIMIT ?1",
    )?;
    let rows = stmt.query_map(params![MAX_RESULTS as i64], |row| row.get(0))?;
    rows.collect()
}

/// Path-prefix query: LIKE match sorted by frecency then path length.
/// Promotes index files (index.ts, index.tsx, index.js) to the front.
fn search_path_prefix(conn: &rusqlite::Connection, query: &str) -> rusqlite::Result<Vec<String>> {
    let pattern = format!("{}%", query.replace('\'', "''"));
    let mut stmt = conn.prepare(
        "SELECT f.path FROM files_fts f
         JOIN file_scores s ON f.path = s.path
         WHERE f.path LIKE ?1
         ORDER BY s.frecency DESC, length(f.path)
         LIMIT ?2",
    )?;
    let mut results: Vec<String> = stmt.query_map(params![pattern, MAX_RESULTS as i64], |row| row.get(0))?
        .collect::<Result<Vec<String>, _>>()?;

    // Promote index files to the front
    if let Some(pos) = results.iter().position(|p| {
        let basename = p.rsplit('/').next().unwrap_or("");
        basename == "index.ts" || basename == "index.tsx" || basename == "index.js"
    }) {
        let index_file = results.remove(pos);
        results.insert(0, index_file);
    }

    Ok(results)
}

/// FTS5 search with BM25 ranking + filename boost + short-path tiebreaker.
/// Fetches FETCH_RESULTS candidates so apply_directory_boost has enough to re-rank.
fn search_fts(conn: &rusqlite::Connection, query: &str) -> rusqlite::Result<Vec<String>> {
    let fts_query = build_fts_query(query);
    if fts_query.is_empty() {
        return Ok(vec![]);
    }

    // bm25 weights: path=1.0, filename=10.0, tokens=2.0
    // type_penalty: test/generated/barrel files ranked lower
    // length tiebreaker: shorter paths slightly preferred
    let sql = format!(
        "SELECT f.path FROM files_fts f
         JOIN file_scores s ON f.path = s.path
         WHERE files_fts MATCH '{fts_query}'
         ORDER BY ROUND(bm25(files_fts, 1.0, 10.0, 2.0) + (s.type_penalty * 0.5), 1) + (length(f.path) * 0.001) - (s.frecency * 0.1)
         LIMIT {FETCH_RESULTS}"
    );

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], |row| row.get(0))?;
    rows.collect()
}

/// Trigram substring search for queries like "config" matching "tsconfig.json".
fn search_trigram(conn: &rusqlite::Connection, query: &str) -> rusqlite::Result<Vec<String>> {
    if query.len() < 3 {
        return Ok(vec![]);
    }
    let escaped = query.replace('\'', "''").replace('"', "");
    // No phrase quotes — detail=none doesn't support phrase queries.
    // Unquoted trigram queries check that all query trigrams appear in the path.
    let sql = format!(
        "SELECT t.path FROM files_trigram t
         JOIN file_scores s ON t.path = s.path
         WHERE files_trigram MATCH '{escaped}'
         ORDER BY (s.type_penalty * 0.5) + (length(t.path) * 0.001)
         LIMIT {MAX_RESULTS}"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], |row| row.get(0))?;
    rows.collect()
}

/// Fallback: LIKE search for partial matches FTS5 missed.
fn search_like_fallback(
    conn: &rusqlite::Connection,
    query: &str,
) -> rusqlite::Result<Vec<String>> {
    let pattern = format!("%{}%", query.replace('\'', "''"));
    let mut stmt = conn.prepare(
        "SELECT f.path FROM files_fts f
         JOIN file_scores s ON f.path = s.path
         WHERE f.path LIKE ?1
         ORDER BY s.frecency DESC, length(f.path)
         LIMIT ?2",
    )?;
    let rows = stmt.query_map(params![pattern, MAX_RESULTS as i64], |row| row.get(0))?;
    rows.collect()
}

/// Load all file paths from the DB for fuzzy matching.
fn load_all_paths(conn: &rusqlite::Connection) -> rusqlite::Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT path FROM file_scores")?;
    let rows = stmt.query_map([], |row| row.get(0))?;
    rows.collect()
}

/// Extract the `apps/X/` or `packages/X/` top-level directory prefix from a path,
/// or `None` if the path doesn't start with either prefix.
fn extract_top_level_dir(path: &str) -> Option<String> {
    if let Some(rest) = path.strip_prefix("apps/") {
        rest.find('/').map(|i| format!("apps/{}/", &rest[..i]))
    } else if let Some(rest) = path.strip_prefix("packages/") {
        rest.find('/').map(|i| format!("packages/{}/", &rest[..i]))
    } else {
        None
    }
}

/// Return true if the final component of `dir` (e.g. `api` in `apps/api/`) contains
/// at least one of the provided query tokens.
fn dir_matches_tokens(dir: &str, tokens: &[String]) -> bool {
    let dir_name = dir
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or("")
        .to_lowercase();
    tokens.iter().any(|tok| dir_name.contains(tok.as_str()))
}

/// Find well-known directory prefixes (`apps/X/` or `packages/X/`) whose directory name
/// contains any token from the query.
///
/// Extracts directory prefixes from the FTS5 results we already have, avoiding a full
/// table scan of file_scores. This is semantically equivalent because the boost only
/// reorders files already in the result set — dirs with no files in results are irrelevant.
fn find_matching_dirs(results: &[String], query: &str) -> Vec<String> {
    let tokens: Vec<String> = query
        .split(|c: char| c == '/' || c == '.' || c == '_' || c == '-' || c.is_whitespace())
        .filter(|t| !t.is_empty())
        .map(|t| t.to_lowercase())
        .collect();

    if tokens.is_empty() {
        return vec![];
    }

    let mut dirs: Vec<String> = results
        .iter()
        .filter_map(|path| extract_top_level_dir(path))
        .collect();
    dirs.sort();
    dirs.dedup();
    dirs.retain(|dir| dir_matches_tokens(dir, &tokens));
    dirs
}

/// Re-rank results by boosting files that live inside a matching directory.
///
/// Files in a matching directory are moved to the front of the list, preserving
/// their relative BM25 order among themselves. Files not in any matching directory
/// follow, also preserving their relative order. The final list is truncated to
/// MAX_RESULTS.
fn apply_directory_boost(results: Vec<String>, matching_dirs: &[String]) -> Vec<String> {
    if matching_dirs.is_empty() {
        return results.into_iter().take(MAX_RESULTS).collect();
    }

    let (mut boosted, mut rest): (Vec<String>, Vec<String>) =
        results.into_iter().partition(|path| {
            matching_dirs.iter().any(|dir| path.starts_with(dir.as_str()))
        });

    boosted.append(&mut rest);
    boosted.truncate(MAX_RESULTS);
    boosted
}

/// Re-rank results by boosting files whose basename exactly matches the query.
///
/// Only activates when the query contains a `.` — a signal of filename intent (e.g.
/// `sanitization.ts`, `booking.service`). This prevents the boost from overriding the
/// directory boost for bare directory-style queries like `temporal-worker`.
///
/// Handles queries with and without extensions:
/// - `sanitization.ts` → exact match against basename `sanitization.ts`
/// - `booking.service` → prefix match against `booking.service.ts` (basename starts with `booking.service.`)
///
/// Files with an exact or prefix-exact filename match are moved to the front of the list,
/// preserving their relative order among themselves. Remaining files follow in their
/// existing order. The list is already truncated to MAX_RESULTS by apply_directory_boost,
/// so no further truncation is needed here.
fn apply_filename_boost(results: Vec<String>, query: &str) -> Vec<String> {
    // Only apply when query contains a `.` — signals filename/extension intent.
    // Without this guard, bare queries like `temporal-worker` would incorrectly boost
    // `docker/temporal-worker` over directory-boosted `apps/temporal-worker/` files.
    if !query.contains('.') {
        return results;
    }

    let query_lower = query.to_lowercase();
    let (mut exact, mut rest): (Vec<String>, Vec<String>) =
        results.into_iter().partition(|path| {
            let basename = path.rsplit('/').next().unwrap_or(path).to_lowercase();
            basename == query_lower || basename.starts_with(&format!("{}.", query_lower))
        });
    exact.append(&mut rest);
    exact.truncate(MAX_RESULTS);
    exact
}

/// Build an FTS5 MATCH expression from a query string.
/// `booking.service` → `"booking"* AND "service"*`
fn build_fts_query(query: &str) -> String {
    let tokens: Vec<&str> = query
        .split(|c: char| c == '/' || c == '.' || c == '_' || c == '-' || c.is_whitespace())
        .filter(|t| !t.is_empty())
        .collect();

    tokens
        .iter()
        .map(|t| {
            let sanitized: String = t.chars().filter(|c| *c != '"' && *c != '\'').collect();
            format!("\"{sanitized}\"*")
        })
        .collect::<Vec<_>>()
        .join(" AND ")
}
