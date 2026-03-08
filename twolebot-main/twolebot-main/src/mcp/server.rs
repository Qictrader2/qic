use crate::cron::CronFeed;
use crate::mcp::conversation_tools::ConversationTools;
use crate::mcp::image_tools::ImageTools;
use crate::mcp::memory_tools::MemoryTools;
use crate::mcp::send_tools::SendTools;
use crate::mcp::tools::CronTools;
use crate::mcp::work_tools::WorkTools;
use crate::semantic::{Embedder, VectorDb};
use crate::storage::CronTopicStore;
use crate::telegram::send::TelegramSender;
use crate::work::WorkApp;
use axum::{body::Body, extract::State, http::Request, response::IntoResponse};
use rmcp::{
    handler::server::tool::ToolCallContext,
    handler::server::ServerHandler,
    model::{
        CallToolRequestParams, CallToolResult, Implementation, ListToolsResult,
        PaginatedRequestParams, ProtocolVersion, ServerCapabilities, ServerInfo, ToolsCapability,
    },
    service::{RequestContext, RoleServer},
    transport::{
        streamable_http_server::session::local::LocalSessionManager, StreamableHttpServerConfig,
        StreamableHttpService,
    },
    ErrorData,
};
use std::path::PathBuf;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

/// Unified MCP Server for twolebot — exposes cron, memory, conversation, and work tools.
#[derive(Clone)]
pub struct TwolebotMcpServer {
    cron_tools: CronTools,
    memory_tools: MemoryTools,
    conversation_tools: ConversationTools,
    work_tools: Option<WorkTools>,
    send_tools: Option<SendTools>,
    image_tools: Option<ImageTools>,
}

impl TwolebotMcpServer {
    pub fn new(
        cron_feed: Arc<CronFeed>,
        memory_dir: PathBuf,
        claude_conversations_dir: PathBuf,
        codex_conversations_dir: PathBuf,
        work_app: Option<Arc<WorkApp>>,
        vector_db: Option<Arc<VectorDb>>,
        embedder: Option<Arc<Embedder>>,
        cron_topic_store: Option<Arc<CronTopicStore>>,
        telegram_sender: Option<Arc<TelegramSender>>,
        send_tools: Option<SendTools>,
        image_tools: Option<ImageTools>,
    ) -> Self {
        let work_tools = work_app.map(WorkTools::new);
        let mut cron_tools = CronTools::new(cron_feed);
        if let (Some(store), Some(sender)) = (cron_topic_store, telegram_sender) {
            cron_tools = cron_tools.with_topic_management(store, sender);
        }
        Self {
            cron_tools,
            memory_tools: MemoryTools::new(memory_dir, vector_db.clone(), embedder.clone()),
            conversation_tools: ConversationTools::new(
                claude_conversations_dir,
                codex_conversations_dir,
                vector_db,
                embedder,
            ),
            work_tools,
            send_tools,
            image_tools,
        }
    }
}

/// State for the MCP HTTP handler
#[derive(Clone)]
pub struct McpHttpState {
    service: StreamableHttpService<TwolebotMcpServer>,
}

impl McpHttpState {
    pub fn new(
        cron_feed: Arc<CronFeed>,
        memory_dir: PathBuf,
        claude_conversations_dir: PathBuf,
        codex_conversations_dir: PathBuf,
        cancellation_token: CancellationToken,
        work_app: Option<Arc<WorkApp>>,
        vector_db: Option<Arc<VectorDb>>,
        embedder: Option<Arc<Embedder>>,
        cron_topic_store: Option<Arc<CronTopicStore>>,
        telegram_sender: Option<Arc<TelegramSender>>,
        send_tools: Option<SendTools>,
        image_tools: Option<ImageTools>,
    ) -> Self {
        let config = StreamableHttpServerConfig {
            stateful_mode: false,
            cancellation_token,
            ..Default::default()
        };

        let service = StreamableHttpService::new(
            move || {
                Ok(TwolebotMcpServer::new(
                    cron_feed.clone(),
                    memory_dir.clone(),
                    claude_conversations_dir.clone(),
                    codex_conversations_dir.clone(),
                    work_app.clone(),
                    vector_db.clone(),
                    embedder.clone(),
                    cron_topic_store.clone(),
                    telegram_sender.clone(),
                    send_tools.clone(),
                    image_tools.clone(),
                ))
            },
            Arc::new(LocalSessionManager::default()),
            config,
        );

        Self { service }
    }
}

/// Axum handler for MCP requests
pub async fn mcp_handler(
    State(state): State<McpHttpState>,
    request: Request<Body>,
) -> impl IntoResponse {
    state.service.handle(request).await
}

impl ServerHandler for TwolebotMcpServer {
    fn get_info(&self) -> ServerInfo {
        // Claude Code v2.1+ uses protocol version 2025-11-25, which rmcp 0.14 doesn't
        // know about yet. Advertise it so the version negotiation picks the client's
        // version instead of downgrading to 2025-03-26 (which makes Claude Code ignore
        // our tools). Fallback to LATEST if deserialization ever fails.
        let protocol_version: ProtocolVersion =
            serde_json::from_value(serde_json::json!("2025-11-25"))
                .unwrap_or(ProtocolVersion::LATEST);
        ServerInfo {
            protocol_version,
            capabilities: ServerCapabilities {
                tools: Some(ToolsCapability {
                    list_changed: Some(false),
                }),
                ..Default::default()
            },
            server_info: Implementation {
                name: "twolebot".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                title: None,
                icons: None,
                website_url: None,
            },
            instructions: Some(
                "Twolebot cron job scheduler. Use these tools to schedule Claude tasks \
                 to run at specific times or on recurring schedules."
                    .to_string(),
            ),
        }
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        let mut items = self.cron_tools.get_tool_router().list_all();
        items.extend(self.memory_tools.get_tool_router().list_all());
        items.extend(self.conversation_tools.get_tool_router().list_all());
        if let Some(ref ct) = self.work_tools {
            items.extend(ct.get_tool_router().list_all());
        }
        if let Some(ref st) = self.send_tools {
            items.extend(st.get_tool_router().list_all());
        }
        if let Some(ref it) = self.image_tools {
            items.extend(it.get_tool_router().list_all());
        }
        // Sanitise inputSchemas so Claude Code accepts them.
        // schemars emits non-standard fields ("$schema", "nullable", "format",
        // "default", etc.) and rmcp can produce empty schemas (`{}`) for tools
        // with no parameters.  Claude Code requires `{"type":"object"}` at minimum.
        for tool in &mut items {
            let mut schema = (*tool.input_schema).clone();
            schema.remove("$schema");
            schema.remove("title");
            // Ensure every schema has "type": "object"
            schema
                .entry("type")
                .or_insert(serde_json::Value::String("object".to_string()));
            if let Some(serde_json::Value::Object(props)) = schema.get_mut("properties") {
                for (_name, val) in props.iter_mut() {
                    if let serde_json::Value::Object(prop) = val {
                        prop.remove("nullable");
                        prop.remove("default");
                        prop.remove("format");
                        prop.remove("minimum");
                        prop.remove("title");
                        // Also strip from nested items (e.g. array item schemas)
                        if let Some(serde_json::Value::Object(items)) = prop.get_mut("items") {
                            items.remove("format");
                            items.remove("nullable");
                            items.remove("default");
                            items.remove("title");
                        }
                    }
                }
            }
            tool.input_schema = Arc::new(schema);
        }
        Ok(ListToolsResult::with_all_items(items))
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let tool_name = request.name.as_ref();

        // Dispatch to the correct tool set based on tool name
        if self.cron_tools.get_tool_router().has_route(tool_name) {
            let tcc = ToolCallContext::new(&self.cron_tools, request, context);
            self.cron_tools.get_tool_router().call(tcc).await
        } else if self.memory_tools.get_tool_router().has_route(tool_name) {
            let tcc = ToolCallContext::new(&self.memory_tools, request, context);
            self.memory_tools.get_tool_router().call(tcc).await
        } else if self
            .conversation_tools
            .get_tool_router()
            .has_route(tool_name)
        {
            let tcc = ToolCallContext::new(&self.conversation_tools, request, context);
            self.conversation_tools.get_tool_router().call(tcc).await
        } else if let Some(ref ct) = self.work_tools {
            if ct.get_tool_router().has_route(tool_name) {
                let tcc = ToolCallContext::new(ct, request, context);
                return ct.get_tool_router().call(tcc).await;
            }
            if let Some(ref st) = self.send_tools {
                if st.get_tool_router().has_route(tool_name) {
                    let tcc = ToolCallContext::new(st, request, context);
                    return st.get_tool_router().call(tcc).await;
                }
            }
            if let Some(ref it) = self.image_tools {
                if it.get_tool_router().has_route(tool_name) {
                    let tcc = ToolCallContext::new(it, request, context);
                    return it.get_tool_router().call(tcc).await;
                }
            }
            Err(ErrorData::invalid_params(
                format!("Unknown tool: {}", tool_name),
                None,
            ))
        } else if let Some(ref st) = self.send_tools {
            if st.get_tool_router().has_route(tool_name) {
                let tcc = ToolCallContext::new(st, request, context);
                return st.get_tool_router().call(tcc).await;
            }
            if let Some(ref it) = self.image_tools {
                if it.get_tool_router().has_route(tool_name) {
                    let tcc = ToolCallContext::new(it, request, context);
                    return it.get_tool_router().call(tcc).await;
                }
            }
            Err(ErrorData::invalid_params(
                format!("Unknown tool: {}", tool_name),
                None,
            ))
        } else if let Some(ref it) = self.image_tools {
            if it.get_tool_router().has_route(tool_name) {
                let tcc = ToolCallContext::new(it, request, context);
                return it.get_tool_router().call(tcc).await;
            }
            Err(ErrorData::invalid_params(
                format!("Unknown tool: {}", tool_name),
                None,
            ))
        } else {
            Err(ErrorData::invalid_params(
                format!("Unknown tool: {}", tool_name),
                None,
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_server_info() {
        let dir = tempdir().unwrap();
        let feed = Arc::new(CronFeed::new(dir.path().join("runtime.sqlite3")).unwrap());
        let server = TwolebotMcpServer::new(
            feed,
            dir.path().join("memory"),
            dir.path().join("conversations"),
            dir.path().join("codex-sessions"),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );

        let info = server.get_info();
        assert_eq!(info.server_info.name, "twolebot");
        assert!(info.capabilities.tools.is_some());
    }
}
