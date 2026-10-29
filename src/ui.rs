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
#[derive(Component)]
pub struct PendingUi;
#[derive(Resource)]
pub struct MapImage(pub Handle<Image>);
#[derive(Resource)]
pub struct CallerPortraits(pub Vec<Handle<Image>>);
#[derive(Resource)]
pub struct DesktopFont(pub Handle<Font>);
pub fn load_map(mut commands: Commands, assets: Res<AssetServer>) {
    commands.insert_resource(DesktopFont(assets.load("fonts/DesktopSans.ttf")));
    commands.insert_resource(MapImage(assets.load("city/map.png")));
    commands.insert_resource(CallerPortraits(
        (0..50)
            .map(|n| assets.load(format!("callers/{n:02}.png")))
            .collect(),
    ));
}
#[derive(Component)]
pub struct UiAction(pub Action);
#[derive(Component)]
pub struct Hud;
const INK: Color = Color::srgb(0.025, 0.025, 0.025);
const PANEL: Color = Color::srgb(0.07, 0.07, 0.07);
const TEXT: Color = Color::srgb(0.9, 0.94, 0.93);
const MUTED: Color = Color::srgb(0.63, 0.63, 0.63);
const TEAL: Color = Color::srgb(0.12, 0.84, 0.38);
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
            Color::srgb(0.13, 0.13, 0.13)
        } else {
            Color::srgb(0.075, 0.075, 0.075)
        }),
    ));
    if enabled {
        e.insert((Button, Hovered::default(), UiAction(action.clone())))
            .observe(
                move |_: On<Activate>, mut actions: ResMut<Actions>, game: Res<Session>| {
                    if action == Action::Answer
                        && let Some(id) = game.ringing
                        && game.selected != id
                    {
                        actions.0.push(Action::Select(id));
                    }
                    actions.0.push(action.clone());
                    if matches!(action, Action::Sit(_)) && game.screen == Screen::Terminal {
                        actions.0.push(Action::OpenDesk);
                    }
                },
            );
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
        Capability::Medical => Color::srgb(0.15, 0.85, 0.47),
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
                MapUnit(unit.id),
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
}
#[derive(Component)]
pub struct MapUnit(u32);
pub fn map_units(game: Res<Session>, mut units: Query<(&MapUnit, &mut Node)>) {
    for (marker, mut node) in &mut units {
        if let Some(unit) = game.sim.units().get(&marker.0) {
            let (x, y) = crate::city::map_percent(unit.position);
            node.left = percent(x);
            node.top = percent(y);
        }
    }
}
#[allow(clippy::too_many_arguments)]
fn terminal(
    p: &mut ChildSpawnerCommands,
    game: &Session,
    image: &MapImage,
    routes: &crate::routes::RouteOverlay,
    chat: &crate::voice::ChatInput,
    voice: &crate::voice::Voice,
    portraits: &CallerPortraits,
    prefs: &crate::frontend::Preferences,
    position: Vec3,
) {
    let running = matches!(game.phase, Phase::Running | Phase::Closing);
    p.spawn((
        Node {
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(px(10)),
            row_gap: px(10),
            ..default()
        },
        BackgroundColor(INK),
    ))
    .with_children(|p| {
        p.spawn(Node {
            height: px(42),
            flex_shrink: 0.0,
            align_items: AlignItems::Center,
            column_gap: px(14),
            ..default()
        })
        .with_children(|p| {
            text(p, "WN  /  DISPATCH", 20.0, TEXT);
            live_text(p, DesktopLive::Clock, 13.0, MUTED);
            for (label, action) in [
                ("Sound", Action::AudioSettings),
                ("Leave computer", Action::Office),
            ] {
                p.spawn(Node {
                    width: px(140),
                    margin: UiRect::left(px(8)),
                    ..default()
                })
                .with_children(|p| button(p, label, action, true));
            }
            if game.seated.is_none() {
                text(p, "Standing view · sit to receive calls", 12.0, MUTED);
            }
        });
        p.spawn(Node {
            flex_grow: 1.0,
            min_height: px(0),
            column_gap: px(12),
            ..default()
        })
        .with_children(|p| {
            // Resource rail: the live funding ledger and actual available responders.
            p.spawn(Node {
                width: percent(22),
                min_width: px(0),
                overflow: Overflow::scroll_y(),
                flex_direction: FlexDirection::Column,
                row_gap: px(10),
                ..default()
            })
            .with_children(|p| {
                card(p, |p| {
                    text(p, "Resources", 19.0, TEXT);
                    text(p, "TOTAL   /   AVAILABLE", 10.0, MUTED);
                    for cap in [Capability::Fire, Capability::Police, Capability::Medical] {
                        let units: Vec<_> = game
                            .sim
                            .units()
                            .values()
                            .filter(|u| u.capability == cap)
                            .collect();
                        p.spawn((
                            Node {
                                padding: UiRect::all(px(12)),
                                flex_direction: FlexDirection::Column,
                                row_gap: px(5),
                                border_radius: BorderRadius::all(px(6)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.09, 0.09, 0.09)),
                        ))
                        .with_children(|p| {
                            text(p, cap_name(cap), 13.0, cap_color(cap));
                            text(
                                p,
                                format!(
                                    "{}    /    {}",
                                    units.len(),
                                    units
                                        .iter()
                                        .filter(|u| u.state == UnitState::Available)
                                        .count()
                                ),
                                24.0,
                                TEXT,
                            );
                            for unit in units {
                                live_text(p, DesktopLive::Unit(unit.id), 11.0, MUTED);
                            }
                        });
                    }
                });
                card(p, |p| {
                    text(p, "Unit status", 17.0, TEXT);
                    for (label, count, color) in [
                        (
                            "Available",
                            game.sim
                                .units()
                                .values()
                                .filter(|u| u.state == UnitState::Available)
                                .count(),
                            TEAL,
                        ),
                        (
                            "Dispatched",
                            game.sim
                                .units()
                                .values()
                                .filter(|u| {
                                    matches!(
                                        u.state,
                                        UnitState::Assigned { .. } | UnitState::EnRoute { .. }
                                    )
                                })
                                .count(),
                            Color::srgb(0.95, 0.25, 0.25),
                        ),
                        (
                            "On scene",
                            game.sim
                                .units()
                                .values()
                                .filter(|u| {
                                    matches!(
                                        u.state,
                                        UnitState::OnScene { .. } | UnitState::Resolving { .. }
                                    )
                                })
                                .count(),
                            AMBER,
                        ),
                        (
                            "Returning / recovery",
                            game.sim
                                .units()
                                .values()
                                .filter(|u| {
                                    matches!(
                                        u.state,
                                        UnitState::Returning { .. } | UnitState::Recovering { .. }
                                    )
                                })
                                .count(),
                            MUTED,
                        ),
                    ] {
                        text(p, format!("{label}     {count}"), 13.0, color);
                    }
                });
                card(p, |p| {
                    text(p, "Funds", 17.0, TEXT);
                    live_text(p, DesktopLive::Cash, 26.0, TEXT);
                    live_text(p, DesktopLive::Income, 12.0, TEAL);
                    live_text(p, DesktopLive::Loss, 12.0, AMBER);
                    live_text(p, DesktopLive::Grant, 11.0, MUTED);
                });
                card(p, |p| {
                    text(p, "Active incidents", 17.0, TEXT);
                    for id in game.visible_ids() {
                        let v = game.sim.call_view(*id).unwrap();
                        if v.state == IncidentState::Active {
                            button(
                                p,
                                format!(
                                    "{}  #{}\n{}",
                                    if game.selected == *id {
                                        "Selected"
                                    } else {
                                        "Incident"
                                    },
                                    id,
                                    crate::city::address(v.location)
                                ),
                                Action::Select(*id),
                                true,
                            );
                        }
                    }
                    if running {
                        button(
                            p,
                            "Finish shift",
                            Action::EndShift,
                            game.can_dispatch() || game.multiplayer && game.local_player == 1,
                        );
                    }
                });
            });
            // The real supplied map occupies the center, with response controls below it.
            p.spawn(Node {
                width: percent(48),
                min_width: px(0),
                overflow: Overflow::scroll_y(),
                flex_direction: FlexDirection::Column,
                row_gap: px(10),
                ..default()
            })
            .with_children(|p| {
                card(p, |p| {
                    p.spawn(Node {
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        ..default()
                    })
                    .with_children(|p| {
                        text(p, "CITY MAP", 15.0, TEAL);
                        p.spawn(Node {
                            width: px(130),
                            ..default()
                        })
                        .with_children(|p| button(p, "Expand map", Action::OpenMap, true));
                    });
                    map(p, game, image, routes);
                });
                card(p, |p| {
                    text(p, "Incident details & dispatch", 18.0, TEXT);
                    live_text(p, DesktopLive::Voice, 11.0, MUTED);
                    if let Ok(view) = game.sim.call_view(game.selected) {
                        text(
                            p,
                            format!(
                                "#{}  ·  {}",
                                game.selected,
                                crate::city::address(view.location)
                            ),
                            17.0,
                            TEXT,
                        );
                        text(p, &view.report, 14.0, MUTED);
                        for fact in &view.facts {
                            text(p, &fact.answer, 13.0, MUTED);
                        }
                        if let Some(c) = game.dialogue.conversations.get(&game.selected) {
                            for evidence in &c.evidence {
                                text(p, format!("{} #{} · dispatcher {} · turn {}: {}", if evidence.verified { "CONFIRMED" } else { &evidence.kind }, evidence.id, evidence.player, evidence.turn, evidence.detail), 12.0, if evidence.verified {TEAL} else {MUTED});
                            }
                        }
                        let actionable =
                            running && game.selected_call_answered() && game.seated.is_some();
                        p.spawn(Node {
                            flex_wrap: FlexWrap::Wrap,
                            column_gap: px(6),
                            ..default()
                        })
                        .with_children(|p| {
                            for unit in game.sim.units().values() {
                                let verb = match unit.state {
                                    UnitState::Available => "Send",
                                    UnitState::Assigned { incident }
                                    | UnitState::EnRoute { incident, .. }
                                        if incident == game.selected =>
                                    {
                                        "Cancel"
                                    }
                                    UnitState::Assigned { .. } | UnitState::EnRoute { .. } => {
                                        "Reroute"
                                    }
                                    _ => "Busy",
                                };
                                p.spawn(Node {
                                    width: percent(31),
                                    min_width: px(0),
                                    ..default()
                                })
                                .with_children(|p| {
                                    button(
                                        p,
                                        format!(
                                            "{verb} {}\nUnit {}",
                                            cap_name(unit.capability),
                                            unit.id
                                        ),
                                        Action::Unit(unit.id),
                                        actionable
                                            && matches!(
                                                unit.state,
                                                UnitState::Available
                                                    | UnitState::Assigned { .. }
                                                    | UnitState::EnRoute { .. }
                                            ),
                                    );
                                });
                            }
                        });
                        button(
                            p,
                            "Send all three services",
                            Action::FullResponse,
                            actionable
                                && [Capability::Police, Capability::Medical, Capability::Fire]
                                    .iter()
                                    .all(|cap| {
                                        game.sim.units().values().any(|u| {
                                            u.capability == *cap && u.state == UnitState::Available
                                        })
                                    }),
                        );
                        button(
                            p,
                            "No response needed / close case",
                            Action::Decline,
                            actionable,
                        );
                        if game.multiplayer && game.conversation == Some(game.selected) {
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
                    } else {
                        text(
                            p,
                            "Select an incident on the map or in the list.",
                            14.0,
                            MUTED,
                        );
                    }
                });
            });
            p.spawn(Node {
                width: percent(28),
                min_width: px(0),
                min_height: px(0),
                flex_direction: FlexDirection::Column,
                row_gap: px(10),
                ..default()
            })
            .with_children(|p| {
                if let Some(id) = phone_call(game) {
                    dispatch_phone(p, game, chat, voice, portraits, id, prefs.subtitles);
                } else {
                    card(p, |p| {
                        text(p, "Dispatch phone", 18.0, TEXT);
                        text(p, if game.seated.is_none() {"NOT RECEIVING CALLS — standing"} else {"Waiting for incoming call"}, 14.0, AMBER);
                        if game.seated.is_none()
                            && let Some((n,_))=crate::world::SEATS.iter().enumerate().filter(|(_,x)| position.distance(Vec3::new(**x,1.65,1.7)) <= 2.5).min_by(|(_,a),(_,b)| (position.x-**a).abs().total_cmp(&(position.x-**b).abs())) {
                                button(p,"Take this seat and receive calls",Action::UseDesk(n),game.occupied(n).is_none());
                        }
                        text(
                            p,
                            if game.phase == Phase::Lobby { "Shift has not started. Hold Interact at the break-room door." } else if game.seated.is_none() { "You are standing. Exit the computer and press Interact at a chair to receive calls." } else { "Line ready. Incoming calls appear here automatically." },
                            13.0,
                            MUTED,
                        );
                    });
                    card(p, |p| {
                        text(p, "Recent calls", 17.0, TEXT);
                        let mut recent: Vec<_> =
                            game.dialogue.conversations.keys().copied().collect();
                        recent.reverse();
                        for id in recent.into_iter().take(5) {
                            if let Ok(v) = game.sim.call_view(id) {
                                button(
                                    p,
                                    format!(
                                        "{}\n{}",
                                        v.caller_name,
                                        crate::city::address(v.location)
                                    ),
                                    Action::Select(id),
                                    true,
                                );
                            }
                        }
                    });
                }
            });
        });
        text(p, &game.notice, 12.0, MUTED);
        if chat.settings {
            p.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(20),
                    top: px(64),
                    bottom: px(35),
                    width: px(430),
                    padding: UiRect::all(px(18)),
                    overflow: Overflow::scroll_y(),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(8),
                    border_radius: BorderRadius::all(px(12)),
                    border: UiRect::all(px(1)),
                    ..default()
                },
                BackgroundColor(PANEL),
                BorderColor::all(TEAL),
            ))
            .with_children(|p| {
                text(p, "Sound settings", 22.0, TEXT);
                audio_meters(p);
                audio_ui(p, voice, chat);
            });
        }
    });
}
fn card(p: &mut ChildSpawnerCommands, contents: impl FnOnce(&mut ChildSpawnerCommands)) {
    p.spawn((
        Node {
            padding: UiRect::all(px(12)),
            flex_direction: FlexDirection::Column,
            row_gap: px(9),
            border_radius: BorderRadius::all(px(10)),
            border: UiRect::all(px(1)),
            flex_shrink: 0.0,
            ..default()
        },
        BackgroundColor(PANEL),
        BorderColor::all(Color::srgb(0.13, 0.13, 0.13)),
    ))
    .with_children(contents);
}
#[derive(Component)]
pub struct TranscriptScroll(bool);
fn dispatch_phone(
    p: &mut ChildSpawnerCommands,
    game: &Session,
    chat: &crate::voice::ChatInput,
    voice: &crate::voice::Voice,
    portraits: &CallerPortraits,
    id: u32,
    subtitles: bool,
) {
    let Ok(view) = game.sim.call_view(id) else {
        return;
    };
    let own = game.conversation == Some(id) && game.call_station == game.seated;
    p.spawn((
        Node {
            flex_grow: 1.0,
            min_height: px(0),
            padding: UiRect::all(px(12)),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(14)),
            flex_direction: FlexDirection::Column,
            row_gap: px(9),
            ..default()
        },
        BackgroundColor(PANEL),
        BorderColor::all(Color::srgb(0.14, 0.14, 0.14)),
    ))
    .with_children(|p| {
        text(
            p,
            if own {
                "Dispatch phone  ·  Connected"
            } else {
                "Dispatch phone  ·  Incoming"
            },
            14.0,
            if own { TEAL } else { TEXT },
        );
        p.spawn((
            Node {
                flex_grow: 1.0,
                min_height: px(0),
                width: percent(100),
                padding: UiRect::all(px(13)),
                border: UiRect::all(px(2)),
                border_radius: BorderRadius::all(px(28)),
                flex_direction: FlexDirection::Column,
                row_gap: px(9),
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(INK),
            BorderColor::all(Color::srgb(0.27, 0.27, 0.27)),
        ))
        .with_children(|p| {
            p.spawn(Node {
                width: percent(100),
                height: px(8),
                justify_content: JustifyContent::Center,
                flex_shrink: 0.0,
                ..default()
            })
            .with_children(|p| {
                p.spawn((
                    Node {
                        width: px(62),
                        height: px(4),
                        border_radius: BorderRadius::MAX,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.16, 0.16, 0.16)),
                ));
            });
            p.spawn(Node {
                column_gap: px(10),
                align_items: AlignItems::Center,
                flex_shrink: 0.0,
                ..default()
            })
            .with_children(|p| {
                p.spawn((
                    ImageNode::new(
                        portraits.0[(view.caller_id.saturating_sub(1000) as usize) % 50].clone(),
                    ),
                    Node {
                        width: px(58),
                        height: px(58),
                        flex_shrink: 0.0,
                        border_radius: BorderRadius::all(px(14)),
                        ..default()
                    },
                ));
                p.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    min_width: px(0),
                    row_gap: px(5),
                    ..default()
                })
                .with_children(|p| {
                    text(
                        p,
                        if view.caller_name.is_empty() {
                            "Unknown caller"
                        } else {
                            &view.caller_name
                        },
                        19.0,
                        TEXT,
                    );
                    live_text(p, DesktopLive::Call(id, view.caller_id), 11.0, MUTED);
                });
            });
            if own {
                p.spawn(Node {
                    column_gap: px(6),
                    ..default()
                })
                .with_children(|p| {
                    p.spawn(Node {
                        width: percent(55),
                        ..default()
                    })
                    .with_children(|p| {
                        button(
                            p,
                            if voice.enabled {
                                "Mute mic"
                            } else {
                                "Unmute mic"
                            },
                            Action::MicToggle,
                            true,
                        )
                    });
                    p.spawn(Node {
                        flex_grow: 1.0,
                        ..default()
                    })
                    .with_children(|p| button(p, "Replay", Action::ReplayCaller, !voice.speaking));
                });
            }
            p.spawn((
                TranscriptScroll(false),
                Node {
                    width: percent(100),
                    flex_grow: 1.0,
                    min_height: px(60),
                    overflow: Overflow::scroll_y(),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(12),
                    padding: UiRect::axes(px(2), px(10)),
                    ..default()
                },
            ))
            .with_children(|p| {
                if let Some(c) = game.dialogue.conversations.get(&id) {
                    for turn in &c.turns {
                        if !turn.question.is_empty() {
                            text(p, format!("YOU\n{}", turn.question), 13.0, MUTED);
                        }
                        if subtitles {
                            text(
                                p,
                                format!(
                                    "{}\n{}",
                                    if view.caller_name.is_empty() {
                                        "CALLER"
                                    } else {
                                        &view.caller_name
                                    },
                                    turn.spoken_text
                                ),
                                15.0,
                                TEXT,
                            );
                        }
                    }
                    if c.pending == Some(game.local_player) {
                        button(
                            p,
                            "Caller is thinking… Cancel",
                            Action::CancelTalk(id),
                            true,
                        );
                    }
                } else {
                    text(p, "Incoming voice call", 18.0, TEXT);
                }
            });
            if own {
                button(
                    p,
                    if chat.editing {
                        format!("> {}_", chat.draft)
                    } else {
                        "Type a message…".into()
                    },
                    Action::ChatFocus,
                    true,
                );
                compact_meters(p);
                // Hang-up stays pinned at the very bottom, outside the scrolling transcript.
                p.spawn((
                    Node {
                        border_radius: BorderRadius::all(px(18)),
                        padding: UiRect::all(px(2)),
                        flex_shrink: 0.0,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.65, 0.08, 0.08)),
                ))
                .with_children(|p| button(p, "Hang up", Action::HangUp, true));
            } else {
                p.spawn(Node {
                    flex_shrink: 0.0,
                    ..default()
                })
                .with_children(|p| {
                    button(
                        p,
                        "Answer phone",
                        Action::Answer,
                        game.call_station == game.seated,
                    )
                });
            }
        });
    });
}
fn compact_meters(p: &mut ChildSpawnerCommands) {
    for caller in [false, true] {
        p.spawn(Node {
            height: px(13),
            min_height: px(13),
            width: percent(100),
            flex_shrink: 0.0,
            align_items: AlignItems::Center,
            column_gap: px(3),
            ..default()
        })
        .with_children(|p| {
            text(p, if caller { "CALLER" } else { "YOU" }, 9.0, MUTED);
            for index in 0..crate::voice::WAVE_BINS {
                p.spawn((
                    WaveBar {
                        caller,
                        index,
                        small: true,
                    },
                    Node {
                        flex_grow: 1.0,
                        flex_basis: px(0),
                        height: px(1),
                        ..default()
                    },
                    BackgroundColor(if caller {
                        Color::srgb(0.75, 0.75, 0.75)
                    } else {
                        TEAL
                    }),
                ));
            }
        });
    }
}
pub fn transcript_scroll(
    mut panels: Query<(&ComputedNode, &mut ScrollPosition, &mut TranscriptScroll)>,
) {
    for (node, mut scroll, mut state) in &mut panels {
        if !state.0 && node.size().y > 0.0 {
            scroll.y =
                ((node.content_size().y - node.size().y) * node.inverse_scale_factor()).max(0.0);
            state.0 = true;
        }
    }
}
fn phone_call(game: &Session) -> Option<u32> {
    game.conversation.or(game.ringing).filter(|id| {
        game.sim
            .call_view(*id)
            .is_ok_and(|v| matches!(v.call, CallState::Ringing | CallState::Answered { .. }))
    })
}
pub(crate) fn audio_ui(
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

    text(
        p,
        format!("Noise floor: {} dBFS", voice.gate_db),
        15.0,
        TEXT,
    );
    text(
        p,
        "Audio below this level counts as silence. Raise it if fans or room noise keep the microphone open.",
        12.0,
        MUTED,
    );
    button(
        p,
        "Raise noise floor (+3 dB)",
        Action::NoiseGate(true),
        voice.gate_db < 0,
    );
    button(
        p,
        "Lower noise floor (-3 dB)",
        Action::NoiseGate(false),
        voice.gate_db > -90,
    );
    text(
        p,
        format!("Maximum listening time: {} seconds", voice.listen_seconds),
        15.0,
        TEXT,
    );
    button(
        p,
        "Shorter listening timer",
        Action::ListenLimit(false),
        voice.listen_seconds > 3,
    );
    button(
        p,
        "Longer listening timer",
        Action::ListenLimit(true),
        voice.listen_seconds < 12,
    );
    button(p, "Test speakers", Action::TestSpeaker, !voice.speaking);
    button(
        p,
        "Replay last caller reply",
        Action::ReplayCaller,
        !voice.speaking,
    );
    button(
        p,
        if voice.enabled {
            if voice.push_to_talk {
                "Microphone: ON / push to talk"
            } else {
                "Microphone: ON / voice activated"
            }
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
            if game.phase == Phase::Lobby && game.break_lobby {
                text(p, format!("{}/{} ready. Hold E at the break-room door. The host or at least half the connected players starts the shift.", game.ready.len(), game.connected.len()), 18.0, TEXT);
            } else if game.phase == Phase::Lobby {
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
            button(p, "Start new run / reset funding & upgrades", Action::NewRun, !game.multiplayer || game.local_player == 1);
            text(p, "Progress saves at shift boundaries. New runs keep career commendations, never team upgrades.", 14.0, MUTED);
            button(p, "Return to office", Action::Office, true);
        });
}

fn desktop_key(
    game: &Session,
    chat: &crate::voice::ChatInput,
    voice: &crate::voice::Voice,
) -> String {
    serde_json::json!({
        "notice":game.notice,"phase":game.phase,"selected":game.selected,"ringing":game.ringing,"conversation":game.conversation,
        "seated":game.seated,"shift":game.shift,
        "units":game.sim.units().values().map(|u|(u.id,&u.state)).collect::<Vec<_>>(),
        "call":game.sim.call_view(game.selected).ok(),
        "incidents":game.visible_ids().map(|id|game.sim.call_view(*id).ok()).collect::<Vec<_>>(),
        "phone_dialogue":game.conversation.and_then(|id|game.dialogue.conversations.get(&id)),
        "dialogue":game.dialogue.conversations.get(&game.selected),
        "chat":(&chat.draft,chat.editing,chat.settings,chat.hide_resources),
        "voice":(voice.enabled,voice.push_to_talk,voice.review_speech,voice.gate_db,voice.listen_seconds,voice.volume,voice.input,voice.output,voice.save_clips),
        "devices":(&voice.inputs,&voice.outputs),
        "speaking":voice.speaking
    }).to_string()
}
/// New trees get one layout frame while the previous tree remains visible.
pub fn reveal_ui(mut commands: Commands, roots: Query<(Entity, Has<PendingUi>), With<UiRoot>>) {
    if roots.iter().any(|(_, pending)| pending) {
        for (e, pending) in &roots {
            if pending {
                commands
                    .entity(e)
                    .remove::<PendingUi>()
                    .insert(Visibility::Inherited);
            } else {
                commands.entity(e).despawn();
            }
        }
    }
}
#[allow(clippy::too_many_arguments)]
pub fn draw(
    mut commands: Commands,
    mut game: ResMut<Session>,
    image: Res<MapImage>,
    routes: Res<crate::routes::RouteOverlay>,
    net: Res<crate::network::Network>,
    player: Single<&Transform, With<Player>>,
    chat: Res<crate::voice::ChatInput>,
    voice: Res<crate::voice::Voice>,
    portraits: Res<CallerPortraits>,
    mut rendered: Local<Option<(Screen, String)>>,
    menu: Res<crate::frontend::Frontend>,
    prefs: Res<crate::frontend::Preferences>,
) {
    if menu.visible {
        *rendered = None;
        return;
    }
    if !game.dirty {
        return;
    }
    game.dirty = false;
    let screen = game.screen;
    if screen == Screen::Terminal {
        let key = desktop_key(&game, &chat, &voice);
        if rendered
            .as_ref()
            .is_some_and(|(screen, old)| *screen == Screen::Terminal && *old == key)
        {
            return;
        }
        *rendered = Some((screen, key));
    } else {
        *rendered = Some((screen, String::new()));
    }

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
            PendingUi,
            Visibility::Hidden,
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
            } else if screen == Screen::Terminal {
                terminal(
                    p,
                    &game,
                    &image,
                    &routes,
                    &chat,
                    &voice,
                    &portraits,
                    &prefs,
                    player.translation,
                );
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
                    Screen::Lobby => {
                        lobby(p, &game, &net);
                        p.spawn(column(30.0)).with_children(|p| {
                            if !menu.hosted_code.is_empty() && net.id == Some(1) {
                                text(p, format!("JOIN CODE\n{}", menu.hosted_code), 18.0, AMBER);
                                crate::frontend::button(
                                    p,
                                    "Copy join code",
                                    crate::frontend::MenuAction::CopyCode,
                                );
                                crate::frontend::button(
                                    p,
                                    "Copy host address",
                                    crate::frontend::MenuAction::CopyAddress,
                                );
                            }
                            crate::frontend::button(
                                p,
                                "Settings",
                                crate::frontend::MenuAction::Settings,
                            );
                            crate::frontend::button(
                                p,
                                "Leave room / main menu",
                                crate::frontend::MenuAction::Leave,
                            );
                        });
                    }
                    Screen::Terminal => terminal(
                        p,
                        &game,
                        &image,
                        &routes,
                        &chat,
                        &voice,
                        &portraits,
                        &prefs,
                        player.translation,
                    ),
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
                                crate::frontend::button(
                                    p,
                                    "Settings / controls",
                                    crate::frontend::MenuAction::Settings,
                                );
                                crate::frontend::button(
                                    p,
                                    "Leave shift / main menu",
                                    crate::frontend::MenuAction::Leave,
                                );
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
    menu: Res<crate::frontend::Frontend>,
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
        if game.phase == Phase::Lobby && !menu.hosted_code.is_empty() {
            value.push_str(&format!(
                "\nJoin code: {} | Esc > Room lobby to copy",
                menu.hosted_code
            ));
        }
        if stats.show {
            value.push_str(&format!("\n{:.0} FPS | {}", stats.fps, stats.gpu));
        }
        if text.0 != value {
            text.0 = value;
        }
    }
}
pub fn hover(
    mut buttons: Query<(&Hovered, &mut BackgroundColor, Option<&UiAction>), With<Button>>,
) {
    for (hovered, mut color, action) in &mut buttons {
        let next = BackgroundColor(if action.is_some_and(|a| a.0 == Action::HangUp) {
            if hovered.get() {
                Color::srgb(0.9, 0.16, 0.2)
            } else {
                Color::srgb(0.68, 0.08, 0.12)
            }
        } else if hovered.get() {
            TEAL
        } else {
            Color::srgb(0.13, 0.13, 0.13)
        });
        if *color != next {
            *color = next;
        }
    }
}

pub fn scale(
    window: Single<&Window>,
    prefs: Res<crate::frontend::Preferences>,
    mut scale: ResMut<UiScale>,
) {
    let value = (window.width() / 1440.0)
        .min(window.height() / 900.0)
        .clamp(0.65, 1.25)
        * prefs.text_scale;
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
        let width_key = match node.width {
            Val::Percent(width) => width as u32,
            Val::Px(width) => 10_000 + width as u32,
            _ => continue,
        };
        let key = (game.screen as u8, width_key);
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

#[derive(Component)]
pub struct WaveBar {
    caller: bool,
    index: usize,
    small: bool,
}
#[derive(Component)]
pub struct AudioReadout(bool);
fn audio_meters(p: &mut ChildSpawnerCommands) {
    for caller in [false, true] {
        p.spawn(Node {
            width: percent(100),
            flex_direction: FlexDirection::Column,
            row_gap: px(3),
            ..default()
        })
        .with_children(|p| {
            p.spawn((
                AudioReadout(caller),
                Text::new(if caller {
                    "Caller output"
                } else {
                    "Your microphone"
                }),
                TextFont {
                    font_size: FontSize::Px(12.0),
                    ..default()
                },
                TextColor(MUTED),
            ));
            p.spawn((
                Node {
                    width: percent(100),
                    height: px(34),
                    align_items: AlignItems::Center,
                    column_gap: px(3),
                    padding: UiRect::axes(px(6), px(2)),
                    ..default()
                },
                BackgroundColor(PANEL),
            ))
            .with_children(|p| {
                for index in 0..crate::voice::WAVE_BINS {
                    p.spawn((
                        WaveBar {
                            caller,
                            index,
                            small: false,
                        },
                        Node {
                            flex_grow: 1.0,
                            flex_basis: px(0),
                            height: px(1),
                            border_radius: BorderRadius::all(px(2)),
                            ..default()
                        },
                        BackgroundColor(if caller { AMBER } else { TEAL }),
                    ));
                }
            });
        });
    }
}
pub fn audio_visuals(
    time: Res<Time<Real>>,
    voice: Res<crate::voice::Voice>,
    mut last: Local<f64>,
    mut bars: Query<(&WaveBar, &mut Node)>,
    mut labels: Query<(&AudioReadout, &mut Text)>,
) {
    if time.elapsed_secs_f64() - *last < 0.05 {
        return;
    }
    *last = time.elapsed_secs_f64();
    let mic = voice.input_trace.lock().unwrap().clone();
    let caller = voice.output_trace.lock().unwrap().clone();
    for (bar, mut node) in &mut bars {
        let trace = if bar.caller { &caller } else { &mic };
        node.height = px(
            (trace.levels[bar.index].sqrt() * if bar.small { 30.0 } else { 90.0 })
                .clamp(1.0, if bar.small { 10.0 } else { 30.0 }),
        );
    }
    for (label, mut text) in &mut labels {
        if label.0 {
            text.0 = format!(
                "CALLER  {:.1}s / {:.1}s  |  {}",
                caller.elapsed, caller.duration, caller.state
            );
        } else {
            let state = if !voice.enabled {
                "Muted"
            } else if voice.recording {
                mic.state.as_str()
            } else {
                "Not listening"
            };
            let db = 20.0 * mic.rms.max(0.00001).log10();
            text.0 = format!(
                "YOU  {:.1}s / {}s  |  {:.0} dB  |  floor {} dB  |  {}",
                if voice.recording { mic.elapsed } else { 0.0 },
                voice.listen_seconds,
                db,
                voice.gate_db,
                state
            );
        }
    }
}
pub fn desktop_fonts(
    game: Res<Session>,
    menu: Res<crate::frontend::Frontend>,
    font: Res<DesktopFont>,
    mut fonts: Query<&mut TextFont, Added<TextFont>>,
) {
    if game.screen == Screen::Terminal || menu.visible {
        for mut f in &mut fonts {
            f.font = bevy::text::FontSource::Handle(font.0.clone());
        }
    }
}

#[derive(Component)]
pub enum DesktopLive {
    Clock,
    Cash,
    Income,
    Loss,
    Grant,
    Unit(u32),
    Call(u32, u32),
    Voice,
}
fn live_text(p: &mut ChildSpawnerCommands, kind: DesktopLive, size: f32, color: Color) {
    p.spawn((
        kind,
        Text::new(" "),
        TextFont {
            font_size: FontSize::Px(size),
            ..default()
        },
        TextColor(color),
        Node {
            flex_shrink: 0.0,
            ..default()
        },
    ));
}
pub fn desktop_values(
    game: Res<Session>,
    voice: Res<crate::voice::Voice>,
    mut texts: Query<(&DesktopLive, &mut Text)>,
) {
    for (kind, mut text) in &mut texts {
        let value = match kind {
            DesktopLive::Clock => format!(
                "Shift {}    {:02}:{:02}",
                game.shift,
                game.remaining() / 60,
                game.remaining() % 60
            ),
            DesktopLive::Cash => format!("$ {}", game.funds),
            DesktopLive::Income => format!(
                "Shift income (projected)   +$ {}",
                (100 + game.sim.score()).max(0)
            ),
            DesktopLive::Loss => {
                format!("Damage deductions         -$ {}", game.damage_assessment())
            }
            DesktopLive::Grant => format!("Shift grant         $ {}", game.grant()),
            DesktopLive::Unit(id) => game
                .sim
                .units()
                .get(id)
                .map(|u| format!("Unit {} / {}", id, unit_label(&u.state, game.sim.now())))
                .unwrap_or_default(),
            DesktopLive::Call(id, caller) => format!(
                "555-{:04} / {:02}:{:02}",
                100 + caller % 100,
                game.call_duration(*id) / 60,
                game.call_duration(*id) % 60
            ),
            DesktopLive::Voice => voice.status.clone(),
        };
        if text.0 != value {
            text.0 = value;
        }
    }
}

#[cfg(test)]
mod stable_desktop_tests {
    use super::*;
    #[test]
    fn clock_and_audio_status_do_not_replace_the_desktop_tree() {
        let mut game = Session::default();
        game.act(Action::Sit(0));
        game.tick(10.0);
        game.act(Action::OpenDesk);
        game.act(Action::Answer);
        let chat = crate::voice::ChatInput::default();
        let mut voice = crate::voice::Voice::default();
        let key = desktop_key(&game, &chat, &voice);
        game.tick(1.0);
        voice.status = "Listening for speech".into();
        voice.recording = true;
        assert_eq!(desktop_key(&game, &chat, &voice), key);
        voice.gate_db = -75;
        assert_ne!(desktop_key(&game, &chat, &voice), key);
    }
}

#[derive(Component)]
pub struct ReadyProgress;
#[derive(Component)]
pub struct ReadyFill;
#[derive(Component)]
pub struct ReadyLabel;
pub fn setup_ready_progress(mut commands: Commands) {
    commands
        .spawn((
            ReadyProgress,
            Visibility::Hidden,
            GlobalZIndex(50),
            Node {
                position_type: PositionType::Absolute,
                bottom: px(24),
                left: percent(32),
                width: percent(36),
                padding: UiRect::all(px(14)),
                flex_direction: FlexDirection::Column,
                row_gap: px(10),
                ..default()
            },
            BackgroundColor(INK),
        ))
        .with_children(|p| {
            p.spawn((
                ReadyLabel,
                Text::new("Hold E to open dispatch"),
                TextFont {
                    font_size: FontSize::Px(18.0),
                    ..default()
                },
                TextColor(TEXT),
            ));
            p.spawn((
                Node {
                    width: percent(100),
                    height: px(12),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
            ))
            .with_children(|p| {
                p.spawn((
                    ReadyFill,
                    Node {
                        width: percent(0),
                        height: percent(100),
                        ..default()
                    },
                    BackgroundColor(TEAL),
                ));
            });
        });
}
pub fn ready_progress(
    ready: Res<crate::world::ReadyInput>,
    game: Res<Session>,
    menu: Res<crate::frontend::Frontend>,
    prefs: Res<crate::frontend::Preferences>,
    mut bars: Query<&mut Visibility, With<ReadyProgress>>,
    mut fills: Query<&mut Node, With<ReadyFill>>,
    mut labels: Query<&mut Text, With<ReadyLabel>>,
) {
    let show = ready.active
        && game.phase == Phase::Lobby
        && game.screen == Screen::Office
        && !menu.visible
        && !game.ready.contains(&game.local_player);
    for mut bar in &mut bars {
        *bar = if show {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    for mut fill in &mut fills {
        fill.width = percent((ready.elapsed / 1.5).clamp(0.0, 1.0) * 100.0);
    }
    for mut label in &mut labels {
        label.0 = format!(
            "Hold {} — {}  ·  {:.0}%",
            prefs.keys[5].trim_start_matches("Key"),
            if game.local_player == 1 {
                "Open dispatch"
            } else {
                "Ready up"
            },
            (ready.elapsed / 1.5).clamp(0.0, 1.0) * 100.0
        );
    }
}

/// Apply contrast to newly created text and react to settings without rebuilding layout.
#[derive(Component)]
pub struct OriginalTextColor(Color);
pub fn accessibility(
    mut commands: Commands,
    prefs: Res<crate::frontend::Preferences>,
    mut texts: Query<(Entity, &mut TextColor, Option<&OriginalTextColor>)>,
) {
    for (entity, mut color, original) in &mut texts {
        let base = original.map_or(color.0, |v| v.0);
        if original.is_none() {
            commands.entity(entity).insert(OriginalTextColor(base));
        }
        let next = if prefs.high_contrast {
            Color::WHITE
        } else {
            base
        };
        if color.0 != next {
            color.0 = next;
        }
    }
}
