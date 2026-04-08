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
