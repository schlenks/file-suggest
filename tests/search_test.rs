use file_suggest::{db, fuzzy, index, scoring, search};
use tempfile::TempDir;

fn build_test_index(files: &[(&str, f64)]) -> (TempDir, std::path::PathBuf) {
    let tmp = TempDir::new().unwrap();
    let db_path = tmp.path().join("test.db");
    let conn = db::open(&db_path).unwrap();
    db::create_schema(&conn).unwrap();

    let tx = conn.unchecked_transaction().unwrap();
    for (file, frecency) in files {
        let filename = index::extract_filename(file);
        let tokens = index::tokenize_path(file);
        let penalty = scoring::type_penalty(file);
        let depth = file.matches('/').count() as i32;
        tx.execute(
            "INSERT INTO files_fts (path, filename, tokens) VALUES (?1, ?2, ?3)",
            rusqlite::params![file, filename, tokens],
        )
        .unwrap();
        tx.execute("INSERT INTO files_trigram (path) VALUES (?1)", [file])
            .unwrap();
        tx.execute(
            "INSERT INTO file_scores (path, frecency, depth, type_penalty) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![file, frecency, depth, penalty],
        )
        .unwrap();
    }
    tx.commit().unwrap();
    db::optimize(&conn).unwrap();

    (tmp, db_path)
}

#[test]
fn source_file_ranks_above_test_file() {
    let (_tmp, db_path) = build_test_index(&[
        ("packages/design-system/src/Button/Button.tsx", 0.0),
        ("packages/design-system/src/Button/Button.test.tsx", 0.0),
    ]);
    let results = search::search("Button", &db_path).unwrap();
    assert!(results.len() >= 2);
    assert_eq!(results[0], "packages/design-system/src/Button/Button.tsx");
}

#[test]
fn exact_filename_match_ranks_first() {
    let (_tmp, db_path) = build_test_index(&[
        ("packages/tsconfig/base.json", 0.0),
        ("tsconfig.json", 0.0),
        ("apps/api/tsconfig.json", 0.0),
    ]);
    let results = search::search("tsconfig", &db_path).unwrap();
    assert!(!results.is_empty());
    // Root tsconfig.json should rank high (shorter path)
    assert!(results[0].contains("tsconfig"));
}

#[test]
fn trigram_finds_substring_matches() {
    let (_tmp, db_path) = build_test_index(&[
        ("tsconfig.json", 0.0),
        ("packages/tsconfig/base.json", 0.0),
        ("src/utils/config.ts", 0.0),
    ]);
    let results = search::search("config", &db_path).unwrap();
    assert!(!results.is_empty());
}

#[test]
fn fuzzy_finds_abbreviations_via_search_pipeline() {
    // "bksvc" won't match FTS5, trigram, or LIKE — falls through to fuzzy
    let (_tmp, db_path) = build_test_index(&[
        ("apps/api/src/domain/booking/services/booking.service.ts", 0.0),
        ("apps/api/src/domain/guide/services/guide.service.ts", 0.0),
        ("packages/data/src/user/user.repository.ts", 0.0),
    ]);
    let results = search::search("bksvc", &db_path).unwrap();
    assert!(!results.is_empty());
    assert!(results[0].contains("booking"));
}

#[test]
fn fuzzy_finds_abbreviations_directly() {
    let paths = vec![
        "apps/api/src/domain/booking/services/booking.service.ts".to_string(),
        "apps/api/src/domain/guide/services/guide.service.ts".to_string(),
        "packages/data/src/user/user.repository.ts".to_string(),
    ];
    let results = fuzzy::fuzzy_search("bksvc", &paths, 5);
    assert!(!results.is_empty());
    assert!(results[0].contains("booking"));
}

#[test]
fn empty_query_returns_frecency_sorted() {
    let (_tmp, db_path) = build_test_index(&[
        ("src/low.ts", 0.1),
        ("src/high.ts", 0.9),
        ("src/mid.ts", 0.5),
    ]);
    let results = search::search("", &db_path).unwrap();
    assert_eq!(results[0], "src/high.ts");
    assert_eq!(results[1], "src/mid.ts");
    assert_eq!(results[2], "src/low.ts");
}

#[test]
fn frecency_breaks_ties_in_fts() {
    // Two files with identical names but different frecency
    let (_tmp, db_path) = build_test_index(&[
        ("apps/admin/src/utils/helper.ts", 0.1),
        ("apps/api/src/utils/helper.ts", 0.9),
    ]);
    let results = search::search("helper", &db_path).unwrap();
    assert!(results.len() >= 2);
    // Higher frecency should rank first
    assert_eq!(results[0], "apps/api/src/utils/helper.ts");
}
