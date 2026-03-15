use crate::db;
use rusqlite::params;
use std::path::Path;

const MAX_RESULTS: usize = 15;

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
        return Ok(results);
    }

    let results = search_trigram(&conn, query)?;
    if !results.is_empty() {
        return Ok(results);
    }

    search_like_fallback(&conn, query)
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
         LIMIT {MAX_RESULTS}"
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
