## Task 9: Update init command for new DB structure

**Purpose:** Update the `init` command to create per-project DB directory and use incremental builds in hooks.

## Files
- Modify: `src/main.rs` (update cmd_init and install_hooks)

## Steps

**Step 1: Update hook body**
Change hook content from `file-suggest build <dir> &` to `file-suggest build &` (incremental by default, uses `CLAUDE_PROJECT_DIR`).

**Step 2: Ensure DB directory is created**
Call `project::ensure_db_dir()` in `cmd_init`.

**Step 3: Update help text**
Add `--full` flag documentation.

**Step 4: Test**
Run: `CLAUDE_PROJECT_DIR=~/Developer/work/hub file-suggest init`

**Step 5: Commit**
