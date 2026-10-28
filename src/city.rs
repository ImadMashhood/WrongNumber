//! Training placements over the imported map; road routing is a separate boundary.
use wrong_number_sim::Position;
pub const STATIONS: [Position; 3] = [
    Position { x: 9, y: 7 },
    Position { x: 6, y: 10 },
    Position { x: 11, y: 4 },
];
pub const CALLS: [Position; 6] = [
    Position { x: 9, y: 8 },
    Position { x: 5, y: 5 },
    Position { x: 6, y: 10 },
    Position { x: 5, y: 9 },
    Position { x: 7, y: 11 },
    Position { x: 11, y: 8 },
];
pub fn map_percent(position: Position) -> (f32, f32) {
    (
        position.x as f32 / 16.0 * 100.0,
        position.y as f32 / 16.0 * 100.0,
    )
}
