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

/// Returns true if the path is a build output file.
fn is_build_output(lower: &str) -> bool {
    lower.starts_with("dist/") || lower.starts_with("build/") || lower.contains("/dist/") || lower.contains("/build/")
}

/// Returns true if the path is an IDE config file.
fn is_ide_config(lower: &str) -> bool {
    lower.starts_with(".vscode/") || lower.starts_with(".idea/") || lower.ends_with("/.vscode/settings.json")
}

/// Returns true if the path is a lockfile.
fn is_lockfile(lower: &str) -> bool {
    lower.ends_with("pnpm-lock.yaml")
        || lower.ends_with("package-lock.json")
        || lower.ends_with("yarn.lock")
}

/// Returns true if the path is a Dockerfile.
fn is_dockerfile(lower: &str) -> bool {
    let filename = lower.rsplit('/').next().unwrap_or(lower);
    filename == "dockerfile" || filename.starts_with("dockerfile.")
}

/// Returns true if the path is a dot config file.
fn is_dot_config(lower: &str) -> bool {
    let filename = lower.rsplit('/').next().unwrap_or(lower);
    filename.starts_with(".")
        && (filename.ends_with("rc") || filename.ends_with("config") || filename.ends_with(".json"))
}

/// Returns true if the path is a migration file.
fn is_migration(lower: &str) -> bool {
    lower.contains("/migrations/")
}

/// Returns true if the path is a type declaration file.
fn is_type_declaration(lower: &str) -> bool {
    lower.ends_with(".d.ts")
}

/// Compute a type-based penalty for a file path.
/// Higher penalty = ranked lower. Source files get 0.0.
pub fn type_penalty(path: &str) -> f64 {
    let lower = path.to_lowercase();

    // Tier 1.0: build_output/generated
    if is_build_output(&lower) {
        return 1.0;
    }
    if is_generated(&lower) {
        return 1.0;
    }

    // Tier 0.9: ide_config/lockfile
    if is_ide_config(&lower) {
        return 0.9;
    }
    if is_lockfile(&lower) {
        return 0.9;
    }

    // Tier 0.8: snapshot
    if is_snapshot(&lower) {
        return 0.8;
    }

    // Tier 0.5: test
    if is_test(&lower) {
        return 0.5;
    }

    // Tier 0.4: dockerfile
    if is_dockerfile(&lower) {
        return 0.4;
    }

    // Tier 0.3: story
    if is_story(&lower) {
        return 0.3;
    }

    // Tier 0.2: type_declaration/migration
    if is_type_declaration(&lower) {
        return 0.2;
    }
    if is_migration(&lower) {
        return 0.2;
    }

    // Tier 0.2: styled
    if is_styled(&lower) {
        return 0.2;
    }

    // Tier 0.15: dot_config
    if is_dot_config(&lower) {
        return 0.15;
    }

    // Tier 0.1: barrel
    if is_barrel(path) {
        return 0.1;
    }

    0.0
}
