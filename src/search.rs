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
        let matching_dirs = find_matching_dirs(&conn, query)?;
        let boosted = apply_directory_boost(results, &matching_dirs);
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
fn search_path_prefix(conn: &rusqlite::Connection, query: &str) -> rusqlite::Result<Vec<String>> {
    let pattern = format!("{}%", query.replace('\'', "''"));
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

/// Find well-known directory prefixes (`apps/X/` or `packages/X/`) whose directory name
/// contains any token from the query.
///
/// Uses a subquery to avoid the SQLite limitation where a column alias defined in SELECT
/// cannot be referenced in a WHERE clause at the same level.
fn find_matching_dirs(
    conn: &rusqlite::Connection,
    query: &str,
) -> rusqlite::Result<Vec<String>> {
    let tokens: Vec<&str> = query
        .split(|c: char| c == '/' || c == '.' || c == '_' || c == '-')
        .filter(|t| !t.is_empty())
        .collect();

    if tokens.is_empty() {
        return Ok(vec![]);
    }

    // Query distinct `apps/X/` and `packages/X/` prefixes from file_scores.
    // The subquery extracts the root prefix so the outer WHERE can filter on it.
    let sql = "
        SELECT DISTINCT dir FROM (
            SELECT
                CASE
                    WHEN path LIKE 'apps/%' THEN
                        'apps/' || substr(path, 6, instr(substr(path, 6), '/') - 1) || '/'
                    WHEN path LIKE 'packages/%' THEN
                        'packages/' || substr(path, 10, instr(substr(path, 10), '/') - 1) || '/'
                    ELSE NULL
                END AS dir
            FROM file_scores
        )
        WHERE dir IS NOT NULL AND dir != 'apps//' AND dir != 'packages//'
    ";

    let mut stmt = conn.prepare(sql)?;
    let all_dirs: Vec<String> = stmt
        .query_map([], |row| row.get(0))?
        .collect::<rusqlite::Result<Vec<String>>>()?;

    // Keep only dirs whose directory name (the X part) matches at least one query token.
    let matching: Vec<String> = all_dirs
        .into_iter()
        .filter(|dir| {
            // Extract X from "apps/X/" or "packages/X/"
            let parts: Vec<&str> = dir.trim_end_matches('/').split('/').collect();
            let dir_name = parts.last().unwrap_or(&"");
            // The directory name must contain at least one query token (case-insensitive).
            tokens.iter().any(|tok| {
                dir_name.to_lowercase().contains(&tok.to_lowercase())
            })
        })
        .collect();

    Ok(matching)
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

/// Build an FTS5 MATCH expression from a query string.
/// `booking.service` → `"booking"* AND "service"*`
fn build_fts_query(query: &str) -> String {
    let tokens: Vec<&str> = query
        .split(|c: char| c == '/' || c == '.' || c == '_' || c == '-')
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
