# Initial architecture decision — M0

- Rust 1.99.1 and Bevy 0.20.0 are pinned; Cargo.lock is committed for repeatable builds.
- The Cargo workspace now contains the Bevy application and `crates/simulation`, a renderer-free rules library with typed commands/events, versioned JSON snapshots, and a headless demo. The Bevy application will consume this library in M2.
- Bevy renders the office and city; a host-owned simulation will validate all gameplay commands for solo and co-op through the same path.
- M1 uses versioned JSON snapshots through Serde. Unknown versions are rejected; migrations are not yet implemented. Scenario templates are authored in Rust for now; data-file content loading is future work.
- Networking is still a decision to validate: evaluate the Bevy-compatible Lightyear version with two processes exchanging a validated command before committing to a transport. No networking implementation is claimed in M0 yet.
- Explicit Bevy features exclude audio, gamepads, and Wayland for this initial build to keep native dependencies small. Linux currently uses X11/XWayland. Revisit these selections when input/audio work begins.
- Log through Bevy's logging plugin. Development failures retain normal Rust panic output; packaged crash reporting is future work.
- The procedural skyline is a startup/renderer check, not a playable shift or an office prototype.
