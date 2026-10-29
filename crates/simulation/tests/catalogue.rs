use wrong_number_sim::{catalogue::*, *};
fn setup() -> Simulation {
    let mut s = Simulation::new_with_config(
        42,
        ShiftConfig {
            queued_calls: true,
            ..Default::default()
        },
    )
    .unwrap();
    s.apply_scenario_rules(
        consequences::ScenarioRules::from_json(include_str!(
            "../../../assets/scenarios/m3_rules.json"
        ))
        .unwrap(),
    )
    .unwrap();
    s.enable_catalogue(vec![
        Location {
            position: Position { x: 4, y: 7 },
            address: "47 Pilot Road".into(),
        },
        Location {
            position: Position { x: 9, y: 12 },
            address: "912 Harbour Street".into(),
        },
    ])
    .unwrap();
    s
}
#[test]
fn generated_truth_distinguishes_false_reports_from_genuine_minor_complaints() {
    let mut s = setup();
    for _ in 1..90 {
        s.queue_catalogue_call().unwrap();
    }
    let all: Vec<_> = s.incidents().values().collect();
    assert_eq!(
        all.iter()
            .filter(|i| i.case_brief.as_ref().unwrap().false_report)
            .count(),
        9
    );
    assert!(
        all.iter()
            .any(|i| !i.case_brief.as_ref().unwrap().false_report && i.truth == Truth::Benign)
    );
    assert!(
        all.iter()
            .any(|i| i.case_brief.as_ref().unwrap().severity == Severity::Critical)
    );
    for i in all {
        if i.case_brief.as_ref().unwrap().false_report {
            assert_eq!(i.truth, Truth::Benign);
            assert_eq!(i.consequence, consequences::ConsequenceModel::default());
        }
        assert_eq!(i.facts.len(), 3);
        assert!(i.caller_name.len() > 3);
    }
}
#[test]
fn caller_private_brief_stays_off_public_wire_and_in_authority_saves() {
    let mut s = setup();
    s.activate_configured_call(1).unwrap();
    s.apply(1, Command::AcceptCall { incident: 1 }).unwrap();
    let public = serde_json::to_string(&s.public_snapshot()).unwrap();
    assert!(
        !public.contains("false_report")
            && !public.contains("caller_private")
            && !public.contains("catalogue_id")
            && !public.contains("psychology")
            && !public.contains("trust_required")
    );
    let resumed = Simulation::load_json(&s.save_json().unwrap()).unwrap();
    assert_eq!(resumed, s);
    let mirror = Simulation::from_public_snapshot(s.public_snapshot()).unwrap();
    assert_eq!(mirror.public_snapshot(), s.public_snapshot());
}
#[test]
fn catalogue_queue_and_save_are_bounded_and_reproducible() {
    let mut a = setup();
    let mut b = Simulation::load_json(&a.save_json().unwrap()).unwrap();
    for _ in 1..96 {
        assert_eq!(
            a.queue_catalogue_call().unwrap(),
            b.queue_catalogue_call().unwrap()
        );
        assert_eq!(a, b);
    }
    assert_eq!(a.queue_catalogue_call(), Err(Error::InvalidTransition));
}
