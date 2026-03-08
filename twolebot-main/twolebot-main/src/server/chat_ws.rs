use axum::{
    extract::{
        ws::{Message, WebSocket},
        Path, Query, State, WebSocketUpgrade,
    },
    response::Response,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tokio_stream::wrappers::BroadcastStream;

/// Events sent to web chat clients via WebSocket
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ChatEvent {
    MessageChunk {
        conversation_id: String,
        content: String,
        sequence: u32,
        is_final: bool,
    },
    ConversationRenamed {
        conversation_id: String,
        name: String,
    },
    TypingIndicator {
        conversation_id: String,
        is_typing: bool,
    },
    Transcribing {
        conversation_id: String,
    },
    MessageUpdated {
        conversation_id: String,
        message_id: String,
        content: String,
    },
    FileMessage {
        conversation_id: String,
        message_id: String,
        filename: String,
        media_path: String,
        mime_type: String,
        caption: String,
    },
}

/// A sequenced event stored in the replay buffer for resumability.
#[derive(Debug, Clone)]
struct SequencedEvent {
    seq: u64,
    event: ChatEvent,
}

/// Per-conversation channel state: broadcast sender + replay buffer.
struct ConversationChannel {
    /// Broadcasts (seq, event) pairs so receivers get the authoritative seq atomically.
    tx: broadcast::Sender<(u64, ChatEvent)>,
    /// Ring buffer of recent events for replay on reconnect.
    replay_buffer: VecDeque<SequencedEvent>,
    /// Monotonically increasing sequence counter for this conversation.
    next_seq: u64,
}

const REPLAY_BUFFER_SIZE: usize = 256;

/// Hub managing per-conversation broadcast channels for WebSocket.
/// Each conversation gets its own broadcast channel created on first subscribe.
/// Maintains a replay buffer per conversation for resumable connections.
pub struct ChatEventHub {
    channels: RwLock<HashMap<String, ConversationChannel>>,
}

impl ChatEventHub {
    pub fn new() -> Self {
        Self {
            channels: RwLock::new(HashMap::new()),
        }
    }

    /// Subscribe to events for a conversation. Creates the channel if it doesn't exist.
    pub async fn subscribe(&self, conversation_id: &str) -> broadcast::Receiver<(u64, ChatEvent)> {
        // Fast path: read lock
        {
            let channels = self.channels.read().await;
            if let Some(ch) = channels.get(conversation_id) {
                return ch.tx.subscribe();
            }
        }

        // Slow path: write lock to create channel
        let mut channels = self.channels.write().await;
        let ch = channels
            .entry(conversation_id.to_string())
            .or_insert_with(|| {
                let (tx, _) = broadcast::channel(64);
                ConversationChannel {
                    tx,
                    replay_buffer: VecDeque::with_capacity(REPLAY_BUFFER_SIZE),
                    next_seq: 1,
                }
            });
        ch.tx.subscribe()
    }

    /// Get events from the replay buffer that are newer than `last_seq`.
    /// Returns the events the client missed, enabling resumable connections.
    pub async fn replay_since(
        &self,
        conversation_id: &str,
        last_seq: u64,
    ) -> Vec<(u64, ChatEvent)> {
        let channels = self.channels.read().await;
        let Some(ch) = channels.get(conversation_id) else {
            return Vec::new();
        };
        ch.replay_buffer
            .iter()
            .filter(|se| se.seq > last_seq)
            .map(|se| (se.seq, se.event.clone()))
            .collect()
    }

    /// Send an event to all subscribers of a conversation.
    /// Also stores it in the replay buffer for late joiners.
    /// No-op if no channel exists (nobody ever subscribed).
    pub async fn send(&self, conversation_id: &str, event: ChatEvent) {
        let mut channels = self.channels.write().await;
        if let Some(ch) = channels.get_mut(conversation_id) {
            let seq = ch.next_seq;
            ch.next_seq += 1;

            // Store in replay buffer (ring buffer eviction)
            if ch.replay_buffer.len() >= REPLAY_BUFFER_SIZE {
                ch.replay_buffer.pop_front();
            }
            ch.replay_buffer.push_back(SequencedEvent {
                seq,
                event: event.clone(),
            });

            // Broadcast (seq, event) atomically — receivers get the exact seq
            // that was assigned, no race with concurrent sends.
            match ch.tx.send((seq, event)) {
                Ok(n) => tracing::debug!(
                    "WS hub: sent seq {seq} to {n} receivers for {conversation_id}"
                ),
                Err(_) => tracing::debug!(
                    "WS hub: no active receivers for {conversation_id} (event buffered as seq {seq})"
                ),
            }
        } else {
            tracing::warn!("WS hub: no channel for conversation {conversation_id}");
        }
    }
}

/// State for chat WebSocket endpoints
#[derive(Clone)]
pub struct ChatWsState {
    pub hub: Arc<ChatEventHub>,
}

/// Query params for the WebSocket upgrade request
#[derive(Deserialize)]
pub struct WsConnectQuery {
    /// Last sequence number the client received. Events after this will be replayed.
    pub last_seq: Option<u64>,
}

/// Envelope sent over the WebSocket with a global sequence number per conversation.
#[derive(Serialize)]
struct WsEnvelope {
    seq: u64,
    #[serde(flatten)]
    event: ChatEvent,
}

/// WebSocket upgrade handler: GET /api/chat/ws/:conversation_id?last_seq=N
pub async fn chat_ws(
    State(state): State<ChatWsState>,
    Path(conversation_id): Path<String>,
    query: Query<WsConnectQuery>,
    ws: WebSocketUpgrade,
) -> Response {
    let last_seq = query.last_seq.unwrap_or(0);
    ws.on_upgrade(move |socket| handle_ws(socket, state, conversation_id, last_seq))
}

async fn handle_ws(
    socket: WebSocket,
    state: ChatWsState,
    conversation_id: String,
    last_seq: u64,
) {
    let (mut ws_tx, mut ws_rx) = socket.split();

    // 1. Replay missed events from the buffer
    let replayed = state.hub.replay_since(&conversation_id, last_seq).await;
    let mut highest_seq = last_seq;

    for (seq, event) in replayed {
        let envelope = WsEnvelope { seq, event };
        match serde_json::to_string(&envelope) {
            Ok(json) => {
                if ws_tx.send(Message::Text(json.into())).await.is_err() {
                    return; // client disconnected during replay
                }
                highest_seq = seq;
            }
            Err(e) => tracing::warn!("WS: failed to serialize replay event: {e}"),
        }
    }

    // 2. Subscribe to live events
    let rx = state.hub.subscribe(&conversation_id).await;
    let mut stream = BroadcastStream::new(rx);

    // Send a "connected" frame so the client knows it's live
    let connected = serde_json::json!({
        "type": "connected",
        "conversation_id": conversation_id,
        "last_seq": highest_seq,
    });
    if ws_tx
        .send(Message::Text(connected.to_string().into()))
        .await
        .is_err()
    {
        return;
    }

    // 3. Forward live events until client disconnects
    // We also need to read from ws_rx to detect close frames and pings.
    let ping_interval = tokio::time::interval(std::time::Duration::from_secs(15));
    tokio::pin!(ping_interval);

    loop {
        tokio::select! {
            // Live event from broadcast channel — seq is paired atomically
            maybe_event = stream.next() => {
                match maybe_event {
                    Some(Ok((seq, event))) => {
                        let envelope = WsEnvelope { seq, event };
                        match serde_json::to_string(&envelope) {
                            Ok(json) => {
                                if ws_tx.send(Message::Text(json.into())).await.is_err() {
                                    break;
                                }
                            }
                            Err(e) => tracing::warn!("WS: failed to serialize event: {e}"),
                        }
                    }
                    Some(Err(_)) => {
                        // Lagged behind — broadcast buffer overflowed.
                        // The client should reconnect with last_seq to catch up.
                        tracing::warn!("WS: broadcast lag for {conversation_id}, closing for reconnect");
                        let _ = ws_tx.send(Message::Close(Some(axum::extract::ws::CloseFrame {
                            code: 4001,
                            reason: "lagged".into(),
                        }))).await;
                        break;
                    }
                    None => break, // channel closed
                }
            }

            // Client messages (we mostly ignore, but need to handle close/pong)
            maybe_msg = ws_rx.next() => {
                match maybe_msg {
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(Message::Pong(_))) => {} // expected response to our pings
                    Some(Ok(_)) => {} // ignore other client messages for now
                    Some(Err(_)) => break,
                }
            }

            // Periodic ping to keep the connection alive through proxies
            _ = ping_interval.tick() => {
                if ws_tx.send(Message::Ping(Vec::new())).await.is_err() {
                    break;
                }
            }
        }
    }

    tracing::debug!("WS: connection closed for {conversation_id}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn transcribing_event_serializes_correctly() {
        let event = ChatEvent::Transcribing {
            conversation_id: "conv-123".to_string(),
        };
        let json = serde_json::to_string(&event).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["type"], "transcribing");
        assert_eq!(parsed["conversation_id"], "conv-123");
        assert_eq!(parsed.as_object().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn hub_delivers_transcribing_event_to_subscriber() {
        let hub = ChatEventHub::new();
        let mut rx = hub.subscribe("conv-abc").await;

        hub.send(
            "conv-abc",
            ChatEvent::Transcribing {
                conversation_id: "conv-abc".to_string(),
            },
        )
        .await;

        let (seq, event) = rx.recv().await.unwrap();
        assert_eq!(seq, 1);
        match event {
            ChatEvent::Transcribing { conversation_id } => {
                assert_eq!(conversation_id, "conv-abc");
            }
            other => panic!("Expected Transcribing, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn hub_isolates_conversations() {
        let hub = ChatEventHub::new();
        let mut rx_a = hub.subscribe("conv-a").await;
        let _rx_b = hub.subscribe("conv-b").await;

        hub.send(
            "conv-a",
            ChatEvent::Transcribing {
                conversation_id: "conv-a".to_string(),
            },
        )
        .await;

        let (_seq, event) = rx_a.recv().await.unwrap();
        assert!(matches!(event, ChatEvent::Transcribing { .. }));
        assert!(_rx_b.is_empty());
    }

    #[tokio::test]
    async fn hub_send_without_subscriber_is_noop() {
        let hub = ChatEventHub::new();
        hub.send(
            "nonexistent",
            ChatEvent::Transcribing {
                conversation_id: "nonexistent".to_string(),
            },
        )
        .await;
    }

    #[tokio::test]
    async fn event_type_mapping_covers_all_variants() {
        let cases: Vec<(ChatEvent, &str)> = vec![
            (
                ChatEvent::MessageChunk {
                    conversation_id: "c".into(),
                    content: "hi".into(),
                    sequence: 0,
                    is_final: false,
                },
                "message_chunk",
            ),
            (
                ChatEvent::ConversationRenamed {
                    conversation_id: "c".into(),
                    name: "New Name".into(),
                },
                "conversation_renamed",
            ),
            (
                ChatEvent::TypingIndicator {
                    conversation_id: "c".into(),
                    is_typing: true,
                },
                "typing_indicator",
            ),
            (
                ChatEvent::Transcribing {
                    conversation_id: "c".into(),
                },
                "transcribing",
            ),
            (
                ChatEvent::MessageUpdated {
                    conversation_id: "c".into(),
                    message_id: "m".into(),
                    content: "updated".into(),
                },
                "message_updated",
            ),
            (
                ChatEvent::FileMessage {
                    conversation_id: "c".into(),
                    message_id: "m".into(),
                    filename: "test.pdf".into(),
                    media_path: "/media/test.pdf".into(),
                    mime_type: "application/pdf".into(),
                    caption: "A file".into(),
                },
                "file_message",
            ),
        ];

        for (event, expected_type) in cases {
            let json = serde_json::to_string(&event).unwrap();
            let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
            assert_eq!(
                parsed["type"].as_str().unwrap(),
                expected_type,
                "Wrong type tag for {:?}",
                event
            );
        }
    }

    #[tokio::test]
    async fn replay_buffer_stores_and_replays() {
        let hub = ChatEventHub::new();
        // Subscribe to create the channel
        let _rx = hub.subscribe("conv-1").await;

        // Send 3 events
        for i in 0..3 {
            hub.send(
                "conv-1",
                ChatEvent::MessageChunk {
                    conversation_id: "conv-1".into(),
                    content: format!("chunk-{i}"),
                    sequence: i,
                    is_final: i == 2,
                },
            )
            .await;
        }

        // Replay from seq 0 should get all 3
        let replayed = hub.replay_since("conv-1", 0).await;
        assert_eq!(replayed.len(), 3);
        assert_eq!(replayed[0].0, 1); // seq starts at 1
        assert_eq!(replayed[2].0, 3);

        // Replay from seq 2 should get only the last one
        let replayed = hub.replay_since("conv-1", 2).await;
        assert_eq!(replayed.len(), 1);
        assert_eq!(replayed[0].0, 3);
    }

    #[tokio::test]
    async fn replay_buffer_evicts_old_events() {
        let hub = ChatEventHub::new();
        let _rx = hub.subscribe("conv-1").await;

        // Fill beyond REPLAY_BUFFER_SIZE
        for i in 0..(REPLAY_BUFFER_SIZE + 10) {
            hub.send(
                "conv-1",
                ChatEvent::MessageChunk {
                    conversation_id: "conv-1".into(),
                    content: format!("chunk-{i}"),
                    sequence: i as u32,
                    is_final: false,
                },
            )
            .await;
        }

        // Buffer should contain exactly REPLAY_BUFFER_SIZE events
        let replayed = hub.replay_since("conv-1", 0).await;
        assert_eq!(replayed.len(), REPLAY_BUFFER_SIZE);

        // First event in buffer should be seq 11 (first 10 evicted)
        assert_eq!(replayed[0].0, 11);
    }

    #[tokio::test]
    async fn ws_envelope_serializes_with_seq() {
        let envelope = WsEnvelope {
            seq: 42,
            event: ChatEvent::Transcribing {
                conversation_id: "conv-1".into(),
            },
        };
        let json = serde_json::to_string(&envelope).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["seq"], 42);
        assert_eq!(parsed["type"], "transcribing");
        assert_eq!(parsed["conversation_id"], "conv-1");
    }
}
