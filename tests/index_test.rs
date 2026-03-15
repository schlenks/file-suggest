use file_suggest::{db, incremental, index};
use tempfile::TempDir;

/// Helper: create a DB with known files via direct insertion.
fn build_test_db(files: &[&str]) -> (TempDir, std::path::PathBuf) {
    let tmp = TempDir::new().unwrap();
    let db_path = tmp.path().join("test.db");
    let conn = db::open(&db_path).unwrap();
    db::create_schema(&conn).unwrap();

    let tx = conn.unchecked_transaction().unwrap();
    for file in files {
        let filename = index::extract_filename(file);
        let tokens = index::tokenize_path(file);
        tx.execute(
            "INSERT INTO files_fts (path, filename, tokens) VALUES (?1, ?2, ?3)",
            rusqlite::params![file, filename, tokens],
        )
        .unwrap();
        tx.execute("INSERT INTO files_trigram (path) VALUES (?1)", [file])
            .unwrap();
        tx.execute(
            "INSERT INTO file_scores (path, frecency, depth, type_penalty) VALUES (?1, 0.0, 0, 0.0)",
            [file],
        )
        .unwrap();
    }
    // Store a fake head_hash so incremental knows we have a baseline
    tx.execute(
        "INSERT INTO metadata VALUES ('head_hash', 'abc123')",
        [],
    )
    .unwrap();
    tx.commit().unwrap();
    db::optimize(&conn).unwrap();

    (tmp, db_path)
}

#[test]
fn incremental_falls_back_to_full_when_no_db() {
    let tmp = TempDir::new().unwrap();
    let db_path = tmp.path().join("nonexistent.db");
    // Non-existent DB should return None (need full build)
    let result = incremental::incremental_build(std::path::Path::new("."), &db_path).unwrap();
    assert!(result.is_none());
}

#[test]
fn incremental_falls_back_when_no_hash() {
    let tmp = TempDir::new().unwrap();
    let db_path = tmp.path().join("test.db");
    let conn = db::open(&db_path).unwrap();
    db::create_schema(&conn).unwrap();
    drop(conn);
    // DB exists but no head_hash in metadata -> None
    let result = incremental::incremental_build(std::path::Path::new("."), &db_path).unwrap();
    assert!(result.is_none());
}

#[test]
fn test_db_has_expected_files() {
    let (_tmp, db_path) = build_test_db(&["src/main.rs", "src/lib.rs"]);
    let conn = db::open(&db_path).unwrap();
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM file_scores", [], |row| row.get(0))
        .unwrap();
    assert_eq!(count, 2);
}
