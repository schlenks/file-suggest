use crate::{db, git, index, scoring};
use std::path::Path;

/// Incrementally update the index based on git diff since last build.
/// Returns Ok(None) if a full rebuild is needed, Ok(Some(count)) on success.
pub fn incremental_build(project_dir: &Path, db_path: &Path) -> rusqlite::Result<Option<usize>> {
    if !db_path.exists() {
        return Ok(None);
    }

    let conn = db::open(db_path)?;

    let stored_hash: Option<String> = conn
        .query_row(
            "SELECT value FROM metadata WHERE key = 'head_hash'",
            [],
            |row| row.get(0),
        )
        .ok();

    let stored_hash = match stored_hash {
        Some(h) => h,
        None => return Ok(None),
    };

    let current_hash = match git::get_head_hash(project_dir) {
        Some(h) => h,
        None => return Ok(None),
    };

    if stored_hash == current_hash {
        return Ok(Some(0));
    }

    let (added, removed) = git::get_changed_files(project_dir, &stored_hash);

    // If git diff returned nothing but hashes differ, the old hash may be
    // unreachable (force push/rebase). Fall back to full rebuild.
    if added.is_empty() && removed.is_empty() && stored_hash != current_hash {
        return Ok(None);
    }

    let delta_count = added.len() + removed.len();

    if delta_count > 500 {
        return Ok(None);
    }

    apply_delta(&conn, &added, &removed)?;

    conn.execute(
        "INSERT OR REPLACE INTO metadata VALUES ('head_hash', ?1)",
        [&current_hash],
    )?;

    Ok(Some(delta_count))
}

fn apply_delta(
    conn: &rusqlite::Connection,
    added: &[String],
    removed: &[String],
) -> rusqlite::Result<()> {
    let tx = conn.unchecked_transaction()?;

    for path in removed {
        tx.execute("DELETE FROM files_fts WHERE path = ?1", [path])?;
        tx.execute("DELETE FROM files_trigram WHERE path = ?1", [path])?;
        tx.execute("DELETE FROM file_scores WHERE path = ?1", [path])?;
    }

    for path in added {
        // Remove first (in case of modify/rename)
        tx.execute("DELETE FROM files_fts WHERE path = ?1", [path])?;
        tx.execute("DELETE FROM files_trigram WHERE path = ?1", [path])?;
        tx.execute("DELETE FROM file_scores WHERE path = ?1", [path])?;

        let filename = index::extract_filename(path);
        let tokens = index::tokenize_path(path);
        let penalty = scoring::type_penalty(path);
        let depth = path.matches('/').count() as i32;

        tx.execute(
            "INSERT INTO files_fts (path, filename, tokens) VALUES (?1, ?2, ?3)",
            rusqlite::params![path, filename, tokens],
        )?;
        tx.execute("INSERT INTO files_trigram (path) VALUES (?1)", [path])?;
        tx.execute(
            "INSERT INTO file_scores (path, frecency, depth, type_penalty) VALUES (?1, 0.0, ?2, ?3)",
            rusqlite::params![path, depth, penalty],
        )?;
    }

    tx.commit()?;
    Ok(())
}
