## Task 12: Full benchmark validation

**Purpose:** Run the full benchmark suite against the v2 binary and compare with baseline. Verify no speed regressions and ranking improvements.

**No code changes. Benchmark-only task.**

## Steps

**Step 1: Rebuild and rebuild index**
Run: `cargo build --release`
Run: `file-suggest build --full ~/Developer/work/hub`

**Step 2: Run benchmark**
Run: `python3 ~/.claude/file-suggestion-bench/benchmark.py custom ~/Developer/personal/file-suggest/target/release/file-suggest`

**Step 3: Compare with baseline**
Run: `python3 ~/.claude/file-suggestion-bench/benchmark.py compare ~/.claude/file-suggestion-bench/baseline.json ~/.claude/file-suggestion-bench/custom.json`

**Expected results:**
- Speed p50: at most 10ms (no regression from 8ms)
- Top-1 hit rate: at least 80%
- Top-3 hit rate: at least 87%
- Reliability: 5/5 pass
