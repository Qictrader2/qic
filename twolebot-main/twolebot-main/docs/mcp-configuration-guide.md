# Claude Code MCP Configuration Guide

This document explains how MCP (Model Context Protocol) servers are configured in Claude Code. It covers the common pitfalls, correct file locations, and working examples.

## TL;DR - What Actually Works

```
CORRECT:
  ~/.claude.json              → User-level MCP servers (works!)
  .mcp.json (project root)    → Project-level MCP servers (works!)
                               NOTE: this repo currently ignores `.mcp.json` in `.gitignore`
                               to avoid accidentally committing secrets. If you want a
                               team-shared project MCP config, remove that ignore rule.
  claude mcp add              → CLI command to add servers (works!)

INCORRECT (MCP IGNORED):
  ~/.claude/settings.json     → Does NOT load MCP servers
  ~/.claude/.claude.json      → Internal state, don't edit manually
  ~/.claude/mcp.json          → Silently ignored
  .claude/.mcp.json           → Doesn't work
```

## The Configuration Problem

There's significant confusion about where to put MCP configuration. Multiple config files exist, but **only specific locations actually work** for MCP server definitions.

### Files That DO Work for MCP Servers

| Location | Scope | Purpose |
|----------|-------|---------|
| `~/.claude.json` | User | MCPs available across all projects |
| `.mcp.json` (project root) | Project | MCPs shared with team (version controlled) |
| `claude mcp add --scope local` | Local | MCPs for current project, private to you |

### Files That DO NOT Work for MCP Servers

| Location | What It's Actually For |
|----------|----------------------|
| `~/.claude/settings.json` | Permissions, plugins, general settings - NOT MCP definitions |
| `~/.claude/.claude.json` | Internal state (sessions, stats) - DON'T EDIT |
| `~/.claude/mcp.json` | Silently ignored |
| `.claude/settings.local.json` | Local settings overrides, not MCP |

## How to Add MCP Servers (Recommended)

Use the CLI command - it handles file locations correctly:

```bash
# Add to user scope (available everywhere)
claude mcp add --scope user my-server -- npx -y @my/mcp-server

# Add to project scope (shared via .mcp.json)
claude mcp add --scope project my-server -- npx -y @my/mcp-server

# Add to local scope (default - private, current project only)
claude mcp add my-server -- npx -y @my/mcp-server

# With environment variables
claude mcp add --env API_KEY=xxx my-server -- npx -y @my/mcp-server

# HTTP/remote server
claude mcp add --transport http my-api https://api.example.com/mcp
```

### Management Commands

```bash
# List all configured servers
claude mcp list

# Get details for a specific server
claude mcp get <name>

# Remove a server
claude mcp remove <name>

# Within Claude Code, check status
/mcp
```

## Scope Hierarchy

When servers with the same name exist at multiple scopes:

```
1. Local (highest priority) - ~/.claude.json under project path
2. Project - .mcp.json in project root
3. User (lowest priority) - ~/.claude.json top-level
```

## Manual Configuration Examples

### User-Level (~/.claude.json)

Add `mcpServers` at the TOP LEVEL of the file:

```json
{
  "numStartups": 123,
  "installMethod": "global",
  "autoUpdates": true,
  "mcpServers": {
    "chrome-devtools": {
      "command": "npx",
      "args": ["chrome-devtools-mcp@latest", "--browserUrl", "http://localhost:9222"]
    },
    "todoist": {
      "command": "npx",
      "args": ["-y", "@my/todoist-mcp", "--api-key", "xxx"]
    }
  }
}
```

**WARNING**: `~/.claude.json` contains critical user data (auth, history, settings). Back up before editing!

### Project-Level (.mcp.json)

Create `.mcp.json` in project root:

```json
{
  "mcpServers": {
    "database": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-postgres", "postgresql://..."]
    }
  }
}
```

### Environment Variable Expansion

Some Claude Code setups support variable expansion in `.mcp.json` (verify with `claude mcp list`):

```json
{
  "mcpServers": {
    "api-server": {
      "type": "http",
      "url": "${API_BASE_URL:-https://api.example.com}/mcp",
      "headers": {
        "Authorization": "Bearer ${API_KEY}"
      }
    }
  }
}
```

## Windows-Specific Requirements

On Windows (not WSL), you MUST use the `cmd /c` wrapper:

```json
{
  "mcpServers": {
    "my-server": {
      "command": "cmd",
      "args": ["/c", "npx", "-y", "@my/mcp-server"]
    }
  }
}
```

Without this, you'll get "Connection closed" errors.

## Common Issues and Solutions

### "No MCP servers configured"

1. Check if servers are in the correct file location
2. Run `claude mcp list` to verify
3. Restart Claude Code after config changes

### MCP servers in settings.json are ignored

This is expected behavior. Move them to `~/.claude.json` or use `claude mcp add`.

### Servers show connected but tools don't appear

1. Check server logs: `claude mcp get <name>`
2. Verify the server is running: test the command manually
3. Check permissions in `/mcp` within Claude Code

### Project-scoped servers need approval

Claude Code prompts for approval before using project-scoped servers from `.mcp.json`. Reset with:
```bash
claude mcp reset-project-choices
```

## Where twolebot Fits In

Twolebot exposes MCP tools via HTTP endpoints on the running server. Register with Claude Code using HTTP transport:

```bash
# Ensure twolebot is running on port 8080, then:
claude mcp add --transport http -s user twolebot http://localhost:8080/mcp
claude mcp add --transport http -s user twolebot-memory http://localhost:8080/mcp/memory
claude mcp add --transport http -s user twolebot-conversations http://localhost:8080/mcp/conversations
```

This connects Claude Code directly to the running twolebot instance. No separate MCP process is spawned.

## Debugging MCP Issues

1. **Check what Claude sees**:
   ```bash
   claude mcp list
   ```

2. **Test server manually**:
   ```bash
   # Run the server command directly to see errors
   npx -y @my/mcp-server
   ```

3. **Enable debug logging**:
   ```bash
   MCP_DEBUG=1 claude
   ```

4. **Check server health in Claude**:
   ```
   /mcp
   ```

## References

- [Official MCP Documentation](https://code.claude.com/docs/en/mcp)
- [GitHub Issue #4976 - Configuration file location](https://github.com/anthropics/claude-code/issues/4976)
- [GitHub Issue #3321 - .mcp.json not read](https://github.com/anthropics/claude-code/issues/3321)
- [Windows MCP Guide](https://github.com/BunPrinceton/claude-mcp-windows-guide)
