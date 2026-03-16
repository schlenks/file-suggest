## Task 10: Make crate a lib+bin for testability (DO FIRST)

**Purpose:** Integration tests (`tests/*.rs`) need `use file_suggest::module` imports, which requires a library target. Currently only a binary target exists.

## Files
- Create: `src/lib.rs`
- Modify: `src/main.rs` (move module declarations to lib.rs)

## Steps

**Step 1: Create src/lib.rs** (only existing modules -- new modules are added by their tasks)
```rust
pub mod db;
pub mod git;
pub mod index;
pub mod search;
```

**Step 2: Update main.rs**
Remove `mod` declarations and add explicit imports:
```rust
use file_suggest::{db, git, index, search};
```
Note: modules that don't exist yet (fuzzy, scoring, project) will cause compile errors until their tasks are complete. Task 10 should only declare the modules that exist now (`db`, `git`, `index`, `search`) in lib.rs, and each subsequent task adds its own `pub mod` line when creating a new module file.

**Step 3: Verify build**
Run: `cargo build --release`
Expected: PASS

**Step 4: Commit**
