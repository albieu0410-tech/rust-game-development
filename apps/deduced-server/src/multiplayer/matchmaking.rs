use std::sync::Arc;

use deduced_protocol::QueueResponse;

use crate::multiplayer::lobby;
use crate::state::AppState;

/// Joins the Quick Match queue. Since only two players are ever needed, the
/// queue is just "is someone already waiting?" — the first caller waits, the
/// second caller pairs with them immediately and both learn the match id
/// (the first player picks it up on their next `status` poll).
pub async fn enqueue(state: &Arc<AppState>, player_id: String) -> QueueResponse {
    let waiting_player = {
        let mut waiting = state
            .multiplayer
            .matchmaking_waiting
            .lock()
            .expect("matchmaking waiting mutex poisoned");
        waiting.take()
    };

    match waiting_player {
        Some(other_player_id) if other_player_id == player_id => {
            // Already queued (e.g. a retried request); stay waiting.
            *state
                .multiplayer
                .matchmaking_waiting
                .lock()
                .expect("matchmaking waiting mutex poisoned") = Some(other_player_id);
            QueueResponse {
                status: "waiting".to_string(),
                match_id: None,
            }
        }
        Some(other_player_id) => {
            let match_id =
                lobby::create_match_for_pair(state, other_player_id.clone(), player_id).await;
            state
                .multiplayer
                .matchmaking_matched
                .lock()
                .expect("matchmaking matched mutex poisoned")
                .insert(other_player_id, match_id.clone());
            QueueResponse {
                status: "matched".to_string(),
                match_id: Some(match_id),
            }
        }
        None => {
            *state
                .multiplayer
                .matchmaking_waiting
                .lock()
                .expect("matchmaking waiting mutex poisoned") = Some(player_id);
            QueueResponse {
                status: "waiting".to_string(),
                match_id: None,
            }
        }
    }
}

/// Polled by whichever player queued first, to learn once someone else has
/// joined and paired with them.
pub fn status(state: &Arc<AppState>, player_id: &str) -> QueueResponse {
    let mut matched = state
        .multiplayer
        .matchmaking_matched
        .lock()
        .expect("matchmaking matched mutex poisoned");

    if let Some(match_id) = matched.remove(player_id) {
        return QueueResponse {
            status: "matched".to_string(),
            match_id: Some(match_id),
        };
    }

    QueueResponse {
        status: "waiting".to_string(),
        match_id: None,
    }
}

pub fn leave(state: &Arc<AppState>, player_id: &str) {
    let mut waiting = state
        .multiplayer
        .matchmaking_waiting
        .lock()
        .expect("matchmaking waiting mutex poisoned");
    if waiting.as_deref() == Some(player_id) {
        *waiting = None;
    }
}
