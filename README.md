# file-suggest

Fast file suggestion for [Claude Code](https://code.claude.com/) with FTS5 full-text search, trigram substring matching, fuzzy matching, and frecency ranking.

Replaces Claude Code's built-in `@` file autocomplete with a pre-built SQLite FTS5 index for faster, smarter results. Supports multiple projects simultaneously.

## Benchmarks

Tested on a 9,600-file TypeScript monorepo:

| Metric | Default | file-suggest v2 | Improvement |
|--------|---------|-----------------|-------------|
| Speed (p50) | 55ms | 3.4ms | **16.1x faster** |
| Speed (p95) | 70ms | 4.8ms | **14.6x faster** |
| Top-1 accuracy | 40% | 80% | **2x better** |
| Top-3 accuracy | 60% | 87% | **1.4x better** |
| Reliability | 5/5 | 5/5 | - |

## How it works

1. **FTS5 search** with BM25 ranking and 10x filename boost
2. **Trigram index** for substring matching (`config` finds `tsconfig.json`)
3. **Fuzzy matching** for abbreviations (`bksvc` finds `booking.service.ts`)
4. **File-type awareness** — test, generated, snapshot, and barrel files rank below source files
5. **BM25-tiered frecency** — among equally-relevant results, recently edited files rank first
6. **Multi-project** — per-project databases in `~/.claude/file-suggest/`, no cross-contamination
7. **Incremental updates** — git diff-based delta updates (~3ms vs ~100ms full rebuild)
8. **Self-healing** — rebuilds the index automatically if the database is missing

## Install

### From source

```bash
cargo install --git https://github.com/schlenks/file-suggest
```

### From releases

Download the binary for your platform from [Releases](https://github.com/schlenks/file-suggest/releases).

## Setup

```bash
# Navigate to your project
cd /path/to/your/project

# Build initial index and install git hooks
file-suggest init
```

This will:
- Create a per-project database in `~/.claude/file-suggest/`
- Build the FTS5 + trigram index for your project
- Install git hooks (`post-checkout`, `post-merge`, `post-commit`, `post-rewrite`) for incremental updates
- Print the settings.json config to add

Then add to `~/.claude/settings.json`:

```json
{
  "fileSuggestion": {
    "type": "command",
    "command": "file-suggest"
  }
}
```

## Commands

```bash
file-suggest                    # Search mode (reads JSON from stdin, used by Claude Code)
file-suggest build [dir]        # Incremental index update (default)
file-suggest build --full [dir] # Full rebuild of the FTS5 index
file-suggest init               # Install git hooks + build index for current project
file-suggest --help             # Show usage
```

## Search pipeline

Queries go through these stages in order, returning at the first match:

1. **Empty query** → frecency-sorted recent files
2. **Path prefix** (contains `/`) → LIKE match sorted by frecency
3. **FTS5** → BM25-ranked with file-type penalties and frecency tie-breaking
4. **Trigram** → substring matching for partial queries (min 3 chars)
5. **LIKE fallback** → simple pattern matching
6. **Fuzzy** → fzf-style scoring for abbreviations

## File-type ranking

Source files rank above auxiliary files with identical relevance:

| Type | Penalty | Example |
|------|---------|---------|
| Source | 0.0 | `Button.tsx` |
| Barrel/index | 0.1 | `index.ts` |
| Styled | 0.2 | `Button.styled.ts` |
| Stories | 0.3 | `Button.stories.tsx` |
| Test | 0.5 | `Button.test.tsx` |
| Snapshot | 0.8 | `Button.test.tsx.snap` |
| Generated | 1.0 | `generated/types.ts` |

## Requirements

- Git (for `git ls-files` and `git log` during index building)
- Claude Code (this is a `fileSuggestion` provider)

No runtime dependencies. The binary statically links SQLite with FTS5 support.

## License

MIT
