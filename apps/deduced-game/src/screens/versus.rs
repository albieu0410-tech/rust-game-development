use std::sync::Mutex;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::Duration;

use bevy::prelude::*;
use tungstenite::Message;
use tungstenite::stream::MaybeTlsStream;

use deduced_protocol::{ClientMessage, ComparisonDto, QueueResponse, ServerMessage};

use crate::screens::nav;
use crate::state::{AppState, ContentRes, SaveRes};
use crate::theme::{
    self, ACCENT, DIFFERENT, MATCH, RADIUS_LG, RADIUS_MD, RADIUS_PILL, RADIUS_SM, SURFACE, TEXT,
    TEXT_DIM, comparison_color, comparison_symbol,
};

#[derive(Component)]
pub struct OnVersusScreen;

#[derive(Component)]
pub(crate) struct VersusContent;

#[derive(Component)]
pub(crate) enum VersusAction {
    QuickMatch,
    Cancel,
    Done,
}

#[derive(Component)]
pub(crate) struct GuessButton(String);

#[derive(Default, Clone, PartialEq)]
pub enum VersusPhase {
    #[default]
    Idle,
    Searching,
    Playing {
        category_id: String,
        your_attempts_used: usize,
        opp_attempts_used: usize,
        max_attempts: usize,
        you_solved: bool,
    },
    Finished {
        you_won: Option<bool>,
    },
    Error(String),
}

enum VersusEvent {
    Matched,
    Server(ServerMessage),
    Failed(String),
}

/// Drives one Quick Match end-to-end: enqueue, poll for an opponent, connect
/// the match WebSocket, then relay `ServerMessage`s back to the game thread
/// and `ClientMessage`s (guesses) out to the server. Runs entirely on a
/// background thread so Bevy's Update loop is never blocked on the network.
#[derive(Resource, Default)]
pub struct VersusRes {
    pub phase: VersusPhase,
    pub log: Vec<String>,
    pub last_comparisons: Vec<ComparisonDto>,
    outbound: Option<Sender<ClientMessage>>,
    inbound: Option<Mutex<Receiver<VersusEvent>>>,
}

impl VersusRes {
    fn reset(&mut self) {
        self.phase = VersusPhase::Idle;
        self.log.clear();
        self.last_comparisons.clear();
        self.outbound = None;
        self.inbound = None;
    }
}

fn read_timeout(socket: &tungstenite::WebSocket<MaybeTlsStream<std::net::TcpStream>>) {
    if let MaybeTlsStream::Plain(stream) = socket.get_ref() {
        let _ = stream.set_read_timeout(Some(Duration::from_millis(150)));
    }
}

fn run_quick_match(
    player_id: String,
    tx: Sender<VersusEvent>,
    outbound_rx: Receiver<ClientMessage>,
) {
    let base = crate::server::BASE_URL;

    let enqueue: Result<QueueResponse, String> = ureq::post(format!("{base}/matchmaking/queue"))
        .send_json(serde_json::json!({ "player_id": player_id }))
        .map_err(|err| err.to_string())
        .and_then(|mut response| {
            response
                .body_mut()
                .read_json::<QueueResponse>()
                .map_err(|err| err.to_string())
        });

    let mut match_id = match enqueue {
        Ok(response) => response.match_id,
        Err(err) => {
            let _ = tx.send(VersusEvent::Failed(err));
            return;
        }
    };

    for _ in 0..120 {
        if match_id.is_some() {
            break;
        }
        std::thread::sleep(Duration::from_secs(1));
        let status: Result<QueueResponse, String> = ureq::get(format!("{base}/matchmaking/status"))
            .query("player_id", &player_id)
            .call()
            .map_err(|err| err.to_string())
            .and_then(|mut response| {
                response
                    .body_mut()
                    .read_json::<QueueResponse>()
                    .map_err(|err| err.to_string())
            });
        match status {
            Ok(response) => match_id = response.match_id,
            Err(err) => {
                let _ = tx.send(VersusEvent::Failed(err));
                return;
            }
        }
    }

    let Some(match_id) = match_id else {
        let _ = tx.send(VersusEvent::Failed(
            "no opponent found - try again later".to_string(),
        ));
        return;
    };
    let _ = tx.send(VersusEvent::Matched);

    let url = format!(
        "ws://127.0.0.1:4000/matches/{match_id}/ws?player_id={player_id}",
        match_id = match_id,
        player_id = player_id
    );
    let mut socket = match tungstenite::connect(url) {
        Ok((socket, _response)) => socket,
        Err(err) => {
            let _ = tx.send(VersusEvent::Failed(err.to_string()));
            return;
        }
    };
    read_timeout(&socket);

    let ready = serde_json::to_string(&ClientMessage::Ready).unwrap_or_default();
    if socket.send(Message::Text(ready.into())).is_err() {
        let _ = tx.send(VersusEvent::Failed("failed to reach opponent".to_string()));
        return;
    }

    loop {
        while let Ok(client_message) = outbound_rx.try_recv() {
            let Ok(json) = serde_json::to_string(&client_message) else {
                continue;
            };
            if socket.send(Message::Text(json.into())).is_err() {
                return;
            }
        }

        match socket.read() {
            Ok(Message::Text(text)) => {
                if let Ok(server_message) = serde_json::from_str::<ServerMessage>(&text) {
                    let finished = matches!(server_message, ServerMessage::MatchFinished { .. });
                    if tx.send(VersusEvent::Server(server_message)).is_err() || finished {
                        return;
                    }
                }
            }
            Ok(Message::Close(_)) => return,
            Ok(_) => {}
            Err(tungstenite::Error::Io(err)) if err.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(_) => return,
        }
    }
}

pub fn setup(mut commands: Commands, mut versus_res: ResMut<VersusRes>, content: Res<ContentRes>) {
    versus_res.reset();

    let mut container_entity = None;

    commands
        .spawn((
            OnVersusScreen,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(24.0)),
                row_gap: Val::Px(14.0),
                ..default()
            },
            theme::app_background(),
        ))
        .with_children(|root| {
            root.spawn((
                Text::new("Versus"),
                theme::heading_font(24.0),
                TextColor(TEXT),
            ));

            container_entity = Some(
                root.spawn((
                    VersusContent,
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(10.0),
                        flex_grow: 1.0,
                        ..default()
                    },
                ))
                .id(),
            );

            nav::spawn(root, AppState::Versus);
        });

    if let Some(container) = container_entity {
        render(&mut commands, container, &content.0, &versus_res);
    }
}

fn render(
    commands: &mut Commands,
    container: Entity,
    content: &deduced_core::GameContent,
    versus_res: &VersusRes,
) {
    commands.entity(container).despawn_children();
    commands.entity(container).with_children(|root| {
        match &versus_res.phase {
            VersusPhase::Idle => {
                root.spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(10.0),
                        padding: UiRect::all(Val::Px(24.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        border_radius: BorderRadius::all(Val::Px(RADIUS_LG)),
                        ..default()
                    },
                    BackgroundColor(SURFACE),
                    BorderColor::all(theme::BORDER),
                    theme::card_shadow(),
                ))
                .with_children(|card| {
                    card.spawn((
                        Text::new("Quick Match"),
                        theme::heading_font(18.0),
                        TextColor(TEXT),
                    ));
                    card.spawn((
                        Text::new("Find a live opponent and race to solve the same puzzle."),
                        theme::body_font(13.0),
                        TextColor(TEXT_DIM),
                    ));
                    card.spawn((
                        VersusAction::QuickMatch,
                        Button,
                        Node {
                            width: Val::Px(200.0),
                            height: Val::Px(48.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            margin: UiRect::top(Val::Px(8.0)),
                            border_radius: BorderRadius::all(Val::Px(RADIUS_MD)),
                            ..default()
                        },
                        BackgroundColor(ACCENT),
                        theme::button_shadow(),
                    ))
                    .with_children(|button| {
                        button.spawn((
                            Text::new("Find Opponent"),
                            theme::label_font(15.0),
                            TextColor(TEXT),
                        ));
                    });
                });
            }
            VersusPhase::Searching => {
                root.spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(10.0),
                        padding: UiRect::all(Val::Px(24.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        border_radius: BorderRadius::all(Val::Px(RADIUS_LG)),
                        ..default()
                    },
                    BackgroundColor(SURFACE),
                    BorderColor::all(theme::BORDER),
                ))
                .with_children(|card| {
                    card.spawn((
                        Text::new("Searching for an opponent..."),
                        theme::label_font(15.0),
                        TextColor(TEXT),
                    ));
                    card.spawn((
                        VersusAction::Cancel,
                        Button,
                        Node {
                            width: Val::Px(160.0),
                            height: Val::Px(42.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            margin: UiRect::top(Val::Px(8.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            border_radius: BorderRadius::all(Val::Px(RADIUS_MD)),
                            ..default()
                        },
                        BackgroundColor(SURFACE),
                        BorderColor::all(theme::BORDER),
                    ))
                    .with_children(|button| {
                        button.spawn((
                            Text::new("Cancel"),
                            theme::label_font(13.0),
                            TextColor(TEXT_DIM),
                        ));
                    });
                });
            }
            VersusPhase::Playing {
                category_id,
                your_attempts_used,
                opp_attempts_used,
                max_attempts,
                you_solved,
            } => {
                root.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                })
                .with_children(|header| {
                    header.spawn((
                        Text::new(category_id.clone()),
                        theme::heading_font(18.0),
                        TextColor(TEXT),
                    ));
                    header.spawn((
                        Text::new(format!(
                            "You {your_attempts_used}/{max_attempts}  -  Opponent {opp_attempts_used}/{max_attempts}"
                        )),
                        theme::body_font(12.0),
                        TextColor(TEXT_DIM),
                    ));
                });

                if !versus_res.last_comparisons.is_empty() {
                    root.spawn(Node {
                        flex_direction: FlexDirection::Row,
                        flex_wrap: FlexWrap::Wrap,
                        column_gap: Val::Px(6.0),
                        row_gap: Val::Px(6.0),
                        ..default()
                    })
                    .with_children(|chips| {
                        for comparison in &versus_res.last_comparisons {
                            let color = comparison_color(parse_comparison(&comparison.comparison));
                            chips
                                .spawn((
                                    Node {
                                        padding: UiRect::axes(Val::Px(8.0), Val::Px(5.0)),
                                        border_radius: BorderRadius::all(Val::Px(RADIUS_PILL)),
                                        ..default()
                                    },
                                    BackgroundColor(color.with_alpha(0.16)),
                                ))
                                .with_children(|chip| {
                                    chip.spawn((
                                        Text::new(format!(
                                            "{} {}",
                                            comparison.label,
                                            comparison_symbol(parse_comparison(&comparison.comparison))
                                        )),
                                        theme::label_font(12.0),
                                        TextColor(color),
                                    ));
                                });
                        }
                    });
                }

                if *you_solved {
                    root.spawn((
                        Text::new("You solved it! Waiting on your opponent..."),
                        theme::label_font(13.0),
                        TextColor(MATCH),
                    ));
                } else {
                    root.spawn((
                        Text::new("YOUR GUESSES"),
                        theme::label_font(10.0),
                        TextColor(TEXT_DIM),
                    ));
                    root.spawn((
                        Node {
                            flex_direction: FlexDirection::Row,
                            flex_wrap: FlexWrap::Wrap,
                            column_gap: Val::Px(8.0),
                            row_gap: Val::Px(8.0),
                            ..default()
                        },
                    ))
                    .with_children(|list| {
                        for answer in content.answers_for_category(category_id) {
                            list.spawn((
                                GuessButton(answer.id.clone()),
                                Button,
                                Node {
                                    padding: UiRect::axes(Val::Px(14.0), Val::Px(9.0)),
                                    border: UiRect::all(Val::Px(1.0)),
                                    border_radius: BorderRadius::all(Val::Px(RADIUS_PILL)),
                                    ..default()
                                },
                                BackgroundColor(SURFACE),
                                BorderColor::all(theme::BORDER),
                            ))
                            .with_children(|button| {
                                button.spawn((
                                    Text::new(answer.name.clone()),
                                    theme::body_font(14.0),
                                    TextColor(TEXT),
                                ));
                            });
                        }
                    });
                }

                root.spawn((
                    Text::new("MATCH LOG"),
                    theme::label_font(10.0),
                    TextColor(TEXT_DIM),
                ));
                root.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(4.0),
                    ..default()
                })
                .with_children(|log| {
                    for line in versus_res.log.iter().rev().take(5) {
                        log.spawn((
                            Text::new(line.clone()),
                            theme::body_font(11.0),
                            TextColor(TEXT_DIM),
                        ));
                    }
                });
            }
            VersusPhase::Finished { you_won } => {
                let (headline, color) = match you_won {
                    Some(true) => ("You won!".to_string(), MATCH),
                    Some(false) => ("You lost this one.".to_string(), DIFFERENT),
                    None => ("Match ended in a draw.".to_string(), TEXT_DIM),
                };
                root.spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(10.0),
                        padding: UiRect::all(Val::Px(24.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        border_radius: BorderRadius::all(Val::Px(RADIUS_LG)),
                        ..default()
                    },
                    BackgroundColor(SURFACE),
                    BorderColor::all(theme::BORDER),
                    theme::card_shadow(),
                ))
                .with_children(|card| {
                    card.spawn((Text::new(headline), theme::heading_font(20.0), TextColor(color)));
                    card.spawn((
                        VersusAction::Done,
                        Button,
                        Node {
                            width: Val::Px(180.0),
                            height: Val::Px(46.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            margin: UiRect::top(Val::Px(8.0)),
                            border_radius: BorderRadius::all(Val::Px(RADIUS_MD)),
                            ..default()
                        },
                        BackgroundColor(ACCENT),
                        theme::button_shadow(),
                    ))
                    .with_children(|button| {
                        button.spawn((Text::new("Done"), theme::label_font(14.0), TextColor(TEXT)));
                    });
                });
            }
            VersusPhase::Error(message) => {
                root.spawn((
                    Text::new(format!("Couldn't set up a match: {message}")),
                    theme::body_font(13.0),
                    TextColor(TEXT_DIM),
                ));
                root.spawn((
                    VersusAction::Done,
                    Button,
                    Node {
                        width: Val::Px(160.0),
                        height: Val::Px(42.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(1.0)),
                        border_radius: BorderRadius::all(Val::Px(RADIUS_SM)),
                        ..default()
                    },
                    BackgroundColor(SURFACE),
                    BorderColor::all(theme::BORDER),
                ))
                .with_children(|button| {
                    button.spawn((Text::new("Back"), theme::label_font(13.0), TextColor(TEXT_DIM)));
                });
            }
        }
    });
}

fn parse_comparison(value: &str) -> deduced_core::Comparison {
    match value {
        "match" => deduced_core::Comparison::Match,
        "higher" => deduced_core::Comparison::Higher,
        "lower" => deduced_core::Comparison::Lower,
        "partial" => deduced_core::Comparison::Partial,
        _ => deduced_core::Comparison::Different,
    }
}

pub fn handle_quick_match_button(
    mut commands: Commands,
    mut interactions: Query<(&Interaction, &VersusAction), Changed<Interaction>>,
    mut versus_res: ResMut<VersusRes>,
    save_res: Res<SaveRes>,
    content: Res<ContentRes>,
    container: Query<Entity, With<VersusContent>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let mut re_render = false;

    for (interaction, action) in &mut interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match action {
            VersusAction::QuickMatch => {
                let (outbound_tx, outbound_rx) = channel::<ClientMessage>();
                let (inbound_tx, inbound_rx) = channel::<VersusEvent>();
                versus_res.outbound = Some(outbound_tx);
                versus_res.inbound = Some(Mutex::new(inbound_rx));
                versus_res.phase = VersusPhase::Searching;
                versus_res.log.clear();
                versus_res.last_comparisons.clear();
                re_render = true;

                let player_id = save_res.profile.player_id.clone();
                std::thread::spawn(move || {
                    run_quick_match(player_id, inbound_tx, outbound_rx);
                });
            }
            VersusAction::Cancel => {
                versus_res.reset();
                re_render = true;
            }
            VersusAction::Done => {
                versus_res.reset();
                next_state.set(AppState::Home);
            }
        }
    }

    if re_render && let Ok(container) = container.single() {
        render(&mut commands, container, &content.0, &versus_res);
    }
}

pub fn handle_guess_buttons(
    mut interactions: Query<(&Interaction, &GuessButton), Changed<Interaction>>,
    versus_res: Res<VersusRes>,
) {
    for (interaction, guess_button) in &mut interactions {
        if *interaction == Interaction::Pressed
            && let Some(outbound) = &versus_res.outbound
        {
            let _ = outbound.send(ClientMessage::Guess {
                answer_id: guess_button.0.clone(),
            });
        }
    }
}

pub fn handle_events(
    mut commands: Commands,
    mut versus_res: ResMut<VersusRes>,
    content: Res<ContentRes>,
    save_res: Res<SaveRes>,
    container: Query<Entity, With<VersusContent>>,
) {
    let events: Vec<VersusEvent> = {
        let Some(receiver) = versus_res.inbound.as_ref() else {
            return;
        };
        let receiver = receiver.lock().unwrap();
        let mut events = Vec::new();
        while let Ok(event) = receiver.try_recv() {
            events.push(event);
        }
        events
    };

    let mut changed = false;
    for event in events {
        changed = true;

        match event {
            VersusEvent::Matched => {
                versus_res
                    .log
                    .push("Opponent found. Connecting...".to_string());
            }
            VersusEvent::Failed(message) => {
                versus_res.phase = VersusPhase::Error(message);
            }
            VersusEvent::Server(message) => match message {
                ServerMessage::MatchStarted {
                    category_id,
                    max_attempts,
                    ..
                } => {
                    versus_res.phase = VersusPhase::Playing {
                        category_id,
                        your_attempts_used: 0,
                        opp_attempts_used: 0,
                        max_attempts,
                        you_solved: false,
                    };
                    versus_res.log.push("Match started!".to_string());
                }
                ServerMessage::GuessResult {
                    attempts_used,
                    comparisons,
                    won,
                    ..
                } => {
                    versus_res.last_comparisons = comparisons;
                    if let VersusPhase::Playing {
                        your_attempts_used,
                        you_solved,
                        ..
                    } = &mut versus_res.phase
                    {
                        *your_attempts_used = attempts_used;
                        *you_solved = won;
                    }
                    versus_res.log.push(if won {
                        "You solved it!".to_string()
                    } else {
                        format!("You guessed wrong ({attempts_used} used)")
                    });
                }
                ServerMessage::OpponentProgress { attempts_used } => {
                    if let VersusPhase::Playing {
                        opp_attempts_used, ..
                    } = &mut versus_res.phase
                    {
                        *opp_attempts_used = attempts_used;
                    }
                    versus_res
                        .log
                        .push(format!("Opponent guessed ({attempts_used} used)"));
                }
                ServerMessage::OpponentSolved { .. } => {
                    versus_res.log.push("Opponent solved it!".to_string());
                }
                ServerMessage::OpponentLeft => {
                    versus_res.log.push("Opponent left the match.".to_string());
                }
                ServerMessage::MatchFinished { winner_id } => {
                    let you_won = winner_id.map(|winner| winner == save_res.profile.player_id);
                    versus_res.phase = VersusPhase::Finished { you_won };
                }
                ServerMessage::Error { message } => {
                    versus_res.log.push(format!("Error: {message}"));
                }
            },
        }
    }

    if changed && let Ok(container) = container.single() {
        render(&mut commands, container, &content.0, &versus_res);
    }
}

pub fn teardown(mut commands: Commands, query: Query<Entity, With<OnVersusScreen>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
