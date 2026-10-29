//! Real imported road coordinates; pilot placements stay inside one connected district.
use serde::Deserialize;
use std::sync::OnceLock;
use wrong_number_sim::{Position, Simulation, roads::RoadNetwork};
#[derive(Deserialize)]
pub struct Config {
    pub unit_speed_milli: [u32; 3],
    pub reticle_colors: ReticleColors,
    pub station_nodes: [u32; 3],
    pub incident_nodes: [u32; 6],
    pub map_width_px: u32,
    pub map_height_px: u32,
    pub recovery_seconds: u32,
    pub radio_recovery_seconds: u32,
    pub initial_delay_seconds: [u32; 2],
    pub next_delay_seconds: [u32; 2],
}
#[derive(Deserialize)]
pub struct ReticleColors {
    pub neutral: [f32; 3],
    pub ready: [f32; 3],
    pub locked: [f32; 3],
    pub conditional: [f32; 3],
}
pub fn config() -> &'static Config {
    static CONFIG: OnceLock<Config> = OnceLock::new();
    CONFIG.get_or_init(|| {
        serde_json::from_str(include_str!("../assets/city/gameplay_config.json"))
            .expect("Valid authored gameplay configuration")
    })
}
pub fn scenario_rules() -> &'static wrong_number_sim::consequences::ScenarioRules {
    static RULES: OnceLock<wrong_number_sim::consequences::ScenarioRules> = OnceLock::new();
    RULES.get_or_init(|| {
        wrong_number_sim::consequences::ScenarioRules::from_json(include_str!(
            "../assets/scenarios/m3_rules.json"
        ))
        .expect("Valid authored M3 rules")
    })
}
pub fn network() -> &'static RoadNetwork {
    static NETWORK: OnceLock<RoadNetwork> = OnceLock::new();
    NETWORK.get_or_init(|| {
        RoadNetwork::from_import(include_str!("../assets/city/road_graph.json"))
            .expect("Valid supplied road graph")
    })
}
pub fn install(sim: &mut Simulation) {
    let config = config();
    let roads = network();
    sim.enable_roads(
        roads.connected_component(config.station_nodes[0]),
        config
            .station_nodes
            .map(|n| roads.node_position(n).unwrap()),
        config
            .incident_nodes
            .map(|n| roads.node_position(n).unwrap()),
    )
    .expect("All pilot placements must have real round-trip routes");
    sim.set_response_speeds(config.unit_speed_milli)
        .expect("Valid configured unit speeds");
}
pub fn map_percent(position: Position) -> (f32, f32) {
    (
        position.x as f32 / config().map_width_px as f32 * 100.0,
        position.y as f32 / config().map_height_px as f32 * 100.0,
    )
}
pub fn address(position: Position) -> String {
    let road = network()
        .roads
        .iter()
        .filter(|r| r.points.contains(&[position.x, position.y]))
        .min_by_key(|r| r.id);
    road.map_or_else(
        || "Reported location pending".into(),
        |r| {
            // A stable pseudo-random building number for each physical location.
            // Keep graph IDs and coordinates internal; imported road data stays intact.
            let hash = (position.x as u32).wrapping_mul(73856093)
                ^ (position.y as u32).wrapping_mul(19349663)
                ^ r.id.wrapping_mul(83492791);
            let mut words: Vec<_> = r.name.split_whitespace().collect();
            while words
                .last()
                .is_some_and(|w| w.chars().all(|c| c.is_ascii_digit()))
            {
                words.pop();
            }
            format!("{} {}", 10 + hash % 9990, words.join(" "))
        },
    )
}

/// Only real nodes with an actual outbound and return route to every station.
pub fn catalogue_locations() -> &'static Vec<wrong_number_sim::catalogue::Location> {
    static LOCATIONS: OnceLock<Vec<wrong_number_sim::catalogue::Location>> = OnceLock::new();
    LOCATIONS.get_or_init(|| {
        let n = network().connected_component(config().station_nodes[0]);
        n.nodes
            .iter()
            .filter_map(|node| {
                let position = Position {
                    x: node.x,
                    y: node.y,
                };
                config()
                    .station_nodes
                    .iter()
                    .all(|id| {
                        let station = n.node_position(*id).unwrap();
                        n.route(station, position, 1000, None).is_ok()
                            && n.route(position, station, 1000, None).is_ok()
                    })
                    .then(|| wrong_number_sim::catalogue::Location {
                        position,
                        address: address(position),
                    })
            })
            .collect()
    })
}

#[cfg(test)]
mod address_tests {
    use super::*;
    #[test]
    fn every_catalogue_location_has_a_stable_number_and_street() {
        let locations = catalogue_locations();
        assert!(!locations.is_empty());
        for location in locations {
            let address = address(location.position);
            assert_eq!(address, location.address);
            let (number, street) = address.split_once(' ').unwrap();
            assert!((2..=4).contains(&number.len()));
            assert!((10..=9999).contains(&number.parse::<u32>().unwrap()));
            assert!(!street.contains("junction") && !street.contains(','));
            assert!(street.chars().last().unwrap().is_alphabetic());
        }
    }
}
