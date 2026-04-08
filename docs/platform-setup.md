# Platform Setup Guide

This guide walks you through getting credentials for each messaging platform supported by quick-reply.

## Configuration File

Create `~/.config/quick-reply/config.toml`:

```toml
[general]
tick_rate_ms = 250

[ai]
claude_binary = "claude"
suggestion_timeout_secs = 30

[memory]
recent_messages_count = 20
search_results_count = 5

# --- Enable platforms below ---

[platforms.telegram]
api_id = 12345678
api_hash = "your_api_hash_here"
phone = "+1234567890"

[platforms.imessage]
# macOS only — uses your local Messages database
# db_path = "/Users/YOU/Library/Messages/chat.db"  # auto-detected

[platforms.slack]
user_token = "xoxp-your-user-token-here"

[platforms.whatsapp]
bridge_port = 9100

[platforms.messenger]
bridge_port = 9101
```

Only include sections for platforms you want to use. Omitted sections are skipped.

---

## 1. Telegram

### What you need
- `api_id` (integer)
- `api_hash` (string)
- Your phone number

### Steps

1. Go to [https://my.telegram.org](https://my.telegram.org)
2. Log in with your phone number
3. Click **"API development tools"**
4. Fill in the form:
   - **App title**: `quick-reply` (or anything)
   - **Short name**: `quickreply`
   - **Platform**: Other
   - **Description**: Personal messaging TUI
5. Click **"Create application"**
6. Copy your **api_id** and **api_hash**

### Config

```toml
[platforms.telegram]
api_id = 12345678
api_hash = "0123456789abcdef0123456789abcdef"
phone = "+1234567890"
```

### First run

On first launch, quick-reply will request a login code sent to your Telegram app. Enter it when prompted. The session is saved to `~/.local/share/quick-reply/telegram.session` — subsequent launches skip auth.

---

## 2. iMessage (macOS only)

### What you need
- macOS with Messages.app configured
- **Full Disk Access** permission for your terminal

### Steps

1. Open **System Settings** > **Privacy & Security** > **Full Disk Access**
2. Click the **+** button
3. Add your terminal app:
   - **Terminal.app**: `/Applications/Utilities/Terminal.app`
   - **iTerm2**: `/Applications/iTerm.app`
   - **Alacritty**: `/Applications/Alacritty.app`
   - **WezTerm**: `/Applications/WezTerm.app`
   - **Kitty**: `/Applications/kitty.app`
4. Toggle it **ON**
5. **Restart your terminal** (required for the permission to take effect)

### Verify access

```bash
sqlite3 ~/Library/Messages/chat.db "SELECT COUNT(*) FROM message;" 2>/dev/null
```

If this returns a number, you have access. If it returns an error, check Full Disk Access.

### Config

```toml
[platforms.imessage]
# db_path is auto-detected — only set if your Messages database is in a non-standard location
# db_path = "/Users/YOU/Library/Messages/chat.db"
```

No additional credentials needed — iMessage uses your local Messages database directly.

---

## 3. Slack

### What you need
- A Slack **user token** (`xoxp-...`) with the right scopes

### Option A: Create a Slack App (recommended)

1. Go to [https://api.slack.com/apps](https://api.slack.com/apps)
2. Click **"Create New App"** > **"From scratch"**
3. Name it `quick-reply`, select your workspace
4. Go to **"OAuth & Permissions"** in the sidebar
5. Under **"User Token Scopes"**, add these scopes:
   - `channels:history` — read public channel messages
   - `channels:read` — list public channels
   - `chat:write` — send messages
   - `groups:history` — read private channel messages
   - `groups:read` — list private channels
   - `im:history` — read DM messages
   - `im:read` — list DMs
   - `im:write` — send DMs
   - `mpim:history` — read group DM messages
   - `mpim:read` — list group DMs
   - `users:read` — resolve user names
6. Scroll up and click **"Install to Workspace"**
7. Click **"Allow"**
8. Copy the **User OAuth Token** (starts with `xoxp-`)

### Config

```toml
[platforms.slack]
user_token = "xoxp-YOUR-TOKEN-HERE"
```

### Note
The token acts as you — messages are sent from your account, not a bot.

---

## 4. WhatsApp (requires sidecar bridge)

WhatsApp has no official personal API. quick-reply uses a Go sidecar process powered by [whatsmeow](https://github.com/tulir/whatsmeow) to bridge WhatsApp Web.

### Prerequisites
- Go 1.21+ installed (`brew install go`)

### Build the bridge

```bash
cd bridges/whatsapp-bridge
go build -o whatsapp-bridge .
```

### Run the bridge

```bash
./whatsapp-bridge
# First run: displays a QR code in the terminal
# Scan it with WhatsApp on your phone:
#   WhatsApp > Settings > Linked Devices > Link a Device
```

The bridge runs on `http://127.0.0.1:9100` by default.

### Config

```toml
[platforms.whatsapp]
bridge_port = 9100
```

### Note
- The bridge must be running before launching quick-reply
- WhatsApp sessions expire if your phone is offline for 14+ days
- Re-scan the QR code if the session expires

---

## 5. Messenger (requires sidecar bridge — Tier 2)

Messenger has no official personal API. quick-reply uses a Node.js sidecar with Puppeteer to automate Messenger web.

**Warning:** This is a Tier 2 (best-effort) integration. Facebook actively blocks automation, so this adapter may break periodically.

### Prerequisites
- Node.js 18+ installed (`brew install node`)

### Build the bridge

```bash
cd bridges/messenger-bridge
npm install
```

### Run the bridge

```bash
npm start
# First run: opens a headless Chrome browser
# You'll need to log in to messenger.com
# Session cookies are saved for subsequent launches
```

The bridge runs on `http://127.0.0.1:9101` by default.

### Config

```toml
[platforms.messenger]
bridge_port = 9101
```

### Known limitations
- Session cookies may expire every few hours to days
- Facebook DOM changes can break the bridge
- If the bridge crashes 3+ times, Messenger shows as unavailable
- All other platforms continue working when Messenger is down

---

## Quick Start (minimal setup)

The fastest way to get started is with just **Telegram + iMessage** — no sidecars needed:

```toml
# ~/.config/quick-reply/config.toml

[platforms.telegram]
api_id = 12345678
api_hash = "your_hash"
phone = "+1234567890"

[platforms.imessage]
# Auto-detected on macOS
```

Then run:

```bash
cargo run
```

---

## Troubleshooting

| Issue | Fix |
|-------|-----|
| `Claude CLI not found` | Install Claude Code: `npm install -g @anthropic-ai/claude-code` |
| iMessage: `Cannot read chat.db` | Grant Full Disk Access to your terminal app, then restart it |
| Telegram: `Authorization required` | Delete `~/.local/share/quick-reply/telegram.session` and re-auth |
| Slack: `Auth failed` | Verify your `xoxp-` token has all required scopes |
| WhatsApp: `Bridge not reachable` | Start the bridge first: `./bridges/whatsapp-bridge/whatsapp-bridge` |
| Messenger: `Bridge failed 3 times` | Restart the bridge manually, re-login if cookies expired |
| No AI suggestions | Ensure `claude` is in your PATH and authenticated (`claude auth`) |
