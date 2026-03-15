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
