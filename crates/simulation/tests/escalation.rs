use wrong_number_sim::{consequences::ScenarioRules, *};
fn rules() -> ScenarioRules {
    ScenarioRules::from_json(include_str!("../../../assets/scenarios/m3_rules.json")).unwrap()
}
fn shift() -> Simulation {
    let mut s = Simulation::new_with_config(
        42,
        ShiftConfig {
            queued_calls: true,
            ..Default::default()
        },
    )
    .unwrap();
    s.apply_scenario_rules(rules()).unwrap();
    s
}
#[test]
fn fire_escalates_once_with_linked_pending_medical_followup_and_replays() {
    let mut a = shift();
    a.activate_configured_call(1).unwrap();
    a.advance(119).unwrap();
    assert!(a.review().escalations.is_empty());
    let mut b = a.clone();
    a.advance(1).unwrap();
    b.advance(1).unwrap();
    b = Simulation::load_json(&b.save_json().unwrap()).unwrap();
    assert_eq!(a, b);
    let event = &a.review().escalations[0];
    assert_eq!(event.at, 120);
    assert_eq!(event.incident, 1);
    let child = event.secondary.unwrap();
    assert_eq!(a.incidents()[&child].parent, Some(1));
    assert_eq!(a.incidents()[&child].required, Capability::Medical);
    assert_eq!(a.incidents()[&child].state, IncidentState::Pending);
    assert!(a.call_view(child).is_err());
    a.advance(180).unwrap();
    assert_eq!(a.review().escalations.len(), 1);
    assert_eq!(a.incidents()[&1].damage, 137_500);
    assert_eq!(Simulation::load_json(&a.save_json().unwrap()).unwrap(), a);
}
#[test]
fn prank_truth_and_timely_response_cannot_trigger_an_explosion() {
    let mut benign = shift();
    benign.activate_configured_call(4).unwrap();
    benign.advance(300).unwrap();
    assert!(benign.review().escalations.is_empty());
    assert_eq!(benign.review().total_damage, 0);
    let mut handled = shift();
    handled.activate_configured_call(1).unwrap();
    handled
        .apply(1, Command::AcceptCall { incident: 1 })
        .unwrap();
    handled
        .apply(
            1,
            Command::Dispatch {
                unit: 3,
                incident: 1,
            },
        )
        .unwrap();
    handled.advance(150).unwrap();
    assert!(handled.review().escalations.is_empty());
}
#[test]
fn escalation_cooldown_global_cap_and_child_lineage_are_enforced() {
    let mut s = shift();
    let location = s.incidents()[&1].location;
    let mut ids = vec![1];
    for _ in 0..3 {
        ids.push(s.queue_scenario(1, location).unwrap());
    }
    for id in ids {
        s.activate_call(id, 600).unwrap();
    }
    s.advance(119).unwrap();
    for _ in 0..200 {
        s.advance(1).unwrap();
        s = Simulation::load_json(&s.save_json().unwrap()).unwrap();
    }
    let events = s.review().escalations;
    assert_eq!(events.len(), 2);
    assert_eq!(events[1].at - events[0].at, 90);
    assert_eq!(
        s.incidents()
            .values()
            .filter(|i| i.parent.is_some())
            .count(),
        2
    );
    let mut corrupt: serde_json::Value = serde_json::from_str(&s.save_json().unwrap()).unwrap();
    let child = events[0].secondary.unwrap().to_string();
    corrupt["simulation"]["incidents"][child]["parent"] = 999.into();
    assert!(Simulation::load_json(&corrupt.to_string()).is_err());
}
#[test]
fn invalid_rules_are_atomic_and_deadlines_are_data_driven() {
    let mut s = Simulation::new_with_config(
        42,
        ShiftConfig {
            queued_calls: true,
            ..Default::default()
        },
    )
    .unwrap();
    let before = s.clone();
    let mut invalid = rules();
    invalid.escalation.max_events = 0;
    assert!(s.apply_scenario_rules(invalid).is_err());
    assert_eq!(s, before);
    let mut r = rules();
    r.profiles
        .iter_mut()
        .find(|p| p.capability == Capability::Fire)
        .unwrap()
        .response_seconds = 200;
    s.apply_scenario_rules(r).unwrap();
    s.advance(20).unwrap();
    s.activate_configured_call(1).unwrap();
    assert_eq!(s.incidents()[&1].deadline, 220);
}
#[test]
fn director_enforces_active_and_total_budgets_and_slows_under_pressure() {
    let mut s = shift();
    let p = rules().director;
    for id in 1..=4 {
        s.activate_configured_call(id).unwrap();
    }
    assert!(!p.can_deliver(&s, 1, false));
    let crowded = p.delay_range(&s, 60, [1, 15]);
    assert!(crowded[0] > 1 && crowded[1] <= 15);
    s.apply(1, Command::AcceptCall { incident: 2 }).unwrap();
    s.apply(1, Command::Decline { incident: 2 }).unwrap();
    assert!(p.can_deliver(&s, 1, false));
    let mut budget = shift();
    let location = budget.incidents()[&2].location;
    for n in 0..32 {
        let id = if n == 0 {
            2
        } else {
            budget.queue_scenario(2, location).unwrap()
        };
        budget.activate_configured_call(id).unwrap();
        budget
            .apply(1, Command::AcceptCall { incident: id })
            .unwrap();
        budget.apply(1, Command::Decline { incident: id }).unwrap();
    }
    assert!(!p.can_deliver(&budget, 1, false));
    assert!(p.can_deliver(&budget, 1, true));
    assert!(p.active_limit(20) <= 6);
    assert!(p.call_budget(20) <= 64);
    assert_eq!(p.template(&budget, 2, 987), p.template(&budget, 2, 987));
}

#[test]
fn explicitly_enabling_rules_also_updates_immediately_ringing_shift_deadlines() {
    let mut s = Simulation::new(42);
    s.apply_scenario_rules(rules()).unwrap();
    assert_eq!(s.incidents()[&1].deadline, 300);
    s.advance(120).unwrap();
    assert_eq!(s.review().escalations.len(), 1);
    Simulation::load_json(&s.save_json().unwrap()).unwrap();
}
