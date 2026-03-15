use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

/// Run fuzzy matching over a list of file paths.
/// Returns up to `limit` results sorted by score descending.
pub fn fuzzy_search(query: &str, paths: &[String], limit: usize) -> Vec<String> {
    let matcher = SkimMatcherV2::default();
    let mut scored: Vec<(i64, &String)> = paths
        .iter()
        .filter_map(|p| matcher.fuzzy_match(p, query).map(|score| (score, p)))
        .collect();

    scored.sort_by(|a, b| b.0.cmp(&a.0));
    scored.into_iter().take(limit).map(|(_, p)| p.clone()).collect()
}
