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
pub fn build(b: &mut Builder, screen: Handle<StandardMaterial>) {
    // Background row sits ahead of the six playable stations and preserves side aisles.
    b.block(
        Paint::Wood,
        Vec3::new(0.0, 0.84, -3.5),
        Vec3::new(12.9, 0.16, 1.45),
    );
    for (n, x) in crate::world::SEATS.iter().enumerate() {
        let first = b
            .commands
            .spawn((Transform::from_xyz(0.0, 0.0, -3.5), Visibility::Visible))
            .id();
        // Builder supports local subtrees so the same workstation geometry cannot drift.
        let old = b.parent.replace(first);
        super::dispatch_room::workstation(b, *x, n + 7, screen.clone());
        b.chair(Vec3::new(*x, 0.0, 1.7), 0.0);
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
    mut walkers: Query<(&Walker, &mut Transform), Without<WalkingLimb>>,
    mut limbs: Query<(&WalkingLimb, &mut Transform), Without<Walker>>,
) {
    let t = time.elapsed_secs()
        + if std::env::args().any(|a| a == "--polish-test") {
            20.0
        } else {
            0.0
        };
    for (w, mut pose) in &mut walkers {
        let (mut p, yaw) = if w.staff {
            let (p, yaw, _) = staff_pose(t, w.index);
            (p, yaw)
        } else {
            rectangle(
                t * 1.25 + w.index as f32 * 12.3,
                Vec2::splat(-19.2),
                Vec2::splat(43.2),
            )
        };
        if !w.staff {
            p.y = super::street::ROAD_Y + 0.14;
        }
        pose.translation = p;
        pose.rotation = Quat::from_rotation_y(yaw);
    }
    for (limb, mut pose) in &mut limbs {
        let seated = limb.staff && staff_pose(t, limb.index).2;
        let angle = if seated {
            if limb.arm {
                -0.9 + if prefs.reduced_motion {
                    0.0
                } else {
                    (t * 9.0 + limb.side).sin() * 0.05
                }
            } else {
                -1.35
            }
        } else if prefs.reduced_motion {
            0.0
        } else {
            (t * 6.0).sin() * 0.35 * limb.side * if limb.arm { -1.0 } else { 1.0 }
        };
        pose.rotation = Quat::from_rotation_x(angle);
    }
}
/// A shared permutation assigns distinct decorative desks each cycle.
pub fn staff_pose(t: f32, index: u8) -> (Vec3, f32, bool) {
    let epoch = (t / 50.0) as usize;
    let phase = t % 50.0;
    let next = (index as usize * 5 + epoch * 5) % 6;
    let previous = (index as usize * 5 + (epoch + 5) * 5) % 6;
    let a = crate::world::SEATS[previous];
    let b = crate::world::SEATS[next];
    if phase >= 14.0 {
        return (Vec3::new(b, -0.24, -1.8), 0.0, true);
    }
    let points = [
        Vec3::new(a, 0.0, -1.8),
        Vec3::new(a, 0.0, -1.02),
        Vec3::new(b, 0.0, -1.02),
        Vec3::new(b, 0.0, -1.8),
    ];
    let total = 1.56 + (a - b).abs();
    let mut d = phase / 14.0 * total;
    for pair in points.windows(2) {
        let delta = pair[1] - pair[0];
        let len = delta.length();
        if d <= len {
            return (
                pair[0] + delta * (d / len.max(0.001)),
                (-delta.x).atan2(-delta.z),
                false,
            );
        }
        d -= len;
    }
    (Vec3::new(b, 0.0, -1.8), 0.0, false)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn background_staff_use_distinct_extra_row_seats() {
        for epoch in 0..20 {
            let mut seats = std::collections::BTreeSet::new();
            for id in 0..5 {
                let (p, _, seated) = staff_pose(epoch as f32 * 50.0 + 25.0, id);
                assert!(seated);
                assert_eq!(p.z, -1.8);
                assert!(seats.insert((p.x * 100.0) as i32));
            }
        }
    }
}
