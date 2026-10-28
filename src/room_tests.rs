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
