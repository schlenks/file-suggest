use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const FRECENCY_HALF_LIFE_DAYS: f64 = 14.0;

/// Get all git-tracked + untracked-but-not-ignored files.
pub fn get_files(project_dir: &Path) -> Vec<String> {
    let mut files = std::collections::HashSet::new();

    if let Ok(output) = Command::new("git")
        .args(["ls-files"])
        .current_dir(project_dir)
        .output()
    {
        for line in String::from_utf8_lossy(&output.stdout).lines() {
            if !line.is_empty() {
                files.insert(line.to_string());
            }
        }
    }

    if let Ok(output) = Command::new("git")
        .args(["ls-files", "--others", "--exclude-standard"])
        .current_dir(project_dir)
        .output()
    {
        for line in String::from_utf8_lossy(&output.stdout).lines() {
            if !line.is_empty() {
                files.insert(line.to_string());
            }
        }
    }

    let mut sorted: Vec<String> = files.into_iter().collect();
    sorted.sort();
    sorted
}

/// Compute frecency scores from git log using exponential decay.
pub fn get_frecency(project_dir: &Path) -> HashMap<String, f64> {
    let output = match Command::new("git")
        .args([
            "log",
            "--format=%at",
            "--name-only",
            "--diff-filter=ACMR",
            "--since=90 days ago",
            "-n",
            "2000",
        ])
        .current_dir(project_dir)
        .output()
    {
        Ok(o) => o,
        Err(_) => return HashMap::new(),
    };

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64();
    let decay = (2.0_f64).ln() / (FRECENCY_HALF_LIFE_DAYS * 86400.0);

    let mut scores: HashMap<String, f64> = HashMap::new();
    let mut current_timestamp: Option<f64> = None;

    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Ok(ts) = trimmed.parse::<f64>() {
            current_timestamp = Some(ts);
        } else if let Some(ts) = current_timestamp {
            let age = now - ts;
            let visit_score = (-decay * age).exp();
            *scores.entry(trimmed.to_string()).or_default() += visit_score;
        }
    }

    scores
}
