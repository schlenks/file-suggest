#!/usr/bin/env python3
"""Test file-suggest queries to determine expected results."""
import json
import os
import subprocess

BINARY = "/Users/schlenks/Developer/personal/file-suggest/target/release/file-suggest"
PROJECT_DIR = "/Users/schlenks/Developer/work/hub"

queries = [
    "guide.entity",
    "user.repository",
    "traveler.entity",
    "review.service",
    "authMagicLink.resolver",
    "administrator.resolver",
    "area.resolver",
    "RegionFiltersAndResults",
    "Modal",
    "Sidebar",
    "useBooking",
    "jest.config",
    "playwright.config",
    "Makefile",
    "env.example",
    "enums.graphql",
    "PaymentProcessorTypeEnum",
    "AGENTS.md",
    "shared",
    "loader",
    "builder",
    "middleware",
    "tour.entity",
    "notification.service",
    "turbo.json",
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
    lines = [l for l in result.stdout.strip().split("\n") if l][:5]
    print(f"=== {q!r} ===")
    for i, line in enumerate(lines):
        print(f"  {i+1}. {line}")
    print()
