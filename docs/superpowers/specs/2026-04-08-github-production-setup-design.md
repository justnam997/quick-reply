# GitHub & Production Setup Design

## Summary

Initialize quick-reply as a production-ready open-source Rust TUI project on GitHub, following conventions from popular projects like gitui, bottom, and ratatui.

## Decisions

- **License**: MIT
- **GitHub user**: adrewtr
- **Repo name**: quick-reply
- **Target platforms**: macOS only (darwin-arm64, darwin-amd64)
- **Release strategy**: Tag-triggered GitHub Releases with pre-built binaries

## Files to Create

### 1. README.md

Structure (modeled after gitui/bottom):

```
# quick-reply

Badges: [CI] [License: MIT] [Release]

> Unified messaging TUI with AI-powered reply suggestions.
> Read and reply to Telegram, iMessage, WhatsApp, Slack, and Messenger
> from one terminal — with Claude-powered context-aware suggestions.

[Terminal screenshot placeholder]

## Features
- Unified inbox across 5 messaging platforms
- Claude Code AI suggestions with memory (facts, tone, context)
- Keyboard-driven 3-panel TUI (ratatui + crossterm)
- Local-first: all data stored on your machine (SQLite + FTS5)
- Per-contact profiles and semantic search

## Installation

### From binary (macOS)
Download from GitHub Releases.

### From source
cargo install quick-reply

### Build from source
git clone ... && cargo build --release

## Quick Start
1. Create ~/.config/quick-reply/config.toml
2. Add platform credentials (see docs/platform-setup.md)
3. Run: quick-reply

## Keybindings
Tab/Shift+Tab — switch panels
j/k or arrows  — navigate
Enter          — select / approve suggestion
r              — request AI suggestion
e              — edit suggestion before sending
/              — search conversations
?              — toggle help
q              — quit

## Platforms
| Platform  | Tier | Method                    | Status       |
|-----------|------|---------------------------|--------------|
| Telegram  | 1    | grammers (MTProto)        | Implemented  |
| iMessage  | 1    | chat.db + osascript       | Implemented  |
| Slack     | 1    | Web API + user token      | Implemented  |
| WhatsApp  | 2    | Go sidecar (whatsmeow)    | Bridge ready |
| Messenger | 2    | Node sidecar (puppeteer)  | Bridge ready |

## Architecture
Brief diagram (from spec) showing TUI → Platform Adapters → Claude CLI → Memory

## Contributing
See CONTRIBUTING.md

## License
MIT — see LICENSE
```

### 2. LICENSE

MIT license, copyright 2026 adrewtr.

### 3. CONTRIBUTING.md

Sections:
- Development setup (Rust toolchain, clone, cargo build)
- Project architecture (brief module map)
- Running checks locally (`make check`)
- PR guidelines (conventional commits, one feature per PR)
- Adding a new platform adapter (implement PlatformAdapter trait)

### 4. CHANGELOG.md

Keep-a-changelog format. Initial entry for v0.1.0 with all features.

### 5. Makefile

```makefile
.PHONY: build fmt lint test check clean release

build:
	cargo build

release:
	cargo build --release

fmt:
	cargo fmt

lint:
	cargo clippy -- -D warnings

test:
	cargo test

check: fmt lint test build
	@echo "All checks passed"

clean:
	cargo clean
```

### 6. .github/workflows/ci.yml

Triggers: push to main, pull requests.

```yaml
Jobs:
  check:
    runs-on: macos-latest
    steps:
      - checkout
      - Install Rust stable
      - Cache cargo registry + target
      - cargo fmt --check
      - cargo clippy -- -D warnings
      - cargo build
      - cargo test
```

### 7. .github/workflows/release.yml

Triggers: push tag matching `v*`.

```yaml
Jobs:
  build:
    strategy:
      matrix:
        include:
          - target: aarch64-apple-darwin
            os: macos-latest
            name: quick-reply-darwin-arm64
          - target: x86_64-apple-darwin
            os: macos-latest
            name: quick-reply-darwin-amd64
    steps:
      - checkout
      - Install Rust stable + target
      - cargo build --release --target ${{ matrix.target }}
      - tar czf ${{ matrix.name }}.tar.gz -C target/${{ matrix.target }}/release quick-reply
      - Upload artifact

  release:
    needs: build
    steps:
      - Download all artifacts
      - Create GitHub Release from tag
      - Attach .tar.gz binaries
```

### 8. .github/ISSUE_TEMPLATE/

Two templates:
- **bug_report.md**: platform, OS version, steps to reproduce, expected vs actual
- **feature_request.md**: description, use case, alternatives considered

### 9. .gitignore (expanded)

Add to existing:
```
# Cargo
/target
Cargo.lock  # remove this line — lock files should be committed for binaries

# Runtime data
/data
*.session
*.db
*.db-wal
*.db-shm

# Environment
.omc/
.env
.DS_Store

# IDE
.idea/
.vscode/
*.swp
*.swo
```

Note: `Cargo.lock` SHOULD be committed for binary projects (Rust convention).

### 10. Git Initial Commit Strategy

Single initial commit containing all existing source code plus the new files above:
```
feat: initial release — unified messaging TUI with AI suggestions

- 3-panel TUI with keyboard navigation (ratatui + crossterm)
- 5 platform adapters: Telegram, iMessage, WhatsApp, Slack, Messenger
- Claude Code CLI integration for AI reply suggestions
- Memory system: contact profiles + conversation archive + FTS5 search
- SQLite storage with WAL mode and tokio-rusqlite
- Full platform setup documentation
```

## Out of Scope

- Homebrew formula (future)
- Docker support (not needed for a TUI binary)
- crates.io publishing (future, after stabilization)
- Linux/Windows CI targets (future)
- Code coverage reporting (future)
