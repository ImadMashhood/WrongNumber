//! Directed, speed-weighted Dijkstra routes over the supplied road polylines.
use crate::{Error, Position};
use serde::{Deserialize, Serialize};
use std::cmp::Reverse;
use std::collections::{BTreeMap, BinaryHeap};
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoadNode {
    pub id: u32,
    pub x: i32,
    pub y: i32,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Road {
    pub id: u32,
    pub from_node: u32,
    pub to_node: u32,
    pub points: Vec<[i32; 2]>,
    pub name: String,
    pub length_mm: u32,
    pub speed_limit_kmh: u32,
    pub one_way: bool,
    pub closed: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoadNetwork {
    pub nodes: Vec<RoadNode>,
    pub roads: Vec<Road>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Waypoint {
    pub position: Position,
    pub at_ms: u32,
    pub road: Option<u32>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Journey {
    pub points: Vec<Waypoint>,
}
impl Journey {
    pub fn duration_ms(&self) -> u32 {
        self.points.last().map_or(0, |p| p.at_ms)
    }
    pub fn seconds(&self) -> u32 {
        self.duration_ms().div_ceil(1000)
    }
    pub fn road_at(&self, elapsed: u32) -> Option<u32> {
        self.points
            .iter()
            .find(|p| p.at_ms >= elapsed.saturating_mul(1000))
            .and_then(|p| p.road)
    }
    /// Expiry/escalation runs before this tick's movement. Select the road at
    /// the timestamp that actually produced the stored position.
    pub fn source_road(&self, position: Position, elapsed: u32) -> Option<u32> {
        let at = if self.position(elapsed) == position {
            elapsed
        } else {
            elapsed.saturating_sub(1)
        };
        self.road_at(at)
    }
    pub fn position(&self, elapsed: u32) -> Position {
        let ms = elapsed.saturating_mul(1000);
        for pair in self.points.windows(2) {
            let [a, b] = pair else { unreachable!() };
            if ms <= b.at_ms {
                let progress =
                    f64::from(ms.saturating_sub(a.at_ms)) / f64::from((b.at_ms - a.at_ms).max(1));
                return Position {
                    x: (f64::from(a.position.x) + f64::from(b.position.x - a.position.x) * progress)
                        .round() as i32,
                    y: (f64::from(a.position.y) + f64::from(b.position.y - a.position.y) * progress)
                        .round() as i32,
                };
            }
        }
        self.points
            .last()
            .expect("Nonempty validated route")
            .position
    }
}
#[derive(Deserialize)]
struct Imported {
    nodes: Vec<RoadNode>,
    roads: Vec<ImportedRoad>,
}
#[derive(Deserialize)]
struct ImportedRoad {
    id: u32,
    from_node: u32,
    to_node: u32,
    points: Vec<[i32; 2]>,
    name: String,
    length_m: f64,
    speed_limit_kmh: u32,
    one_way: bool,
    closed: bool,
}
type Edge = (usize, u32, u32); // destination, milliseconds, road id
struct Graph {
    positions: Vec<Position>,
    edges: Vec<Vec<Edge>>,
    endpoint_ids: BTreeMap<u32, usize>,
    members: Vec<Vec<u32>>,
}
impl RoadNetwork {
    pub fn from_import(json: &str) -> Result<Self, Error> {
        let raw: Imported =
            serde_json::from_str(json).map_err(|e| Error::InvalidSave(e.to_string()))?;
        let endpoints: BTreeMap<_, _> = raw.nodes.iter().map(|n| (n.id, [n.x, n.y])).collect();
        let roads = raw
            .roads
            .into_iter()
            .map(|mut r| {
                if !r.length_m.is_finite() || r.length_m <= 0.0 || r.length_m > 1_000_000.0 {
                    return Err(Error::NoRoute);
                }
                // The tracer merges adjacent junction pixels into a declared node.
                // Join only that declared endpoint, with at most one pixel per axis.
                for (id, front) in [(r.from_node, true), (r.to_node, false)] {
                    let endpoint = *endpoints.get(&id).ok_or(Error::NoRoute)?;
                    let point = *if front {
                        r.points.first()
                    } else {
                        r.points.last()
                    }
                    .ok_or(Error::NoRoute)?;
                    if point != endpoint {
                        if point[0].abs_diff(endpoint[0]) > 1 || point[1].abs_diff(endpoint[1]) > 1
                        {
                            return Err(Error::NoRoute);
                        }
                        if front {
                            r.points.insert(0, endpoint);
                        } else {
                            r.points.push(endpoint);
                        }
                    }
                }
                Ok(Road {
                    id: r.id,
                    from_node: r.from_node,
                    to_node: r.to_node,
                    points: r.points,
                    name: r.name,
                    length_mm: (r.length_m * 1000.0).round() as u32,
                    speed_limit_kmh: r.speed_limit_kmh,
                    one_way: r.one_way,
                    closed: r.closed,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let network = Self {
            nodes: raw.nodes,
            roads,
        };
        network.validate()?;
        Ok(network)
    }
    pub fn connected_component(&self, start: u32) -> Self {
        let mut seen = std::collections::BTreeSet::from([start]);
        let mut stack = vec![start];
        while let Some(n) = stack.pop() {
            for r in self
                .roads
                .iter()
                .filter(|r| !r.closed && (r.from_node == n || r.to_node == n))
            {
                let other = if r.from_node == n {
                    r.to_node
                } else {
                    r.from_node
                };
                if seen.insert(other) {
                    stack.push(other);
                }
            }
        }
        Self {
            nodes: self
                .nodes
                .iter()
                .filter(|n| seen.contains(&n.id))
                .cloned()
                .collect(),
            roads: self
                .roads
                .iter()
                .filter(|r| seen.contains(&r.from_node) && seen.contains(&r.to_node))
                .cloned()
                .collect(),
        }
    }
    pub fn round_trip_connected(&self, start: Position) -> bool {
        let Some(root) = self
            .nodes
            .iter()
            .find(|n| n.x == start.x && n.y == start.y)
            .map(|n| n.id)
        else {
            return false;
        };
        for reverse in [false, true] {
            let mut seen = std::collections::BTreeSet::from([root]);
            let mut q = vec![root];
            while let Some(node) = q.pop() {
                for r in self.roads.iter().filter(|r| !r.closed) {
                    let (a, b) = if reverse {
                        (r.to_node, r.from_node)
                    } else {
                        (r.from_node, r.to_node)
                    };
                    if a == node && seen.insert(b) {
                        q.push(b);
                    }
                    if !r.one_way && b == node && seen.insert(a) {
                        q.push(a);
                    }
                }
            }
            if seen.len() != self.nodes.len() {
                return false;
            }
        }
        true
    }
    pub fn node_position(&self, id: u32) -> Option<Position> {
        self.nodes
            .iter()
            .find(|n| n.id == id)
            .map(|n| Position { x: n.x, y: n.y })
    }
    pub fn validate(&self) -> Result<(), Error> {
        let invalid = || Error::InvalidSave("Invalid road network".into());
        if self.nodes.is_empty()
            || self.nodes.len() > 10_000
            || self.roads.len() > 20_000
            || self.roads.iter().map(|r| r.points.len()).sum::<usize>() > 100_000
        {
            return Err(invalid());
        }
        let nodes: BTreeMap<_, _> = self.nodes.iter().map(|n| (n.id, [n.x, n.y])).collect();
        let mut ids = std::collections::BTreeSet::new();
        if nodes.len() != self.nodes.len()
            || nodes.values().flatten().any(|v| v.abs_diff(0) > 10_000)
        {
            return Err(invalid());
        }
        for r in &self.roads {
            if !ids.insert(r.id)
                || r.points.len() < 2
                || r.length_mm == 0
                || r.speed_limit_kmh == 0
                || r.speed_limit_kmh > 300
                || r.points.first() != nodes.get(&r.from_node)
                || r.points.last() != nodes.get(&r.to_node)
                || r.points.iter().flatten().any(|v| v.abs_diff(0) > 10_000)
                || r.points.windows(2).any(|p| p[0] == p[1])
            {
                return Err(invalid());
            }
        }
        Ok(())
    }
    fn graph(&self, speed: u32, closed_edges: bool) -> Graph {
        let mut g = Graph {
            positions: self
                .nodes
                .iter()
                .map(|n| Position { x: n.x, y: n.y })
                .collect(),
            edges: vec![vec![]; self.nodes.len()],
            endpoint_ids: self
                .nodes
                .iter()
                .enumerate()
                .map(|(i, n)| (n.id, i))
                .collect(),
            members: vec![vec![]; self.nodes.len()],
        };
        for road in &self.roads {
            let mut vertices = vec![g.endpoint_ids[&road.from_node]];
            for p in road.points.iter().skip(1).take(road.points.len() - 2) {
                vertices.push(g.positions.len());
                g.positions.push(Position { x: p[0], y: p[1] });
                g.edges.push(vec![]);
                g.members.push(vec![road.id]);
            }
            vertices.push(g.endpoint_ids[&road.to_node]);
            g.members[vertices[0]].push(road.id);
            g.members[*vertices.last().unwrap()].push(road.id);
            let lengths: Vec<f64> = road
                .points
                .windows(2)
                .map(|p| f64::from(p[1][0] - p[0][0]).hypot(f64::from(p[1][1] - p[0][1])))
                .collect();
            let total: f64 = lengths.iter().sum();
            let duration = f64::from(road.length_mm) * 3600.0
                / f64::from(road.speed_limit_kmh)
                / f64::from(speed);
            for (pair, len) in vertices.windows(2).zip(lengths) {
                let cost = (duration * len / total).ceil().max(1.0) as u32;
                if !road.closed || closed_edges {
                    g.edges[pair[0]].push((pair[1], cost, road.id));
                    if !road.one_way {
                        g.edges[pair[1]].push((pair[0], cost, road.id));
                    }
                }
            }
        }
        g
    }
    pub fn route(
        &self,
        from: Position,
        to: Position,
        speed: u32,
        source_road: Option<u32>,
    ) -> Result<Journey, Error> {
        if !(100..=10_000).contains(&speed) {
            return Err(Error::NoRoute);
        }
        let g = self.graph(speed, false);
        let mut distance = vec![u32::MAX; g.positions.len()];
        let mut prev = vec![None; g.positions.len()];
        let mut heap = BinaryHeap::new();
        for (i, p) in g.positions.iter().enumerate() {
            if *p == from && source_road.is_none_or(|r| g.members[i].contains(&r)) {
                distance[i] = 0;
                heap.push(Reverse((0_u32, i)));
            }
        }
        let targets: std::collections::BTreeSet<_> = self
            .nodes
            .iter()
            .filter(|n| n.x == to.x && n.y == to.y)
            .map(|n| g.endpoint_ids[&n.id])
            .collect();
        while let Some(Reverse((cost, node))) = heap.pop() {
            if cost != distance[node] {
                continue;
            }
            if targets.contains(&node) {
                let mut result = vec![];
                let mut current = node;
                while let Some((parent, road)) = prev[current] {
                    result.push(Waypoint {
                        position: g.positions[current],
                        at_ms: distance[current],
                        road: Some(road),
                    });
                    current = parent;
                }
                result.push(Waypoint {
                    position: from,
                    at_ms: 0,
                    road: source_road,
                });
                result.reverse();
                return Ok(Journey { points: result });
            }
            for &(next, weight, road) in &g.edges[node] {
                let next_cost = cost.saturating_add(weight);
                if next_cost < distance[next] {
                    distance[next] = next_cost;
                    prev[next] = Some((node, road));
                    heap.push(Reverse((next_cost, next)));
                }
            }
        }
        Err(Error::NoRoute)
    }
    pub fn valid_journey(&self, journey: &Journey, speed: u32) -> bool {
        if journey.points.is_empty()
            || journey.points.len() > 100_000
            || journey.points[0].at_ms != 0
        {
            return false;
        }
        // Validate actual directed edges and timings, including routes already in flight.
        let g = self.graph(speed, true);
        let mut candidates: Vec<_> = g
            .positions
            .iter()
            .enumerate()
            .filter(|(i, p)| {
                **p == journey.points[0].position
                    && journey.points[0]
                        .road
                        .is_none_or(|r| g.members[*i].contains(&r))
            })
            .map(|(i, _)| i)
            .collect();
        for pair in journey.points.windows(2) {
            if pair[1].at_ms <= pair[0].at_ms {
                return false;
            }
            let mut next = vec![];
            for node in candidates {
                for &(target, ms, road) in &g.edges[node] {
                    if g.positions[target] == pair[1].position
                        && ms == pair[1].at_ms - pair[0].at_ms
                        && Some(road) == pair[1].road
                    {
                        next.push(target);
                    }
                }
            }
            next.sort_unstable();
            next.dedup();
            if next.is_empty() {
                return false;
            }
            candidates = next;
        }
        !candidates.is_empty()
    }
}
