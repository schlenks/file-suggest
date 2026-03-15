use crate::{db, git};
use rusqlite::Connection;
use std::collections::HashMap;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// Tokenize a file path into space-separated segments for FTS5.
pub fn tokenize_path(path: &str) -> String {
    path.replace('/', " ")
        .replace('.', " ")
        .replace('-', " ")
        .replace('_', " ")
}

/// Extract the filename portion, tokenized.
pub fn extract_filename(path: &str) -> String {
    let name = path.rsplit('/').next().unwrap_or(path);
    name.replace('.', " ").replace('-', " ").replace('_', " ")
}

/// Build the full index for a project directory.
pub fn build(project_dir: &Path, db_path: &Path) -> rusqlite::Result<usize> {
    let files = git::get_files(project_dir);
    let frecency = git::get_frecency(project_dir);
    let max_frecency = frecency.values().cloned().fold(1.0_f64, f64::max);

    let tmp_path = db_path.with_extension("tmp");
    let conn = db::open(&tmp_path)?;
    db::create_schema(&conn)?;

    insert_files(&conn, &files, &frecency, max_frecency)?;
    insert_metadata(&conn, project_dir, files.len())?;
    db::optimize(&conn)?;

    conn.close().map_err(|(_, e)| e)?;

    std::fs::rename(&tmp_path, db_path).map_err(|e| {
        rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error::new(1),
            Some(format!("rename failed: {e}")),
        )
    })?;

    Ok(files.len())
}

fn insert_files(
    conn: &Connection,
    files: &[String],
    frecency: &HashMap<String, f64>,
    max_frecency: f64,
) -> rusqlite::Result<()> {
    let tx = conn.unchecked_transaction()?;

    {
        let mut fts_stmt =
            tx.prepare("INSERT INTO files_fts (path, filename, tokens) VALUES (?1, ?2, ?3)")?;
        let mut score_stmt =
            tx.prepare("INSERT INTO file_scores (path, frecency, depth) VALUES (?1, ?2, ?3)")?;

        for file in files {
            let filename = extract_filename(file);
            let tokens = tokenize_path(file);
            let norm_frecency = frecency.get(file.as_str()).unwrap_or(&0.0) / max_frecency;
            let depth = file.matches('/').count() as i32;

            fts_stmt.execute(rusqlite::params![file, filename, tokens])?;
            score_stmt.execute(rusqlite::params![file, norm_frecency, depth])?;
        }
    }

    tx.commit()?;
    Ok(())
}

fn insert_metadata(
    conn: &Connection,
    project_dir: &Path,
    file_count: usize,
) -> rusqlite::Result<()> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    conn.execute(
        "INSERT INTO metadata VALUES ('project_dir', ?1)",
        [project_dir.to_string_lossy().as_ref()],
    )?;
    conn.execute(
        "INSERT INTO metadata VALUES ('file_count', ?1)",
        [&file_count.to_string()],
    )?;
    conn.execute(
        "INSERT INTO metadata VALUES ('built_at', ?1)",
        [&now.to_string()],
    )?;

    Ok(())
}
