# Initial architecture decision — M0

- Rust 1.99.1 and Bevy 0.20.0 are pinned; Cargo.lock is committed for repeatable builds.
- The Cargo workspace now contains the Bevy application and `crates/simulation`, a renderer-free rules library with typed commands/events, versioned JSON snapshots, and a headless demo. The M2 Bevy application consumes this library through `src/game.rs`, which owns the solo phase/upgrade flow. `src/environment/` builds the room and props; `src/world.rs` handles navigation, seating and the phase-gated rear door; `src/ui.rs` presents public call views and unit states.
- Bevy renders the office and city; a host-owned simulation will validate all gameplay commands for solo and co-op through the same path.
- M1 uses versioned JSON snapshots through Serde. Unknown versions are rejected; migrations are not yet implemented. Scenario templates are authored in Rust for now; data-file content loading is future work.
- Networking is still a decision to validate: evaluate the Bevy-compatible Lightyear version with two processes exchanging a validated command before committing to a transport. No networking implementation is claimed in M0 yet.
- Explicit Bevy features exclude audio, gamepads, and Wayland for this initial build to keep native dependencies small. Linux currently uses X11/XWayland. Revisit these selections when input/audio work begins.
- Log through Bevy's logging plugin. Development failures retain normal Rust panic output; packaged crash reporting is future work.
- The procedural skyline is a startup/renderer check, not a playable shift or an office prototype.

- `src/city.rs` supplies authored training placements over the imported coastal image. Raw graph/config/address data is retained under `assets/city`; it is not yet an authority routing graph. An audit script reports disconnected components before pathfinding integration.
- Graphical automation uses hidden-window rendering and enabled UI button observers. It avoids physical input so desktop work cannot accidentally alter a verification run. Normal gameplay uses the visible window and standard controls.

- `src/wall_map.rs` renders a shared live situational board into one image target. The wall display and monitor surfaces reuse that image; fullscreen terminal details continue to use the public call views. The main 3D camera is the explicit default UI camera, keeping the offscreen board separate from player menus.
- The environment uses a daytime exterior, warm pendant/task lighting, two shadowed interior spots and one limited-resolution sun shadow cascade. It uses shared primitive meshes and procedural tiled materials; no photoreal assets or new dependencies were introduced.

- `Session` starts a 600-second solo shift immediately, schedules eligible calls after seeded 1–10 second delays, owns workstation/line state, and closes dispatch before waiting for active responses. Pending scenarios remain hidden from call views; simulation events record delivery and hang-up independently of incident resolution. Player IDs now validate 1–6 for the future co-op target, but network sessions and shared chair claims are not implemented.
- Room interactions use camera-ray/AABB proxies with distance checks and partition occlusion. Seating and computer focus are independent states; the seated camera remains free to look around.
- `src/performance.rs` owns optional frame statistics, reduced shadow settings and a fixed-camera benchmark. The Linux script selects hardware rendering normally; software Vulkan is opt-in.
