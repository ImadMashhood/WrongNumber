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
pub fn load_map(mut commands: Commands, assets: Res<AssetServer>) {
    commands.insert_resource(MapImage(assets.load("city/map.png")));
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
    e.with_children(|p| text(p, label, 14.0, if enabled { TEXT } else { MUTED }));
}
fn column(width: f32) -> Node {
    Node {
        width: percent(width),
        min_width: px(0),
        flex_direction: FlexDirection::Column,
        padding: UiRect::all(px(16)),
        row_gap: px(8),
        ..default()
    }
}
fn state_name(state: IncidentState) -> &'static str {
    match state {
        IncidentState::Pending => "QUEUED",
        IncidentState::Unreported => "UNREPORTED",
        IncidentState::Active => "OPEN",
        IncidentState::Resolved => "RESOLVED",
        IncidentState::Expired => "EXPIRED",
        IncidentState::Dismissed => "REFUSED",
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
fn map(p: &mut ChildSpawnerCommands, game: &Session, image: &MapImage) {
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
            let color = if view.state == IncidentState::Active {
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
    text(p, "TRAINING GRID / ROAD ROUTING PENDING", 11.0, AMBER);
}
fn terminal(p: &mut ChildSpawnerCommands, game: &Session, image: &MapImage) {
    let running = matches!(game.phase, Phase::Running | Phase::Closing);
    p.spawn((column(23.0), BackgroundColor(PANEL)))
        .with_children(|p| {
            text(p, "INCOMING / CASE QUEUE", 14.0, MUTED);
            for id in game.visible_ids() {
                let v = game.sim.call_view(*id).unwrap();
                let tag = if v.state != IncidentState::Active {
                    state_name(v.state)
                } else {
                    match v.call {
                        CallState::Queued => "QUEUED",
                        CallState::Ringing => "RINGING",
                        CallState::Answered { .. } => "ON LINE",
                        CallState::Declined => "DECLINED",
                        CallState::Closed => "CLOSED",
                    }
                };
                let selected = if *id == game.selected { "> " } else { "" };
                button(
                    p,
                    format!("{selected}{id:02} / {tag}\n{}", v.report),
                    Action::Select(*id),
                    true,
                );
            }
            if running {
                button(
                    p,
                    "Finish shift early / skip remaining time",
                    Action::EndShift,
                    true,
                );
            }
            button(p, "Leave console [Esc]", Action::Office, true);
        });
    p.spawn((column(36.0),BackgroundColor(PANEL))).with_children(|p| {
        let Ok(view)=game.sim.call_view(game.selected) else {text(p,"Waiting for your first caller",25.0,TEXT);text(p,"Calls arrive 1–10 seconds after sitting. The clock keeps running outside computer focus.",17.0,MUTED);return;};
        text(p,format!("CALL {:02} / {}",game.selected,state_name(view.state)),14.0,AMBER);
        text(p,view.report,25.0,TEXT);
        text(p,format!("Grid {}, {}",view.location.x,view.location.y),13.0,MUTED);
        if game.phase==Phase::AwaitingReview {text(p,"Shift finished. Leave the console and walk through the rear door into the meeting room.",19.0,TEXT);}
        else {
            if view.state==IncidentState::Active {text(p,format!("Response window: {}s",game.sim.incidents()[&game.selected].deadline.saturating_sub(game.sim.now())),13.0,MUTED);}
            if view.call==CallState::Ringing {button(p,"ANSWER CALL",Action::Answer,running&&game.call_station==game.seated&&game.conversation.is_none());}
            if view.facts.is_empty() {text(p,"No evidence collected yet. Ask questions before committing limited teams.",15.0,MUTED);}
            for fact in view.facts {text(p,format!("YOU: {}",fact.question),13.0,MUTED); text(p,format!("CALLER: {}",fact.answer),17.0,TEXT);}
            for (id,question) in view.questions {button(p,format!("Ask: {question}"),Action::Ask(id),running&&game.conversation==Some(game.selected)&&game.call_station==game.seated);}
            if view.state==IncidentState::Active {button(p,"Decline response",Action::Decline,running&&game.conversation==Some(game.selected)&&game.call_station==game.seated);}
        }
        button(p,"End conversation",Action::HangUp,game.conversation.is_some()&&game.call_station==game.seated);
        text(p,"ASSIGN / CANCEL / REROUTE",13.0,MUTED);
        for unit in game.sim.units().values() {
            let verb=match unit.state {UnitState::Available=>"Send",UnitState::Assigned {incident} | UnitState::EnRoute {incident,..} if incident==game.selected=>"Cancel",UnitState::Assigned {..} | UnitState::EnRoute {..}=>"Reroute",_=>"Busy"};
            let enabled=running&&game.selected_call_answered()&&matches!(unit.state,UnitState::Available|UnitState::Assigned {..}|UnitState::EnRoute {..});
            button(p,format!("{verb} {} {} / {}",cap_name(unit.capability),unit.id,unit_label(&unit.state,game.sim.now())),Action::Unit(unit.id),enabled);
        }
    });
    p.spawn((column(39.0),BackgroundColor(PANEL))).with_children(|p| {
        text(p,"CITY OPERATIONS",14.0,MUTED); map(p, game, image);
        text(p,"SHARED RESPONSE CAPACITY",14.0,MUTED);
        text(p,format!("Run funding ${} | Recovery {}s",game.funds,if game.radio {2} else {5}),12.0,MUTED);
        for u in game.sim.units().values() {text(p,format!("{} {}: {}",cap_name(u.capability),u.id,unit_label(&u.state,game.sim.now())),13.0,cap_color(u.capability));}
        text(p,"Gather evidence. Appropriate refusals earn funding; unnecessary deployments occupy real capacity.",15.0,TEXT);
    });
}
fn review(p: &mut ChildSpawnerCommands, game: &Session) {
    let result = game.sim.review();
    p.spawn((column(66.0),BackgroundColor(PANEL))).with_children(|p| {
        text(p,format!("SHIFT REVIEW / GRADE {}",game.grade()),24.0,TEXT);
        text(p,format!("{} resolved | {} refused | {} expired",result.resolved,result.dismissed,result.expired),18.0,AMBER);
        for entry in &result.score_entries {text(p,format!("{:02}:{:02}  Call {}  {:+} pts\n{}",entry.at/60,entry.at%60,entry.incident,entry.points,entry.reason),16.0,TEXT);}
        text(p,format!("Score {}  /  Base grant 100 + score = {} funding (minimum 0)",result.score,game.grant()),17.0,AMBER);
        text(p,"Recorded decisions drive this review. Upgrades apply to the next shift in this run.",14.0,MUTED);
    });
    p.spawn((column(33.0), BackgroundColor(PANEL)))
        .with_children(|p| {
            text(p, format!("AVAILABLE FUNDING\n${}", game.funds), 28.0, TEXT);
            text(p, "One purchase per review", 15.0, MUTED);
            button(
                p,
                if game.radio {
                    "Radios installed"
                } else {
                    "$50 / Improved radios\nRecovery: 5 seconds -> 2 seconds"
                },
                Action::BuyRadio,
                !game.radio && !game.bought_this_review && game.funds >= 50,
            );
            button(
                p,
                if game.extra_police {
                    "Patrol installed"
                } else {
                    "$90 / Additional patrol\nA second police unit next shift"
                },
                Action::BuyPolice,
                !game.extra_police && !game.bought_this_review && game.funds >= 90,
            );
            button(p, "Prepare next shift ->", Action::NextShift, true);
            button(p, "Return to office", Action::Office, true);
        });
}
pub fn draw(
    mut commands: Commands,
    roots: Query<Entity, With<UiRoot>>,
    mut game: ResMut<Session>,
    image: Res<MapImage>,
    player: Single<&Transform, With<Player>>,
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
                    text(p, "WRONG NUMBER  /  DISPATCH AUTHORITY", 22.0, TEXT);
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
                    Screen::Terminal => terminal(p, &game, &image),
                    Screen::CityMap => {
                        p.spawn((column(70.0), BackgroundColor(PANEL)))
                            .with_children(|p| {
                                map(p, &game, &image);
                                button(p, "Return to room", Action::Office, true);
                            });
                    }
                    Screen::Review => review(p, &game),
                    Screen::Pause => {
                        p.spawn((column(50.0), BackgroundColor(PANEL)))
                            .with_children(|p| {
                                text(p, "PAUSED", 32.0, TEXT);
                                text(p, "The shift clock is stopped.", 18.0, MUTED);
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
        let next = BorderColor::all(match target {
            Some(Action::Door | Action::OpenReview)
                if !crate::world::review_unlocked(game.phase) =>
            {
                Color::srgb(1.0, 0.25, 0.2)
            }
            Some(_) => TEAL,
            None => TEXT.with_alpha(0.6),
        });
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
