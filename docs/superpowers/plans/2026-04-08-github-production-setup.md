# GitHub & Production Setup Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make quick-reply a production-ready open-source project with GitHub CI/CD, documentation, and release automation.

**Architecture:** Add standard open-source project files (README, LICENSE, CI, Makefile) around the existing Rust source. No code changes — this is all project scaffolding.

**Tech Stack:** GitHub Actions, Make, Markdown

---

### Task 1: Expand .gitignore

**Files:**
- Modify: `.gitignore`

- [ ] **Step 1: Replace .gitignore with expanded version**

```gitignore
# Cargo
/target

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

Note: `Cargo.lock` is NOT ignored — it must be committed for binary projects (Rust convention).

- [ ] **Step 2: Verify Cargo.lock is tracked**

Run: `git status | grep Cargo.lock`
Expected: `Cargo.lock` appears in untracked files (will be committed in Task 10).

---

### Task 2: Create LICENSE

**Files:**
- Create: `LICENSE`

- [ ] **Step 1: Create MIT license file**

```
MIT License

Copyright (c) 2026 adrewtr

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

---

### Task 3: Create Makefile

**Files:**
- Create: `Makefile`

- [ ] **Step 1: Create Makefile**

```makefile
.PHONY: build release fmt lint test check clean

build:
	cargo build

release:
	cargo build --release

fmt:
	cargo fmt

fmt-check:
	cargo fmt --check

lint:
	cargo clippy -- -D warnings

test:
	cargo test

check: fmt-check lint test build
	@echo "All checks passed"

clean:
	cargo clean

run:
	cargo run
```

- [ ] **Step 2: Verify make check works**

Run: `make check`
Expected: fmt, clippy, test, and build all pass. Final line: "All checks passed"

---

### Task 4: Create README.md

**Files:**
- Create: `README.md`

- [ ] **Step 1: Create README.md**

```markdown
# quick-reply

[![CI](https://github.com/adrewtr/quick-reply/actions/workflows/ci.yml/badge.svg)](https://github.com/adrewtr/quick-reply/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

> Unified messaging TUI with AI-powered reply suggestions.
> Read and reply to Telegram, iMessage, WhatsApp, Slack, and Messenger from one terminal — with Claude-powered context-aware suggestions.

<!-- TODO: Add terminal screenshot here -->
<!-- ![quick-reply demo](docs/screenshot.png) -->

## Features

- **Unified inbox** across 5 messaging platforms in one terminal
- **AI reply suggestions** via Claude Code CLI with conversation memory
- **Memory-aware** — recalls personal facts, matches your tone per contact, references recent threads
- **Keyboard-driven** 3-panel TUI built with ratatui + crossterm
- **Local-first** — all data stored on your machine (SQLite + FTS5)
- **Per-contact profiles** with semantic search over conversation history

## Installation

### From binary (macOS)

Download the latest release from [GitHub Releases](https://github.com/adrewtr/quick-reply/releases):

```bash
# Apple Silicon (M1/M2/M3/M4)
curl -LO https://github.com/adrewtr/quick-reply/releases/latest/download/quick-reply-darwin-arm64.tar.gz
tar xzf quick-reply-darwin-arm64.tar.gz
sudo mv quick-reply /usr/local/bin/

# Intel Mac
curl -LO https://github.com/adrewtr/quick-reply/releases/latest/download/quick-reply-darwin-amd64.tar.gz
tar xzf quick-reply-darwin-amd64.tar.gz
sudo mv quick-reply /usr/local/bin/
```

### Build from source

```bash
git clone https://github.com/adrewtr/quick-reply.git
cd quick-reply
cargo build --release
cp target/release/quick-reply /usr/local/bin/
```

## Quick Start

1. Create your config file:

```bash
mkdir -p ~/.config/quick-reply
cp config/default.toml ~/.config/quick-reply/config.toml
```

2. Add your platform credentials (see [Platform Setup Guide](docs/platform-setup.md))

3. Launch:

```bash
quick-reply
```

## Keybindings

| Key | Action |
|-----|--------|
| `Tab` / `Shift+Tab` | Switch panels |
| `j` / `k` or arrows | Navigate |
| `Enter` | Select / approve suggestion |
| `r` | Request AI suggestion |
| `e` | Edit suggestion before sending |
| `/` | Search conversations |
| `?` | Toggle help |
| `q` | Quit |

## Supported Platforms

| Platform | Tier | Method | Status |
|----------|------|--------|--------|
| Telegram | 1 | grammers (MTProto) | Implemented |
| iMessage | 1 | chat.db + osascript (macOS) | Implemented |
| Slack | 1 | Web API + user token | Implemented |
| WhatsApp | 2 | Go sidecar (whatsmeow) | Bridge ready |
| Messenger | 2 | Node sidecar (puppeteer) | Bridge ready |

**Tier 1**: Fully supported, native Rust implementation.
**Tier 2**: Best-effort via sidecar bridges. May require manual setup.

## Architecture

```
┌─────────────────────────────────────────────────┐
│                  quick-reply TUI                 │
│  ┌──────────┬───────────────┬─────────────────┐ │
│  │Platforms │ Conversations │    Messages      │ │
│  │──────────│───────────────│─────────────────│ │
│  │▶Telegram │ @john         │ john: Hey!      │ │
│  │ iMessage │ @sarah        │ you: Hi there   │ │
│  │ WhatsApp │ #team         │─────────────────│ │
│  │ Slack    │               │ AI Suggestion:  │ │
│  │ Msgr     │               │ "See you at 3!" │ │
│  └──────────┴───────────────┴─────────────────┘ │
└──────────┬──────────────────────┬────────────────┘
           │                      │
    ┌──────▼──────┐      ┌───────▼────────┐
    │  Platform    │      │  Claude CLI     │
    │  Adapters    │      │  (subprocess)   │
    └──────┬──────┘      └───────┬────────┘
           │                      │
    ┌──────▼──────────────────────▼──────┐
    │          Local Storage              │
    │  SQLite (WAL) + Contact Profiles   │
    └────────────────────────────────────┘
```

## Prerequisites

- **Rust** 1.75+ (for building from source)
- **Claude Code CLI** (for AI suggestions) — [Install](https://docs.anthropic.com/claude-code)
- **macOS** (required for iMessage; other platforms work on any OS)

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup, architecture overview, and PR guidelines.

## License

[MIT](LICENSE)
```

---

### Task 5: Create CONTRIBUTING.md

**Files:**
- Create: `CONTRIBUTING.md`

- [ ] **Step 1: Create CONTRIBUTING.md**

```markdown
# Contributing to quick-reply

Thanks for your interest in contributing! Here's how to get started.

## Development Setup

```bash
# Clone the repo
git clone https://github.com/adrewtr/quick-reply.git
cd quick-reply

# Build
cargo build

# Run all checks (fmt, clippy, test, build)
make check
```

### Requirements

- Rust 1.75+ (`rustup update stable`)
- macOS (for iMessage adapter development; other adapters work on any OS)

## Project Architecture

```
src/
├── main.rs              # Entry point, event loop, keybindings
├── app.rs               # App state machine (panels, input modes)
├── config.rs            # TOML configuration loading
├── error.rs             # Error types (thiserror)
├── models/              # Data types: Message, Conversation, Contact, Notification
├── platform/
│   ├── mod.rs           # PlatformAdapter trait definition
│   ├── manager.rs       # Manages all adapters, unified notification stream
│   ├── telegram.rs      # Telegram via grammers (MTProto)
│   ├── imessage.rs      # iMessage via chat.db + osascript
│   ├── slack.rs         # Slack Web API
│   ├── whatsapp.rs      # HTTP client to Go sidecar bridge
│   └── messenger.rs     # HTTP client to Node sidecar bridge
├── ai/
│   ├── claude.rs        # Claude CLI subprocess wrapper
│   └── context.rs       # Memory context assembly for prompts
├── memory/
│   ├── archive.rs       # SQLite conversation archive
│   ├── profile.rs       # Per-contact TOML profiles
│   └── search.rs        # FTS5 full-text search
├── storage/
│   └── sqlite.rs        # Database connection, WAL mode, migrations
└── ui/
    ├── layout.rs        # 3-panel layout
    ├── theme.rs         # Colors and styles
    └── panels/          # Platform list, conversation list, message view
```

## Running Checks

```bash
# Full CI check (what GitHub Actions runs)
make check

# Individual checks
make fmt        # Format code
make lint       # Run clippy
make test       # Run tests
make build      # Build debug binary
```

## Pull Request Guidelines

1. **One feature per PR** — keep changes focused
2. **Conventional commits** — use prefixes: `feat:`, `fix:`, `docs:`, `ci:`, `refactor:`, `test:`
3. **Run `make check` before pushing** — CI will catch the same issues
4. **Update CHANGELOG.md** for user-facing changes

## Adding a New Platform Adapter

1. Create `src/platform/yourplatform.rs`
2. Implement the `PlatformAdapter` trait:

```rust
#[async_trait]
impl PlatformAdapter for YourAdapter {
    fn name(&self) -> &str;
    async fn connect(&mut self) -> Result<()>;
    async fn disconnect(&mut self) -> Result<()>;
    fn is_connected(&self) -> bool;
    async fn fetch_conversations(&self) -> Result<Vec<Conversation>>;
    async fn fetch_messages(&self, conv_id: &str, limit: usize) -> Result<Vec<Message>>;
    async fn send_message(&self, conv_id: &str, content: &str) -> Result<DeliveryConfirmation>;
    async fn subscribe_notifications(&self) -> Result<mpsc::UnboundedReceiver<Notification>>;
}
```

3. Add config struct to `src/config.rs`
4. Register in `register_platforms()` in `src/main.rs`
5. Add to `src/platform/mod.rs`

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
```

---

### Task 6: Create CHANGELOG.md

**Files:**
- Create: `CHANGELOG.md`

- [ ] **Step 1: Create CHANGELOG.md**

```markdown
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-04-08

### Added

- 3-panel TUI with keyboard navigation (ratatui + crossterm)
- Telegram adapter via grammers (MTProto personal account access)
- iMessage adapter via macOS chat.db + osascript
- Slack adapter via Web API with user tokens
- WhatsApp adapter (HTTP client for Go sidecar bridge)
- Messenger adapter (HTTP client for Node sidecar bridge, Tier 2)
- Claude Code CLI integration for AI reply suggestions
- Memory-aware context assembly (contact profiles + recent messages + FTS5 search)
- SQLite storage with WAL mode and tokio-rusqlite
- Per-contact TOML profiles for facts, preferences, and tone
- Conversation archive with FTS5 full-text search
- Real-time notification polling for Telegram and iMessage
- Platform setup documentation

[0.1.0]: https://github.com/adrewtr/quick-reply/releases/tag/v0.1.0
```

---

### Task 7: Create GitHub Actions CI workflow

**Files:**
- Create: `.github/workflows/ci.yml`

- [ ] **Step 1: Create CI workflow**

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always

jobs:
  check:
    name: Check
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy

      - name: Cache cargo registry
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
          restore-keys: ${{ runner.os }}-cargo-

      - name: Check formatting
        run: cargo fmt --check

      - name: Run clippy
        run: cargo clippy -- -D warnings

      - name: Build
        run: cargo build

      - name: Run tests
        run: cargo test
```

---

### Task 8: Create GitHub Actions release workflow

**Files:**
- Create: `.github/workflows/release.yml`

- [ ] **Step 1: Create release workflow**

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

permissions:
  contents: write

jobs:
  build:
    name: Build ${{ matrix.name }}
    runs-on: ${{ matrix.os }}
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
      - uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - name: Build release binary
        run: cargo build --release --target ${{ matrix.target }}

      - name: Package binary
        run: |
          cd target/${{ matrix.target }}/release
          tar czf ../../../${{ matrix.name }}.tar.gz quick-reply
          cd ../../..

      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: ${{ matrix.name }}
          path: ${{ matrix.name }}.tar.gz

  release:
    name: Create Release
    needs: build
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Download all artifacts
        uses: actions/download-artifact@v4
        with:
          path: artifacts

      - name: Create GitHub Release
        uses: softprops/action-gh-release@v2
        with:
          generate_release_notes: true
          files: |
            artifacts/**/*.tar.gz
```

---

### Task 9: Create GitHub issue templates

**Files:**
- Create: `.github/ISSUE_TEMPLATE/bug_report.md`
- Create: `.github/ISSUE_TEMPLATE/feature_request.md`

- [ ] **Step 1: Create bug report template**

```markdown
---
name: Bug Report
about: Report a bug to help improve quick-reply
title: ''
labels: bug
assignees: ''
---

## Environment

- **OS**: macOS [version]
- **quick-reply version**: [e.g. 0.1.0]
- **Platform affected**: [Telegram / iMessage / Slack / WhatsApp / Messenger / All]

## Description

A clear description of what the bug is.

## Steps to Reproduce

1. Launch quick-reply with config: ...
2. Navigate to ...
3. Press ...
4. See error

## Expected Behavior

What you expected to happen.

## Actual Behavior

What actually happened.

## Logs

Paste relevant output from `/tmp/quick-reply.log` if available.
```

- [ ] **Step 2: Create feature request template**

```markdown
---
name: Feature Request
about: Suggest an idea for quick-reply
title: ''
labels: enhancement
assignees: ''
---

## Description

A clear description of the feature you'd like.

## Use Case

Why would this be useful? What problem does it solve?

## Alternatives Considered

Any alternative solutions or features you've considered.
```

---

### Task 10: Update Cargo.toml metadata and initial commit

**Files:**
- Modify: `Cargo.toml` (add metadata fields)

- [ ] **Step 1: Add package metadata to Cargo.toml**

Add these fields to the `[package]` section:

```toml
[package]
name = "quick-reply"
version = "0.1.0"
edition = "2021"
description = "Unified messaging TUI with AI-powered reply suggestions"
license = "MIT"
repository = "https://github.com/adrewtr/quick-reply"
homepage = "https://github.com/adrewtr/quick-reply"
readme = "README.md"
keywords = ["tui", "messaging", "telegram", "claude", "terminal"]
categories = ["command-line-interface"]
```

- [ ] **Step 2: Stage all files**

Run: `git add .gitignore LICENSE Makefile README.md CONTRIBUTING.md CHANGELOG.md .github/ Cargo.toml Cargo.lock src/ config/ docs/ bridges/ tests/`

- [ ] **Step 3: Create initial commit**

```bash
git commit -m "$(cat <<'EOF'
feat: initial release — unified messaging TUI with AI suggestions

- 3-panel TUI with keyboard navigation (ratatui + crossterm)
- 5 platform adapters: Telegram, iMessage, WhatsApp, Slack, Messenger
- Claude Code CLI integration for AI reply suggestions
- Memory system: contact profiles + conversation archive + FTS5 search
- SQLite storage with WAL mode and tokio-rusqlite
- GitHub Actions CI (build, clippy, fmt, test)
- Tag-triggered release workflow for macOS binaries
- Full platform setup documentation

Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>
EOF
)"
```

- [ ] **Step 4: Verify commit**

Run: `git log --oneline -1`
Expected: Shows the commit with the feat message.

- [ ] **Step 5: Verify make check passes on committed code**

Run: `make check`
Expected: All checks pass.

---

### Task 11: Tag v0.1.0

**Files:** None (git operation only)

- [ ] **Step 1: Create annotated tag**

```bash
git tag -a v0.1.0 -m "v0.1.0 — initial release"
```

- [ ] **Step 2: Verify tag**

Run: `git tag -l`
Expected: Shows `v0.1.0`

Note: Do NOT push yet. The user will create the GitHub repo and push when ready.
