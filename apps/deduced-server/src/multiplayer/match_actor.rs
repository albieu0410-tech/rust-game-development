use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::{mpsc, oneshot};

use deduced_core::{CategoryDefinition, Round, RoundConfig, RoundStatus};
use deduced_protocol::{ClientMessage, ComparisonDto, ServerMessage};

use crate::services::matches as match_history;
use crate::state::AppState;

/// How long a disconnected player has to reconnect (same match id + player
/// id, new WebSocket) before the match is forfeited to their opponent.
const RECONNECT_GRACE_PERIOD: Duration = Duration::from_secs(15);

#[derive(Clone)]
pub struct MatchHandle {
    sender: mpsc::UnboundedSender<ActorEvent>,
}

impl MatchHandle {
    pub fn send(&self, event: ActorEvent) {
        // The match actor task only stops after both players disconnect, so a
        // send failure just means the match already ended; nothing to do.
        let _ = self.sender.send(event);
    }
}

pub enum ActorEvent {
    Join {
        player_id: String,
        respond_to: oneshot::Sender<Result<(), String>>,
    },
    Connect {
        player_id: String,
        outbound: mpsc::UnboundedSender<ServerMessage>,
    },
    Client {
        player_id: String,
        message: ClientMessage,
    },
    Disconnect {
        player_id: String,
    },
    /// Sent by a delayed task after `RECONNECT_GRACE_PERIOD`. `epoch` lets a
    /// reconnect-then-disconnect-again sequence invalidate a stale timer
    /// instead of forfeiting for the wrong disconnect.
    ForfeitCheck {
        player_id: String,
        epoch: u64,
    },
}

struct PlayerSession {
    ready: bool,
    round: Option<Round>,
    outbound: Option<mpsc::UnboundedSender<ServerMessage>>,
    connected: bool,
    disconnect_epoch: u64,
}

impl PlayerSession {
    fn new() -> Self {
        Self {
            ready: false,
            round: None,
            outbound: None,
            connected: false,
            disconnect_epoch: 0,
        }
    }
}

/// Spawns the background task that owns all state for one match: both
/// players' sessions, their independent (but same-target) `Round`s, and the
/// win/loss/forfeit determination. All match mutation happens serially
/// inside this task, so there is no shared-state locking to get wrong.
pub fn spawn_match(state: Arc<AppState>, match_id: String, host_player_id: String) -> MatchHandle {
    let (sender, mut receiver) = mpsc::unbounded_channel::<ActorEvent>();
    let self_sender = sender.clone();

    tokio::spawn(async move {
        let mut players: HashMap<String, PlayerSession> = HashMap::new();
        players.insert(host_player_id, PlayerSession::new());

        let mut category: Option<CategoryDefinition> = None;
        let mut finished = false;

        while let Some(event) = receiver.recv().await {
            match event {
                ActorEvent::Join {
                    player_id,
                    respond_to,
                } => {
                    let outcome = if finished {
                        Err("match has already finished".to_string())
                    } else if players.contains_key(&player_id) {
                        Err("already in this match".to_string())
                    } else if players.len() >= 2 {
                        Err("match is full".to_string())
                    } else {
                        players.insert(player_id, PlayerSession::new());
                        Ok(())
                    };
                    let _ = respond_to.send(outcome);
                }

                ActorEvent::Connect {
                    player_id,
                    outbound,
                } => {
                    if let Some(session) = players.get_mut(&player_id) {
                        session.outbound = Some(outbound);
                        session.connected = true;
                    }
                }

                ActorEvent::Disconnect { player_id } => {
                    let Some(session) = players.get_mut(&player_id) else {
                        continue;
                    };
                    session.connected = false;
                    session.outbound = None;
                    session.disconnect_epoch += 1;
                    let epoch = session.disconnect_epoch;

                    if finished || category.is_none() {
                        // Match hasn't started (or already ended) — nothing
                        // to forfeit; just drop their connection state.
                        continue;
                    }

                    let requeue = self_sender.clone();
                    let player_id_for_timer = player_id.clone();
                    tokio::spawn(async move {
                        tokio::time::sleep(RECONNECT_GRACE_PERIOD).await;
                        let _ = requeue.send(ActorEvent::ForfeitCheck {
                            player_id: player_id_for_timer,
                            epoch,
                        });
                    });
                }

                ActorEvent::ForfeitCheck { player_id, epoch } => {
                    if finished {
                        continue;
                    }
                    let still_disconnected = players
                        .get(&player_id)
                        .map(|session| !session.connected && session.disconnect_epoch == epoch)
                        .unwrap_or(false);

                    if still_disconnected
                        && let Some(opponent_id) = opponent_of(&players, &player_id)
                    {
                        send_to(&players, &opponent_id, ServerMessage::OpponentLeft);
                        finish_match(
                            &state,
                            &match_id,
                            category.as_ref(),
                            &mut players,
                            &mut finished,
                            Some(opponent_id),
                        )
                        .await;
                    }
                }

                ActorEvent::Client { player_id, message } => {
                    if finished || !players.contains_key(&player_id) {
                        continue;
                    }

                    match message {
                        ClientMessage::Ready => {
                            if let Some(session) = players.get_mut(&player_id) {
                                session.ready = true;
                            }
                            if category.is_none()
                                && players.len() == 2
                                && players.values().all(|session| session.ready)
                            {
                                category = Some(start_match(&state, &mut players).await);
                            }
                        }

                        ClientMessage::Guess { answer_id } => {
                            let Some(category_ref) = category.clone() else {
                                send_error(&players, &player_id, "match has not started yet");
                                continue;
                            };

                            handle_guess(
                                &state,
                                &match_id,
                                &category_ref,
                                &mut players,
                                &mut finished,
                                &player_id,
                                &answer_id,
                            )
                            .await;
                        }

                        ClientMessage::Leave => {
                            if category.is_some()
                                && let Some(opponent_id) = opponent_of(&players, &player_id)
                            {
                                send_to(&players, &opponent_id, ServerMessage::OpponentLeft);
                                finish_match(
                                    &state,
                                    &match_id,
                                    category.as_ref(),
                                    &mut players,
                                    &mut finished,
                                    Some(opponent_id),
                                )
                                .await;
                            }
                        }
                    }
                }
            }
        }
    });

    MatchHandle { sender }
}

async fn start_match(
    state: &Arc<AppState>,
    players: &mut HashMap<String, PlayerSession>,
) -> CategoryDefinition {
    let category = pick_random_category(&state.content);
    let seed: u64 = rand::random();

    for session in players.values_mut() {
        let round = Round::new(
            &state.content.answers,
            RoundConfig {
                category: category.id.clone(),
                seed,
                max_attempts: category.attempts,
            },
        )
        .expect("every content category has at least one answer");
        session.round = Some(round);
    }

    broadcast(
        players,
        ServerMessage::MatchStarted {
            category_id: category.id.clone(),
            seed,
            content_version: state.content.content_version.clone(),
            max_attempts: category.attempts,
        },
    );

    category
}

async fn handle_guess(
    state: &Arc<AppState>,
    match_id: &str,
    category: &CategoryDefinition,
    players: &mut HashMap<String, PlayerSession>,
    finished: &mut bool,
    player_id: &str,
    answer_id: &str,
) {
    let Some(answer) = state
        .content
        .answers_for_category(&category.id)
        .find(|answer| answer.id == answer_id)
        .cloned()
    else {
        send_error(players, player_id, "no answer matches that id");
        return;
    };

    let Some(session) = players.get_mut(player_id) else {
        return;
    };
    let Some(round) = session.round.as_mut() else {
        send_error(players, player_id, "match has not started yet");
        return;
    };
    if round.status != RoundStatus::Playing {
        send_error(players, player_id, "you have already finished this round");
        return;
    }

    let Ok(result) = round.submit_guess(category, &answer) else {
        send_error(players, player_id, "guess was rejected");
        return;
    };

    let comparisons = result
        .comparisons
        .iter()
        .map(|comparison| ComparisonDto {
            key: comparison.key.clone(),
            label: comparison.label.clone(),
            guessed_value: comparison.guessed_value.display_value(),
            comparison: comparison_tag(comparison.comparison),
        })
        .collect();
    let won = round.status == RoundStatus::Won;
    let attempts_used = round.attempts_used();
    let max_attempts = round.max_attempts;

    send_to(
        players,
        player_id,
        ServerMessage::GuessResult {
            attempts_used,
            max_attempts,
            comparisons,
            won,
        },
    );

    if let Some(opponent_id) = opponent_of(players, player_id) {
        let message = if won {
            ServerMessage::OpponentSolved { attempts_used }
        } else {
            ServerMessage::OpponentProgress { attempts_used }
        };
        send_to(players, &opponent_id, message);
    }

    if won {
        finish_match(
            state,
            match_id,
            Some(category),
            players,
            finished,
            Some(player_id.to_string()),
        )
        .await;
    } else if players.values().all(|session| {
        matches!(
            session.round.as_ref().map(|r| &r.status),
            Some(RoundStatus::Won | RoundStatus::Lost)
        )
    }) {
        finish_match(state, match_id, Some(category), players, finished, None).await;
    }
}

async fn finish_match(
    state: &Arc<AppState>,
    match_id: &str,
    category: Option<&CategoryDefinition>,
    players: &mut HashMap<String, PlayerSession>,
    finished: &mut bool,
    winner_id: Option<String>,
) {
    if *finished {
        return;
    }
    *finished = true;
    broadcast(
        players,
        ServerMessage::MatchFinished {
            winner_id: winner_id.clone(),
        },
    );

    // Only a started match (has a category) has anything worth recording;
    // a lobby that never filled up isn't a "match" for history purposes.
    if let Some(category) = category {
        let mut player_ids: Vec<&String> = players.keys().collect();
        player_ids.sort();
        if let [player_a, player_b] = player_ids[..]
            && let Err(err) = match_history::record_match_result(
                &state.pool,
                match_id,
                &category.id,
                player_a,
                player_b,
                winner_id.as_deref(),
            )
            .await
        {
            eprintln!("failed to record match history for {match_id}: {err}");
        }
    }
}

fn pick_random_category(content: &deduced_core::GameContent) -> CategoryDefinition {
    let index = rand::random_range(0..content.categories.len());
    content.categories[index].clone()
}

fn comparison_tag(comparison: deduced_core::Comparison) -> String {
    match comparison {
        deduced_core::Comparison::Match => "match",
        deduced_core::Comparison::Higher => "higher",
        deduced_core::Comparison::Lower => "lower",
        deduced_core::Comparison::Different => "different",
        deduced_core::Comparison::Partial => "partial",
    }
    .to_string()
}

fn opponent_of(players: &HashMap<String, PlayerSession>, player_id: &str) -> Option<String> {
    players.keys().find(|id| id.as_str() != player_id).cloned()
}

fn send_to(players: &HashMap<String, PlayerSession>, player_id: &str, message: ServerMessage) {
    if let Some(session) = players.get(player_id)
        && let Some(outbound) = &session.outbound
    {
        let _ = outbound.send(message);
    }
}

fn send_error(players: &HashMap<String, PlayerSession>, player_id: &str, message: &str) {
    send_to(
        players,
        player_id,
        ServerMessage::Error {
            message: message.to_string(),
        },
    );
}

fn broadcast(players: &HashMap<String, PlayerSession>, message: ServerMessage) {
    for session in players.values() {
        if let Some(outbound) = &session.outbound {
            let _ = outbound.send(message.clone());
        }
    }
}
