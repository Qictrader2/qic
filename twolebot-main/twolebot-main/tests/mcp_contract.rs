//! MCP protocol contract tests (initialize -> tools/list -> tools/call).

use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::Arc;

use serde_json::{json, Value};
use tempfile::tempdir;
use tokio_util::sync::CancellationToken;

use twolebot::{
    cron::CronFeed,
    logging::SharedLogger,
    mcp::{McpHttpState, TwolebotMcpServer},
    server::{create_router, handlers::AppState},
    storage::{ChatMetadataStore, MediaStore, MessageStore, PromptFeed, ResponseFeed, SettingsStore},
};

fn find_available_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .expect("bind ephemeral port")
        .local_addr()
        .expect("read local addr")
        .port()
}

fn create_test_state(dir: &tempfile::TempDir) -> AppState {
    AppState {
        prompt_feed: Arc::new(PromptFeed::new(dir.path().join("runtime.sqlite3")).unwrap()),
        response_feed: Arc::new(ResponseFeed::new(dir.path().join("runtime.sqlite3")).unwrap()),
        message_store: Arc::new(MessageStore::new(dir.path().join("runtime.sqlite3")).unwrap()),
        media_store: Arc::new(MediaStore::new(dir.path().join("media")).unwrap()),
        cron_feed: Arc::new(CronFeed::new(dir.path().join("runtime.sqlite3")).unwrap()),
        settings_store: Arc::new(SettingsStore::new(dir.path().join("runtime.sqlite3")).unwrap()),
        chat_metadata_store: Arc::new(ChatMetadataStore::new(dir.path().join("runtime.sqlite3")).unwrap()),
        logger: SharedLogger::new(dir.path().join("logs.jsonl")).unwrap(),
        data_dir: dir.path().to_path_buf(),
    }
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

    let parsed = parse_jsonrpc_body(&text)?;
    Ok((parsed, session_id_from_headers(&headers)))
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

async fn mcp_contract_flow(url: &str, marker: &str) -> Result<(), String> {
    let client = reqwest::Client::new();
    let mut session: Option<String> = None;

    // initialize
    let initialize = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2025-06-18",
            "capabilities": {},
            "clientInfo": { "name": "twolebot-test", "version": "0.0.0" }
        }
    });
    let (init_resp, new_session) = post_jsonrpc(&client, url, None, initialize).await?;
    session = new_session.or(session);
    if init_resp.get("error").is_some() {
        return Err(format!("initialize error: {}", init_resp));
    }
    // In stateless mode the server may not return a session id — that's fine.
    if let Some(sid) = session.as_deref() {
        post_notification(
            &client,
            url,
            sid,
            json!({
                "jsonrpc": "2.0",
                "method": "notifications/initialized",
                "params": {}
            }),
        )
        .await?;
    }

    // tools/list
    let (list_resp, sid2) = post_jsonrpc(
        &client,
        url,
        session.as_deref(),
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list",
            "params": {}
        }),
    )
    .await?;
    session = sid2.or(session);

    let tools = list_resp
        .get("result")
        .and_then(|r| r.get("tools"))
        .and_then(|t| t.as_array())
        .ok_or_else(|| format!("tools/list missing tools array: {}", list_resp))?;

    let has_memory_search = tools.iter().any(|tool| {
        tool.get("name")
            .and_then(|n| n.as_str())
            .map(|n| n == "memory_search")
            .unwrap_or(false)
    });
    if !has_memory_search {
        return Err(format!(
            "tools/list did not advertise memory_search: {}",
            list_resp
        ));
    }

    // tools/call memory_search
    let (call_resp, _sid3) = post_jsonrpc(
        &client,
        url,
        session.as_deref(),
        json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": "memory_search",
                "arguments": {
                    "query": marker,
                    "limit": 5,
                    "offset": 0
                }
            }
        }),
    )
    .await?;
    if call_resp.get("error").is_some() {
        return Err(format!("tools/call error: {}", call_resp));
    }

    let text_payload = call_resp
        .get("result")
        .and_then(|r| r.get("content"))
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|first| first.get("text"))
        .and_then(|t| t.as_str())
        .ok_or_else(|| format!("tools/call missing text result: {}", call_resp))?;

    let search_results: Value = serde_json::from_str(text_payload).map_err(|e| {
        format!(
            "memory_search non-JSON text payload: {} ({})",
            text_payload, e
        )
    })?;
    let found_marker = search_results
        .as_array()
        .map(|rows| rows.iter().any(|r| r.to_string().contains(marker)))
        .unwrap_or(false);
    if !found_marker {
        return Err(format!(
            "memory_search response did not include marker '{}': {}",
            marker, text_payload
        ));
    }

    Ok(())
}

#[tokio::test]
async fn test_mcp_contract_on_unified_endpoint() {
    let marker = "TWOLEBOT_MCP_CONTRACT_MARKER_13v2";
    let dir = tempdir().unwrap();

    let memory_dir = dir.path().join("memory");
    std::fs::create_dir_all(&memory_dir).unwrap();
    std::fs::write(
        memory_dir.join("marker.md"),
        format!("# marker\n\n{}\n", marker),
    )
    .unwrap();

    let state = create_test_state(&dir);
    let cancellation = CancellationToken::new();
    let mcp_state = McpHttpState::new(
        state.cron_feed.clone(),
        memory_dir,
        PathBuf::from("/tmp/nonexistent"),
        PathBuf::from("/tmp/nonexistent"),
        cancellation,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );
    let router = create_router(state, Some(mcp_state), None);

    let port = find_available_port();
    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{port}"))
        .await
        .unwrap();
    let server = tokio::spawn(async move {
        axum::serve(listener, router).await.ok();
    });

    let unified = format!("http://localhost:{port}/mcp");
    mcp_contract_flow(&unified, marker).await.unwrap();

    let memory_alias = format!("http://localhost:{port}/mcp/memory");
    mcp_contract_flow(&memory_alias, marker).await.unwrap();

    server.abort();
}

/// Stdio transport contract test: initialize -> list_tools -> call_tool over duplex channel.
#[tokio::test]
async fn test_mcp_stdio_contract() {
    use rmcp::{handler::client::ClientHandler, model::ClientInfo, ServiceExt};

    #[derive(Debug, Clone, Default)]
    struct MinimalClient;
    impl ClientHandler for MinimalClient {
        fn get_info(&self) -> ClientInfo {
            ClientInfo::default()
        }
    }

    let marker = "STDIO_CONTRACT_MARKER_42";
    let dir = tempdir().unwrap();

    let memory_dir = dir.path().join("memory");
    std::fs::create_dir_all(&memory_dir).unwrap();
    std::fs::write(
        memory_dir.join("marker.md"),
        format!("# marker\n\n{}\n", marker),
    )
    .unwrap();

    let cron_feed = Arc::new(
        CronFeed::new(dir.path().join("runtime.sqlite3")).unwrap(),
    );

    let (server_transport, client_transport) = tokio::io::duplex(65536);

    // Spawn server side
    let server_handle = tokio::spawn({
        let cron_feed = cron_feed.clone();
        let memory_dir = memory_dir.clone();
        async move {
            let server = TwolebotMcpServer::new(
                cron_feed,
                memory_dir,
                PathBuf::from("/tmp/nonexistent"),
                PathBuf::from("/tmp/nonexistent"),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            );
            server.serve(server_transport).await?.waiting().await?;
            anyhow::Ok(())
        }
    });

    // Client side
    let client = MinimalClient.serve(client_transport).await.unwrap();

    // list_tools — should include memory_search
    let tools = client.list_all_tools().await.unwrap();
    let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_ref()).collect();
    assert!(
        tool_names.contains(&"memory_search"),
        "expected memory_search in tools, got: {:?}",
        tool_names
    );

    // call memory_search
    let result = client
        .call_tool(rmcp::model::CallToolRequestParams {
            meta: None,
            name: "memory_search".into(),
            arguments: Some(
                serde_json::json!({
                    "query": marker,
                    "limit": 5,
                    "offset": 0
                })
                .as_object()
                .unwrap()
                .clone(),
            ),
            task: None,
        })
        .await
        .unwrap();

    let text = result
        .content
        .first()
        .and_then(|c| c.raw.as_text())
        .map(|t| t.text.as_str())
        .expect("expected text content in memory_search result");
    assert!(
        text.contains(marker),
        "memory_search result should contain marker '{}', got: {}",
        marker,
        text
    );

    client.cancel().await.unwrap();
    server_handle.await.unwrap().unwrap();
}
