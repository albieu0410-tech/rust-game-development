use std::sync::Arc;

use rand::Rng;
use tokio::sync::oneshot;

use crate::multiplayer::match_actor::{self, ActorEvent};
use crate::state::AppState;

/// Letters/digits with visually ambiguous characters (0/O, 1/I) removed, so a
/// join code is easy to read aloud or retype.
const JOIN_CODE_ALPHABET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";

pub fn create_match(state: &Arc<AppState>, host_player_id: String) -> (String, String) {
    let match_id = random_hex_id(16);
    let join_code = random_join_code();

    let handle = match_actor::spawn_match(Arc::clone(state), match_id.clone(), host_player_id);

    state
        .multiplayer
        .matches
        .lock()
        .expect("multiplayer matches mutex poisoned")
        .insert(match_id.clone(), handle);
    state
        .multiplayer
        .join_codes
        .lock()
        .expect("multiplayer join_codes mutex poisoned")
        .insert(join_code.clone(), match_id.clone());

    (match_id, join_code)
}

pub async fn join_match(
    state: &Arc<AppState>,
    join_code: &str,
    player_id: String,
) -> Result<String, String> {
    let match_id = state
        .multiplayer
        .join_codes
        .lock()
        .expect("multiplayer join_codes mutex poisoned")
        .get(join_code)
        .cloned()
        .ok_or_else(|| "unknown join code".to_string())?;

    let handle = state
        .multiplayer
        .matches
        .lock()
        .expect("multiplayer matches mutex poisoned")
        .get(&match_id)
        .cloned()
        .ok_or_else(|| "match no longer exists".to_string())?;

    let (respond_to, response) = oneshot::channel();
    handle.send(ActorEvent::Join {
        player_id,
        respond_to,
    });

    response
        .await
        .map_err(|_| "match is no longer accepting players".to_string())??;

    Ok(match_id)
}

/// Used by matchmaking, where both players are already known and there is no
/// join code to hand out.
pub async fn create_match_for_pair(
    state: &Arc<AppState>,
    player_a: String,
    player_b: String,
) -> String {
    let match_id = random_hex_id(16);
    let handle = match_actor::spawn_match(Arc::clone(state), match_id.clone(), player_a);

    state
        .multiplayer
        .matches
        .lock()
        .expect("multiplayer matches mutex poisoned")
        .insert(match_id.clone(), handle.clone());

    let (respond_to, response) = oneshot::channel();
    handle.send(ActorEvent::Join {
        player_id: player_b,
        respond_to,
    });
    // The actor was just created with no one else in it, so joining the
    // second (and only other) player cannot fail.
    let _ = response.await;

    match_id
}

fn random_hex_id(len: usize) -> String {
    let mut rng = rand::rng();
    (0..len)
        .map(|_| format!("{:x}", rng.random_range(0u8..16u8)))
        .collect()
}

fn random_join_code() -> String {
    let mut rng = rand::rng();
    (0..6)
        .map(|_| JOIN_CODE_ALPHABET[rng.random_range(0..JOIN_CODE_ALPHABET.len())] as char)
        .collect()
}
