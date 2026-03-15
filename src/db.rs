use rusqlite::{Connection, Result};
use std::path::Path;

/// Open (or create) the database at the given path.
pub fn open(path: &Path) -> Result<Connection> {
    let conn = Connection::open(path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")?;
    Ok(conn)
}

/// Create the FTS5 table and scoring table. Drops existing tables first.
pub fn create_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        DROP TABLE IF EXISTS files_fts;
        CREATE VIRTUAL TABLE files_fts USING fts5(
            path,
            filename,
            tokens,
            tokenize='porter unicode61',
            prefix='2,3',
            detail=column
        );

        DROP TABLE IF EXISTS file_scores;
        CREATE TABLE file_scores (
            path TEXT PRIMARY KEY,
            frecency REAL DEFAULT 0.0,
            depth INTEGER DEFAULT 0
        );

        DROP TABLE IF EXISTS metadata;
        CREATE TABLE metadata (key TEXT PRIMARY KEY, value TEXT);
        ",
    )?;
    Ok(())
}

/// Optimize the FTS5 index (merge all b-trees for fastest reads).
pub fn optimize(conn: &Connection) -> Result<()> {
    conn.execute("INSERT INTO files_fts(files_fts) VALUES('optimize')", [])?;
    Ok(())
}
