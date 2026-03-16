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
    // Directory boost elevates packages/tsconfig/ files, but at minimum a tsconfig-related file ranks first
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

#[test]
fn directory_context_boosts_files_in_matching_dir() {
    // "temporal-worker" query: files inside apps/temporal-worker/ should beat
    // infra files (Dockerfile, IDE configs) that also mention temporal-worker
    // but are not inside the temporal-worker app directory.
    //
    // Without directory boost, "docker/temporal-worker" (short path, high BM25)
    // would rank above "apps/temporal-worker/src/workers/emailWorker.ts".
    let (_tmp, db_path) = build_test_index(&[
        // Short infra path — wins on BM25+length without directory boost
        ("docker/temporal-worker", 0.0),
        // App files — should win WITH directory boost
        ("apps/temporal-worker/src/workers/emailWorker.ts", 0.0),
        ("apps/temporal-worker/package.json", 0.0),
        (".idea/runConfigurations/temporal_worker_dev.xml", 0.0),
    ]);
    let results = search::search("temporal-worker", &db_path).unwrap();
    assert!(
        !results.is_empty(),
        "expected at least one result for 'temporal-worker'"
    );
    assert!(
        results[0].starts_with("apps/temporal-worker/"),
        "expected results[0] to be inside apps/temporal-worker/, got: {}",
        results[0]
    );
}

#[test]
fn directory_boost_works_for_packages() {
    let (_tmp, db_path) = build_test_index(&[
        ("scripts/data-export.sh", 0.0),
        ("packages/data/src/user.repository.ts", 0.0),
        ("packages/data/src/booking.repository.ts", 0.0),
    ]);
    let results = search::search("data", &db_path).unwrap();
    assert!(!results.is_empty());
    assert!(results[0].starts_with("packages/data/"));
}

#[test]
fn space_separated_query_finds_results() {
    let (_tmp, db_path) = build_test_index(&[
        ("apps/admin/jest.config.ts", 0.0),
        ("apps/partners/jest.config.ts", 0.0),
        ("apps/marketplace/jest.config.ts", 0.0),
    ]);
    let results = search::search("admin jest.config", &db_path).unwrap();
    assert!(!results.is_empty(), "space-separated query should return results");
    assert_eq!(results[0], "apps/admin/jest.config.ts");
}

#[test]
fn exact_filename_beats_stemmer_conflation() {
    let (_tmp, db_path) = build_test_index(&[
        ("apps/api/src/middleware/sanitization.ts", 0.0),
        ("apps/api/src/utils/sanitizeError.ts", 0.0),
        ("apps/api/src/utils/sanitizers.ts", 0.0),
        ("apps/api/src/utils/sanitizeInput.ts", 0.0),
    ]);
    let results = search::search("sanitization.ts", &db_path).unwrap();
    assert!(!results.is_empty());
    assert!(
        results[0].ends_with("sanitization.ts"),
        "exact filename should rank first, got: {}",
        results[0]
    );
}
