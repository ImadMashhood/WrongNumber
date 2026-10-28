use wrong_number_sim::*;
#[test]
fn public_replication_omits_pending_truth_and_unrevealed_answers_and_is_read_only() {
    let mut s = Simulation::new_with_config(
        42,
        ShiftConfig {
            queued_calls: true,
            ..Default::default()
        },
    )
    .unwrap();
    let initial = serde_json::to_string(&s.public_snapshot()).unwrap();
    assert!(!initial.contains("Smoke in the laundromat"));
    assert!(!initial.contains("truth"));
    s.activate_configured_call(1).unwrap();
    let json = serde_json::to_string(&s.public_snapshot()).unwrap();
    assert!(!json.contains("Thick black smoke"));
    s.apply(1, Command::AcceptCall { incident: 1 }).unwrap();
    s.apply(
        1,
        Command::Ask {
            incident: 1,
            fact: "smoke".into(),
        },
    )
    .unwrap();
    let public = s.public_snapshot();
    let mut mirror = Simulation::from_public_snapshot(public.clone()).unwrap();
    assert_eq!(mirror.public_snapshot(), public);
    assert!(mirror.advance(1).is_err());
    assert!(
        mirror
            .apply(
                1,
                Command::Dispatch {
                    unit: 3,
                    incident: 1
                }
            )
            .is_err()
    );
    assert!(mirror.save_json().is_err());
}
#[test]
fn transfer_preserves_evidence_changes_owner_and_rejects_former_owner() {
    let mut s = Simulation::new(42);
    s.apply(1, Command::AcceptCall { incident: 1 }).unwrap();
    s.apply(
        1,
        Command::Ask {
            incident: 1,
            fact: "smoke".into(),
        },
    )
    .unwrap();
    s.apply(1, Command::TransferCall { incident: 1, to: 6 })
        .unwrap();
    assert_eq!(s.call_view(1).unwrap().facts.len(), 1);
    assert!(s.apply(1, Command::EndCall { incident: 1 }).is_err());
    s.apply(6, Command::EndCall { incident: 1 }).unwrap();
    Simulation::load_json(&s.save_json().unwrap()).unwrap();
}
