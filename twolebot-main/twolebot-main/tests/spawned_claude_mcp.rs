//! Integration test: spawned Claude MCP access via strict one-off config.
//!
//! These tests are ignored by default because they:
//! - require Claude CLI installed + authenticated
//! - call real Claude APIs (cost/latency)
//! - depend on network and local machine environment

use std::io::Write;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;
use tokio_util::sync::CancellationToken;

use serde_json::{json, Value};
use twolebot::{
    cron::CronFeed,
    logging::SharedLogger,
    mcp::McpHttpState,
    server::{create_router, handlers::AppState},
    storage::{ChatMetadataStore, MediaStore, MessageStore, PromptFeed, ResponseFeed, SettingsStore},
};

const MARKER: &str = "TWOLEBOT_MCP_TEST_MARKER_7x9k2m";
const SERVER_NAME: &str = "twolebot-test";
const EXPECTED_TOOL: &str = "tw.memory.search";

fn find_available_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .expect("bind ephemeral port")
        .local_addr()
        .expect("read local address")
        .port()
}

fn check_claude_cli() -> Result<String, String> {
    let output = std::process::Command::new("claude")
        .arg("--version")
        .output()
        .map_err(|e| format!("Failed to run claude CLI: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err("Claude CLI not found or not working".to_string())
    }
}

fn create_test_state(dir: &TempDir) -> Result<AppState, String> {
    let memory_dir = dir.path().join("memory");
    std::fs::create_dir_all(&memory_dir)
        .map_err(|e| format!("Failed to create memory dir: {}", e))?;

    let marker_path = memory_dir.join("test-marker.md");
    let mut file = std::fs::File::create(&marker_path)
        .map_err(|e| format!("Failed to create marker: {}", e))?;
    writeln!(file, "# MCP Access Test Marker\n\nKeywords: {}", MARKER)
        .map_err(|e| format!("Failed to write marker: {}", e))?;

    let state = AppState {
        prompt_feed: Arc::new(
            PromptFeed::new(dir.path().join("runtime.sqlite3"))
                .map_err(|e| format!("Failed to create prompt feed: {}", e))?,
        ),
        response_feed: Arc::new(
            ResponseFeed::new(dir.path().join("runtime.sqlite3"))
                .map_err(|e| format!("Failed to create response feed: {}", e))?,
        ),
        message_store: Arc::new(
            MessageStore::new(dir.path().join("runtime.sqlite3"))
                .map_err(|e| format!("Failed to create message store: {}", e))?,
        ),
        media_store: Arc::new(
            MediaStore::new(dir.path().join("media"))
                .map_err(|e| format!("Failed to create media store: {}", e))?,
        ),
        cron_feed: Arc::new(
            CronFeed::new(dir.path().join("runtime.sqlite3"))
                .map_err(|e| format!("Failed to create cron feed: {}", e))?,
        ),
        settings_store: Arc::new(
            SettingsStore::new(dir.path().join("runtime.sqlite3"))
                .map_err(|e| format!("Failed to create settings: {}", e))?,
        ),
        chat_metadata_store: Arc::new(ChatMetadataStore::new(dir.path().join("runtime.sqlite3")).map_err(|e| format!("Failed to create chat_metadata_store: {}", e))?),
        logger: SharedLogger::new(dir.path().join("logs.jsonl"))
            .map_err(|e| format!("Failed to create logger: {}", e))?,
        data_dir: dir.path().to_path_buf(),
    };

    Ok(state)
}

fn create_strict_mcp_config(dir: &TempDir, port: u16) -> Result<PathBuf, String> {
    let mcp_config = dir.path().join("test.mcp.json");
    let config_json = json!({
        "mcpServers": {
            SERVER_NAME: {
                "type": "http",
                "url": format!("http://localhost:{}/mcp/memory", port)
            }
        }
    });
    std::fs::write(
        &mcp_config,
        serde_json::to_string_pretty(&config_json).unwrap(),
    )
    .map_err(|e| format!("Failed to write MCP config file: {}", e))?;
    Ok(mcp_config)
}

fn session_id_from_headers(headers: &reqwest::header::HeaderMap) -> Option<String> {
    headers
        .get("mcp-session-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

fn parse_jsonrpc_body(text: &str) -> Result<Value, String> {
    if let Ok(v) = serde_json::from_str::<Value>(text) {
        return Ok(v);
    }

    let mut last_json: Option<Value> = None;
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(data) = trimmed.strip_prefix("data:") {
            let payload = data.trim();
            if payload.is_empty() || payload == "[DONE]" {
                continue;
            }
            if let Ok(v) = serde_json::from_str::<Value>(payload) {
                last_json = Some(v);
            }
        }
    }

    last_json.ok_or_else(|| format!("invalid JSON-RPC response body: '{}'", text))
}

async fn post_jsonrpc(
    client: &reqwest::Client,
    url: &str,
    session_id: Option<&str>,
    body: Value,
) -> Result<(Value, Option<String>), String> {
    let mut req = client.post(url).json(&body);
    req = req.header("accept", "application/json, text/event-stream");
    if let Some(sid) = session_id {
        req = req.header("mcp-session-id", sid);
    }
    let response = req
        .send()
        .await
        .map_err(|e| format!("request failed: {}", e))?;
    let headers = response.headers().clone();
    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|e| format!("read body failed: {}", e))?;
    if !status.is_success() {
        return Err(format!("HTTP {}: {}", status, text));
    }
    let json = parse_jsonrpc_body(&text)?;
    Ok((json, session_id_from_headers(&headers)))
}

async fn post_notification(
    client: &reqwest::Client,
    url: &str,
    session_id: &str,
    body: Value,
) -> Result<(), String> {
    let response = client
        .post(url)
        .header("accept", "application/json, text/event-stream")
        .header("mcp-session-id", session_id)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("notification request failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response
            .text()
            .await
            .unwrap_or_else(|_| "<failed to read body>".to_string());
        return Err(format!("notification failed {}: {}", status, text));
    }
    Ok(())
}

/// Deterministic preflight: fail before Claude prompting if server contract is broken.
async fn assert_server_advertises_expected_tool(port: u16) -> Result<(), String> {
    let client = reqwest::Client::new();
    let mut session: Option<String> = None;
    let endpoint = format!("http://localhost:{}/mcp/memory", port);

    let (init, sid1) = post_jsonrpc(
        &client,
        &endpoint,
        None,
        json!({
            "jsonrpc":"2.0",
            "id": 1,
            "method":"initialize",
            "params":{
                "protocolVersion":"2025-06-18",
                "capabilities":{},
                "clientInfo":{"name":"twolebot-spawned-test","version":"0.0.0"}
            }
        }),
    )
    .await?;
    session = sid1.or(session);
    if init.get("error").is_some() {
        return Err(format!("initialize failed: {}", init));
    }
    let sid = session
        .clone()
        .ok_or_else(|| "initialize response missing mcp-session-id".to_string())?;
    post_notification(
        &client,
        &endpoint,
        &sid,
        json!({
            "jsonrpc":"2.0",
            "method":"notifications/initialized",
            "params":{}
        }),
    )
    .await?;

    let (list, _sid2) = post_jsonrpc(
        &client,
        &endpoint,
        session.as_deref(),
        json!({
            "jsonrpc":"2.0",
            "id": 2,
            "method":"tools/list",
            "params":{}
        }),
    )
    .await?;
    let tools = list
        .get("result")
        .and_then(|r| r.get("tools"))
        .and_then(|t| t.as_array())
        .ok_or_else(|| format!("tools/list missing tools array: {}", list))?;

    let has_expected = tools.iter().any(|tool| {
        tool.get("name")
            .and_then(|n| n.as_str())
            .map(|n| n == EXPECTED_TOOL)
            .unwrap_or(false)
    });
    if !has_expected {
        return Err(format!(
            "Expected '{}' not found in tools/list: {}",
            EXPECTED_TOOL, list
        ));
    }
    Ok(())
}

#[derive(Debug)]
struct SpawnResult {
    used_expected_tool: bool,
    marker_found_in_output: bool,
    init_advertised_expected_tool: bool,
    raw_output: String,
    tool_use_names: Vec<String>,
}

fn collect_tool_use_names(v: &Value, out: &mut Vec<String>) {
    match v {
        Value::Object(map) => {
            let is_tool_use = map
                .get("type")
                .and_then(|t| t.as_str())
                .map(|t| t == "tool_use")
                .unwrap_or(false);
            if is_tool_use {
                if let Some(name) = map.get("name").and_then(|n| n.as_str()) {
                    out.push(name.to_string());
                }
            }
            for value in map.values() {
                collect_tool_use_names(value, out);
            }
        }
        Value::Array(arr) => {
            for value in arr {
                collect_tool_use_names(value, out);
            }
        }
        _ => {}
    }
}

fn looks_like_expected_tool(name: &str) -> bool {
    name.contains("memory.search")
        || name.contains("memory_search")
        || name.ends_with("tw.memory.search")
        || name.ends_with("tw_memory_search")
}

async fn spawn_claude_and_test(
    timeout: Duration,
    mcp_config_path: &Path,
    cwd: &Path,
) -> Result<SpawnResult, String> {
    let prompt = format!(
        "Use an MCP tool to find the marker '{}'. \
         Use the tool that maps to tw.memory.search (namespaced names are fine). \
         Return exactly MCP_ACCESS_CONFIRMED if found, else MCP_ACCESS_FAILED.",
        MARKER
    );

    let mcp_config = mcp_config_path
        .to_str()
        .ok_or_else(|| "mcp config path is not utf-8".to_string())?
        .to_string();
    let output = tokio::time::timeout(timeout, async move {
        tokio::process::Command::new("claude")
            .current_dir(cwd)
            .arg("--dangerously-skip-permissions")
            .arg("--strict-mcp-config")
            .arg("--mcp-config")
            .arg(mcp_config)
            .arg("-p")
            .arg(prompt)
            .arg("--output-format")
            .arg("stream-json")
            .arg("--verbose")
            .arg("--max-turns")
            .arg("4")
            .output()
            .await
    })
    .await
    .map_err(|_| format!("Claude spawn timed out after {:?}", timeout))?
    .map_err(|e| format!("Failed to spawn Claude: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let raw_output = format!("{}\n{}", stdout, stderr);

    let mut init_tools: Vec<String> = Vec::new();
    let mut tool_use_names: Vec<String> = Vec::new();
    for line in stdout.lines().chain(stderr.lines()) {
        if let Ok(v) = serde_json::from_str::<Value>(line) {
            if v.get("type").and_then(|x| x.as_str()) == Some("system")
                && v.get("subtype").and_then(|x| x.as_str()) == Some("init")
            {
                if let Some(tools) = v.get("tools").and_then(|x| x.as_array()) {
                    init_tools = tools
                        .iter()
                        .filter_map(|x| x.as_str().map(|s| s.to_string()))
                        .collect();
                }
            }
            collect_tool_use_names(&v, &mut tool_use_names);
        }
    }

    let init_advertised_expected_tool = init_tools.iter().any(|t| looks_like_expected_tool(t));
    let used_expected_tool = tool_use_names.iter().any(|n| looks_like_expected_tool(n));
    let marker_found_in_output = raw_output.contains(MARKER);

    Ok(SpawnResult {
        used_expected_tool,
        marker_found_in_output,
        init_advertised_expected_tool,
        raw_output,
        tool_use_names,
    })
}

#[tokio::test]
#[ignore] // Run with: cargo test test_spawned_claude_mcp_access --ignored -- --nocapture
async fn test_spawned_claude_mcp_access() {
    println!("\n╔═══════════════════════════════════════════════════════════════╗");
    println!("║     Spawned Claude HTTP MCP Access Test                       ║");
    println!("╚═══════════════════════════════════════════════════════════════╝\n");

    let _claude_version = match check_claude_cli() {
        Ok(v) => {
            println!("✓ Claude CLI found: {}", v);
            v
        }
        Err(e) => {
            println!("✗ {}", e);
            println!("\nSkipping test: Claude CLI not available");
            return;
        }
    };

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    println!("✓ Test directory: {}", temp_dir.path().display());

    let state = create_test_state(&temp_dir).expect("Failed to create test state");
    println!("✓ Test state created with marker file");

    let port = find_available_port();
    println!("✓ Using port: {}", port);

    let cancellation = CancellationToken::new();
    let mcp_state = McpHttpState::new(
        state.cron_feed.clone(),
        temp_dir.path().join("memory"),
        PathBuf::from("/tmp/nonexistent"),
        PathBuf::from("/tmp/nonexistent"),
        cancellation.clone(),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );

    let router = create_router(state, Some(mcp_state), None);
    let addr = format!("127.0.0.1:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind");
    let server_handle = tokio::spawn(async move {
        axum::serve(listener, router).await.ok();
    });

    tokio::time::sleep(Duration::from_millis(500)).await;

    let client = reqwest::Client::new();
    let status_url = format!("http://localhost:{}/api/status", port);
    let status_resp = client
        .get(&status_url)
        .send()
        .await
        .expect("Failed to connect to server");
    assert!(
        status_resp.status().is_success(),
        "Status endpoint failed: {}",
        status_resp.status()
    );
    println!("✓ Server responding to requests");

    assert_server_advertises_expected_tool(port)
        .await
        .expect("MCP preflight contract check failed");
    println!("✓ MCP preflight confirms expected tool is advertised");

    let mcp_config =
        create_strict_mcp_config(&temp_dir, port).expect("Failed to write strict mcp config");
    println!("✓ Strict MCP config created: {}", mcp_config.display());

    let result = spawn_claude_and_test(Duration::from_secs(90), &mcp_config, temp_dir.path()).await;
    server_handle.abort();

    match result {
        Ok(r) => {
            assert!(
                r.init_advertised_expected_tool,
                "Claude init did not advertise expected memory tool. Output:\n{}",
                &r.raw_output[..r.raw_output.len().min(3000)]
            );
            assert!(
                r.used_expected_tool,
                "Claude did not produce structured tool_use for expected memory tool. tool_uses={:?}\nOutput:\n{}",
                r.tool_use_names,
                &r.raw_output[..r.raw_output.len().min(3000)]
            );
            assert!(
                r.marker_found_in_output,
                "Claude used tool but marker not found in output.\nOutput:\n{}",
                &r.raw_output[..r.raw_output.len().min(3000)]
            );
        }
        Err(e) => panic!("Test failed: {}", e),
    }
}
