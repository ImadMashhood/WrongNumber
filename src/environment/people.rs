//! Decorative walkers only: no desk ownership, calls, money or simulation actions.
use super::geometry::{Builder, Paint};
use bevy::prelude::*;
#[derive(Component)]
pub struct Walker {
    index: u8,
    staff: bool,
}
#[derive(Component)]
pub struct WalkingLimb {
    side: f32,
    arm: bool,
    staff: bool,
    index: u8,
}
pub const ROW_Z: f32 = -4.6;
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct StaffState {
    pub position: Vec3,
    pub yaw: f32,
    seated: bool,
    moving: bool,
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
    for staff in [false, true] {
        for i in 0..if staff { 5 } else { 20 } {
            let root = b
                .commands
                .spawn((
                    Walker { index: i, staff },
                    Transform::default(),
                    Visibility::Visible,
                ))
                .id();
            let old = b.parent.replace(root);
            b.block(
                if i % 3 == 0 {
                    Paint::Cool
                } else if i % 3 == 1 {
                    Paint::Green
                } else {
                    Paint::Red
                },
                Vec3::new(0.0, 1.12, 0.0),
                Vec3::new(0.42, 0.53, 0.25),
            );
            b.round(
                Paint::Cream,
                Vec3::new(0.0, 1.58, 0.0),
                Vec3::new(0.28, 0.32, 0.27),
            );
            b.block(
                Paint::Metal,
                Vec3::new(0.0, 1.73, 0.03),
                Vec3::new(0.29, 0.08, 0.25),
            );
            if staff {
                b.block(
                    Paint::Paper,
                    Vec3::new(0.1, 1.17, -0.135),
                    Vec3::new(0.09, 0.12, 0.012),
                );
            }
            for x in [-0.055, 0.055] {
                b.block(
                    Paint::Metal,
                    Vec3::new(x, 1.62, -0.137),
                    Vec3::new(0.025, 0.025, 0.012),
                );
            }
            for side in [-1.0, 1.0] {
                for arm in [false, true] {
                    let limb = b
                        .commands
                        .spawn((
                            WalkingLimb {
                                side,
                                arm,
                                staff,
                                index: i,
                            },
                            Transform::from_xyz(
                                side * if arm { 0.29 } else { 0.12 },
                                if arm { 1.32 } else { 0.82 },
                                0.0,
                            ),
                            Visibility::Visible,
                        ))
                        .id();
                    b.commands.entity(root).add_child(limb);
                    b.parent = Some(limb);
                    b.block(
                        if arm { Paint::Cream } else { Paint::Plastic },
                        Vec3::new(0.0, if arm { -0.24 } else { -0.36 }, 0.0),
                        Vec3::new(0.12, if arm { 0.48 } else { 0.70 }, 0.14),
                    );
                }
            }
            b.parent = old;
        }
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
pub fn update(
    time: Res<Time>,
    prefs: Res<crate::frontend::Preferences>,
    game: Res<crate::game::Session>,
    mut walkers: Query<(&Walker, &mut Transform, &mut Visibility), Without<WalkingLimb>>,
    mut limbs: Query<(&WalkingLimb, &mut Transform), Without<Walker>>,
) {
    let t = time.elapsed_secs()
        + if std::env::args().any(|a| a == "--polish-test") {
            20.0
        } else {
            0.0
        };
    let crowd = &game.fun.staff;
    for (w, mut pose, mut visible) in &mut walkers {
        *visible = if w.staff && game.fun.down(100 + w.index) {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
        let (mut p, yaw) = if w.staff {
            let m = &crowd.members[w.index as usize];
            (m.position, m.yaw)
        } else {
            rectangle(
                t * 1.25 + w.index as f32 * 12.3,
                if w.index.is_multiple_of(2) {
                    Vec2::new(-60.5, -8.5)
                } else {
                    Vec2::new(-8.5, -60.5)
                },
                if w.index.is_multiple_of(2) {
                    Vec2::new(-19.5, 32.5)
                } else {
                    Vec2::new(32.5, -19.5)
                },
            )
        };
        if !w.staff {
            p.y = super::street::ROAD_Y + 0.14;
        }
        let blend =
            if w.staff && pose.translation != Vec3::ZERO && pose.translation.distance(p) < 1.0 {
                (time.delta_secs() * 14.0).min(1.0)
            } else {
                1.0
            };
        pose.translation = pose.translation.lerp(p, blend);
        pose.rotation = pose.rotation.slerp(Quat::from_rotation_y(yaw), blend);
    }
    for (limb, mut pose) in &mut limbs {
        let seated = limb.staff && crowd.members[limb.index as usize].seated;
        let angle = if seated {
            if limb.arm {
                0.9 + if prefs.reduced_motion {
                    0.0
                } else {
                    (t * 9.0 + limb.side).sin() * 0.05
                }
            } else {
                1.35
            }
        } else if prefs.reduced_motion || (limb.staff && !crowd.members[limb.index as usize].moving)
        {
            0.0
        } else {
            (t * 6.0).sin() * 0.35 * limb.side * if limb.arm { -1.0 } else { 1.0 }
        };
        pose.rotation = Quat::from_rotation_x(angle);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
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
