//! Shared authoritative furniture bounds, used by walking and thrown props.
use bevy::prelude::*;
#[derive(Clone, Copy)]
pub struct BoxShape {
    pub center: Vec3,
    pub half: Vec3,
}
impl BoxShape {
    pub fn contains(self, p: Vec3) -> bool {
        (p - self.center).abs().cmple(self.half).all()
    }
    pub fn blocks_walker(self, p: Vec2) -> bool {
        if self.center.y - self.half.y > 1.8 {
            return false;
        }
        let delta = (p - Vec2::new(self.center.x, self.center.z)).abs()
            - Vec2::new(self.half.x, self.half.z);
        delta.max(Vec2::ZERO).length_squared() < 0.22 * 0.22
    }
}
pub fn furniture() -> &'static [BoxShape] {
    static BOUNDS: std::sync::OnceLock<Vec<BoxShape>> = std::sync::OnceLock::new();
    BOUNDS.get_or_init(|| {
        let mut out = vec![BoxShape {
            center: Vec3::new(13.25, 1.2, 4.4),
            half: Vec3::new(0.3, 1.2, 0.75),
        }];
        let mut add = |c: Vec3, s: Vec3| {
            out.push(BoxShape {
                center: c,
                half: s * 0.5,
            })
        };
        for z in [0.0, crate::environment::people::ROW_Z] {
            add(Vec3::new(0.0, 0.84, z), Vec3::new(12.9, 0.16, 1.45));
            for (index, x) in crate::world::SEATS.into_iter().enumerate() {
                add(Vec3::new(x, 1.4, z - 0.39), Vec3::new(1.13, 0.74, 0.13));
                add(Vec3::new(x + 0.72, 0.39, z), Vec3::new(0.43, 0.78, 1.05));
                if (index + 1) % 3 != 0 {
                    add(
                        Vec3::new(x + 0.72, 0.4, z + 0.69),
                        Vec3::new(0.43, 0.78, 0.36),
                    );
                }
                add(
                    Vec3::new(x, 0.72, z + if z == 0.0 { 1.7 } else { 1.05 }),
                    Vec3::new(0.7, 1.2, 0.65),
                );
            }
        }
        for (x, z) in [(-7.5, -5.8), (7.4, -5.8), (13.4, 5.7)] {
            add(Vec3::new(x, 0.55, z), Vec3::new(0.65, 1.1, 0.65));
        }
        for x in [-7.8, -6.6] {
            add(Vec3::new(x, 0.5, 6.45), Vec3::new(1.0, 1.0, 0.9));
        }
        add(Vec3::new(11.5, 0.82, 1.2), Vec3::new(2.0, 0.13, 1.0));
        add(Vec3::new(11.0, 0.5, -4.25), Vec3::new(3.25, 1.0, 1.0));
        add(Vec3::new(13.25, 1.1, -3.9), Vec3::new(1.1, 2.2, 1.0));
        for x in [10.9, 12.1] {
            for z in [0.3, 2.1] {
                add(Vec3::new(x, 0.65, z), Vec3::new(0.7, 1.2, 0.65));
            }
        }
        add(Vec3::new(9.8, 0.4, 5.8), Vec3::new(0.45, 0.8, 0.45));
        out
    })
}
pub fn solid(p: Vec3) -> bool {
    furniture().iter().any(|b| b.contains(p))
}
pub fn overlaps(center: Vec3, half: Vec3) -> bool {
    furniture()
        .iter()
        .any(|b| (center - b.center).abs().cmple(half + b.half).all())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn screens_plants_and_both_rows_block_props() {
        for p in [
            Vec3::new(-1.05, 1.4, -0.39),
            Vec3::new(-1.05, 1.4, -4.99),
            Vec3::new(-7.5, 0.4, -5.8),
            Vec3::new(0.0, 0.84, 0.0),
            Vec3::new(0.0, 0.84, -4.6),
        ] {
            assert!(solid(p), "{p}");
        }
    }
    #[test]
    fn player_approaches_remain_clear() {
        for n in 0..6 {
            let p = crate::world::desk_approach(n);
            assert!(crate::world::walkable(
                Vec2::new(p.x, p.z),
                crate::game::Phase::Running
            ));
        }
        assert!(!crate::world::walkable(
            Vec2::new(-7.5, -5.8),
            crate::game::Phase::Running
        ));
    }
}
#[cfg(test)]
mod overlap_tests {
    use super::*;
    #[test]
    fn broad_props_cannot_straddle_thin_monitor_or_desktop() {
        assert!(overlaps(
            Vec3::new(0.0, 0.84, -4.6),
            Vec3::new(0.15, 0.3, 0.15)
        ));
        assert!(overlaps(Vec3::new(-1.05, 1.4, -0.39), Vec3::splat(0.15)));
    }
}
