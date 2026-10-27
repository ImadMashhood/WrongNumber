# Initial architecture decision — M0

- Rust 1.99.1 and Bevy 0.20.0 are pinned; Cargo.lock is committed for repeatable builds.
- Start with one application package. Extract a simulation library in M1, with typed commands/events and no rendering requirement. Split into a workspace only when boundaries need separate dependencies.
- Bevy renders the office and city; a host-owned simulation will validate all gameplay commands for solo and co-op through the same path.
- Scenario data and save snapshots will use versioned Serde models. Select JSON/RON and migration rules in M1 before persisting player data.
- Networking is still a decision to validate: evaluate the Bevy-compatible Lightyear version with two processes exchanging a validated command before committing to a transport. No networking implementation is claimed in M0 yet.
- Explicit Bevy features exclude audio, gamepads, and Wayland for this initial build to keep native dependencies small. Linux currently uses X11/XWayland. Revisit these selections when input/audio work begins.
- Log through Bevy's logging plugin. Development failures retain normal Rust panic output; packaged crash reporting is future work.
- The procedural skyline is a startup/renderer check, not a playable shift or an office prototype.
