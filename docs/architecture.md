# Initial architecture decision — M0

- Rust 1.99.1 and Bevy 0.20.0 are pinned; Cargo.lock is committed for repeatable builds.
- The Cargo workspace now contains the Bevy application and `crates/simulation`, a renderer-free rules library with typed commands/events, versioned JSON snapshots, and a headless demo. The M2 Bevy application consumes this library through `src/game.rs`, which owns the solo phase/upgrade flow. `src/environment/` builds the room and props; `src/world.rs` handles navigation, seating and the phase-gated rear door; `src/ui.rs` presents public call views and unit states.
- Bevy renders the office and city; a host-owned simulation will validate all gameplay commands for solo and co-op through the same path.
- M1 uses versioned JSON snapshots through Serde. Unknown versions are rejected; migrations are not yet implemented. Scenario templates are authored in Rust for now; data-file content loading is future work.
- Networking is still a decision to validate: evaluate the Bevy-compatible Lightyear version with two processes exchanging a validated command before committing to a transport. No networking implementation is claimed in M0 yet.
- Explicit Bevy features include WAV audio for spatial telephone ringing and exclude gamepads and Wayland to keep native dependencies small. Linux currently uses X11/XWayland. Revisit these selections when input/audio work begins.
- Log through Bevy's logging plugin. Development failures retain normal Rust panic output; packaged crash reporting is future work.
- The procedural skyline is a startup/renderer check, not a playable shift or an office prototype.

- `src/city.rs` supplies authored training placements over the imported coastal image. Raw graph/config/address data is retained under `assets/city`; the authority now uses its connected pilot district for real routing. An audit script reports disconnected components before pathfinding integration.
- Graphical automation uses hidden-window rendering and enabled UI button observers. It avoids physical input so desktop work cannot accidentally alter a verification run. Normal gameplay uses the visible window and standard controls.

- `src/wall_map.rs` renders a shared live situational board into one image target. The wall display and monitor surfaces reuse that image; fullscreen terminal details continue to use the public call views. The main 3D camera is the explicit default UI camera, keeping the offscreen board separate from player menus.
- The environment uses a daytime exterior, warm pendant/task lighting, two shadowed interior spots and one limited-resolution sun shadow cascade. It uses shared primitive meshes and procedural tiled materials; no photoreal assets or new dependencies were introduced.

- `Session` starts a 600-second solo shift immediately, schedules eligible calls after seeded 1–10 second delays, owns workstation/line state, and closes dispatch before waiting for active responses. Pending scenarios remain hidden from call views; simulation events record delivery and hang-up independently of incident resolution. Player IDs now validate 1–6 for the future co-op target, but network sessions and shared chair claims are not implemented.
- Room interactions use camera-ray/AABB proxies with distance checks and partition occlusion. Seating and computer focus are independent states; the seated camera remains free to look around.
- `src/performance.rs` owns optional frame statistics, reduced shadow settings and a fixed-camera benchmark. The Linux script selects hardware rendering normally; software Vulkan is opt-in.

- The gameplay design PDF and exact source are versioned in `docs/design/`; review evidence lives in `docs/reviews/gameplay-loop.md`. `AGENTS.md` requires a comparison after every milestone/gameplay-changing commit and authorizes starting M3 automatically after M2 passes.
- `src/routes.rs` caches a transparent route overlay; `src/phone.rs` provides spatial ringing and a physical beacon. The local launcher still selects hardware by default. Linux audio compilation requires ALSA development linkage.

- M3's first slice adds bounded scenario consequence models, authoritative per-tick damage events, and an event-derived review ledger. The UI never reads potential damage/truth to invent an outcome. Full escalation chains and the adaptive director are still pending; this change starts M3 without marking the whole milestone complete.

- M3's next slice stores `ScenarioRules` in authority snapshots and loads authored JSON from `assets/scenarios/m3_rules.json`. A pure `DirectorPolicy` controls solo pressure/budgets; source/child relations and escalation events preserve causal replay.
- `src/city_effects.rs` consumes new live escalation events for a one-shot office flash and generated spatial boom. It ignores old fast-forwarded/closing events. Map and review views derive urgency and the follow-up link from authority state/events. Runtime road closures and skyline destruction are not yet implemented.


## M3 completion and M4 authority decision — 2026-10-28

M3 is complete for this vertical slice: recorded explosions drive facade damage, safe timed closures, rerouting, linked calls and the event review. Earlier “pending” statements above describe historical slices.

M4 adds `crates/net` for bounded background TCP/JSON transport and `src/room.rs` for host-owned membership, player IDs, ordered commands, validated movement and exclusive chairs. `Session` stores independent `Desk` states around one shared simulation. `src/network.rs` integrates the room into Bevy; both the host viewport and clients consume public read-only mirrors, leaving private truth and unrevealed content in authority memory. Simple shared-mesh avatars interpolate remote poses. Offline solo still consumes the same original rules directly.

The TCP spike passed two/six-process full-shift and reconnect checks. It avoids engine-plugin version coupling at the cost of reliable-stream head-of-line blocking and full public snapshot bandwidth. Protocol version 1 includes bounded queues, frame/rate limits, sequence acknowledgements and same-room reconnect tokens. It is a direct LAN prototype with no host migration, account service, relay or voice. See [multiplayer setup and acceptance evidence](multiplayer.md).


## M5 caller boundary — 2026-10-28

`crates/caller` defines a provider-neutral bounded context/reply protocol, authored fallback and strict validator. `crates/caller-server` is a separate optional trusted executable with the compatible HTTP adapter; only that process reads provider credentials. The game's dependency tree excludes the HTTP provider client. Actor-owned asynchronous dialogue in `src/dialogue.rs` commits only validated fact references or conversation goodbye, while scenario truth and all resource/city commands remain in the existing simulation.

`src/voice.rs` owns the typed-question draft, explicit hold-V microphone capture, bounded local STT/TTS subprocess adapters, caller output selection/volume and opt-in local clip retention/replay/deletion. Workers never run on the rendering/simulation loop. Recognized speech is editable text before submission. The live room protocol is now version 2 because snapshots include public conversation turns; older M4 clients are rejected. Text transcripts reset per shift; saved local clips remain until retention cleanup or user deletion. See [the caller guide](callers.md) and milestone review for tested scope and manual checks.
