use axum::{
    extract::State,
    response::sse::{Event, KeepAlive, Sse},
};
use futures_core::Stream;
use std::convert::Infallible;
use std::sync::Arc;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;

use crate::work::{models::WorkEvent, WorkApp};

/// State for SSE endpoints
#[derive(Clone)]
pub struct SseState {
    pub app: Arc<WorkApp>,
}

/// SSE endpoint: GET /api/work/events
/// Each client gets their own broadcast receiver.
pub async fn work_events(
    State(state): State<SseState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = state.app.event_tx().subscribe();

    let stream = BroadcastStream::new(rx).filter_map(|result| {
        match result {
            Ok(event) => {
                let event_type = match &event {
                    WorkEvent::SelectionUpdated(_) => "selection_updated",
                    WorkEvent::AgentProgress { .. } => "agent_progress",
                };

                match serde_json::to_string(&event) {
                    Ok(data) => Some(Ok(Event::default().event(event_type).data(data))),
                    Err(_) => None,
                }
            }
            Err(_) => None, // Lagged receiver, skip missed events
        }
    });

    Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(std::time::Duration::from_secs(5))
            .text("ping"),
    )
}
