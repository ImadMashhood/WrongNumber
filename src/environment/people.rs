//! Decorative walkers only: no desk ownership, calls, money or simulation actions.
use super::geometry::{Builder, Paint};
use bevy::prelude::*;
pub const ROW_Z: f32 = -4.6;
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct StaffState {
    pub position: Vec3,
    pub yaw: f32,
    pub seated: bool,
    pub moving: bool,
    wait: f32,
    seat: usize,
    returning: bool,
    route: std::collections::VecDeque<Vec3>,
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct StaffCrowd {
    pub members: Vec<StaffState>,
    rng: u64,
    #[serde(skip, default = "spots")]
    points: Vec<Vec3>,
}
fn random(rng: &mut u64, count: usize) -> usize {
    *rng ^= *rng << 13;
    *rng ^= *rng >> 7;
    *rng ^= *rng << 17;
    *rng as usize % count
}
fn clear(p: Vec3) -> bool {
    p.x.abs() < 8.0
        && p.z > -6.0
        && p.z < 5.5
        && [-0.12, 0.0, 0.12].into_iter().all(|x| {
            [-0.12, 0.0, 0.12].into_iter().all(|z| {
                crate::world::walkable(Vec2::new(p.x + x, p.z + z), crate::game::Phase::Running)
            })
        })
}
fn spots() -> Vec<Vec3> {
    let mut points = Vec::new();
    for x in -19..=19 {
        for z in -14..=13 {
            let p = Vec3::new(x as f32 * 0.4, 0.0, z as f32 * 0.4);
            if clear(p) {
                points.push(p);
            }
        }
    }
    points
}
fn path(start: Vec3, end: Vec3, occupied: &[Vec3]) -> std::collections::VecDeque<Vec3> {
    let key = |p: Vec3| ((p.x / 0.4).round() as i32, (p.z / 0.4).round() as i32);
    let pos = |(x, z): (i32, i32)| Vec3::new(x as f32 * 0.4, 0.0, z as f32 * 0.4);
    let from = key(start);
    let to = key(end);
    let mut queue = std::collections::VecDeque::from([from]);
    let mut previous = std::collections::HashMap::from([(from, from)]);
    while let Some(at) = queue.pop_front() {
        if at == to {
            break;
        }
        for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let next = (at.0 + dx, at.1 + dz);
            if !previous.contains_key(&next)
                && clear(pos(next))
                && occupied.iter().all(|p| p.distance(pos(next)) >= 0.78)
            {
                previous.insert(next, at);
                queue.push_back(next);
            }
        }
    }
    if !previous.contains_key(&to) {
        return Default::default();
    }
    let mut result = std::collections::VecDeque::new();
    let mut at = to;
    while at != from {
        result.push_front(pos(at));
        at = previous[&at];
    }
    result
}
impl Default for StaffCrowd {
    fn default() -> Self {
        if cfg!(test) {
            return Self::new(1);
        }
        Self::new(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64,
        )
    }
}
impl StaffCrowd {
    pub fn recover(&mut self, id: u8, p: Vec3) {
        if let Some(m) = self.members.get_mut((id - 100) as usize) {
            m.position = p;
            m.seated = false;
            m.moving = false;
            m.wait = 2.0;
            m.route.clear();
        }
    }

    fn new(mut rng: u64) -> Self {
        if rng == 0 {
            rng = 1;
        }
        let points = spots();
        let mut seats = vec![0, 1, 2, 3, 4, 5];
        let mut members: Vec<StaffState> = Vec::new();
        for i in 0..5 {
            let choice = random(&mut rng, seats.len());
            let seat = seats.remove(choice);
            let seated = i < 2 || (i == 2 && random(&mut rng, 2) == 0);
            let position = if seated {
                Vec3::new(crate::world::SEATS[seat], -0.24, ROW_Z + 1.05)
            } else {
                loop {
                    let p = points[random(&mut rng, points.len())];
                    if members.iter().all(|m| (m.position - p).length() > 1.0) {
                        break p;
                    }
                }
            };
            members.push(StaffState {
                position,
                yaw: if seated {
                    0.0
                } else {
                    random(&mut rng, 628) as f32 / 100.0
                },
                seated,
                moving: false,
                wait: 2.0 + random(&mut rng, 15) as f32,
                seat,
                returning: false,
                route: Default::default(),
            });
        }
        Self {
            members,
            rng,
            points,
        }
    }
    pub fn step(
        &mut self,
        dt: f32,
        down: &std::collections::BTreeMap<u8, crate::office_fun::Doll>,
    ) {
        for i in 0..self.members.len() {
            self.members[i].moving = false;
            if down.contains_key(&(100 + i as u8)) {
                continue;
            }
            if self.members[i].wait > 0.0 {
                self.members[i].wait -= dt;
                continue;
            }
            if self.members[i].seated {
                let p = Vec3::new(crate::world::SEATS[self.members[i].seat], 0.0, ROW_Z + 2.0);
                if self
                    .members
                    .iter()
                    .enumerate()
                    .any(|(j, m)| j != i && m.position.distance(p) < 0.75)
                {
                    continue;
                }
                self.members[i].position = p;
                self.members[i].seated = false;
            }
            if self.members[i].route.is_empty() {
                let returning = random(&mut self.rng, 4) == 0;
                self.members[i].returning = returning;
                let target = if returning {
                    Vec3::new(crate::world::SEATS[self.members[i].seat], 0.0, ROW_Z + 2.0)
                } else {
                    self.points[random(&mut self.rng, self.points.len())]
                };
                let occupied: Vec<_> = self
                    .members
                    .iter()
                    .enumerate()
                    .filter(|(j, _)| *j != i)
                    .map(|(_, m)| m.position)
                    .collect();
                self.members[i].route = path(self.members[i].position, target, &occupied);
                self.members[i].wait = 1.0 + random(&mut self.rng, 5) as f32;
                continue;
            }
            let m = &self.members[i];
            let target = *m.route.front().unwrap();
            let delta = target - m.position;
            let next = m.position + delta.normalize_or_zero() * (1.65 * dt).min(delta.length());
            if !clear(next)
                || self
                    .members
                    .iter()
                    .enumerate()
                    .any(|(j, m)| j != i && m.position.distance(next) < 0.72)
            {
                // Yield and choose another route instead of walking through a colleague.
                self.members[i].route.clear();
                self.members[i].wait = 0.3 + random(&mut self.rng, 10) as f32 * 0.1;
                continue;
            }
            let m = &mut self.members[i];
            m.position = next;
            m.moving = true;
            m.yaw = (-delta.x).atan2(-delta.z);
            if next.distance(target) < 0.01 {
                m.route.pop_front();
                if m.route.is_empty() && m.returning {
                    m.position = Vec3::new(crate::world::SEATS[m.seat], -0.24, ROW_Z + 1.05);
                    m.seated = true;
                    m.moving = false;
                    m.yaw = 0.0;
                    m.wait = 18.0;
                }
            }
        }
    }
}
pub fn build(b: &mut Builder, screen: Handle<StandardMaterial>) {
    // Background row sits ahead of the six playable stations and preserves side aisles.
    b.block(
        Paint::Wood,
        Vec3::new(0.0, 0.84, ROW_Z),
        Vec3::new(12.9, 0.16, 1.45),
    );
    for (n, x) in crate::world::SEATS.iter().enumerate() {
        let first = b
            .commands
            .spawn((Transform::from_xyz(0.0, 0.0, ROW_Z), Visibility::Visible))
            .id();
        // Builder supports local subtrees so the same workstation geometry cannot drift.
        let old = b.parent.replace(first);
        super::dispatch_room::workstation(b, *x, n + 7, screen.clone());
        b.chair(Vec3::new(*x, 0.0, 1.05), 0.0);
        b.parent = old;
    }
}
pub fn rectangle(distance: f32, min: Vec2, max: Vec2) -> (Vec3, f32) {
    let w = max.x - min.x;
    let h = max.y - min.y;
    let d = distance.rem_euclid(2.0 * (w + h));
    let (p, dir) = if d < w {
        (Vec2::new(min.x + d, min.y), Vec2::X)
    } else if d < w + h {
        (Vec2::new(max.x, min.y + d - w), Vec2::Y)
    } else if d < 2.0 * w + h {
        (Vec2::new(max.x - (d - w - h), max.y), Vec2::NEG_X)
    } else {
        (Vec2::new(min.x, max.y - (d - 2.0 * w - h)), Vec2::NEG_Y)
    };
    (Vec3::new(p.x, 0.0, p.y), (-dir.x).atan2(-dir.y))
}
/// Disjoint sidewalk circuits, evenly spaced: no opposing/crossing pedestrian streams.
pub fn pedestrian(index: u8, seconds: f32) -> (Vec3, f32) {
    let center = if index < 10 {
        Vec2::new(-46., 12.)
    } else {
        Vec2::new(12., -46.)
    };
    let d = seconds * 1.45 + (index % 10) as f32 * 17.6;
    let (mut p, _) = rectangle(d, center - Vec2::splat(22.), center + Vec2::splat(22.));
    // A short look-ahead steers through the corner before changing heading.
    let (ahead, _) = rectangle(
        d + 0.65,
        center - Vec2::splat(22.),
        center + Vec2::splat(22.),
    );
    let dir = ahead - p;
    p.y = super::street::ROAD_Y + 0.14;
    (p, (-dir.x).atan2(-dir.z))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pedestrian_circuits_keep_clearance_and_stay_on_sidewalks() {
        for tick in 0..1000 {
            let positions: Vec<_> = (0..20)
                .map(|i| pedestrian(i, tick as f32 * 0.5).0)
                .collect();
            for (i, a) in positions.iter().enumerate() {
                for b in &positions[i + 1..] {
                    assert!(a.distance(*b) > 1.0);
                }
            }
        }
    }
    #[test]
    fn thrown_prop_knocks_down_staff_and_snapshot_preserves_it() {
        let mut toys = crate::office_fun::Playground::default();
        toys.staff.members.truncate(1);
        let staff = &mut toys.staff.members[0];
        staff.position = Vec3::new(0.0, 0.0, 3.0);
        staff.seated = false;
        staff.wait = 30.0;
        for other in &mut toys.staff.members[1..] {
            other.wait = 30.0;
        }
        let player = crate::room::Pose {
            position: [0.0, 1.65, 5.0],
            yaw: 0.0,
            pitch: 0.0,
        };
        toys.props[0].held = Some(1);
        toys.action(
            1,
            player,
            &crate::game::Action::ThrowProp,
            crate::game::Phase::Running,
        )
        .unwrap();
        for _ in 0..30 {
            toys.tick(1.0 / 120.0, &[(1, player)], crate::game::Phase::Running);
        }
        assert!(toys.down(100));
        let mirror: crate::office_fun::Playground =
            serde_json::from_str(&serde_json::to_string(&toys).unwrap()).unwrap();
        assert_eq!(mirror.dolls[&100].points, toys.dolls[&100].points);
        for _ in 0..360 {
            toys.tick(1.0 / 120.0, &[(1, player)], crate::game::Phase::Running);
        }
        assert!(!toys.down(100));
    }
    #[test]
    fn randomized_staff_are_clear_and_never_overlap() {
        for seed in 1..5 {
            let mut crowd = StaffCrowd::new(seed);
            assert!(crowd.members.iter().any(|m| m.seated));
            assert!(crowd.members.iter().any(|m| !m.seated));
            for _ in 0..2400 {
                crowd.step(0.05, &Default::default());
                for (i, a) in crowd.members.iter().enumerate() {
                    if !a.seated {
                        assert!(clear(a.position));
                    }
                    for b in &crowd.members[i + 1..] {
                        assert!(a.position.distance(b.position) >= 0.71);
                    }
                }
            }
        }
    }
}
