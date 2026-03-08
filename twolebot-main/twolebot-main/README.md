# twolebot

A self-hosted Telegram bot with Claude AI integration. Rust backend with pure Elm frontend.

## Features

- **Telegram Chat Interface** - Talk to Claude Opus 4.5 through Telegram
- **Voice Message Support** - Automatic transcription via Google Gemini
- **Media Handling** - Photos, videos, documents, stickers, and more
- **Cron Scheduling** - Schedule recurring Claude prompts
- **MCP Server** - Integrate with Claude Code for memory, cron jobs, and conversation search
- **Web Dashboard** - Monitor queues, browse messages, view logs
- **Local-First** - Data stored locally (SQLite + files), no cloud dependencies

## Install

```bash
curl -fsSL https://raw.githubusercontent.com/sjalq/twolebot/main/install.sh | bash
```

Then run `twolebot` to start the setup wizard at `http://localhost:8080/setup`.

### Install Modes

#### Runtime (default, no sudo)

```bash
curl -fsSL https://raw.githubusercontent.com/sjalq/twolebot/main/install.sh | bash
```

This installs:
- `twolebot` binary
- Node.js/npm (user-local)
- Claude CLI (user-local)

#### Dev / source build

```bash
curl -fsSL https://raw.githubusercontent.com/sjalq/twolebot/main/install.sh | bash -s -- --dev
```

Dev mode may require `sudo` to install missing system compiler/toolchain packages.

## Architecture

```
┌─────────────────┐     ┌─────────────┐     ┌─────────────────┐
│ Telegram Poller │────▶│    Feed     │────▶│  Claude Manager │
│                 │     │  (queues)   │     │                 │
└─────────────────┘     └─────────────┘     └─────────────────┘
        ▲                                           │
        │                                           ▼
┌─────────────────┐                       ┌─────────────────┐
│ Cron Scheduler  │──────────────────────▶│    Responses    │
└─────────────────┘                       └─────────────────┘
```

- **Telegram Poller**: Long-polling for messages, media download, typing indicators
- **Feed**: SQLite-backed prompt/response queues in unified runtime DB
- **Claude Manager**: Spawns Claude CLI processes with streaming output
- **Cron Scheduler**: Time-based prompt scheduling with MCP tool integration

## Configuration

### API Keys (Runtime DB)

Telegram/Gemini keys are stored in `data/runtime/runtime.sqlite3` and managed through the setup flow:

```bash
twolebot
# then open http://localhost:8080/setup
```

### CLI Arguments

```bash
twolebot \
  --telegram-token "BOT_TOKEN" \
  --gemini-key "API_KEY" \
  --data-dir ./data \
  --port 8080 \
  --claude-model "claude-opus-4-6" \
  --process-timeout-ms 600000
```

### Setup Wizard

On first run, the web UI presents a setup wizard at `http://localhost:8080/setup` for interactive configuration and key validation/storage.

## MCP Server

Twolebot exposes MCP tools via HTTP. Register with Claude Code (adjust port if using `--port`):

```bash
claude mcp add --transport http -s user twolebot http://localhost:8080/mcp
claude mcp add --transport http -s user twolebot-memory http://localhost:8080/mcp/memory
claude mcp add --transport http -s user twolebot-conversations http://localhost:8080/mcp/conversations
```

This enables three tool categories:

**Cron Tools** - Schedule and manage automated Claude jobs
- `schedule_job` - Create one-shot or recurring jobs
- `list_jobs` - View active/paused jobs
- `cancel_job` / `snooze_job` - Job management

**Memory Tools** - Persistent markdown storage
- `memory_read` / `memory_write` - File operations
- `memory_search` - Full-text search across memory files

**Conversation Tools** - Search Claude Code history
- `conversation_search` - Full-text search with context

## Web Dashboard

The Elm frontend provides:

- **Dashboard** - Queue status, recent completions
- **Messages** - Browse chat history by user with media display
- **Logs** - Real-time structured event log
- **Cron Jobs** - View and manage scheduled jobs
- **Settings** - Toggle Claude output options

## Data Storage

```
data/
├── runtime/
│   └── runtime.sqlite3   # Unified general DB (queues + work + runtime secrets + messages + cron + settings + active chats)
├── media/{chatId}/       # Downloaded files
├── vectors.db            # Semantic memory+conversation embeddings (vector DB)
├── memory/               # MCP memory files
└── logs.jsonl            # Event log (10k entries)
```

## API Endpoints

| Endpoint | Description |
|----------|-------------|
| `GET /api/status` | System health |
| `GET /api/feed` | Prompt queue status |
| `GET /api/responses` | Response queue status |
| `GET /api/messages` | All messages |
| `GET /api/messages/{chatId}` | Chat history |
| `GET /api/media/{chatId}/{file}` | Media files |
| `GET /api/logs` | Event log |
| `GET /api/cron/jobs` | List cron jobs |
| `POST /api/cron/jobs` | Create cron job |
| `GET /api/settings` | Current settings |
| `PUT /api/settings` | Update settings |

## Telegram Commands

- `/clear` - Reset conversation context

## Key Constants

| Setting | Value |
|---------|-------|
| Claude Model | `claude-opus-4-6` |
| Gemini Model | `gemini-2.0-flash` |
| Process Timeout | 10 minutes |
| Typing Indicator | Every 6 seconds |
| Log Buffer | 10,000 entries |

## Building

```bash
# Full build with tests
./compile.sh

# Development
cargo build
cd frontend && lamdera make src/Main.elm --output=dist/elm.js
```

## Requirements

- Rust 1.70+
- Elm (via Lamdera CLI)
- **Claude CLI** with Max or Pro subscription (`npm install -g @anthropic-ai/claude-code`)
- Telegram Bot Token (from [@BotFather](https://t.me/BotFather))
- Google Gemini API Key (for voice transcription)

## Data Locations (XDG)

**Linux:**
```
~/.config/twolebot/config.toml      # Configuration (secrets)
~/.local/share/twolebot/            # Data directory
~/.local/state/twolebot/            # Logs
```

**macOS:**
```
~/Library/Application Support/twolebot/config.toml  # Configuration
~/Library/Application Support/twolebot/             # Data directory
```

Override with `--data-dir` or `--config-dir` CLI flags. The general DB and vector DB are:
- `{data_dir}/runtime/runtime.sqlite3`
- `{data_dir}/vectors.db`

## Troubleshooting

### Bot not responding to messages

1. Check the Logs page in the dashboard for errors
2. Verify Telegram token: `curl https://api.telegram.org/bot<TOKEN>/getMe`
3. Ensure Claude CLI is authenticated: `claude --version`
4. Check if a prompt is stuck in "running" state on the Dashboard

### Voice notes / images not working

1. Verify Gemini API key is valid at [Google AI Studio](https://aistudio.google.com/)
2. Check logs for transcription errors
3. Ensure media files are being downloaded: `ls data/media/`

### MCP tools not available in Claude Code

1. Ensure twolebot is running: `curl http://localhost:8080/api/status`
2. Register the MCP servers (see MCP Server section above)
3. **Restart Claude Code** (required to reload MCP connections)
4. Verify with `claude mcp list` - should show twolebot endpoints as connected

### Setup wizard not loading

1. Navigate directly to `http://localhost:8080/setup`
2. Check if port is in use: `lsof -i :8080`
3. Try a different port: `--port 8081`

### Claude responses timing out

1. Default timeout is 10 minutes, increase with `--process-timeout-ms`
2. Check if Claude CLI is rate-limited (Max/Pro subscription required)
3. View the running prompt on Dashboard to see progress

## License

MIT
