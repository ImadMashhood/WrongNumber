use wrong_number_sim::*;
fn shift() -> Simulation {
    let mut sim = Simulation::new_with_config(
        42,
        ShiftConfig {
            queued_calls: true,
            ..Default::default()
        },
    )
    .unwrap();
    sim.enable_consequences().unwrap();
    sim
}
#[test]
fn declining_does_not_apply_immediate_damage_but_delayed_damage_continues() {
    let mut sim = shift();
    sim.activate_call(1, 300).unwrap();
    sim.apply(1, Command::AcceptCall { incident: 1 }).unwrap();
    sim.apply(1, Command::Decline { incident: 1 }).unwrap();
    assert_eq!(sim.review().total_damage, 0);
    sim.advance(45).unwrap();
    assert_eq!(sim.review().total_damage, 0);
    sim.advance(1).unwrap();
    assert_eq!(sim.review().total_damage, 500);
    sim.advance(254).unwrap();
    assert_eq!(sim.review().total_damage, 127_500);
    assert_eq!(sim.incidents()[&1].state, IncidentState::Expired);
    let damage = sim.review().total_damage;
    sim.advance(20).unwrap();
    assert_eq!(sim.review().total_damage, damage);
}
#[test]
fn benign_truth_cannot_generate_monetary_damage_and_pending_calls_stay_untouched() {
    let mut sim = shift();
    sim.activate_call(2, 300).unwrap();
    sim.apply(1, Command::AcceptCall { incident: 2 }).unwrap();
    sim.apply(1, Command::Decline { incident: 2 }).unwrap();
    sim.advance(400).unwrap();
    assert_eq!(sim.review().total_damage, 0);
    assert_eq!(sim.incidents()[&1].state, IncidentState::Pending);
}
#[test]
fn prompt_response_stops_damage_and_budget_caps_each_incident() {
    let mut sim = shift();
    sim.activate_call(1, 300).unwrap();
    sim.apply(1, Command::AcceptCall { incident: 1 }).unwrap();
    sim.apply(
        1,
        Command::Dispatch {
            unit: 3,
            incident: 1,
        },
    )
    .unwrap();
    sim.advance(90).unwrap();
    assert_eq!(sim.review().total_damage, 0);
    let mut untreated = shift();
    untreated.activate_call(5, 1000).unwrap();
    untreated.advance(1000).unwrap();
    assert_eq!(untreated.incidents()[&5].damage, 25_000);
}
#[test]
fn damage_replays_deterministically_and_corrupt_ledgers_are_rejected() {
    let mut a = shift();
    a.activate_call(1, 300).unwrap();
    let mut b = a.clone();
    a.advance(90).unwrap();
    for _ in 0..90 {
        b.advance(1).unwrap();
        b = Simulation::load_json(&b.save_json().unwrap()).unwrap();
    }
    assert_eq!(a, b);
    let mut corrupt: serde_json::Value = serde_json::from_str(&a.save_json().unwrap()).unwrap();
    corrupt["simulation"]["incidents"]["1"]["damage"] = 999.into();
    assert!(Simulation::load_json(&corrupt.to_string()).is_err());
    assert_eq!(a.review().damage_entries[0].first_at, 46);
    assert_eq!(a.review().damage_entries[0].damage, 22_500);
}
