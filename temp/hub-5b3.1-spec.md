[SPEC-REVIEW] hub-5b3.1 wave-1

## Findings

### Requirements Met

All 6 specification steps fully implemented:

1. **Tests Written (scoring_test.rs:41-84)**: 7 new tests added for all required categories:
   - dockerfile_has_penalty (line 41)
   - ide_config_has_high_penalty (line 47)
   - lockfile_has_high_penalty (line 55)
   - build_output_has_max_penalty (line 63)
   - dot_config_has_small_penalty (line 69)
   - migration_has_penalty (line 75)
   - type_declaration_has_penalty (line 81)

2. **Tests Verified Failing**: Per implementation report, tests confirmed written before predicates implemented.

3. **Predicates Implemented (scoring.rs:37-74)**: All 7 required predicates present:
   - is_build_output (line 37-39): checks dist/, build/ prefixes/paths
   - is_ide_config (line 42-44): checks .vscode/, .idea/ directories
   - is_lockfile (line 47-51): checks pnpm-lock.yaml, package-lock.json, yarn.lock
   - is_dockerfile (line 54-57): checks Dockerfile and Dockerfile.* variants
   - is_dot_config (line 60-64): checks dot-prefixed files with rc/config/.json suffixes
   - is_migration (line 67-69): checks /migrations/ directory
   - is_type_declaration (line 72-74): checks .d.ts extension

4. **type_penalty() Updated (scoring.rs:78-141)**: Complete tier implementation matches specification exactly:
   - Tier 1.0: build_output and generated (lines 82-87)
   - Tier 0.9: ide_config and lockfile (lines 90-95)
   - Tier 0.8: snapshot (lines 98-100)
   - Tier 0.5: test (lines 103-105)
   - Tier 0.4: dockerfile (lines 108-110)
   - Tier 0.3: story (lines 113-115)
   - Tier 0.2: type_declaration, migration, and styled (lines 118-128)
   - Tier 0.15: dot_config (lines 131-133)
   - Tier 0.1: barrel (lines 136-138)

5. **Tests Pass**: Confirmed via cargo test output: 13/13 tests pass (exit code 0)

6. **Commit Present**: Commit 769978cdbe36096e8d2564a8f4043b9d55ed00bd in git log

### Code Quality

- All new predicates follow existing patterns: lowercase conversion, rsplit for filenames
- Predicates are properly documented with descriptive comments
- Penalty ordering correctly prioritizes source files (0.0) over infrastructure/config/test files
- Test assertions verify both presence of penalties and correct ordering between tiers
- No YAGNI violations: each tier has explicit test coverage
- No scope creep: only requested files modified (src/scoring.rs, tests/scoring_test.rs)

## Conclusion

Spec compliant. All requirements implemented correctly with proper test coverage and working code.
