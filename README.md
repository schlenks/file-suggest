# file-suggest

Fast file suggestion for [Claude Code](https://code.claude.com/) with FTS5 full-text search and frecency ranking.

Replaces Claude Code's built-in `@` file autocomplete with a pre-built SQLite FTS5 index for faster, smarter results.

## Benchmarks

Tested on a 9,600-file TypeScript monorepo:

| Metric | Default | file-suggest | Improvement |
|--------|---------|-------------|-------------|
| Speed (p50) | 55ms | 8ms | **6.7x faster** |
| Speed (p95) | 70ms | 12ms | **5.8x faster** |
| Top-1 accuracy | 40% | 80% | **2x better** |
| Top-3 accuracy | 60% | 87% | **1.4x better** |

## How it works

1. **Indexes** all git-tracked files into a SQLite FTS5 virtual table with tokenized paths
2. **Ranks** results using BM25 with a 10x filename boost (filename matches rank higher than directory matches)
3. **Frecency** for empty queries and path browsing — shows your most recently/frequently edited files first
4. **Self-heals** — rebuilds the index automatically if the database is missing or corrupt

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
- Build the FTS5 index for your project
- Install git hooks (`post-checkout`, `post-merge`, `post-commit`, `post-rewrite`) to keep the index fresh
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
file-suggest              # Search mode (reads JSON from stdin, used by Claude Code)
file-suggest build [dir]  # Rebuild the index for a project directory
file-suggest init         # Install git hooks + build index for current project
file-suggest --help       # Show usage
```

## How ranking works

- **BM25 with filename boost**: When you type `button`, files with "button" in the filename (like `Button.tsx`) rank higher than files where "button" only appears in a directory name (like `RadioButton/index.ts`)
- **Short path preference**: Among equal BM25 scores, shorter paths win (`tsconfig.json` over `apps/api/tsconfig.json`)
- **Frecency for browsing**: Empty queries and path-prefix queries (`apps/api/src/`) return files sorted by how recently and frequently you've edited them
- **Prefix matching**: Partial queries work — `book` matches `booking.service.ts`

## Requirements

- Git (for `git ls-files` and `git log` during index building)
- Claude Code (this is a `fileSuggestion` provider)

No runtime dependencies. The binary statically links SQLite with FTS5 support.

## License

MIT
