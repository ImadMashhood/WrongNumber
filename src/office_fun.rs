//! Small, bounded host-authoritative office toy simulation, independent of dispatch truth.
use crate::{
    game::{Action, Phase, Session},
    room::Pose,
};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
const STEP: f32 = 1.0 / 120.0;
pub const LINKS: [(usize, usize); 10] = [
    (0, 1),
    (1, 2),
    (1, 3),
    (3, 4),
    (1, 5),
    (5, 6),
    (2, 7),
    (7, 8),
    (2, 9),
    (9, 10),
];
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Prop {
    pub id: u8,
    pub kind: u8,
    pub position: Vec3,
    pub velocity: Vec3,
    pub spin: f32,
    pub held: Option<u8>,
    pub thrower: Option<u8>,
    pub age: f32,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Doll {
    pub points: [Vec3; 11],
    previous: [Vec3; 11],
    lengths: [f32; 10],
    pub remaining: f32,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Playground {
    pub props: Vec<Prop>,
    pub dolls: BTreeMap<u8, Doll>,
    pub immunity: BTreeMap<u8, f32>,
    #[serde(skip)]
    accumulator: f32,
}
impl Default for Playground {
    fn default() -> Self {
        Self {
            props: [
                (0, Vec3::new(10.4, 0.38, 5.65)),
                (0, Vec3::new(-7.75, 0.38, 4.8)),
                (1, Vec3::new(11.5, 1.14, 1.2)),
                (1, Vec3::new(10.65, 1.16, -4.0)),
                (2, Vec3::new(12.8, 0.22, 5.8)),
                (2, Vec3::new(-7.5, 0.22, 5.3)),
            ]
            .into_iter()
            .enumerate()
            .map(|(i, (kind, position))| Prop {
                id: i as u8,
                kind,
                position,
                velocity: Vec3::ZERO,
                spin: 0.0,
                held: None,
                thrower: None,
                age: 0.0,
            })
            .collect(),
            dolls: BTreeMap::new(),
            immunity: BTreeMap::new(),
            accumulator: 0.0,
        }
    }
}
pub fn is_action(a: &Action) -> bool {
    matches!(a, Action::Pickup(_) | Action::ThrowProp | Action::DropProp)
}
pub fn forward(pose: Pose) -> Vec3 {
    Quat::from_euler(EulerRot::YXZ, pose.yaw, pose.pitch, 0.0) * Vec3::NEG_Z
}
fn hand(pose: Pose) -> Vec3 {
    Vec3::from_array(pose.position) + forward(pose) * 0.65 - Vec3::Y * 0.24
}
pub fn solid(p: Vec3, phase: Phase) -> bool {
    let in_rect = |x1, x2, z1, z2| p.x > x1 && p.x < x2 && p.z > z1 && p.z < z2;
    let room = in_rect(-8.8, 8.8, -6.8, 6.8)
        || in_rect(9.2, 13.8, -4.8, 6.8)
        || in_rect(8.4, 9.6, -1.1, 2.1)
        || (crate::world::review_unlocked(phase)
            && (in_rect(2.8, 8.8, 7.2, 13.8) || in_rect(5.4, 6.6, 6.5, 7.5)));
    crate::colliders::solid(p)
        || !room
        || p.y < 0.02
        || p.y > 3.95
        || (phase == Phase::Lobby && in_rect(8.75, 9.25, -1.2, 2.2))
        || (p.y < 0.92 && in_rect(-6.45, 6.45, -0.725, 0.725))
        || (p.y < 0.885 && in_rect(10.5, 12.5, 0.7, 1.7))
        || (p.y < 1.0 && in_rect(9.4, 12.6, -4.8, -3.7))
        || (p.y < 2.2 && in_rect(12.7, 13.8, -4.5, -3.45))
        || (p.y < 1.0 && in_rect(3.8, 7.6, 9.7, 11.7))
        || (p.y < 0.45 && in_rect(12.7, 13.7, 6.1, 6.7))
}
fn prop_solid(p: Vec3, kind: u8, phase: Phase) -> bool {
    let (radius, height) = match kind {
        0 => (0.15, 0.30),
        1 => (0.10, 0.075),
        _ => (0.15, 0.11),
    };
    if crate::colliders::overlaps(p, Vec3::new(radius, height, radius)) {
        return true;
    }
    // A small swept box keeps visible bodies above surfaces and away from walls.
    [-radius, radius].into_iter().any(|x| {
        [-radius, radius].into_iter().any(|z| {
            solid(p + Vec3::new(x, -height, z), phase) || solid(p + Vec3::new(x, height, z), phase)
        })
    })
}
fn clear(a: Vec3, b: Vec3, phase: Phase) -> bool {
    let steps = (a.distance(b) / 0.06).ceil() as usize;
    (1..=steps).all(|i| !solid(a.lerp(b, i as f32 / steps.max(1) as f32), phase))
}
impl Playground {
    pub fn held(&self, id: u8) -> Option<u8> {
        self.props.iter().find(|p| p.held == Some(id)).map(|p| p.id)
    }
    pub fn down(&self, id: u8) -> bool {
        self.dolls.contains_key(&id)
    }
    pub fn action(&mut self, id: u8, pose: Pose, a: &Action, phase: Phase) -> Result<(), String> {
        if self.down(id) {
            return Err("Getting back up…".into());
        }
        match a {
            Action::Pickup(n) => {
                if self.held(id).is_some() {
                    return Err("Already carrying an object".into());
                }
                let p = self
                    .props
                    .iter_mut()
                    .find(|p| p.id == *n)
                    .ok_or("Unknown prop")?;
                let origin = Vec3::from_array(pose.position);
                if p.held.is_some()
                    || origin.distance(p.position) > 2.2
                    || !clear(origin, p.position, phase)
                {
                    return Err("Move closer to an available object".into());
                }
                if forward(pose).dot((p.position - origin).normalize_or_zero()) < 0.8 {
                    return Err("Aim at the object".into());
                }
                p.held = Some(id);
                p.thrower = None;
                p.velocity = Vec3::ZERO;
            }
            Action::ThrowProp | Action::DropProp => {
                let p = self
                    .props
                    .iter_mut()
                    .find(|p| p.held == Some(id))
                    .ok_or("You are not holding anything")?;
                let origin = Vec3::from_array(pose.position);
                let target = hand(pose);
                if !clear(origin, target, phase) {
                    return Err("Move the object away from the wall".into());
                }
                p.position = target;
                p.held = None;
                p.age = 0.0;
                p.thrower = if matches!(a, Action::ThrowProp) {
                    Some(id)
                } else {
                    None
                };
                p.velocity = if matches!(a, Action::ThrowProp) {
                    forward(pose) * 10.0 + Vec3::Y * 1.8
                } else {
                    Vec3::ZERO
                };
            }
            _ => return Err("Not a prop action".into()),
        }
        Ok(())
    }
    pub fn tick(&mut self, seconds: f32, players: &[(u8, Pose)], phase: Phase) {
        if !seconds.is_finite() || seconds <= 0.0 {
            return;
        }
        self.accumulator += seconds.min(0.1);
        while self.accumulator >= STEP {
            self.accumulator -= STEP;
            self.step(players, phase);
        }
    }
    fn step(&mut self, players: &[(u8, Pose)], phase: Phase) {
        self.immunity.retain(|_, t| {
            *t -= STEP;
            *t > 0.0
        });
        self.dolls.retain(|id, d| {
            d.remaining -= STEP;
            if !players.iter().any(|(p, _)| p == id) || d.remaining <= 0.0 {
                return false;
            }
            for i in 0..11 {
                let old = d.points[i];
                let proposed = old + (old - d.previous[i]) * 0.985 - Vec3::Y * 9.8 * STEP * STEP;
                d.previous[i] = old;
                d.points[i] = if proposed.y < 0.12 || solid(proposed, phase) {
                    old
                } else {
                    proposed
                };
            }
            for _ in 0..6 {
                for (n, (a, b)) in LINKS.iter().copied().enumerate() {
                    let delta = d.points[b] - d.points[a];
                    let len = delta.length();
                    if len < 0.0001 {
                        continue;
                    }
                    let correction = delta * ((len - d.lengths[n]) / len) * 0.5;
                    let pa = d.points[a] + correction;
                    let pb = d.points[b] - correction;
                    if pa.y >= 0.12 && !solid(pa, phase) {
                        d.points[a] = pa;
                    }
                    if pb.y >= 0.12 && !solid(pb, phase) {
                        d.points[b] = pb;
                    }
                }
            }
            true
        });
        let mut hits = vec![];
        for p in &mut self.props {
            if let Some(id) = p.held {
                if let Some((_, pose)) = players.iter().find(|(n, _)| *n == id) {
                    let target = hand(*pose);
                    let origin = Vec3::from_array(pose.position);
                    p.position = if clear(origin, target, phase) {
                        target
                    } else {
                        origin
                    };
                    continue;
                }
                p.held = None;
                p.thrower = None;
                p.velocity = Vec3::ZERO;
            }
            p.age += STEP;
            p.velocity.y -= 9.8 * STEP;
            let old = p.position;
            for axis in 0..3 {
                let mut next = p.position;
                next[axis] += p.velocity[axis] * STEP;
                if prop_solid(next, p.kind, phase) {
                    if axis == 1 && p.velocity.length() < 0.5 {
                        p.spin = 0.0;
                    }
                    p.velocity[axis] *= -0.32;
                    p.velocity *= 0.86;
                } else {
                    p.position = next;
                }
            }
            p.spin += p.velocity.length() * STEP;
            if p.age > 4.0 {
                p.thrower = None;
            }
            if p.thrower.is_some() && p.velocity.length() > 2.5 && clear(old, p.position, phase) {
                for (id, pose) in players {
                    if Some(*id) == p.thrower
                        || self.immunity.contains_key(id)
                        || self.dolls.contains_key(id)
                    {
                        continue;
                    }
                    let center = Vec3::from_array(pose.position) - Vec3::Y * 0.5;
                    let delta = p.position - old;
                    let t = ((center - old).dot(delta) / delta.length_squared().max(0.0001))
                        .clamp(0.0, 1.0);
                    let closest = old + delta * t;
                    if (closest.x - center.x).hypot(closest.z - center.z) < 0.48
                        && (closest.y - center.y).abs() < 0.75
                    {
                        hits.push((*id, *pose, p.velocity));
                        p.thrower = None;
                        p.velocity *= -0.22;
                        break;
                    }
                }
            }
            if p.age > 25.0 && p.velocity.length() < 0.5 {
                // Settled props stay collectible; never spawn replacements during a shift.
                p.velocity = Vec3::ZERO;
            }
        }
        for (id, pose, impulse) in hits {
            if self.immunity.contains_key(&id) {
                continue;
            }
            self.immunity.insert(id, 6.0);
            self.dolls.insert(id, Doll::new(pose, impulse));
            for p in &mut self.props {
                if p.held == Some(id) {
                    p.held = None;
                    p.thrower = None;
                    p.velocity = impulse * 0.2;
                }
            }
        }
    }
}
impl Doll {
    pub(crate) fn new(pose: Pose, impulse: Vec3) -> Self {
        let origin = Vec3::new(pose.position[0], 0.0, pose.position[2]);
        let rot = Quat::from_rotation_y(pose.yaw);
        let local = [
            (0.0, 1.58, 0.0),
            (0.0, 1.32, 0.0),
            (0.0, 0.78, 0.0),
            (-0.33, 1.14, 0.0),
            (-0.45, 0.85, 0.0),
            (0.33, 1.14, 0.0),
            (0.45, 0.85, 0.0),
            (-0.14, 0.46, 0.0),
            (-0.14, 0.15, -0.08),
            (0.14, 0.46, 0.0),
            (0.14, 0.15, -0.08),
        ];
        let points = local.map(|(x, y, z)| origin + rot * Vec3::new(x, y, z));
        let lengths = LINKS.map(|(a, b)| points[a].distance(points[b]));
        let previous = std::array::from_fn(|i| {
            points[i] - (impulse * 0.18 + Vec3::new((i as f32 - 5.0) * 0.15, 0.5, 0.0)) * STEP
        });
        Self {
            points,
            previous,
            lengths,
            remaining: 2.6,
        }
    }
}
pub fn apply(game: &mut Session, id: u8, pose: Pose, a: &Action) -> Result<(), String> {
    if game.desk(id).seated.is_some() {
        return Err("Stand up before handling props".into());
    }
    game.fun.action(id, pose, a, game.phase)
}
#[cfg(test)]
mod tests {
    use super::*;
    fn pose(x: f32, z: f32) -> Pose {
        Pose {
            position: [x, 1.65, z],
            yaw: 0.0,
            pitch: 0.0,
        }
    }
    #[test]
    fn throw_hits_recovers_and_does_not_chain_stun() {
        let mut toys = Playground::default();
        let a = pose(0.0, 5.0);
        let b = pose(0.0, 3.0);
        toys.props[0].held = Some(1);
        toys.action(1, a, &Action::ThrowProp, Phase::Running)
            .unwrap();
        let players = [(1, a), (2, b)];
        for _ in 0..30 {
            toys.tick(1.0 / 120.0, &players, Phase::Running);
        }
        assert!(toys.down(2));
        assert!(!toys.down(1));
        let initial = toys.dolls[&2].points;
        for _ in 0..120 {
            toys.tick(1.0 / 120.0, &players, Phase::Running);
        }
        assert_ne!(initial, toys.dolls[&2].points);
        assert!(
            toys.dolls[&2]
                .points
                .iter()
                .all(|p| p.is_finite() && p.y >= 0.12)
        );
        for _ in 0..240 {
            toys.tick(1.0 / 120.0, &players, Phase::Running);
        }
        assert!(!toys.down(2));
        assert!(toys.immunity.contains_key(&2));
        toys.props[0].held = Some(1);
        toys.action(1, a, &Action::ThrowProp, Phase::Running)
            .unwrap();
        for _ in 0..30 {
            toys.tick(1.0 / 120.0, &players, Phase::Running);
        }
        assert!(!toys.down(2));
    }
    #[test]
    fn pickup_validates_range_aim_walls_and_ownership() {
        let mut toys = Playground::default();
        let p = pose(0.0, 5.0);
        toys.props[0].position = Vec3::new(0.0, 1.4, 4.0);
        assert!(
            toys.action(1, pose(4.0, 5.0), &Action::Pickup(0), Phase::Running)
                .is_err()
        );
        toys.action(1, p, &Action::Pickup(0), Phase::Running)
            .unwrap();
        assert!(
            toys.action(2, p, &Action::Pickup(0), Phase::Running)
                .is_err()
        );
        toys.tick(0.05, &[(2, p)], Phase::Running);
        assert_eq!(toys.held(1), None);
        toys.props[0].position = Vec3::new(8.5, 1.4, 0.5);
        let across = Pose {
            position: [9.8, 1.65, 0.5],
            yaw: std::f32::consts::FRAC_PI_2,
            pitch: 0.0,
        };
        assert!(
            toys.action(1, across, &Action::Pickup(0), Phase::Lobby)
                .is_err()
        );
    }
    #[test]
    fn serialization_and_long_simulation_are_bounded() {
        let mut toys = Playground::default();
        for _ in 0..3600 {
            toys.tick(1.0 / 60.0, &[], Phase::Running);
        }
        assert_eq!(toys.props.len(), 6);
        assert!(
            toys.props
                .iter()
                .all(|p| p.position.is_finite() && p.position.y >= 0.09)
        );
        let encoded = serde_json::to_string(&toys).unwrap();
        let mirror: Playground = serde_json::from_str(&encoded).unwrap();
        assert_eq!(toys.props[0].position, mirror.props[0].position);
    }
}
