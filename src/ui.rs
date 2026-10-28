use crate::{
    game::{Action, Phase, Screen, Session},
    world::{Player, hud_text},
};
use bevy::{
    picking::hover::Hovered,
    prelude::*,
    ui_widgets::{Activate, Button},
};
use wrong_number_sim::{CallState, Capability, IncidentState, UnitState};

#[derive(Resource, Default)]
pub struct Actions(pub Vec<Action>);
#[derive(Component)]
pub struct UiRoot;
#[derive(Resource)]
pub struct MapImage(pub Handle<Image>);
#[derive(Resource)]
pub struct CallerPortraits(pub Vec<Handle<Image>>);
pub fn load_map(mut commands: Commands, assets: Res<AssetServer>) {
    commands.insert_resource(MapImage(assets.load("city/map.png")));
    commands.insert_resource(CallerPortraits(
        (0..24)
            .map(|n| assets.load(format!("callers/{n:02}.png")))
            .collect(),
    ));
}
#[derive(Component)]
pub struct UiAction(pub Action);
#[derive(Component)]
pub struct Hud;
const INK: Color = Color::srgb(0.035, 0.055, 0.075);
const PANEL: Color = Color::srgb(0.065, 0.095, 0.12);
const TEXT: Color = Color::srgb(0.9, 0.94, 0.93);
const MUTED: Color = Color::srgb(0.55, 0.66, 0.69);
const TEAL: Color = Color::srgb(0.08, 0.52, 0.48);
const AMBER: Color = Color::srgb(0.92, 0.6, 0.22);
fn text(p: &mut ChildSpawnerCommands, value: impl Into<String>, size: f32, color: Color) {
    p.spawn((
        Node {
            flex_shrink: 0.0,
            ..default()
        },
        Text::new(value),
        TextFont {
            font_size: FontSize::Px(size),
            ..default()
        },
        TextColor(color),
    ));
}
fn button(p: &mut ChildSpawnerCommands, label: impl Into<String>, action: Action, enabled: bool) {
    let mut e = p.spawn((
        Node {
            width: percent(100),
            min_height: px(34),
            flex_shrink: 0.0,
            padding: UiRect::axes(px(10), px(8)),
            margin: UiRect::bottom(px(6)),
            border_radius: BorderRadius::all(px(4)),
            ..default()
        },
        BackgroundColor(if enabled {
            Color::srgb(0.12, 0.23, 0.26)
        } else {
            Color::srgb(0.1, 0.13, 0.15)
        }),
    ));
    if enabled {
        e.insert((Button, Hovered::default(), UiAction(action.clone())))
            .observe(move |_: On<Activate>, mut actions: ResMut<Actions>| {
                actions.0.push(action.clone());
            });
    }
    e.with_children(|p| {
        p.spawn((
            Node {
                width: percent(100),
                min_width: px(0),
                flex_shrink: 1.0,
                ..default()
            },
            Text::new(label),
            TextFont {
                font_size: FontSize::Px(14.0),
                ..default()
            },
            TextColor(if enabled { TEXT } else { MUTED }),
        ));
    });
}
fn column(width: f32) -> Node {
    Node {
        width: percent(width),
        min_height: px(0),
        height: percent(100),
        overflow: Overflow::scroll_y(),
        min_width: px(0),
        flex_direction: FlexDirection::Column,
        padding: UiRect::all(px(16)),
        row_gap: px(8),
        ..default()
    }
}
pub fn unit_label(state: &UnitState, now: u32) -> String {
    match state {
        UnitState::Available => "AVAILABLE".into(),
        UnitState::Assigned { incident } => format!("ASSIGNED / CALL {incident}"),
        UnitState::EnRoute {
            incident,
            arrive_at,
            ..
        } => format!(
            "TO CALL {incident} / ETA {}s",
            arrive_at.saturating_sub(now)
        ),
        UnitState::OnScene { .. } => "ON SCENE".into(),
        UnitState::Resolving { complete_at, .. } => {
            format!("WORKING / {}s", complete_at.saturating_sub(now))
        }
        UnitState::Returning { arrive_at, .. } => {
            format!("RETURNING / {}s", arrive_at.saturating_sub(now))
        }
        UnitState::Recovering { available_at } => {
            format!("RECOVERY / {}s", available_at.saturating_sub(now))
        }
    }
}
fn cap_name(cap: Capability) -> &'static str {
    match cap {
        Capability::Police => "POLICE",
        Capability::Medical => "MEDICAL",
        Capability::Fire => "FIRE",
    }
}
fn cap_color(cap: Capability) -> Color {
    match cap {
        Capability::Police => Color::srgb(0.3, 0.65, 1.0),
        Capability::Medical => Color::srgb(0.9, 0.9, 0.9),
        Capability::Fire => Color::srgb(1.0, 0.4, 0.25),
    }
}
fn map(
    p: &mut ChildSpawnerCommands,
    game: &Session,
    image: &MapImage,
    routes: &crate::routes::RouteOverlay,
) {
    p.spawn((
        Node {
            width: percent(100),
            aspect_ratio: Some(1.5),
            flex_shrink: 0.0,
            position_type: PositionType::Relative,
            overflow: Overflow::clip(),
            ..default()
        },
        BackgroundColor(Color::srgb(0.09, 0.16, 0.17)),
    ))
    .with_children(|p| {
        p.spawn((
            ImageNode::new(image.0.clone()),
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                ..default()
            },
        ));
        p.spawn((
            ImageNode::new(routes.0.clone()),
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                ..default()
            },
        ));
        for unit in game.sim.units().values().filter(|u| u.id <= 3) {
            p.spawn(Node {
                position_type: PositionType::Absolute,
                left: percent(crate::city::map_percent(unit.station).0),
                top: percent(crate::city::map_percent(unit.station).1),
                margin: UiRect {
                    left: px(-20),
                    top: px(-14),
                    ..default()
                },
                ..default()
            })
            .with_children(|p| text(p, format!("H{}", unit.id), 13.0, cap_color(unit.capability)));
        }
        for id in game.visible_ids() {
            let view = game.sim.call_view(*id).unwrap();
            let color = if view.escalated {
                Color::srgb(1.0, 0.15, 0.1)
            } else if view.state == IncidentState::Active {
                AMBER
            } else {
                MUTED
            };
            let offset = if *id == 6 { 18.0 } else { 0.0 };
            let mut e = p.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: percent(crate::city::map_percent(view.location).0),
                    top: percent(crate::city::map_percent(view.location).1),
                    margin: UiRect::left(px(offset)),
                    padding: UiRect::all(px(4)),
                    ..default()
                },
                BackgroundColor(INK),
                Button,
                Hovered::default(),
            ));
            let id = *id;
            e.observe(move |_: On<Activate>, mut a: ResMut<Actions>| a.0.push(Action::Select(id)))
                .with_children(|p| text(p, format!("{id}"), 14.0, color));
        }
        for unit in game.sim.units().values() {
            p.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: percent(crate::city::map_percent(unit.position).0),
                    top: percent(crate::city::map_percent(unit.position).1),
                    width: px(6),
                    height: px(6),
                    ..default()
                },
                BackgroundColor(cap_color(unit.capability)),
            ));
        }
    });
    text(
        p,
        "H1 Police HQ | H2 Hospital | H3 Fire station",
        12.0,
        MUTED,
    );
    text(
        p,
        "Numbered pins = calls | Squares = responders",
        12.0,
        MUTED,
    );
    text(p, "PILOT DISTRICT / LIVE ROAD ROUTES", 11.0, AMBER);
}
fn terminal(
    p: &mut ChildSpawnerCommands,
    game: &Session,
    _image: &MapImage,
    _routes: &crate::routes::RouteOverlay,
    chat: &crate::voice::ChatInput,
    voice: &crate::voice::Voice,
    portraits: &CallerPortraits,
) {
    let running = matches!(game.phase, Phase::Running | Phase::Closing);
    // The desktop contains only resource status and the actual funding ledger.
    p.spawn((
        Node {
            width: percent(100),
            height: percent(100),
            padding: UiRect::all(px(28)),
            column_gap: px(36),
            ..default()
        },
        BackgroundColor(Color::srgb(0.075, 0.13, 0.17)),
    ))
    .with_children(|p| {
        p.spawn(Node {
            width: percent(34),
            height: percent(100),
            min_height: px(0),
            overflow: Overflow::scroll_y(),
            min_width: px(0),
            flex_direction: FlexDirection::Column,
            row_gap: px(18),
            ..default()
        })
        .with_children(|p| {
            text(p, "DISPATCH DESKTOP", 14.0, MUTED);
            text(p, "Dispatch Center", 26.0, TEXT);
            p.spawn((
                Node {
                    padding: UiRect::all(px(22)),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(12),
                    border_radius: BorderRadius::all(px(12)),
                    ..default()
                },
                BackgroundColor(PANEL),
            ))
            .with_children(|p| {
                text(p, "CASH AVAILABLE", 12.0, MUTED);
                text(p, format!("$ {}", game.funds), 38.0, TEXT);
                text(
                    p,
                    format!(
                        "Shift income (projected)   +$ {}",
                        (100 + game.sim.score()).max(0)
                    ),
                    14.0,
                    TEAL,
                );
                text(
                    p,
                    format!("Damage deductions         -$ {}", game.damage_assessment()),
                    14.0,
                    AMBER,
                );
                text(
                    p,
                    format!("Net paid at review         $ {}", game.grant()),
                    13.0,
                    MUTED,
                );
            });
            text(p, "RESPONSE RESOURCES", 13.0, MUTED);
            for cap in [Capability::Police, Capability::Medical, Capability::Fire] {
                p.spawn((
                    Node {
                        padding: UiRect::all(px(16)),
                        flex_direction: FlexDirection::Column,
                        row_gap: px(8),
                        border_radius: BorderRadius::all(px(10)),
                        ..default()
                    },
                    BackgroundColor(PANEL),
                ))
                .with_children(|p| {
                    let total = game
                        .sim
                        .units()
                        .values()
                        .filter(|u| u.capability == cap)
                        .count();
                    let free = game
                        .sim
                        .units()
                        .values()
                        .filter(|u| u.capability == cap && u.state == UnitState::Available)
                        .count();
                    text(
                        p,
                        format!("{}   {free}/{total} available", cap_name(cap)),
                        17.0,
                        cap_color(cap),
                    );
                    for u in game.sim.units().values().filter(|u| u.capability == cap) {
                        text(
                            p,
                            format!("Unit {} / {}", u.id, unit_label(&u.state, game.sim.now())),
                            12.0,
                            MUTED,
                        );
                    }
                });
            }
            button(p, "Leave desktop [Esc]", Action::Office, true);
            if running {
                button(p, "Finish shift", Action::EndShift, true);
            }
        });
        if let Some(id) = phone_call(game) {
            let Ok(view) = game.sim.call_view(id) else {
                return;
            };
            let own = game.conversation == Some(id) && game.call_station == game.seated;
            p.spawn((
                Node {
                    width: percent(62),
                    height: percent(100),
                    min_height: px(0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(px(22)),
                    row_gap: px(14),
                    border_radius: BorderRadius::all(px(20)),
                    border: UiRect::all(px(1)),
                    ..default()
                },
                BackgroundColor(INK),
                BorderColor::all(Color::srgb(0.22, 0.34, 0.39)),
            ))
            .with_children(|p| {
                p.spawn(Node {
                    column_gap: px(18),
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|p| {
                    p.spawn((
                        ImageNode::new(portraits.0[(view.caller_id as usize) % 24].clone()),
                        Node {
                            width: px(96),
                            height: px(96),
                            border_radius: BorderRadius::all(px(12)),
                            ..default()
                        },
                    ));
                    p.spawn(Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: px(8),
                        ..default()
                    })
                    .with_children(|p| {
                        text(
                            p,
                            if own {
                                "PHONE  /  CONNECTED"
                            } else {
                                "PHONE  /  INCOMING CALL"
                            },
                            12.0,
                            TEAL,
                        );
                        text(
                            p,
                            if view.caller_name.is_empty() {
                                "Unknown caller"
                            } else {
                                &view.caller_name
                            },
                            23.0,
                            TEXT,
                        );
                        text(
                            p,
                            format!(
                                "555-{:04}  /  {:02}:{:02}",
                                100 + view.caller_id % 100,
                                game.call_duration(id) / 60,
                                game.call_duration(id) % 60
                            ),
                            13.0,
                            MUTED,
                        );
                    });
                });
                if !own {
                    text(p, "Incoming voice call", 24.0, TEXT);
                    button(
                        p,
                        "Answer phone",
                        Action::Answer,
                        running && game.call_station == game.seated,
                    );
                    text(
                        p,
                        "Answering connects your microphone. You can mute at any time.",
                        13.0,
                        MUTED,
                    );
                } else {
                    p.spawn(Node {
                        column_gap: px(10),
                        width: percent(100),
                        ..default()
                    })
                    .with_children(|p| {
                        p.spawn(Node {
                            width: percent(65),
                            min_width: px(0),
                            ..default()
                        })
                        .with_children(|p| {
                            button(
                                p,
                                if voice.enabled {
                                    "Mic on / click to mute"
                                } else {
                                    "Mic muted / click to unmute"
                                },
                                Action::MicToggle,
                                true,
                            );
                        });
                        p.spawn(Node {
                            flex_grow: 1.0,
                            min_width: px(0),
                            ..default()
                        })
                        .with_children(|p| {
                            button(p, "Hang up", Action::HangUp, true);
                        });
                    });
                    p.spawn(Node {
                        flex_grow: 1.0,
                        min_height: px(80),
                        width: percent(100),
                        overflow: Overflow::scroll_y(),
                        flex_direction: FlexDirection::Column,
                        row_gap: px(12),
                        ..default()
                    })
                    .with_children(|p| {
                        conversation_ui(p, game, chat, voice);
                        text(p, crate::city::address(view.location), 13.0, MUTED);
                        text(p, "SEND HELP", 12.0, MUTED);
                        button(
                            p,
                            "Send all three services",
                            Action::FullResponse,
                            running
                                && game.selected_call_answered()
                                && [Capability::Police, Capability::Medical, Capability::Fire]
                                    .into_iter()
                                    .all(|cap| {
                                        game.sim.units().values().any(|u| {
                                            u.capability == cap && u.state == UnitState::Available
                                        })
                                    }),
                        );
                        for unit in game.sim.units().values() {
                            let verb = match unit.state {
                                UnitState::Available => "Send",
                                UnitState::Assigned { incident }
                                | UnitState::EnRoute { incident, .. }
                                    if incident == id =>
                                {
                                    "Cancel"
                                }
                                UnitState::Assigned { .. } | UnitState::EnRoute { .. } => "Reroute",
                                _ => "Busy",
                            };
                            button(
                                p,
                                format!("{verb} {} / Unit {}", cap_name(unit.capability), unit.id),
                                Action::Unit(unit.id),
                                running
                                    && matches!(
                                        unit.state,
                                        UnitState::Available
                                            | UnitState::Assigned { .. }
                                            | UnitState::EnRoute { .. }
                                    ),
                            );
                        }
                        button(
                            p,
                            "No response needed / close case",
                            Action::Decline,
                            running && view.state == IncidentState::Active,
                        );
                        if game.multiplayer {
                            for other in game
                                .connected
                                .iter()
                                .copied()
                                .filter(|v| *v != game.local_player)
                            {
                                let d = game.desk(other);
                                button(
                                    p,
                                    format!("Transfer to dispatcher {other}"),
                                    Action::Transfer(other),
                                    d.seated.is_some()
                                        && d.ringing.is_none()
                                        && d.conversation.is_none(),
                                );
                            }
                        }
                        audio_ui(p, voice, chat);
                    });
                }
            });
        }
    });
}
fn phone_call(game: &Session) -> Option<u32> {
    game.conversation.or(game.ringing).filter(|id| {
        game.sim
            .call_view(*id)
            .is_ok_and(|v| matches!(v.call, CallState::Ringing | CallState::Answered { .. }))
    })
}
fn conversation_ui(
    p: &mut ChildSpawnerCommands,
    game: &Session,
    chat: &crate::voice::ChatInput,
    voice: &crate::voice::Voice,
) {
    text(p, "CONVERSATION", 12.0, MUTED);
    if let Some(c) = game.dialogue.conversations.get(&game.selected) {
        for turn in c.turns.iter().rev().take(6).rev() {
            if !turn.question.is_empty() {
                text(
                    p,
                    format!("DISPATCHER {}: {}", turn.player, turn.question),
                    13.0,
                    MUTED,
                );
            }
            text(p, format!("CALLER: {}", turn.spoken_text), 16.0, TEXT);
        }

        if c.pending == Some(game.local_player) {
            button(
                p,
                "Caller is connecting… Cancel",
                Action::CancelTalk(game.selected),
                true,
            );
        }
    }
    let own = game.conversation == Some(game.selected) && game.call_station == game.seated;
    button(
        p,
        if chat.editing {
            format!("> {}_  [Enter sends]", chat.draft)
        } else if chat.draft.is_empty() {
            "Type a question...".into()
        } else {
            format!("Edit / send: {}", chat.draft)
        },
        Action::ChatFocus,
        own,
    );
    text(
        p,
        "Speak naturally, then pause to send. Your microphone pauses while the caller speaks. You can also type a message.",
        12.0,
        MUTED,
    );
    text(
        p,
        &voice.status,
        12.0,
        if voice.recording { AMBER } else { MUTED },
    );
}
fn audio_ui(
    p: &mut ChildSpawnerCommands,
    voice: &crate::voice::Voice,
    chat: &crate::voice::ChatInput,
) {
    button(
        p,
        if chat.settings {
            "Hide audio settings"
        } else {
            "Audio / microphone settings"
        },
        Action::AudioSettings,
        true,
    );
    if !chat.settings {
        return;
    }
    button(
        p,
        if voice.enabled {
            "Microphone: ON / voice activated"
        } else {
            "Microphone: MUTED / click to unmute"
        },
        Action::MicToggle,
        true,
    );
    button(
        p,
        if voice.review_speech {
            "Speech: review transcript before sending"
        } else {
            "Speech: send after a pause"
        },
        Action::SpeechReviewToggle,
        true,
    );
    button(
        p,
        format!("Input: {} / next", voice.inputs[voice.input]),
        Action::InputDevice,
        !voice.recording,
    );
    button(
        p,
        format!("Caller output: {} / next", voice.outputs[voice.output]),
        Action::OutputDevice,
        true,
    );
    text(
        p,
        format!("Caller volume: {:.0}%", voice.volume * 100.0),
        13.0,
        AMBER,
    );
    button(p, "Caller volume +", Action::Volume(true), true);
    button(p, "Caller volume −", Action::Volume(false), true);
    button(
        p,
        if voice.save_clips {
            "Saving local audio: ON / revoke consent"
        } else {
            "Save my microphone and caller audio locally (opt in)"
        },
        Action::SaveClips,
        true,
    );
    text(
        p,
        "Clips stay on this PC. Other players' microphones are never recorded or relayed. Delete removes saved WAVs and disables future recording.",
        12.0,
        MUTED,
    );
    button(
        p,
        "Delete all my saved audio clips",
        Action::DeleteClips,
        true,
    );
}
fn lobby(p: &mut ChildSpawnerCommands, game: &Session, net: &crate::network::Network) {
    p.spawn((column(65.0), BackgroundColor(PANEL)))
        .with_children(|p| {
            text(p, "DISPATCH ROOM / UP TO SIX PLAYERS", 26.0, TEXT);
            text(p, format!("{} / {}", net.address, net.status), 16.0, AMBER);
            for player in &net.players {
                text(
                    p,
                    format!(
                        "{} / dispatcher {} / {} / {}",
                        player.name,
                        player.id,
                        if player.id == 1 { "HOST" } else { "DISPATCHER" },
                        if !player.connected {
                            "disconnected"
                        } else if player.desk.seated.is_some() {
                            "at workstation"
                        } else {
                            "standing"
                        }
                    ),
                    18.0,
                    TEXT,
                );
            }
            text(
                p,
                format!(
                    "Room frame {} | RTT {} ms | rejected actions {}",
                    net.revision, net.rtt_ms, net.rejected
                ),
                14.0,
                MUTED,
            );
            if game.phase == Phase::Lobby {
                button(
                    p,
                    "Start shared shift",
                    Action::StartShift,
                    net.id == Some(1),
                );
            } else {
                button(
                    p,
                    "End shared shift",
                    Action::EndShift,
                    net.id == Some(1) && matches!(game.phase, Phase::Running | Phase::Closing),
                );
            }
            button(
                p,
                "Return to office / take a seat",
                Action::Office,
                net.id.is_some(),
            );
            if net.client() && net.id.is_none() {
                button(p, "Reconnect", Action::Reconnect, true);
            }
            button(p, "Leave room / quit", Action::Quit, true);
        });
    p.spawn((column(34.0),BackgroundColor(PANEL))).with_children(|p|{text(p,"SHARED AUTHORITY",20.0,TEXT);text(p,"The host owns the clock, incidents, responders and funding. Seat claims and critical actions are validated in order.",16.0,TEXT);text(p,"Use E at a free chair. Open your monitor to answer calls. Transfer a conversation to a connected, seated, idle teammate.",16.0,MUTED);text(p,"Escape opens your local menu; the online shift keeps running.",16.0,AMBER);});
}
fn review(p: &mut ChildSpawnerCommands, game: &Session, voice: &crate::voice::Voice) {
    let result = game.sim.review();
    p.spawn((column(66.0),BackgroundColor(PANEL))).with_children(|p| {
        text(p,format!("SHIFT REVIEW / GRADE {}",game.grade()),24.0,TEXT);
        text(p,format!("{} resolved | {} refused | {} expired",result.resolved,result.dismissed,result.expired),18.0,AMBER);
        for entry in &result.score_entries {text(p,format!("{:02}:{:02}  Call {}  {:+} pts\n{}",entry.at/60,entry.at%60,entry.incident,entry.points,entry.reason),16.0,TEXT);}
        text(p,format!("City damage: ${} | Funding assessment: {}",result.total_damage,game.damage_assessment()),19.0,AMBER);
        for entry in &result.escalations {text(p,format!("{:02}:{:02} / Call {} / structure explosion / {}",entry.at/60,entry.at%60,entry.incident,entry.secondary.map_or_else(||"no follow-up queued".into(),|id|format!("linked call {id}"))),15.0,AMBER);}
        for entry in &result.damage_entries {text(p,format!("Call {} / ${} damage / {:02}:{:02} to {:02}:{:02} / recorded consequences",entry.incident,entry.damage,entry.first_at/60,entry.first_at%60,entry.last_at/60,entry.last_at%60),14.0,TEXT);}
        text(p,format!("Base 100 + score {} - damage assessment {} = {} funding (minimum 0)",result.score,game.damage_assessment(),game.grant()),17.0,AMBER);
        text(p,"DECISION AND CITY TIMELINE (damage grouped above)",16.0,AMBER);
        for event in game.sim.events(){if let Some(line)=timeline_line(&event.kind){text(p,format!("{:02}:{:02}  {line}",event.at/60,event.at%60),13.0,TEXT);}}
        text(p,"Recorded decisions drive this review. Upgrades apply to the next shift in this run.",14.0,MUTED);
    });
    p.spawn((column(33.0), BackgroundColor(PANEL)))
        .with_children(|p| {
            text(p, format!("AVAILABLE FUNDING\n${}", game.funds), 28.0, TEXT);
            text(p, "One purchase per review", 15.0, MUTED);
            text(p, "LOCAL CONVERSATION CLIPS (your device)", 14.0, AMBER);
            for (n, clip) in voice.clips.iter().enumerate() {
                button(
                    p,
                    format!("Replay {}", clip.label),
                    Action::ReplayClip(n),
                    true,
                );
            }
            button(
                p,
                "Delete all my saved audio clips",
                Action::DeleteClips,
                true,
            );
            for (id, c) in &game.dialogue.conversations {
                if let Some(t) = c.turns.iter().rev().find(|t| !t.question.is_empty()) {
                    text(
                        p,
                        format!("Call {id}: YOU: {} / CALLER: {}", t.question, t.spoken_text),
                        13.0,
                        MUTED,
                    );
                }
            }
            button(
                p,
                if game.radio {
                    "Radios installed"
                } else {
                    "$50 / Improved radios\nRecovery: 15 seconds -> 10 seconds"
                },
                Action::BuyRadio,
                (!game.multiplayer || game.local_player == 1)
                    && !game.radio
                    && !game.bought_this_review
                    && game.funds >= 50,
            );
            button(
                p,
                if game.extra_police {
                    "Patrol installed"
                } else {
                    "$90 / Additional patrol\nA second police unit next shift"
                },
                Action::BuyPolice,
                (!game.multiplayer || game.local_player == 1)
                    && !game.extra_police
                    && !game.bought_this_review
                    && game.funds >= 90,
            );
            button(
                p,
                "Prepare next shift ->",
                Action::NextShift,
                !game.multiplayer || game.local_player == 1,
            );
            button(p, "Return to office", Action::Office, true);
        });
}
#[allow(clippy::too_many_arguments)]
pub fn draw(
    mut commands: Commands,
    roots: Query<Entity, With<UiRoot>>,
    mut game: ResMut<Session>,
    image: Res<MapImage>,
    routes: Res<crate::routes::RouteOverlay>,
    net: Res<crate::network::Network>,
    player: Single<&Transform, With<Player>>,
    chat: Res<crate::voice::ChatInput>,
    voice: Res<crate::voice::Voice>,
    portraits: Res<CallerPortraits>,
) {
    if !game.dirty {
        return;
    }
    game.dirty = false;
    for e in &roots {
        commands.entity(e).despawn();
    }
    let screen = game.screen;
    let root = Node {
        position_type: PositionType::Absolute,
        left: px(20),
        right: px(20),
        top: px(20),
        bottom: px(20),
        flex_direction: FlexDirection::Column,
        row_gap: px(12),
        ..default()
    };
    commands
        .spawn((
            UiRoot,
            root,
            BackgroundColor(if screen == Screen::Office {
                Color::NONE
            } else {
                INK
            }),
        ))
        .with_children(|p| {
            if screen == Screen::Office {
                p.spawn((
                    Hud,
                    Text::new(hud_text(&game, &player)),
                    TextFont {
                        font_size: FontSize::Px(14.0),
                        ..default()
                    },
                    TextColor(TEXT),
                    Node {
                        padding: UiRect::all(px(14)),
                        width: px(780),
                        position_type: PositionType::Absolute,
                        bottom: px(0),
                        left: px(0),
                        max_width: percent(65),
                        ..default()
                    },
                    BackgroundColor(INK.with_alpha(0.72)),
                ));
                p.spawn((
                    Reticle,
                    Node {
                        position_type: PositionType::Absolute,
                        left: percent(50),
                        top: percent(50),
                        width: px(10),
                        height: px(10),
                        margin: UiRect::all(px(-5)),
                        border: UiRect::all(px(2)),
                        border_radius: BorderRadius::MAX,
                        ..default()
                    },
                    BorderColor::all(TEXT.with_alpha(0.6)),
                ));
            } else {
                p.spawn(Node {
                    height: px(64),
                    padding: UiRect::all(px(12)),
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                })
                .with_children(|p| {
                    text(
                        p,
                        if screen == Screen::Terminal {
                            "wn / Dispatch desktop"
                        } else {
                            "WRONG NUMBER / DISPATCH AUTHORITY"
                        },
                        22.0,
                        TEXT,
                    );
                    text(
                        p,
                        format!(
                            "SHIFT {}  |  {}  |  {:02}:{:02}  |  SCORE {}",
                            game.shift,
                            game.status(),
                            game.remaining() / 60,
                            game.remaining() % 60,
                            game.sim.score()
                        ),
                        16.0,
                        AMBER,
                    );
                });
                p.spawn(Node {
                    flex_grow: 1.0,
                    min_height: px(0),
                    column_gap: px(12),
                    ..default()
                })
                .with_children(|p| match screen {
                    Screen::Lobby => lobby(p, &game, &net),
                    Screen::Terminal => {
                        terminal(p, &game, &image, &routes, &chat, &voice, &portraits)
                    }
                    Screen::CityMap => {
                        p.spawn((column(70.0), BackgroundColor(PANEL)))
                            .with_children(|p| {
                                map(p, &game, &image, &routes);
                                button(p, "Return to room", Action::Office, true);
                            });
                    }
                    Screen::Review => {
                        review(p, &game, &voice);
                    }
                    Screen::Pause => {
                        p.spawn((column(50.0), BackgroundColor(PANEL)))
                            .with_children(|p| {
                                text(
                                    p,
                                    if net.online() { "ROOM MENU" } else { "PAUSED" },
                                    32.0,
                                    TEXT,
                                );
                                text(
                                    p,
                                    if net.online() {
                                        "The online shift keeps running."
                                    } else {
                                        "The shift clock is stopped."
                                    },
                                    18.0,
                                    MUTED,
                                );
                                if net.online() {
                                    button(p, "Room lobby / players", Action::OpenLobby, true);
                                }
                                audio_ui(p, &voice, &chat);
                                button(p, "Resume", Action::Resume, true);
                                button(p, "Quit game", Action::Quit, true);
                            });
                    }
                    _ => {}
                });
                p.spawn((
                    Node {
                        min_height: px(52),
                        padding: UiRect::all(px(12)),
                        ..default()
                    },
                    BackgroundColor(PANEL),
                ))
                .with_children(|p| text(p, &game.notice, 16.0, AMBER));
            }
        });
}
#[derive(Component)]
pub struct Reticle;
pub fn update_hud(
    game: Res<Session>,
    player: Single<&Transform, With<Player>>,
    stats: Res<crate::performance::Frames>,
    mut reticle: Query<&mut BorderColor, With<Reticle>>,
    mut hud: Query<&mut Text, With<Hud>>,
) {
    for mut color in &mut reticle {
        let target = crate::world::aimed_action(&player, &game);
        let colors = &crate::city::config().reticle_colors;
        let rgb = match target {
            Some(Action::Door | Action::OpenReview)
                if !crate::world::review_unlocked(game.phase) =>
            {
                colors.locked
            }
            Some(Action::UsePhone) if game.ringing.is_none() => colors.conditional,
            Some(_) => colors.ready,
            None => colors.neutral,
        };
        let next = BorderColor::all(Color::srgb(rgb[0], rgb[1], rgb[2]));
        if *color != next {
            *color = next;
        }
    }
    for mut text in &mut hud {
        let mut value = hud_text(&game, &player);
        if stats.show {
            value.push_str(&format!("\n{:.0} FPS | {}", stats.fps, stats.gpu));
        }
        if text.0 != value {
            text.0 = value;
        }
    }
}
pub fn hover(mut buttons: Query<(&Hovered, &mut BackgroundColor), With<Button>>) {
    for (hovered, mut color) in &mut buttons {
        let next = BackgroundColor(if hovered.get() {
            TEAL
        } else {
            Color::srgb(0.12, 0.23, 0.26)
        });
        if *color != next {
            *color = next;
        }
    }
}

pub fn scale(window: Single<&Window>, mut scale: ResMut<UiScale>) {
    let value = (window.width() / 1440.0)
        .min(window.height() / 900.0)
        .clamp(0.65, 1.25);
    if (scale.0 - value).abs() > 0.001 {
        scale.0 = value;
    }
}

/// Scroll only the panel under the pointer; preserve its offset across clock redraws.
pub fn scroll_panels(
    game: Res<Session>,
    window: Single<&Window>,
    mut wheel: MessageReader<bevy::input::mouse::MouseWheel>,
    mut panels: Query<(
        Ref<Node>,
        &ComputedNode,
        &UiGlobalTransform,
        &mut ScrollPosition,
    )>,
    mut memory: Local<std::collections::BTreeMap<(u8, u32), f32>>,
) {
    use bevy::input::mouse::MouseScrollUnit;
    let delta: f32 = wheel
        .read()
        .map(|e| {
            -e.y * if e.unit == MouseScrollUnit::Line {
                40.0
            } else {
                1.0
            }
        })
        .sum();
    let cursor = window.cursor_position().map(|p| p * window.scale_factor());
    for (node, computed, transform, mut position) in &mut panels {
        if node.overflow.y != OverflowAxis::Scroll {
            continue;
        }
        let Val::Percent(width) = node.width else {
            continue;
        };
        let key = (game.screen as u8, width as u32);
        if node.is_added() {
            position.y = *memory.get(&key).unwrap_or(&0.0);
        }
        if delta != 0.0
            && cursor.is_some_and(|p| {
                transform.try_inverse().is_some_and(|inverse| {
                    let local = inverse.transform_point2(p);
                    local.abs().cmple(computed.size() / 2.0).all()
                })
            })
        {
            let max = ((computed.content_size().y - computed.size().y)
                * computed.inverse_scale_factor())
            .max(0.0);
            position.y = (position.y + delta).clamp(0.0, max);
            memory.insert(key, position.y);
        }
    }
}

fn timeline_line(event: &wrong_number_sim::EventKind) -> Option<String> {
    use wrong_number_sim::EventKind::*;
    Some(match event {
        CallTransferred { incident, from, to } => {
            format!("Player {from} transferred call {incident} to player {to}")
        }
        CallArrived { incident } => format!("Call {incident} arrived"),
        CallAccepted { incident, player } => format!("Player {player} answered call {incident}"),
        FactRevealed {
            incident,
            fact,
            player,
        } => format!("Player {player} recorded {fact} evidence for call {incident}"),
        CallEnded { incident, player } => format!("Player {player} ended conversation {incident}"),
        CallDeclined { incident, player } => {
            format!("Player {player} declined response {incident}")
        }
        DispatchOrdered {
            unit,
            incident,
            player,
        } => format!("Player {player} sent unit {unit} to incident {incident}"),
        FullResponseOrdered { incident, player } => {
            format!("Player {player} requested full response for incident {incident}")
        }
        DispatchCancelled { unit, player } => format!("Player {player} cancelled unit {unit}"),
        IncidentEnded {
            incident,
            score_delta,
            reason,
            ..
        } => format!("Incident {incident}: {reason} ({score_delta:+})"),
        IncidentEscalated {
            incident,
            secondary,
            ..
        } => format!(
            "Incident {incident} exploded; follow-up {}",
            secondary.map_or_else(|| "not queued".into(), |id| id.to_string())
        ),
        RoadClosed {
            incident,
            road,
            reopen_at,
        } => format!("Incident {incident} closed road {road}; reopening at {reopen_at}s"),
        RoadReopened { incident, road } => {
            format!("Road {road} reopened after incident {incident}")
        }
        UnitChanged {
            unit,
            state: UnitState::OnScene { incident },
        } => format!("Unit {unit} arrived at incident {incident}"),
        UnitChanged {
            unit,
            state: UnitState::Available,
        } => format!("Unit {unit} is available again"),
        _ => return None,
    })
}

#[cfg(test)]
mod phone_tests {
    use super::*;
    #[test]
    fn phone_only_exists_while_ringing_or_connected() {
        let mut game = Session::default();
        assert_eq!(phone_call(&game), None);
        game.act(Action::Sit(0));
        game.tick(10.0);
        game.act(Action::OpenDesk);
        assert_eq!(phone_call(&game), game.ringing);
        assert!(phone_call(&game).is_some());
        game.act(Action::Answer);
        assert_eq!(phone_call(&game), game.conversation);
        game.act(Action::HangUp);
        assert_eq!(phone_call(&game), None);
    }
}
