//! Repeatable scripted shift; requires no window, GPU, microphone or API key.
use wrong_number_sim::{Command, Simulation};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut sim = Simulation::new(42);
    println!("Wrong Number — headless M1 shift (seed 42)");
    for id in 1..=6 {
        sim.apply(1, Command::AcceptCall { incident: id })?;
        let question = sim.call_view(id)?.questions[0].clone();
        sim.apply(
            1,
            Command::Ask {
                incident: id,
                fact: question.0,
            },
        )?;
        let view = sim.call_view(id)?;
        println!("Call {id}: {}\n  {}", view.report, view.facts[0].answer);
    }
    for incident in [2, 3, 4] {
        sim.apply(1, Command::Decline { incident })?;
    }
    for (unit, incident) in [(1, 6), (2, 5), (3, 1)] {
        sim.apply(1, Command::Dispatch { unit, incident })?;
    }
    sim.advance(90)?;
    let saved = sim.save_json()?;
    let restored = Simulation::load_json(&saved)?;
    assert_eq!(sim, restored);
    println!("\nShift at {}s — score: {}", sim.now(), sim.score());
    for event in sim.events() {
        println!("{:03}s {:?}", event.at, event.kind);
    }
    println!(
        "\nSnapshot round-trip preserved the shift. All three units: {:?}",
        sim.units().values().map(|u| &u.state).collect::<Vec<_>>()
    );
    Ok(())
}
