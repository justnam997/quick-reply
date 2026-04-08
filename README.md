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
