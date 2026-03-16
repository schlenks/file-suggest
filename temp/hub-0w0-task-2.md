## Task 2: File-type scoring rules

**Purpose:** Compute penalty scores for file types so test files, generated files, snapshots, and barrel files rank below source files with identical BM25 relevance.

## Files
- Create: `src/scoring.rs`
- Create: `tests/scoring_test.rs`

## Steps

**Step 1: Write failing tests** in `tests/scoring_test.rs`
- `source_file_has_zero_penalty`
- `test_file_has_penalty`
- `snapshot_file_has_higher_penalty_than_test`
- `generated_file_has_penalty`
- `index_barrel_has_small_penalty`
- `stories_file_has_penalty`

**Step 2: Run test to verify it fails**

**Step 3: Write implementation** in `src/scoring.rs`
- `type_penalty(path: &str) -> f64` function
- Penalty tiers: generated=1.0, snapshot=0.8, test=0.5, stories=0.3, styled=0.2, barrel=0.1, source=0.0
- Add `pub mod scoring;` to `src/lib.rs`

**Step 4: Run test to verify it passes**

**Step 5: Commit**
