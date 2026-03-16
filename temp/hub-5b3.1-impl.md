[IMPL-REPORT] hub-5b3.1 wave-1

### Evidence
- Commit: 769978cdbe36096e8d2564a8f4043b9d55ed00bd
- Files: 2 changed, 126 insertions(+) (src/scoring.rs +80, tests/scoring_test.rs +46)
- Tests: 13/13 pass, exit code 0

### Summary
Expanded file-type penalties to penalize infrastructure, IDE, lockfile, build-output, config, migration, and type-declaration files. Added 7 new predicate functions (is_build_output, is_ide_config, is_lockfile, is_dockerfile, is_dot_config, is_migration, is_type_declaration) and reorganized type_penalty() into 10 penalty tiers (1.0 down to 0.0) with documented rationale matching epic Key Decisions.

Files modified: src/scoring.rs, tests/scoring_test.rs (both in allowed list).

Self-review: All 7 new tests written and passing. Predicates follow existing code patterns (lowercase conversion, rsplit for filenames). Penalty ordering matches epic specification. No YAGNI violations - each tier has explicit test coverage. No scope violations.
