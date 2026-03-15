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

/// Get files added/removed since a given commit hash.
pub fn get_changed_files(project_dir: &Path, since_hash: &str) -> (Vec<String>, Vec<String>) {
    let output = Command::new("git")
        .args(["diff", "--name-status", since_hash, "HEAD"])
        .current_dir(project_dir)
        .output();

    let mut added = Vec::new();
    let mut removed = Vec::new();

    if let Ok(output) = output {
        if !output.status.success() {
            return (added, removed);
        }
        for line in String::from_utf8_lossy(&output.stdout).lines() {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() < 2 {
                continue;
            }
            let status = parts[0].chars().next().unwrap_or(' ');
            match status {
                'A' | 'C' => added.push(parts[1].to_string()),
                'D' => removed.push(parts[1].to_string()),
                'M' => added.push(parts[1].to_string()),
                'R' => {
                    removed.push(parts[1].to_string());
                    if parts.len() >= 3 {
                        added.push(parts[2].to_string());
                    }
                }
                _ => {}
            }
        }
    }
    (added, removed)
}

/// Get the current HEAD commit hash.
pub fn get_head_hash(project_dir: &Path) -> Option<String> {
    Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(project_dir)
        .output()
        .ok()
        .and_then(|o| {
            if !o.status.success() {
                return None;
            }
            let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if s.is_empty() { None } else { Some(s) }
        })
}
