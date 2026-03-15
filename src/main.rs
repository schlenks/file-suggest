use file_suggest::{index, search};
use serde::Deserialize;
use std::io::Read;
use std::path::PathBuf;

#[derive(Deserialize)]
struct QueryInput {
    #[serde(default)]
    query: String,
}

fn db_path() -> PathBuf {
    let home = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default();
    home.join(".claude").join("file-suggestion.db")
}

fn project_dir() -> PathBuf {
    std::env::var_os("CLAUDE_PROJECT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
}

fn cmd_search() {
    let mut input = String::new();
    if std::io::stdin().read_to_string(&mut input).is_err() {
        return;
    }

    let query_input: QueryInput = serde_json::from_str(&input).unwrap_or(QueryInput {
        query: String::new(),
    });

    let db = db_path();

    // Self-healing: rebuild index if DB is missing
    if !db.exists() {
        let _ = index::build(&project_dir(), &db);
    }

    match search::search(&query_input.query, &db) {
        Ok(results) => {
            for path in results {
                println!("{path}");
            }
        }
        Err(_) => {
            // Silent failure — Claude Code falls back to default
        }
    }
}

fn cmd_build(dir: Option<&str>) {
    let project = dir.map(PathBuf::from).unwrap_or_else(project_dir);
    let db = db_path();

    match index::build(&project, &db) {
        Ok(count) => eprintln!("Indexed {count} files from {}", project.display()),
        Err(e) => {
            eprintln!("Error building index: {e}");
            std::process::exit(1);
        }
    }
}

fn cmd_init() {
    let project = project_dir();
    let hooks_dir = project.join(".git").join("hooks");

    if !hooks_dir.exists() {
        eprintln!("Not a git repository: {}", project.display());
        std::process::exit(1);
    }

    let binary = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("file-suggest"));

    install_hooks(&hooks_dir, &binary, &project);

    // Build initial index
    cmd_build(None);
    eprintln!("\nDone! Add to ~/.claude/settings.json:");
    eprintln!(
        "  \"fileSuggestion\": {{\"type\": \"command\", \"command\": \"{}\"}}",
        binary.display()
    );
}

fn install_hooks(hooks_dir: &PathBuf, binary: &PathBuf, project: &PathBuf) {
    let hook_body = format!(
        "\"{}\" build \"{}\" &\n",
        binary.display(),
        project.display()
    );
    let marker = "# file-suggest index rebuild";

    for hook_name in &["post-checkout", "post-merge", "post-commit", "post-rewrite"] {
        let hook_path = hooks_dir.join(hook_name);

        if hook_path.exists() {
            let existing = std::fs::read_to_string(&hook_path).unwrap_or_default();
            if existing.contains(marker) {
                continue;
            }
            let appended = format!("{existing}\n{marker}\n{hook_body}");
            if std::fs::write(&hook_path, appended).is_err() {
                eprintln!("Failed to update {hook_name}");
                continue;
            }
        } else {
            let content = format!("#!/bin/sh\n{marker}\n{hook_body}");
            if std::fs::write(&hook_path, &content).is_err() {
                eprintln!("Failed to create {hook_name}");
                continue;
            }
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = std::fs::set_permissions(
                    &hook_path,
                    std::fs::Permissions::from_mode(0o755),
                );
            }
        }

        eprintln!("Installed {hook_name} hook");
    }
}

fn print_help() {
    eprintln!(
        "file-suggest - Fast file suggestion for Claude Code

USAGE:
  file-suggest              Read JSON from stdin, output file paths (fileSuggestion mode)
  file-suggest build [dir]  Build/rebuild the FTS5 index
  file-suggest init         Install git hooks and configure for current project

ENVIRONMENT:
  CLAUDE_PROJECT_DIR        Project root (set by Claude Code automatically)"
    );
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("build") => cmd_build(args.get(2).map(|s| s.as_str())),
        Some("init") => cmd_init(),
        Some("--help" | "-h") => print_help(),
        _ => cmd_search(),
    }
}
