#[test]
fn source_file_has_zero_penalty() {
    let penalty = file_suggest::scoring::type_penalty("packages/design-system/src/Button/Button.tsx");
    assert_eq!(penalty, 0.0);
}

#[test]
fn test_file_has_penalty() {
    let penalty = file_suggest::scoring::type_penalty("packages/design-system/src/Button/Button.test.tsx");
    assert!(penalty > 0.0);
}

#[test]
fn snapshot_file_has_higher_penalty_than_test() {
    let snap = file_suggest::scoring::type_penalty("src/Button/__snapshots__/Button.test.tsx.snap");
    let test = file_suggest::scoring::type_penalty("src/Button/Button.test.tsx");
    assert!(snap > test);
}

#[test]
fn generated_file_has_penalty() {
    let penalty = file_suggest::scoring::type_penalty("apps/marketplace/graphql/generated/types.ts");
    assert!(penalty > 0.0);
}

#[test]
fn index_barrel_has_small_penalty() {
    let penalty = file_suggest::scoring::type_penalty("packages/design-system/src/Button/index.ts");
    assert!(penalty > 0.0);
    let source = file_suggest::scoring::type_penalty("packages/design-system/src/Button/Button.tsx");
    assert!(penalty > source);
}

#[test]
fn stories_file_has_penalty() {
    let penalty = file_suggest::scoring::type_penalty("src/Button/Button.stories.tsx");
    assert!(penalty > 0.0);
}

#[test]
fn dockerfile_has_penalty() {
    let penalty = file_suggest::scoring::type_penalty("apps/api/Dockerfile");
    assert!(penalty > 0.0);
}

#[test]
fn ide_config_has_high_penalty() {
    let vscode = file_suggest::scoring::type_penalty(".vscode/settings.json");
    assert!(vscode > 0.0);
    let test = file_suggest::scoring::type_penalty("src/test.ts");
    assert!(vscode > test);
}

#[test]
fn lockfile_has_high_penalty() {
    let lockfile = file_suggest::scoring::type_penalty("pnpm-lock.yaml");
    assert!(lockfile > 0.0);
    let test = file_suggest::scoring::type_penalty("src/test.ts");
    assert!(lockfile > test);
}

#[test]
fn build_output_has_max_penalty() {
    let build_output = file_suggest::scoring::type_penalty("dist/index.js");
    assert_eq!(build_output, 1.0);
}

#[test]
fn dot_config_has_small_penalty() {
    let dot_config = file_suggest::scoring::type_penalty(".prettierrc");
    assert!(dot_config > 0.0);
}

#[test]
fn migration_has_penalty() {
    let migration = file_suggest::scoring::type_penalty("migrations/migrations/2024-01-01-add-users.js");
    assert!(migration > 0.0);
}

#[test]
fn type_declaration_has_penalty() {
    let type_decl = file_suggest::scoring::type_penalty("src/types.d.ts");
    assert!(type_decl > 0.0);
}
