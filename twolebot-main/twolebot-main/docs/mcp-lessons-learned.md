# The Book of MCP: Lessons Wrought in Fire

*Herein lieth the account of our tribulations with the Model Context Protocol, that those who follow may not stumble upon the same stones.*

## Chapter I: Of the Great Schema Pestilence

And lo, we did build an MCP server most righteous, with fifty and three tools of great utility. And the server did respond faithfully to all who queried it, returning JSON of proper form. Yet Claude Code looked upon our works and declared: **"Capabilities: none."**

And there was much gnashing of teeth.

### The Afflictions of Schemars

The `schemars` crate (v1.0), employed by `rmcp` to generate `inputSchema` for each tool, doth produce schemas polluted with fields that Claude Code's strict validator **silently rejecteth**. Not a single tool shall appear if even ONE tool beareth a transgression.

These are the unclean fields which must be cast out:

| Field | Whence it Cometh | Why it Offendeth |
|-------|-----------------|------------------|
| `"$schema"` | Top-level of inputSchema | schemars declareth the JSON Schema draft; Claude Code forbiddeth it |
| `"title"` | Top-level (e.g. `"title": "ScheduleJobRequest"`) | The struct name leaketh through; Claude careth not for it |
| `"nullable": true` | Property-level | This is an OpenAPI 3.0 abomination, not valid JSON Schema |
| `"default": null` | Property-level | Paired with nullable; unwanted |
| `"format": "int64"` | Property-level and nested `items` | Non-standard format values; Claude rejecteth them |
| `"format": "uint"` | Property-level | Likewise unclean |
| `"minimum": 0` | Property-level | Produced for unsigned integers; causes rejection |

### The Sin of the Empty Schema

And there were tools which required no parameters, such as `board_list` and `live_clear_completed`. And rmcp did render their inputSchema as `{}` — an empty object, void and without form.

But Claude Code demandeth that every inputSchema contain at minimum `{"type": "object"}`. An empty `{}` is an abomination unto the validator, and it shall cast out ALL thy tools for this transgression.

## Chapter II: Of the Protocol Version Schism

Claude Code v2.1.32 doth send `protocolVersion: "2025-11-25"` in its initialize request. Yet rmcp v0.14, knowing only of `"2025-03-26"`, did respond with the lesser version. And Claude Code, seeing this disagreement, turned its face from our tools.

**The remedy:** Override `get_info()` to advertise `"2025-11-25"` via serde deserialization, for the `ProtocolVersion` struct hath a private inner field that cannot be set directly:

```rust
let protocol_version: ProtocolVersion =
    serde_json::from_value(serde_json::json!("2025-11-25"))
        .unwrap_or(ProtocolVersion::LATEST);
```

## Chapter III: Of the Silent Rejection

And this is the cruellest teaching of all: **Claude Code doth not cry out when it rejecteth thy tools.** It showeth "connected" with a cheerful checkmark, yet declareth "Capabilities: none" and offereth no explanation, no error, no log of what displeased it.

The only path to understanding is to intercept the traffic with a proxy and compare thy responses against a minimal working MCP server, tool by tool, field by field, until the offending element is found.

### The Debugging Proxy

A Python proxy script (`/tmp/twolebot-mcp-proxy2.py`) was fashioned to sit between Claude Code and the server, logging all traffic to `/tmp/mcp-proxy2.log`. Register it thusly:

```bash
claude mcp add --transport stdio -s user twolebot -- python3 /tmp/twolebot-mcp-proxy2.py
```

## Chapter IV: The Sanitisation Covenant

All schema cleansing is performed in `src/mcp/server.rs` within the `list_tools()` method of `ServerHandler`. This runneth for BOTH stdio and HTTP transports, for they share the same handler.

The covenant is simple: after gathering tools from all routers, iterate and purge every unclean field before returning them unto the client.

This must be maintained whenever new tools are added. If a new tool appeareth and Claude Code suddenly showeth "Capabilities: none" once more, suspect the schemas first.

## Chapter V: Of Registration Upon Startup

When `twolebot` starteth (the `Run` command), it calleth `register_mcp_with_claude()` which:

1. Removeth any stale registration: `claude mcp remove -s user twolebot`
2. Registereth anew at **user scope**: `claude mcp add --transport stdio -s user twolebot -- <exe> mcp-stdio`

User scope maketh the MCP available to ALL Claude Code sessions. The binary path is resolved via `std::env::current_exe()`.

## Epilogue

Let it be known: the MCP protocol is unforgiving of schema impurity. Test thy tools with a minimal client. Compare thy output against servers that work. And trust not the "connected" status, for it is a false prophet.

*Written in the year of our Lord 2026, after three days wandering in the wilderness of "Capabilities: none".*
