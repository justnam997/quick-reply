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
