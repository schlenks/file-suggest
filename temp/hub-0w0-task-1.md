## Task 1: Multi-project DB resolution

**Purpose:** Enable simultaneous indexes for multiple repos. Without this, switching Claude Code sessions between projects serves stale results from the wrong repo.

**Not In Scope:** DB migration from v1 single-file format. Users will simply `file-suggest build` to create the new per-project DB. The old `~/.claude/file-suggestion.db` is ignored.

## Files
- Create: `src/project.rs`
- Modify: `src/main.rs` (replace `db_path()` function)
- Create: `tests/project_test.rs`

## Steps

**Step 1: Write failing tests** in `tests/project_test.rs`
- `db_path_is_deterministic_for_same_project`
- `db_path_differs_for_different_projects`
- `db_path_lives_under_dot_claude`

**Step 2: Run test to verify it fails**

**Step 3: Write implementation** in `src/project.rs`
- `db_path_for(project_dir: &Path) -> PathBuf` using FNV-1a 64-bit hash
- `ensure_db_dir()` to create `~/.claude/file-suggest/`
- Add `pub mod project;` to `src/lib.rs`

**Step 4: Update main.rs** to use `project::db_path_for` instead of hardcoded `db_path()`

**Step 5: Run test to verify it passes**

**Step 6: Commit**
