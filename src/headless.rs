//! Separate-process socket playtest runner; never creates a graphics/audio device.
use crate::{
    bot::Agent,
    game::Phase,
    network::{Link, Network},
};
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};
#[derive(serde::Serialize)]
struct Report {
    role: String,
    player: u8,
    peak_players: usize,
    completed_shift: u32,
    digest: u64,
    revision: u64,
    mirror_matches: bool,
    clock: u32,
    score: i32,
    rejected: u64,
}
pub fn run_from_args() -> bool {
    let args: Vec<_> = std::env::args().collect();
    if args.iter().any(|a| a == "--progress-test") {
        crate::progression::check_launch();
        return true;
    }
    if !args.iter().any(|a| a == "--headless-host" || a == "--bot") {
        return false;
    }
    let result = run(&args);
    if let Err(error) = result {
        eprintln!("NETWORK PLAYTEST FAILED: {error}");
        std::process::exit(1);
    }
    true
}
fn run(args: &[String]) -> Result<(), String> {
    let value = |flag: &str| {
        args.iter()
            .position(|a| a == flag)
            .and_then(|n| args.get(n + 1))
    };
    let expected = value("--expected-players")
        .and_then(|n| n.parse().ok())
        .unwrap_or(2);
    let speed: f32 = value("--fast-shift")
        .and_then(|n| n.parse().ok())
        .unwrap_or(1.0);
    if !(1.0..=60.0).contains(&speed) {
        return Err("Fast shift must be 1-60x".into());
    }
    let report = value("--network-report").map(PathBuf::from);
    let automated = args.iter().any(|a| a == "--one-shift") || args.iter().any(|a| a == "--bot");
    let mut net = Network::from_args()?;
    let mut game = net.initial_game();
    let mut agent = Agent::new(1);
    let start = Instant::now();
    let mut last = start;
    let mut peak = 1;
    let mut goal = None;
    let mut finished = None;
    let mut completed_score = 0;
    let mut logged = (0, Phase::Lobby);
    loop {
        let now = Instant::now();
        let dt = now.duration_since(last).as_secs_f32().min(0.1);
        last = now;
        if game.shift == 1 {
            completed_score = game.sim.score();
        }
        net.poll(&mut game);
        let is_host = matches!(net.link, Link::Host { .. });
        net.tick(&mut game, dt * speed);
        if automated {
            agent.update(&mut net, &mut game, dt, expected);
        }
        net.publish(&mut game, dt, agent.pose);
        peak = peak.max(net.players.iter().filter(|p| p.connected).count());
        if logged != (game.shift, game.phase) {
            println!("NETWORK PHASE {:?} SHIFT {}", game.phase, game.shift);
            logged = (game.shift, game.phase);
        }
        if game.phase == Phase::Review {
            completed_score = game.sim.score();
        }
        if game.shift >= 2 && finished.is_none() {
            let sim = game.sim.public_snapshot();
            let hash = crate::room::digest(
                &sim,
                game.phase,
                game.shift,
                game.funds,
                game.radio,
                game.extra_police,
            );
            finished = Some(hash);
        }
        if let Some(hash) = finished {
            let ready = if let Link::Host { room, .. } = &net.link {
                room.observed.len() >= expected.saturating_sub(1)
                    && room.observed.values().all(|h| *h == hash)
            } else {
                true
            };
            if ready && goal.is_none() {
                goal = Some(Instant::now());
            }
            if goal.is_some_and(|at| {
                at.elapsed() > Duration::from_millis(if is_host { 500 } else { 1200 })
            }) {
                let result = Report {
                    role: if is_host { "host" } else { "client" }.into(),
                    player: net.id.unwrap_or(game.local_player),
                    peak_players: peak,
                    completed_shift: 1,
                    digest: hash,
                    revision: net.revision,
                    mirror_matches: true,
                    clock: game.sim.now(),
                    score: completed_score,
                    rejected: net.rejected,
                };
                if let Some(path) = report {
                    if let Some(parent) = path.parent() {
                        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                    }
                    std::fs::write(path, serde_json::to_vec_pretty(&result).unwrap())
                        .map_err(|e| e.to_string())?;
                }
                println!(
                    "NETWORK SHIFT PASSED: {} players, player {}, digest {hash:016x}",
                    peak, result.player
                );
                break;
            }
        }
        if net.id.is_none() && net.status.starts_with("Disconnected") && finished.is_none() {
            return Err(net.status);
        }
        if automated && start.elapsed() > Duration::from_secs(90) {
            return Err(format!(
                "Timed out: {} / shift {} / {:?} / seated {:?} / {}",
                net.status, game.shift, game.phase, game.seated, game.notice
            ));
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    Ok(())
}
