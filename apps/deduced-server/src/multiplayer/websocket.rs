use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use tokio::sync::mpsc;

use deduced_protocol::{ClientMessage, ServerMessage};

use crate::multiplayer::match_actor::ActorEvent;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct WsQuery {
    player_id: String,
}

pub async fn handler(
    ws: WebSocketUpgrade,
    Path(match_id): Path<String>,
    Query(query): Query<WsQuery>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state, match_id, query.player_id))
}

async fn handle_socket(
    socket: WebSocket,
    state: Arc<AppState>,
    match_id: String,
    player_id: String,
) {
    let handle = {
        let matches = state
            .multiplayer
            .matches
            .lock()
            .expect("multiplayer matches mutex poisoned");
        matches.get(&match_id).cloned()
    };
    let Some(handle) = handle else {
        return;
    };

    let (mut ws_sender, mut ws_receiver) = socket.split();
    let (outbound_tx, mut outbound_rx) = mpsc::unbounded_channel::<ServerMessage>();

    handle.send(ActorEvent::Connect {
        player_id: player_id.clone(),
        outbound: outbound_tx,
    });

    let writer_task = tokio::spawn(async move {
        while let Some(message) = outbound_rx.recv().await {
            let Ok(json) = serde_json::to_string(&message) else {
                continue;
            };
            if ws_sender.send(Message::Text(json.into())).await.is_err() {
                break;
            }
        }
    });

    while let Some(Ok(message)) = ws_receiver.next().await {
        if let Message::Text(text) = message
            && let Ok(client_message) = serde_json::from_str::<ClientMessage>(&text)
        {
            handle.send(ActorEvent::Client {
                player_id: player_id.clone(),
                message: client_message,
            });
        }
    }

    handle.send(ActorEvent::Disconnect { player_id });
    writer_task.abort();
}
