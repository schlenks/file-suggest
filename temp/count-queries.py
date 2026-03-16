#!/usr/bin/env python3
import json

data = json.load(open('/Users/schlenks/.claude/file-suggestion-bench/golden-queries.json'))
total = len(data['queries'])
scored = sum(1 for q in data['queries'] if q.get('expected_top3'))
new_only = sum(1 for q in data['queries'] if q.get('scored') is True)
print(f"Total queries: {total}")
print(f"Queries with expected_top3 (non-empty): {scored}")
print(f"New queries (with scored=True field): {new_only}")
print("JSON valid: True")
