use wrong_number_sim::{roads::*, *};
fn network() -> RoadNetwork {
    let nodes = vec![
        RoadNode { id: 0, x: 0, y: 0 },
        RoadNode { id: 1, x: 4, y: 0 },
        RoadNode { id: 2, x: 4, y: 4 },
        RoadNode { id: 3, x: 0, y: 4 },
    ];
    let roads = [
        (0, 1, vec![[0, 0], [1, 0], [2, 0], [3, 0], [4, 0]]),
        (1, 2, vec![[4, 0], [4, 1], [4, 2], [4, 3], [4, 4]]),
        (0, 3, vec![[0, 0], [0, 1], [0, 2], [0, 3], [0, 4]]),
        (3, 2, vec![[0, 4], [1, 4], [2, 4], [3, 4], [4, 4]]),
    ]
    .into_iter()
    .enumerate()
    .map(|(id, (from_node, to_node, points))| Road {
        id: id as u32,
        from_node,
        to_node,
        points,
        name: format!("Road {id}"),
        length_mm: 100_000,
        speed_limit_kmh: if id < 2 { 60 } else { 30 },
        one_way: false,
        closed: false,
    })
    .collect();
    RoadNetwork { nodes, roads }
}
#[test]
fn dijkstra_uses_speed_closures_and_unit_multiplier() {
    let mut n = network();
    n.validate().unwrap();
    let a = Position { x: 0, y: 0 };
    let b = Position { x: 4, y: 4 };
    let route = n.route(a, b, 1000, None).unwrap();
    assert!(
        route
            .points
            .iter()
            .any(|p| p.position == Position { x: 4, y: 0 })
    );
    assert!(n.valid_journey(&route, 1000));
    let fast = n.route(a, b, 2000, None).unwrap();
    assert!(fast.duration_ms() < route.duration_ms());
    n.roads[0].closed = true;
    let alternate = n.route(a, b, 1000, None).unwrap();
    assert!(
        alternate
            .points
            .iter()
            .any(|p| p.position == Position { x: 0, y: 4 })
    );
    n.roads[2].closed = true;
    assert_eq!(n.route(a, b, 1000, None), Err(Error::NoRoute));
}
#[test]
fn one_way_and_disconnected_destinations_are_not_bridged() {
    let mut n = network();
    n.roads.truncate(1);
    n.roads[0].one_way = true;
    assert!(
        n.route(Position { x: 0, y: 0 }, Position { x: 4, y: 0 }, 1000, None)
            .is_ok()
    );
    assert_eq!(
        n.route(Position { x: 4, y: 0 }, Position { x: 0, y: 0 }, 1000, None),
        Err(Error::NoRoute)
    );
    assert_eq!(
        n.route(Position { x: 0, y: 0 }, Position { x: 4, y: 4 }, 1000, None),
        Err(Error::NoRoute)
    );
}
#[test]
fn road_dispatch_cancel_and_reroute_keep_positions_on_real_edges_and_resume() {
    let mut sim = Simulation::new(42);
    sim.enable_roads(
        network(),
        [Position { x: 0, y: 0 }; 3],
        [Position { x: 4, y: 4 }; 6],
    )
    .unwrap();
    sim.apply(1, Command::AcceptCall { incident: 1 }).unwrap();
    sim.apply(
        1,
        Command::Dispatch {
            unit: 3,
            incident: 1,
        },
    )
    .unwrap();
    sim.advance(4).unwrap();
    let current = sim.units()[&3].position;
    assert_ne!(current, sim.units()[&3].station);
    sim.apply(1, Command::Cancel { unit: 3 }).unwrap();
    assert_eq!(sim.units()[&3].position, current);
    assert!(matches!(sim.units()[&3].state, UnitState::Returning { .. }));
    for _ in 0..30 {
        sim = Simulation::load_json(&sim.save_json().unwrap()).unwrap();
        sim.advance(1).unwrap();
    }
    assert_eq!(sim.units()[&3].state, UnitState::Available);
    assert_eq!(sim.units()[&3].position, sim.units()[&3].station);
}
#[test]
fn full_response_reservation_is_atomic_and_all_support_units_return() {
    let mut sim = Simulation::new(42);
    sim.apply(1, Command::AcceptCall { incident: 1 }).unwrap();
    sim.apply(1, Command::FullResponse { incident: 1 }).unwrap();
    assert!(
        sim.units()
            .values()
            .all(|u| u.state.assignment() == Some(1))
    );
    let before = sim.clone();
    sim.apply(1, Command::AcceptCall { incident: 5 }).unwrap();
    let answered = sim.clone();
    assert_eq!(
        sim.apply(1, Command::FullResponse { incident: 5 }),
        Err(Error::InsufficientResources)
    );
    assert_eq!(answered, sim);
    assert!(
        before
            .units()
            .values()
            .all(|u| u.state.assignment() == Some(1))
    );
    for _ in 0..100 {
        sim = Simulation::load_json(&sim.save_json().unwrap()).unwrap();
        sim.advance(1).unwrap();
    }
    assert!(
        sim.units()
            .values()
            .all(|u| u.state == UnitState::Available)
    );
    assert_eq!(sim.incidents()[&1].state, IncidentState::Resolved);
}

#[test]
fn import_normalizes_only_declared_adjacent_junction_pixels() {
    let mut raw = serde_json::json!({"nodes":[{"id":0,"x":0,"y":0},{"id":1,"x":4,"y":0}],"roads":[{"id":0,"from_node":0,"to_node":1,"points":[[1,0],[2,0],[3,0],[4,0]],"name":"Pilot Road","length_m":100.0,"speed_limit_kmh":40,"one_way":false,"closed":false}]});
    let n = RoadNetwork::from_import(&raw.to_string()).unwrap();
    assert_eq!(n.roads[0].points[0], [0, 0]);
    raw["roads"][0]["points"] = serde_json::json!([[2, 0], [3, 0], [4, 0]]);
    assert!(RoadNetwork::from_import(&raw.to_string()).is_err());
}
#[test]
fn rerouting_on_roads_preserves_position_and_takes_a_valid_return_branch() {
    let mut sim = Simulation::new(42);
    let mut locations = [Position { x: 4, y: 4 }; 6];
    locations[3] = Position { x: 0, y: 4 };
    sim.enable_roads(network(), [Position { x: 0, y: 0 }; 3], locations)
        .unwrap();
    sim.apply(1, Command::AcceptCall { incident: 1 }).unwrap();
    sim.apply(
        1,
        Command::Dispatch {
            unit: 3,
            incident: 1,
        },
    )
    .unwrap();
    sim.advance(4).unwrap();
    let before = sim.units()[&3].position;
    sim.apply(1, Command::AcceptCall { incident: 4 }).unwrap();
    sim.apply(
        1,
        Command::Reroute {
            unit: 3,
            incident: 4,
        },
    )
    .unwrap();
    assert_eq!(sim.units()[&3].position, before);
    for _ in 0..70 {
        sim = Simulation::load_json(&sim.save_json().unwrap()).unwrap();
        sim.advance(1).unwrap();
    }
    assert_eq!(sim.units()[&3].state, UnitState::Available);
    assert_eq!(sim.incidents()[&4].state, IncidentState::Resolved);
    assert_eq!(sim.incidents()[&1].state, IncidentState::Active);
}

#[test]
fn blast_closures_preserve_connectivity_replan_and_reopen_without_teleporting() {
    use wrong_number_sim::consequences::ScenarioRules;
    let mut s = Simulation::new_with_config(
        42,
        ShiftConfig {
            queued_calls: true,
            ..Default::default()
        },
    )
    .unwrap();
    s.enable_roads(
        network(),
        [Position { x: 0, y: 0 }; 3],
        [Position { x: 4, y: 4 }; 6],
    )
    .unwrap();
    s.apply_scenario_rules(
        ScenarioRules::from_json(include_str!("../../../assets/scenarios/m3_rules.json")).unwrap(),
    )
    .unwrap();
    s.activate_configured_call(1).unwrap();
    s.advance(115).unwrap();
    s.activate_configured_call(5).unwrap();
    s.apply(1, Command::AcceptCall { incident: 5 }).unwrap();
    s.apply(
        1,
        Command::Dispatch {
            unit: 2,
            incident: 5,
        },
    )
    .unwrap();
    s.advance(4).unwrap();
    let position = s.units()[&2].position;
    s.advance(1).unwrap();
    assert_eq!(s.units()[&2].position, position);
    let closed: Vec<_> = s
        .network()
        .unwrap()
        .roads
        .iter()
        .filter(|r| r.closed)
        .map(|r| r.id)
        .collect();
    assert_eq!(closed.len(), 1);
    assert!(
        s.network()
            .unwrap()
            .round_trip_connected(Position { x: 0, y: 0 })
    );
    assert!(
        !s.units()[&2]
            .journey
            .as_ref()
            .unwrap()
            .points
            .iter()
            .any(|p| p.road == Some(closed[0]))
    );
    let before = s.clone();
    let other = s
        .network()
        .unwrap()
        .roads
        .iter()
        .find(|r| !r.closed)
        .unwrap()
        .id;
    assert!(s.close_road(1, other, 120).is_err());
    assert_eq!(s, before);
    s = Simulation::load_json(&s.save_json().unwrap()).unwrap();
    s.advance(120).unwrap();
    assert!(s.network().unwrap().roads.iter().all(|r| !r.closed));
    assert!(
        s.events()
            .iter()
            .any(|e| matches!(e.kind, EventKind::RoadReopened { .. }))
    );
    Simulation::load_json(&s.save_json().unwrap()).unwrap();
}
