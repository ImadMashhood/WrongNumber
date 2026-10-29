use crate::{
    game::{Action, Phase, Session},
    room::{Request, Response, Room, apply_snapshot},
    world,
};
use wrong_number_sim::{Command, UnitState};
fn joined() -> Room {
    let mut r = Room::new("Host".into());
    r.join(10, wrong_number_net::PROTOCOL, "Partner".into(), None)
        .unwrap();
    for id in [1, 2] {
        let p = r.members.get_mut(&id).unwrap();
        p.pose.position = [world::SEATS[usize::from(id - 1)], 1.65, 3.0];
    }
    r
}
fn act(r: &mut Room, id: u8, action: Action) -> crate::room::Ack {
    let seq = r.members[&id].last_seq + 1;
    r.action(id, seq, r.game.shift, action)
}
#[test]
fn seat_claims_host_permissions_and_duplicate_requests_are_authoritative() {
    let mut r = joined();
    assert!(!act(&mut r, 2, Action::StartShift).accepted);
    assert!(act(&mut r, 1, Action::Sit(0)).accepted);
    r.members.get_mut(&2).unwrap().pose.position = [world::SEATS[0], 1.65, 3.0];
    assert!(!act(&mut r, 2, Action::Sit(0)).accepted);
    let seq = r.members[&1].last_seq;
    let before = r.game.sim.save_json().unwrap();
    assert!(r.action(1, seq, 1, Action::Sit(0)).accepted);
    assert_eq!(r.game.sim.save_json().unwrap(), before);
    assert_eq!(r.game.occupied(0), Some(1));
    assert!(
        serde_json::from_str::<Request>(
            r#"{"Action":{"seq":2,"shift":1,"action":"StartShift","player":1}}"#
        )
        .is_err()
    );
}
#[test]
fn independent_lines_transfer_and_reconnect_keep_shared_world_intact() {
    let mut r = joined();
    act(&mut r, 1, Action::Sit(0));
    act(&mut r, 2, Action::Sit(1));
    act(&mut r, 1, Action::StartShift);
    r.tick(10.0);
    let one = r.game.desk(1).ringing.unwrap();
    let two = r.game.desk(2).ringing.unwrap();
    assert_ne!(one, two);
    act(&mut r, 2, Action::OpenDesk);
    act(&mut r, 2, Action::Answer);
    act(&mut r, 2, Action::Decline);
    act(&mut r, 1, Action::OpenDesk);
    act(&mut r, 1, Action::Answer);
    act(&mut r, 1, Action::Ask("smoke".into()));
    assert!(act(&mut r, 1, Action::Transfer(2)).accepted);
    assert_eq!(r.game.desk(2).conversation, Some(one));
    let token = r.members[&2].token.clone();
    r.disconnect(10);
    assert_eq!(r.game.occupied(1), None);
    assert!(r.game.sim.call_view(one).is_ok());
    let response = r
        .join(
            11,
            wrong_number_net::PROTOCOL,
            "Partner".into(),
            Some(token),
        )
        .unwrap();
    assert!(matches!(response, Response::Welcome { id: 2, .. }));
    let snapshot = r.snapshot();
    let mut client = Session::default();
    apply_snapshot(&mut client, &snapshot, 2).unwrap();
    assert_eq!(client.sim.public_snapshot(), r.game.sim.public_snapshot());
}
#[test]
fn racing_send_requests_allocate_once_and_old_shift_actions_are_rejected() {
    let mut r = joined();
    act(&mut r, 1, Action::Sit(0));
    act(&mut r, 2, Action::Sit(1));
    act(&mut r, 1, Action::StartShift);
    r.game.sim.activate_configured_call(1).unwrap();
    r.game.sim.activate_configured_call(4).unwrap();
    r.game
        .sim
        .apply(1, Command::AcceptCall { incident: 1 })
        .unwrap();
    r.game
        .sim
        .apply(2, Command::AcceptCall { incident: 4 })
        .unwrap();
    let mut d = r.game.desk(1);
    d.screen = crate::game::Screen::Terminal;
    d.selected = 1;
    d.conversation = Some(1);
    d.call_station = Some(0);
    r.game.load_desk(1, d);
    let mut d = r.game.desk(2);
    d.screen = crate::game::Screen::Terminal;
    d.selected = 4;
    d.conversation = Some(4);
    d.call_station = Some(1);
    r.game.desks.insert(2, d);
    assert!(
        act(
            &mut r,
            1,
            Action::Order {
                command: Command::Dispatch {
                    unit: 3,
                    incident: 1
                },
                expected: None
            }
        )
        .accepted
    );
    assert!(
        !act(
            &mut r,
            2,
            Action::Order {
                command: Command::Dispatch {
                    unit: 3,
                    incident: 4
                },
                expected: None
            }
        )
        .accepted
    );
    assert_eq!(
        r.game.sim.units()[&3].state,
        UnitState::Assigned { incident: 1 }
    );
    let seq = r.members[&2].last_seq + 1;
    assert!(!r.action(2, seq, 0, Action::Stand).accepted);
}
#[test]
fn online_menus_do_not_pause_the_host_clock_and_resume_tokens_are_not_player_ids() {
    let mut r = joined();
    act(&mut r, 1, Action::StartShift);
    act(&mut r, 1, Action::Pause);
    r.tick(5.0);
    assert_eq!(r.game.sim.now(), 5);
    assert!(
        r.join(
            12,
            wrong_number_net::PROTOCOL,
            "Pretender".into(),
            Some("2".into())
        )
        .is_err()
    );
    r.game.phase = Phase::Review;
    r.game.screen = crate::game::Screen::Review;
    r.game.funds = 100;
    assert!(!act(&mut r, 2, Action::BuyRadio).accepted);
    assert!(act(&mut r, 1, Action::BuyRadio).accepted);
    assert_eq!(r.game.funds, 50);
}
#[test]
fn explosion_closure_and_damage_replicate_without_pending_caller_secrets() {
    let mut r = joined();
    act(&mut r, 1, Action::StartShift);
    r.game.sim.activate_configured_call(1).unwrap();
    r.tick(121.0);
    assert!(
        r.game
            .sim
            .events()
            .iter()
            .any(|e| matches!(e.kind, wrong_number_sim::EventKind::RoadClosed { .. }))
    );
    assert!(r.game.sim.review().total_damage > 0);
    let snapshot = r.snapshot();
    for id in [1, 2] {
        let mut view = Session::default();
        apply_snapshot(&mut view, &snapshot, id).unwrap();
        assert_eq!(view.sim.public_snapshot(), r.game.sim.public_snapshot());
        assert!(view.sim.advance(1).is_err());
    }
    let packet = serde_json::to_string(&snapshot).unwrap();
    assert!(!packet.contains("\"truth\"") && !packet.contains("\"seed\""));
}
#[test]
fn lost_ringing_line_is_reassigned_and_bad_movement_is_corrected() {
    let mut r = joined();
    act(&mut r, 1, Action::Sit(0));
    act(&mut r, 2, Action::Sit(1));
    act(&mut r, 1, Action::StartShift);
    r.tick(10.0);
    let orphan = r.game.desk(2).ringing.unwrap();
    act(&mut r, 1, Action::OpenDesk);
    act(&mut r, 1, Action::Answer);
    act(&mut r, 1, Action::HangUp);
    r.disconnect(10);
    assert!(r.game.active_ringing_unclaimed(orphan));
    assert_eq!(r.game.occupied(1), None);
    act(&mut r, 1, Action::Stand);
    act(&mut r, 1, Action::Sit(0));
    r.tick(15.0);
    assert_eq!(r.game.desk(1).ringing, Some(orphan));
    let before = r.members[&1].pose;
    let mut bad = before;
    bad.position[0] = 999.0;
    assert!(r.pose(1, bad).is_err());
    assert_eq!(r.members[&1].pose.position, before.position);
}

#[test]
fn text_dialogue_is_owned_by_the_host_and_replicates_only_spoken_evidence() {
    let mut r = joined();
    act(&mut r, 1, Action::Sit(0));
    act(&mut r, 2, Action::Sit(1));
    act(&mut r, 1, Action::StartShift);
    r.tick(10.0);
    let incident = r.game.desk(2).ringing.unwrap();
    act(&mut r, 2, Action::OpenDesk);
    act(&mut r, 2, Action::Answer);
    assert!(
        !act(
            &mut r,
            1,
            Action::Talk {
                incident,
                text: "What happened?".into()
            }
        )
        .accepted
    );
    assert!(
        act(
            &mut r,
            2,
            Action::Talk {
                incident,
                text: "What happened?".into()
            }
        )
        .accepted
    );
    assert!(!act(&mut r, 2, Action::Transfer(1)).accepted);
    assert_eq!(r.game.dialogue.conversations[&incident].pending, Some(2));
    r.game.poll_dialogue();
    let state = r.snapshot();
    assert_eq!(state.conversations[&incident].turns.len(), 2);
    let mut mirror = Session::default();
    apply_snapshot(&mut mirror, &state, 2).unwrap();
    assert_eq!(
        mirror.dialogue.conversations[&incident].turns[1].spoken_text,
        r.game.dialogue.conversations[&incident].turns[1].spoken_text
    );
    assert_eq!(mirror.sim.call_view(incident).unwrap().facts.len(), 1);
    assert!(!serde_json::to_string(&state).unwrap().contains("\"truth\""));
}

#[test]
fn standing_computer_access_is_nearby_and_does_not_claim_a_seat() {
    let mut r = joined();
    assert!(act(&mut r, 2, Action::OpenDesk).accepted);
    assert_eq!(r.game.desk(2).screen, crate::game::Screen::Terminal);
    assert!(r.game.desk(2).seated.is_none());
    act(&mut r, 2, Action::Office);
    r.members.get_mut(&2).unwrap().pose.position = [12.0, 1.65, 4.0];
    assert!(!act(&mut r, 2, Action::OpenDesk).accepted);
}

fn staging_room(players: u8) -> Room {
    let mut r = Room::new("Host".into());
    r.game.staging_lobby();
    for id in 2..=players {
        r.join(
            u64::from(id),
            wrong_number_net::PROTOCOL,
            format!("Player{id}"),
            None,
        )
        .unwrap();
    }
    for m in r.members.values_mut() {
        m.pose.position = [9.8, 1.65, 0.5];
    }
    r
}
fn held_ready(r: &mut Room, id: u8) {
    assert!(act(r, id, Action::Ready(true)).accepted);
    let now = std::time::Instant::now();
    r.ready_holds
        .insert(id, (now - std::time::Duration::from_millis(1600), now));
    r.tick(0.0);
}
#[test]
fn door_ready_requires_half_connected_players_or_the_host() {
    let mut r = staging_room(6);
    held_ready(&mut r, 2);
    assert_eq!(r.game.phase, Phase::Lobby);
    assert_eq!(r.game.ready.len(), 1);
    held_ready(&mut r, 3);
    assert_eq!(r.game.phase, Phase::Lobby);
    let snapshot = r.snapshot();
    let mut mirror = Session::default();
    apply_snapshot(&mut mirror, &snapshot, 2).unwrap();
    assert_eq!(mirror.ready, r.game.ready);
    held_ready(&mut r, 4);
    assert_eq!(r.game.phase, Phase::Running);
    assert!(r.game.ready.is_empty());
    let mut r = staging_room(6);
    held_ready(&mut r, 1);
    assert_eq!(r.game.phase, Phase::Running);
}
#[test]
fn ready_hold_rejects_remote_short_cancelled_and_stale_attempts() {
    let mut r = staging_room(6);
    r.members.get_mut(&2).unwrap().pose.position = [12.0, 1.65, 5.0];
    assert!(!act(&mut r, 2, Action::Ready(true)).accepted);
    r.members.get_mut(&2).unwrap().pose.position = [9.8, 1.65, 0.5];
    assert!(act(&mut r, 2, Action::Ready(true)).accepted);
    r.tick(0.0);
    assert!(r.game.ready.is_empty());
    assert!(act(&mut r, 2, Action::Ready(false)).accepted);
    assert!(r.ready_holds.is_empty());
    let old = std::time::Instant::now() - std::time::Duration::from_secs(2);
    r.ready_holds.insert(2, (old, old));
    r.tick(0.0);
    assert!(r.game.ready.is_empty());
    held_ready(&mut r, 2);
    held_ready(&mut r, 3);
    r.disconnect(6);
    assert_eq!(r.game.phase, Phase::Lobby);
    r.disconnect(5);
    assert_eq!(r.game.phase, Phase::Running);
}
#[test]
fn completed_shift_returns_all_players_to_breakroom_and_clears_votes() {
    let mut r = staging_room(2);
    held_ready(&mut r, 1);
    act(&mut r, 1, Action::EndShift);
    for _ in 0..700 {
        r.tick(1.0);
        if r.game.phase == Phase::Review {
            break;
        }
    }
    assert_eq!(r.game.shift, 1);
    assert_eq!(r.game.phase, Phase::Review);
    assert_eq!(r.game.screen, crate::game::Screen::Review);
    assert!(r.game.ready.is_empty());
    assert!(
        !r.action(2, r.members[&2].last_seq + 1, 1, Action::NextShift)
            .accepted
    );
    act(&mut r, 1, Action::NextShift);
    assert_eq!(r.game.shift, 2);
    assert_eq!(r.game.phase, Phase::Lobby);
    for m in r.members.values() {
        assert_eq!(m.pose.position, crate::room::Pose::spawn(m.id).position);
        assert!(r.game.desk(m.id).seated.is_none());
    }
    assert!(!world::walkable(
        bevy::prelude::Vec2::new(8.8, 0.5),
        Phase::Lobby
    ));
    assert!(world::walkable(
        bevy::prelude::Vec2::new(8.8, 0.5),
        Phase::Running
    ));
}

#[test]
fn door_start_then_sitting_receives_a_call_within_initial_delay() {
    let mut r = staging_room(1);
    held_ready(&mut r, 1);
    assert_eq!(r.game.phase, Phase::Running);
    // Merely focusing a computer while standing must not silently imply phone readiness.
    r.game.as_player(1, Action::OpenDesk).unwrap();
    r.tick(11.0);
    assert!(r.game.ringing.is_none());
    r.game.as_player(1, Action::Sit(0)).unwrap();
    r.tick(11.0);
    assert!(r.game.ringing.is_some());
}

#[test]
fn throwable_authority_replication_and_recovery_preserve_dispatch() {
    let mut r = joined();
    r.game.phase = Phase::Running;
    r.members.get_mut(&1).unwrap().pose.position = [0.0, 1.65, 5.0];
    r.members.get_mut(&2).unwrap().pose.position = [0.0, 1.65, 3.0];
    r.game.fun.props[0].position = bevy::prelude::Vec3::new(0.0, 1.4, 4.0);
    assert!(act(&mut r, 1, Action::Pickup(0)).accepted);
    assert!(!act(&mut r, 2, Action::Pickup(0)).accepted);
    assert!(act(&mut r, 1, Action::ThrowProp).accepted);
    let funds = r.game.funds;
    for _ in 0..30 {
        r.tick(1.0 / 120.0);
    }
    assert!(r.game.fun.down(2));
    let state = r.snapshot();
    let mut mirror = Session::default();
    apply_snapshot(&mut mirror, &state, 2).unwrap();
    assert!(mirror.fun.down(2));
    assert_eq!(mirror.fun.dolls[&2].points, r.game.fun.dolls[&2].points);
    let mut moved = r.members[&2].pose;
    moved.position[0] += 0.1;
    assert!(r.pose(2, moved).is_err());
    assert!(!act(&mut r, 2, Action::Sit(0)).accepted);
    assert_eq!(r.game.funds, funds);
    for _ in 0..360 {
        r.tick(1.0 / 120.0);
    }
    assert!(!r.game.fun.down(2));
    r.game.phase = Phase::Review;
    assert!(act(&mut r, 1, Action::NextShift).accepted);
    assert!(r.game.fun.dolls.is_empty());
    assert!(r.game.fun.props.iter().all(|p| p.held.is_none()));
}

#[test]
fn ragdoll_keeps_a_seated_dispatchers_call_connected() {
    let mut r = joined();
    assert!(act(&mut r, 2, Action::Sit(1)).accepted);
    assert!(act(&mut r, 1, Action::StartShift).accepted);
    for _ in 0..20 {
        r.tick(1.0);
        if r.game.desk(2).ringing.is_some() {
            break;
        }
    }
    assert!(act(&mut r, 2, Action::OpenDesk).accepted);
    let answer = act(&mut r, 2, Action::Answer);
    assert!(answer.accepted, "{}: {:?}", answer.message, r.game.desk(2));
    let before = r.game.desk(2);
    let pose = r.members[&2].pose;
    r.game.fun.dolls.insert(
        2,
        crate::office_fun::Doll::new(pose, bevy::prelude::Vec3::X * 5.0),
    );
    for _ in 0..60 {
        r.tick(1.0 / 60.0);
    }
    assert_eq!(r.game.desk(2).conversation, before.conversation);
    assert_eq!(r.game.desk(2).seated, before.seated);
    assert_eq!(r.game.desk(2).screen, before.screen);
    let mut mirror = Session::default();
    apply_snapshot(&mut mirror, &r.snapshot(), 2).unwrap();
    assert_eq!(mirror.conversation, before.conversation);
}

#[test]
fn monitor_claims_chair_atomically_and_delivers_after_long_standing_wait() {
    let mut r = joined();
    assert!(act(&mut r, 1, Action::StartShift).accepted);
    r.tick(120.0);
    assert!(r.game.desk(1).ringing.is_none());
    assert!(act(&mut r, 1, Action::UseDesk(0)).accepted);
    assert_eq!(r.game.desk(1).seated, Some(0));
    assert_eq!(r.game.desk(1).screen, crate::game::Screen::Terminal);
    r.members.get_mut(&2).unwrap().pose.position = [world::SEATS[0], 1.65, 3.0];
    assert!(!act(&mut r, 2, Action::UseDesk(0)).accepted);
    assert_eq!(r.game.desk(2).seated, None);
    for _ in 0..11 {
        r.tick(1.0);
    }
    assert!(r.game.desk(1).ringing.is_some());
    let mut mirror = Session::default();
    apply_snapshot(&mut mirror, &r.snapshot(), 1).unwrap();
    assert_eq!(mirror.seated, Some(0));
    assert!(mirror.ringing.is_some());
    assert!(!act(&mut r, 2, Action::UseDesk(99)).accepted);
}
