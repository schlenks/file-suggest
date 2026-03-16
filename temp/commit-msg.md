fix(scoring): root-level migration match and strengthen test assertions

- Fix is_migration() to match root-level migrations with starts_with("migrations/")
- Update ide_config_has_high_penalty test: compare against actual test file (.test.ts)
- Update lockfile_has_high_penalty test: compare against actual test file (.test.ts)
