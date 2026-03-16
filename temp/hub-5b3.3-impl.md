[IMPL-REPORT] hub-5b3.3 wave-1

### Evidence
- File modified: ~/.claude/file-suggestion-bench/golden-queries.json
- 3 expectations updated per specification
- JSON validation: PASS

### Summary
Updated 3 debatable benchmark expectations in golden-queries.json to match real-world intent:
1. `booking.service`: Reordered to put booking.service.ts first
2. `temporal-worker`: Reordered to put src/workers/emailWorker.ts first
3. `CancellationUpsell`: Added organism component as acceptable first result

### Files Modified
1 file: ~/.claude/file-suggestion-bench/golden-queries.json

### Self-Review
- Scope: CLEAN - only allowed file modified
- Expected values: All 3 expectations updated as specified in task
- JSON validity: Confirmed via python3 -m json.tool
- No code logic changes needed (benchmark expectations only)
- No dependencies needed (standalone file)
