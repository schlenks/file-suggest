#!/usr/bin/env python3
"""Verify specific queries to update golden file."""
import json
import os
import subprocess

BINARY = "/Users/schlenks/Developer/personal/file-suggest/target/release/file-suggest"
PROJECT_DIR = "/Users/schlenks/Developer/work/hub"

queries = [
    "temporal-worker",
    "apps/api/src/resolvers",
    "tour.entity",
]

env = os.environ.copy()
env["CLAUDE_PROJECT_DIR"] = PROJECT_DIR

for q in queries:
    input_json = json.dumps({"query": q})
    result = subprocess.run(
        [BINARY],
        input=input_json,
        capture_output=True,
        text=True,
        cwd=PROJECT_DIR,
        timeout=10,
        env=env,
    )
    lines = [l for l in result.stdout.strip().split("\n") if l][:10]
    print(f"=== {q!r} ===")
    for i, line in enumerate(lines):
        print(f"  {i+1}. {line}")
    print()
