use wrong_number_sim::*;

fn accept(sim: &mut Simulation, id: u32) {
    sim.apply(1, Command::AcceptCall { incident: id }).unwrap();
}
fn investigate(sim: &mut Simulation, id: u32) {
    accept(sim, id);
    let fact = sim.call_view(id).unwrap().questions[0].0.clone();
    sim.apply(1, Command::Ask { incident: id, fact }).unwrap();
}
fn dispatched() -> Simulation {
    let mut sim = Simulation::new(42);
    investigate(&mut sim, 1);
    sim.apply(
        1,
        Command::Dispatch {
            unit: 3,
            incident: 1,
        },
    )
    .unwrap();
    sim
}
#[test]
fn same_seed_and_commands_replay_identically() {
    let mut a = dispatched();
    let mut b = dispatched();
    a.advance(90).unwrap();
    b.advance(30).unwrap();
    b.advance(60).unwrap();
    assert_eq!(a.save_json().unwrap(), b.save_json().unwrap());
    assert_ne!(
        Simulation::new(1).incidents()[&1].location,
        Simulation::new(2).incidents()[&1].location
    );
}
#[test]
fn clients_only_receive_revealed_caller_facts() {
    let mut sim = Simulation::new(42);
    let view = sim.call_view(5).unwrap();
    assert!(view.facts.is_empty());
    assert!(view.questions.is_empty());
    let json = serde_json::to_string(&view).unwrap();
    for hidden in [
        "Emergency",
        "Underreported",
        "barely breathing",
        "truth",
        "category",
    ] {
        assert!(!json.contains(hidden));
    }
    investigate(&mut sim, 5);
    assert!(
        sim.call_view(5).unwrap().facts[0]
            .answer
            .contains("barely breathing")
    );
    assert_eq!(sim.incidents()[&5].truth, Truth::Emergency);
}
#[test]
fn unknown_facts_and_foreign_call_owner_cannot_change_state() {
    let mut sim = Simulation::new(42);
    accept(&mut sim, 2);
    let before = sim.clone();
    assert_eq!(
        sim.apply(
            1,
            Command::Ask {
                incident: 2,
                fact: "make_prank_real".into()
            }
        ),
        Err(Error::FactUnavailable)
    );
    assert_eq!(sim, before);
    assert_eq!(
        sim.apply(
            2,
            Command::Ask {
                incident: 2,
                fact: "story".into()
            }
        ),
        Err(Error::NotCallOwner)
    );
    assert_eq!(sim, before);
    assert_eq!(sim.incidents()[&2].truth, Truth::Benign);
}
#[test]
fn competing_dispatches_allocate_the_unit_once() {
    let mut sim = Simulation::new(42);
    accept(&mut sim, 1);
    accept(&mut sim, 4);
    sim.apply(
        1,
        Command::Dispatch {
            unit: 3,
            incident: 1,
        },
    )
    .unwrap();
    let before = sim.clone();
    assert_eq!(
        sim.apply(
            2,
            Command::Dispatch {
                unit: 3,
                incident: 4
            }
        ),
        Err(Error::UnitBusy)
    );
    assert_eq!(sim, before);
    assert_eq!(sim.units()[&3].state.assignment(), Some(1));
}
#[test]
fn validates_player_ids_resources_capabilities_and_transitions() {
    let mut sim = Simulation::new(42);
    for command in [
        Command::Cancel { unit: 1 },
        Command::Reroute {
            unit: 1,
            incident: 6,
        },
    ] {
        assert_eq!(sim.apply(1, command), Err(Error::InvalidTransition));
    }
    assert_eq!(
        sim.apply(0, Command::AcceptCall { incident: 1 }),
        Err(Error::InvalidPlayer)
    );
    assert_eq!(
        sim.apply(5, Command::AcceptCall { incident: 1 }),
        Err(Error::InvalidPlayer)
    );
    assert_eq!(
        sim.apply(1, Command::AcceptCall { incident: 999 }),
        Err(Error::UnknownIncident)
    );
    assert_eq!(
        sim.apply(
            1,
            Command::Dispatch {
                unit: 999,
                incident: 1
            }
        ),
        Err(Error::UnknownUnit)
    );
    assert_eq!(
        sim.apply(
            1,
            Command::Dispatch {
                unit: 3,
                incident: 1
            }
        ),
        Err(Error::CallUnavailable)
    );
    accept(&mut sim, 1);
    assert_eq!(
        sim.apply(
            1,
            Command::Dispatch {
                unit: 1,
                incident: 1
            }
        ),
        Err(Error::WrongCapability)
    );
}
#[test]
fn responder_completes_every_lifecycle_stage_and_recovers() {
    let mut sim = dispatched();
    assert!(matches!(sim.units()[&3].state, UnitState::Assigned { .. }));
    sim.advance(90).unwrap();
    assert_eq!(sim.units()[&3].state, UnitState::Available);
    assert_eq!(sim.units()[&3].position, sim.units()[&3].station);
    assert_eq!(sim.incidents()[&1].state, IncidentState::Resolved);
    assert_eq!(sim.score(), 50);
    let states: Vec<_> = sim
        .events()
        .iter()
        .filter_map(|e| match &e.kind {
            EventKind::UnitChanged { unit: 3, state } => Some(state),
            _ => None,
        })
        .collect();
    assert!(matches!(
        states.as_slice(),
        [
            UnitState::Assigned { .. },
            UnitState::EnRoute { .. },
            UnitState::OnScene { .. },
            UnitState::Resolving { .. },
            UnitState::Returning { .. },
            UnitState::Recovering { .. },
            UnitState::Available
        ]
    ));
}
#[test]
fn cancellation_returns_and_recovers_instead_of_freeing_capacity() {
    let mut sim = dispatched();
    sim.advance(3).unwrap();
    assert_ne!(sim.units()[&3].position, sim.units()[&3].station);
    sim.apply(1, Command::Cancel { unit: 3 }).unwrap();
    assert!(matches!(sim.units()[&3].state, UnitState::Returning { .. }));
    assert_eq!(
        sim.apply(
            1,
            Command::Dispatch {
                unit: 3,
                incident: 1
            }
        ),
        Err(Error::UnitBusy)
    );
    sim.advance(20).unwrap();
    assert_eq!(sim.units()[&3].state, UnitState::Available);
    assert_eq!(sim.incidents()[&1].state, IncidentState::Active);
}
#[test]
fn reroute_keeps_position_and_releases_the_old_assignment() {
    let mut sim = dispatched();
    accept(&mut sim, 4);
    sim.advance(4).unwrap();
    let position = sim.units()[&3].position;
    sim.apply(
        2,
        Command::Reroute {
            unit: 3,
            incident: 4,
        },
    )
    .unwrap();
    assert_eq!(sim.units()[&3].position, position);
    assert_eq!(sim.units()[&3].state.assignment(), Some(4));
    assert_eq!(sim.incidents()[&1].state, IncidentState::Active);
    let before = sim.clone();
    assert_eq!(
        sim.apply(
            1,
            Command::Reroute {
                unit: 3,
                incident: 5
            }
        ),
        Err(Error::CallUnavailable)
    );
    assert_eq!(sim, before);
}
#[test]
fn evidence_based_refusal_reward_and_waste_penalty_are_logged() {
    let mut sim = Simulation::new(42);
    investigate(&mut sim, 2);
    sim.apply(1, Command::Decline { incident: 2 }).unwrap();
    assert_eq!(sim.score(), 20);
    assert_eq!(sim.incidents()[&2].state, IncidentState::Dismissed);
    assert_eq!(
        sim.apply(1, Command::Decline { incident: 2 }),
        Err(Error::IncidentClosed)
    );
    investigate(&mut sim, 4);
    sim.apply(
        1,
        Command::Dispatch {
            unit: 3,
            incident: 4,
        },
    )
    .unwrap();
    sim.advance(90).unwrap();
    assert_eq!(sim.score(), 5);
    assert!(sim.events().iter().any(|e| matches!(
        e.kind,
        EventKind::IncidentEnded {
            incident: 4,
            score_delta: -15,
            ..
        }
    )));
}
#[test]
fn expired_reports_respect_deadline_and_available_evidence() {
    let mut known = Simulation::new(42);
    let mut unknown = known.clone();
    investigate(&mut known, 1);
    accept(&mut unknown, 1);
    known.apply(1, Command::Decline { incident: 1 }).unwrap();
    assert_eq!(known.incidents()[&1].state, IncidentState::Active);
    known.advance(119).unwrap();
    assert_eq!(known.incidents()[&1].state, IncidentState::Active);
    known.advance(1).unwrap();
    unknown.advance(120).unwrap();
    assert_eq!(known.incidents()[&1].state, IncidentState::Expired);
    assert_eq!(known.score(), -60);
    assert_eq!(unknown.score(), -5);
    assert_eq!(known.incidents()[&1].truth, Truth::Emergency);
}
#[test]
fn deadline_wins_over_resolution_at_same_timestamp() {
    let mut sim = Simulation::new(42);
    let distance = sim.incidents()[&1].location.x as u32 + sim.incidents()[&1].location.y as u32;
    let dispatch_time = 120 - distance - 10; // assigned + travel + on-scene + 8 seconds work
    sim.advance(dispatch_time).unwrap();
    investigate(&mut sim, 1);
    sim.apply(
        1,
        Command::Dispatch {
            unit: 3,
            incident: 1,
        },
    )
    .unwrap();
    sim.advance(120 - dispatch_time).unwrap();
    assert_eq!(sim.incidents()[&1].state, IncidentState::Expired);
    assert_eq!(sim.score(), -60);
    assert!(matches!(sim.units()[&3].state, UnitState::Returning { .. }));
}
#[test]
fn save_at_every_tick_resumes_to_the_same_result() {
    let mut sim = dispatched();
    for _ in 0..140 {
        let mut restored = Simulation::load_json(&sim.save_json().unwrap()).unwrap();
        sim.advance(1).unwrap();
        restored.advance(1).unwrap();
        assert_eq!(sim, restored);
    }
}
#[test]
fn corrupt_unsupported_and_oversized_saves_are_rejected() {
    let sim = dispatched();
    let json = sim.save_json().unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    for mutate in [
        |v: &mut serde_json::Value| v["version"] = 999.into(),
        |v: &mut serde_json::Value| v["simulation"]["score"] = 999.into(),
        |v: &mut serde_json::Value| {
            v["simulation"]["incidents"]["1"]["revealed"] = serde_json::json!(["invented"])
        },
        |v: &mut serde_json::Value| {
            v["simulation"]["units"]["3"]["state"] =
                serde_json::json!({"Assigned":{"incident":999}})
        },
        |v: &mut serde_json::Value| {
            v["simulation"]["units"]["3"]["position"]["x"] = i32::MIN.into()
        },
        |v: &mut serde_json::Value| v["simulation"]["events"][0]["at"] = 999.into(),
    ] {
        let mut v = value.clone();
        mutate(&mut v);
        assert!(Simulation::load_json(&v.to_string()).is_err());
    }
    assert!(Simulation::load_json("{}").is_err());
    assert!(Simulation::load_json(&" ".repeat(1_048_577)).is_err());
}
#[test]
fn time_validation_is_atomic() {
    let mut sim = Simulation::new(0);
    let before = sim.clone();
    assert_eq!(sim.advance(0), Err(Error::InvalidAdvance));
    assert_eq!(sim.advance(3601), Err(Error::InvalidAdvance));
    assert_eq!(sim, before);
}

#[test]
fn serialized_commands_cannot_invent_facts_or_mutate_truth() {
    let mut sim = Simulation::new(42);
    accept(&mut sim, 2);
    let before = sim.clone();
    for json in [
        r#"{"ChangeTruth":{"incident":2,"truth":"Emergency"}}"#,
        r#"{"Ask":{"incident":2,"fact":"story","truth":"Emergency"}}"#,
        r#"{"Dispatch":{"unit":-1,"incident":2}}"#,
    ] {
        assert!(matches!(
            sim.apply_json(1, json),
            Err(Error::InvalidCommand(_))
        ));
        assert_eq!(sim, before);
    }
    assert_eq!(
        sim.apply_json(1, r#"{"Ask":{"incident":2,"fact":"invented"}}"#),
        Err(Error::FactUnavailable)
    );
    assert!(sim.apply_json(1, &" ".repeat(8193)).is_err());
    sim.apply_json(1, r#"{"Ask":{"incident":2,"fact":"story"}}"#)
        .unwrap();
    assert_eq!(sim.incidents()[&2].truth, Truth::Benign);
}

#[test]
fn raw_deserialization_also_validates_simulation_invariants() {
    let mut raw: serde_json::Value = serde_json::to_value(dispatched()).unwrap();
    raw["units"]["3"]["state"] = serde_json::json!({"Assigned":{"incident":999}});
    assert!(serde_json::from_value::<Simulation>(raw).is_err());
}

#[test]
fn corrupted_event_references_and_duplicate_outcomes_are_rejected() {
    let mut sim = Simulation::new(42);
    investigate(&mut sim, 2);
    sim.apply(1, Command::Decline { incident: 2 }).unwrap();
    let original: serde_json::Value = serde_json::from_str(&sim.save_json().unwrap()).unwrap();
    let mut bad_ref = original.clone();
    bad_ref["simulation"]["events"][0]["kind"]["CallAccepted"]["incident"] = 999.into();
    assert!(Simulation::load_json(&bad_ref.to_string()).is_err());
    let mut duplicate = original;
    let events = duplicate["simulation"]["events"].as_array_mut().unwrap();
    let mut last = events.last().unwrap().clone();
    last["sequence"] = (events.len() as u64).into();
    events.push(last);
    assert!(Simulation::load_json(&duplicate.to_string()).is_err());
}

#[test]
fn shift_review_matches_logged_outcomes() {
    let mut sim = Simulation::new(42);
    for id in [2, 3, 4] {
        investigate(&mut sim, id);
        sim.apply(1, Command::Decline { incident: id }).unwrap();
    }
    let review = sim.review();
    assert_eq!(review.score, 60);
    assert_eq!(review.dismissed, 3);
    assert_eq!(review.active, 3);
    assert_eq!(review.available_units, 3);
    assert_eq!(
        review.score_entries.iter().map(|e| e.points).sum::<i32>(),
        review.score
    );
    assert!(review.score_entries.iter().all(|e| !e.reason.is_empty()));
}

#[test]
fn shift_configuration_changes_stations_recovery_and_capacity() {
    let config = ShiftConfig {
        recovery_seconds: 2,
        extra_police: true,
        stations: [Position { x: 2, y: 3 }; 3],
        ..Default::default()
    };
    let mut sim = Simulation::new_with_config(42, config).unwrap();
    assert_eq!(sim.units().len(), 4);
    assert!(
        sim.units()
            .values()
            .all(|u| u.station == Position { x: 2, y: 3 }
                && u.position == u.station
                && u.recovery_seconds == 2)
    );
    investigate(&mut sim, 6);
    sim.apply(
        1,
        Command::Dispatch {
            unit: 1,
            incident: 6,
        },
    )
    .unwrap();
    assert_eq!(
        sim.apply(
            2,
            Command::Dispatch {
                unit: 4,
                incident: 6
            }
        ),
        Err(Error::IncidentAlreadyAssigned)
    );
    assert_eq!(
        Simulation::load_json(&sim.save_json().unwrap()).unwrap(),
        sim
    );
    assert!(
        Simulation::new_with_config(
            42,
            ShiftConfig {
                recovery_seconds: 0,
                ..config
            }
        )
        .is_err()
    );
}
