# file-suggest

Fast file suggestion for [Claude Code](https://code.claude.com/) with FTS5 full-text search, trigram substring matching, fuzzy matching, and frecency ranking.

Replaces Claude Code's built-in `@` file autocomplete with a pre-built SQLite FTS5 index for faster, smarter results. Supports multiple projects simultaneously.

## Benchmarks

Tested on a 9,600-file TypeScript monorepo (53 scored queries):

| Metric | Default | file-suggest v2.2 | Improvement |
|--------|---------|-------------------|-------------|
| Speed (p50) | 55ms | 4.3ms | **13x faster** |
| Speed (p95) | 70ms | 6.6ms | **11x faster** |
| Top-1 accuracy | 40% | 94% | **2.4x better** |
| Top-3 accuracy | 60% | 96% | **1.6x better** |
| Reliability | 5/5 | 5/5 | - |

## How it works

1. **FTS5 search** with BM25 ranking and 10x filename boost
2. **Trigram index** for substring matching (`config` finds `tsconfig.json`)
3. **Fuzzy matching** for abbreviations (`bksvc` finds `booking.service.ts`)
4. **File-type awareness** — build outputs, IDE configs, lockfiles, tests, dockerfiles, snapshots, and more rank below source files
5. **Directory-context boost** — queries matching `apps/X` or `packages/X` directory names boost files inside those directories
6. **Filename match boost** — exact basename matches outrank stemmer-conflated results (e.g., `sanitization.ts` beats `sanitize.ts`)
7. **Index file promotion** — path-prefix queries promote `index.ts`/`index.tsx`/`index.js` to the front
8. **Space-separated queries** — `admin jest.config` finds `apps/admin/jest.config.ts`
9. **BM25-tiered frecency** — among equally-relevant results, recently edited files rank first
10. **Multi-project** — per-project databases in `~/.claude/file-suggest/`, no cross-contamination
11. **Incremental updates** — git diff-based delta updates (~3ms vs ~100ms full rebuild)
12. **Self-healing** — rebuilds the index automatically if the database is missing

## Quick Start

```bash
# 1. Install
cargo install --git https://github.com/schlenks/file-suggest

# 2. Initialize for your project (builds index + installs git hooks)
cd /path/to/your/project
file-suggest init

# 3. Add to Claude Code settings (init prints this for you)
#    Edit ~/.claude/settings.json and add:
#    "fileSuggestion": {"type": "command", "command": "file-suggest"}
```

That's it. Start a new Claude Code session and file suggestions will use the FTS5 index.

## Install

### From source (requires Rust toolchain)

```bash
cargo install --git https://github.com/schlenks/file-suggest
```

### Clone and build

```bash
git clone https://github.com/schlenks/file-suggest.git
cd file-suggest
cargo build --release
# Binary is at target/release/file-suggest — copy or symlink to your PATH
```

### From releases

Download the binary for your platform from [Releases](https://github.com/schlenks/file-suggest/releases).

### Verify installation

```bash
file-suggest --help
```

## Setup

### 1. Initialize your project

```bash
cd /path/to/your/project
file-suggest init
```

This will:
- Build the FTS5 + trigram index for your project (~100ms for 10k files)
- Install git hooks (`post-checkout`, `post-merge`, `post-commit`, `post-rewrite`) for automatic incremental updates
- Create a per-project database in `~/.claude/file-suggest/`
- Print the settings.json config to add

### 2. Configure Claude Code

Add to your **personal** Claude Code settings (`~/.claude/settings.json`):

```json
{
  "fileSuggestion": {
    "type": "command",
    "command": "file-suggest"
  }
}
```

If you installed via clone/build and didn't add to PATH, use the full path:

```json
{
  "fileSuggestion": {
    "type": "command",
    "command": "/path/to/file-suggest"
  }
}
```

### 3. Start using it

Start a new Claude Code session in your project. The `@` file autocomplete and file suggestions now use the FTS5 index. No other changes needed.

### Multiple projects

Each project gets its own database. Run `file-suggest init` in each project directory. The `fileSuggestion` setting is global — the binary automatically selects the right database based on `CLAUDE_PROJECT_DIR` (set by Claude Code).

### Keeping the index fresh

Git hooks handle most updates automatically. To manually rebuild after large changes (e.g., switching branches with many new files):

```bash
file-suggest build --full /path/to/your/project
```

### Uninstall

1. Remove the `fileSuggestion` block from `~/.claude/settings.json`
2. Delete the database: `rm -rf ~/.claude/file-suggest/`
3. Remove the `# file-suggest index rebuild` lines from `.git/hooks/post-*`
4. Uninstall the binary: `cargo uninstall file-suggest` (or delete the binary)

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
2. **Path prefix** (contains `/`) → LIKE match sorted by frecency, index files promoted
3. **FTS5** → BM25-ranked with file-type penalties, directory boost, filename boost, and frecency tie-breaking
4. **Trigram** → substring matching for partial queries (min 3 chars)
5. **LIKE fallback** → simple pattern matching
6. **Fuzzy** → fzf-style scoring for abbreviations

## File-type ranking

Source files rank above auxiliary files with identical relevance:

| Type | Penalty | Example |
|------|---------|---------|
| Source | 0.0 | `Button.tsx` |
| Barrel/index | 0.1 | `index.ts` |
| Dot config | 0.15 | `.eslintrc.json` |
| Styled | 0.2 | `Button.styled.ts` |
| Type declaration | 0.2 | `types.d.ts` |
| Migration | 0.2 | `migrations/20240101-add-field.js` |
| Stories | 0.3 | `Button.stories.tsx` |
| Dockerfile | 0.4 | `Dockerfile.production` |
| Test | 0.5 | `Button.test.tsx` |
| Snapshot | 0.8 | `Button.test.tsx.snap` |
| IDE config | 0.9 | `.idea/runConfigurations/*.xml` |
| Lockfile | 0.9 | `pnpm-lock.yaml` |
| Build output | 1.0 | `.next/server/pages/index.js` |
| Generated | 1.0 | `generated/types.ts` |

## Requirements

- Git (for `git ls-files` and `git log` during index building)
- Claude Code (this is a `fileSuggestion` provider)

No runtime dependencies. The binary statically links SQLite with FTS5 support.

## License

MIT
