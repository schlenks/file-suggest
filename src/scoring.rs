/// Returns true if the path is a generated file.
fn is_generated(lower: &str) -> bool {
    lower.contains("/generated/") || lower.contains("/gen/")
}

/// Returns true if the path is a snapshot file.
fn is_snapshot(lower: &str) -> bool {
    lower.ends_with(".snap") || lower.contains("__snapshots__")
}

/// Returns true if the path is a test file.
fn is_test(lower: &str) -> bool {
    lower.ends_with(".test.ts")
        || lower.ends_with(".test.tsx")
        || lower.ends_with(".spec.ts")
        || lower.ends_with(".spec.tsx")
        || lower.contains("__tests__")
}

/// Returns true if the path is a story file.
fn is_story(lower: &str) -> bool {
    lower.ends_with(".stories.ts") || lower.ends_with(".stories.tsx")
}

/// Returns true if the path is a styled file.
fn is_styled(lower: &str) -> bool {
    lower.ends_with(".styled.ts") || lower.ends_with(".styled.tsx")
}

/// Returns true if the path is a barrel/index file.
fn is_barrel(path: &str) -> bool {
    let filename = path.rsplit('/').next().unwrap_or(path);
    filename == "index.ts" || filename == "index.tsx" || filename == "index.js"
}

/// Compute a type-based penalty for a file path.
/// Higher penalty = ranked lower. Source files get 0.0.
pub fn type_penalty(path: &str) -> f64 {
    let lower = path.to_lowercase();

    if is_generated(&lower) {
        return 1.0;
    }
    if is_snapshot(&lower) {
        return 0.8;
    }
    if is_test(&lower) {
        return 0.5;
    }
    if is_story(&lower) {
        return 0.3;
    }
    if is_styled(&lower) {
        return 0.2;
    }
    if is_barrel(path) {
        return 0.1;
    }

    0.0
}
