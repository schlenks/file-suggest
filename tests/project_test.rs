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
