//! Live frame timings and a reproducible steady-state renderer benchmark.
use bevy::{app::AppExit, prelude::*, render::renderer::RenderAdapterInfo};
#[derive(Resource)]
pub struct Graphics {
    pub low: bool,
    pub benchmark: bool,
}
impl Graphics {
    pub fn from_args() -> Self {
        let args: Vec<_> = std::env::args().collect();
        Self {
            low: args
                .iter()
                .any(|a| a == "--software" || a == "--low-graphics"),
            benchmark: args.iter().any(|a| a == "--benchmark"),
        }
    }
}
#[derive(Resource, Default)]
pub struct Frames {
    pub fps: f64,
    pub gpu: String,
    pub show: bool,
    elapsed: f64,
    count: u32,
    frames: u32,
    samples: Vec<f64>,
}
pub fn measure(
    time: Res<Time<Real>>,
    keys: Res<ButtonInput<KeyCode>>,
    graphics: Res<Graphics>,
    adapter: Option<Res<RenderAdapterInfo>>,
    mut stats: ResMut<Frames>,
    mut exit: MessageWriter<AppExit>,
) {
    if keys.just_pressed(KeyCode::F3) {
        stats.show = !stats.show;
    }
    if stats.gpu.is_empty()
        && let Some(adapter) = adapter
    {
        stats.gpu = adapter.name.clone();
    }
    let seconds = time.delta_secs_f64();
    stats.frames += 1;
    stats.elapsed += seconds;
    stats.count += 1;
    if stats.elapsed >= 0.5 {
        stats.fps = f64::from(stats.count) / stats.elapsed;
        stats.elapsed = 0.0;
        stats.count = 0;
    }
    if graphics.benchmark && stats.frames > 90 {
        stats.samples.push(seconds);
    }
    if graphics.benchmark && stats.samples.len() == 300 {
        let total: f64 = stats.samples.iter().sum();
        let mut sorted = stats.samples.clone();
        sorted.sort_by(f64::total_cmp);
        println!(
            "PERFORMANCE gpu={} average_fps={:.1} p95_ms={:.2} samples=300 low_graphics={}",
            stats.gpu,
            300.0 / total,
            sorted[284] * 1000.0,
            graphics.low
        );
        exit.write(AppExit::Success);
    }
}

pub fn configure(
    graphics: Res<Graphics>,
    mut spots: Query<&mut SpotLight>,
    mut suns: Query<&mut DirectionalLight>,
    mut shadow: ResMut<bevy::light::DirectionalLightShadowMap>,
) {
    if graphics.low {
        shadow.size = 512;
        for mut light in &mut spots {
            light.shadow_maps_enabled = false;
        }
        for mut light in &mut suns {
            light.shadow_maps_enabled = false;
        }
    }
}
