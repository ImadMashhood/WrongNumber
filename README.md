# Wrong Number

**A 1–4 player cooperative emergency-dispatch comedy game built with Rust and Bevy.**

Players work a shift in a busy dispatch office, investigate calls from unreliable callers, and decide when to send scarce response teams. The city remembers those choices: incidents resolve, responders stay occupied, new calls can emerge, and the team reviews its performance before buying upgrades.

The central challenge is sound triage. A refusal can be the right call, while a credible emergency left unanswered can become much worse. Humor comes from callers, player arguments, limited resources, and consequences the game can show and explain.

This repository is the project home. The first goal is a small, complete, playable shift. AI callers, voice features, character creation, and Steam Workshop support come later, after the core loop is fun and reliable.

## Project status

**M1 implemented.** The repository contains a Bevy startup scene and a renderer-free dispatch simulation with a scripted terminal demo. The office and interactive shift UI are planned for M2. Milestones below are proposed work and acceptance criteria; they are not claims that features already exist. Scope and technical choices may change as prototypes and playtests provide evidence.

## Design pillars

1. **Triage is the game.** Players investigate incomplete reports and choose whether a response is justified.
2. **The simulation owns the truth.** Caller dialogue can be mistaken or deceptive, but it cannot change an incident's established facts.
3. **Consequences are visible and traceable.** Important simulation events appear in the map, the city, and the shift review.
4. **Coordination matters.** Players share calls and limited response units; simultaneous actions must resolve consistently.
5. **The core loop works offline.** Scripted callers keep the game playable when AI, speech, or network services are unavailable.

## Proposed technical direction

- **Language and engine:** Rust and Bevy for the Windows/Linux PC game.
- **Simulation:** typed, data-driven rules for incident truth, deadlines, unit availability, travel, escalation, scoring, and upgrades. Keep game rules testable without opening a rendered scene.
- **Networking:** server-authoritative shared state. Start with a host running the simulation for a small co-op group; choose the Bevy networking library and versions after a build and multiplayer compatibility spike.
- **Presentation:** a first-person dispatch office, interactive call terminals, a readable city map, and a stylized skyline. Simulate only the city details that affect decisions; present other activity selectively.
- **AI and voice:** optional adapters behind a server-side boundary. The game client must not contain paid-provider credentials. Validate all generated output against known scenario facts and allowed actions.
- **Content:** versioned data files for scenarios and, later, user-created caller packs. Treat imported text and media as untrusted data; never load executable content from a pack.

These are starting choices, not locked dependency versions. Before implementation, record the selected Rust toolchain, Bevy version, networking approach, and supported platform build commands in this README or an architecture decision record.

## Development milestones

Milestones are ordered by dependency. Each ends with a reviewable build or artifact, a short playtest or verification pass, and a decision to continue, revise, or reduce scope. No milestone should be described as complete until its acceptance checks pass.

### M0 — Repository and build foundation

- [ ] **Milestone status:** In progress — Linux build verified; Windows/Linux CI pending

**Purpose:** Make changes repeatable and produce a minimal app on target platforms.

**Deliverables**

- [x] Cargo workspace or package structure chosen for the prototype, with a minimal Bevy application that opens and exits cleanly.
- [x] Formatting, linting, logging, error reporting, and a documented development setup.
- [x] Continuous integration that checks formatting, compiles the project, and runs fast automated checks on Linux; add a Windows build check as soon as practical.
- [x] A first architecture note naming crate boundaries, data serialization approach, and the initial networking experiment.
- [x] Placeholder asset policy and licensing notes for any third-party assets.

**Acceptance checks**

- [ ] A new contributor can follow the README to install the Rust toolchain and run the app.
- [ ] CI gives a clear pass/fail result for the documented checks.
- [ ] The selected Bevy and Rust versions build together in CI; incompatible versions are not guessed or left implicit.

### M1 — Headless dispatch simulation

- [x] **Milestone status:** Implemented and locally verified

**Purpose:** Prove the central rules before investing in office art or AI.

**Deliverables**

- [x] Typed models for incidents, known facts, locations, response units, commands, simulation events, and shift results.
- [x] An explicit incident lifecycle and responder lifecycle, including assignment, travel, arrival, resolution, return, and recovery.
- [x] A deterministic simulation clock and seeded scenario setup so a bug can be reproduced.
- [x] Validated commands for accepting a call, revealing an available clue, dispatching, cancelling or rerouting, and declining.
- [x] Basic scoring that rewards justified refusal and timely appropriate response, and explains penalties for waste or delay.
- [x] A small scenario set covering real, fake, non-emergency, exaggerated, and underreported calls.

**Acceptance checks**

- [x] The same seed and command sequence produce the same result.
- [x] Hidden scenario truth is fixed before dialogue begins and cannot be changed by a caller response.
- [x] Invalid actions, fabricated fact identifiers, impossible unit transitions, and double allocation are rejected.
- [x] Automated checks cover truth preservation, unit transitions, deadlines, scoring, and save/load of a shift snapshot.

### M2 — First-person dispatch office and playable solo shift

- [ ] **Milestone status:** Not started

**Purpose:** Turn the simulation into a complete, understandable game loop.

**Deliverables**

- [ ] A compact navigable office with a player viewpoint, one working desk, call terminal, city map, windows, and a review area.
- [ ] Interactions for answering, asking scripted questions, reading clues, dispatching or declining, and checking unit availability.
- [ ] Three response types—police, ambulance, and fire—with distinct capabilities and visible busy/available states.
- [ ] A compact city with several districts, locations, simple travel times, incident markers, and a stylized skyline.
- [ ] A complete short shift: preparation, incoming calls, incident outcomes, performance review, and one upgrade choice.
- [ ] Clear text and visual feedback for action results, incident status, and why score changed.

**Acceptance checks**

- [ ] A new player can finish a shift without using developer tools or reading source code.
- [ ] At least one real and one fake call require different sound decisions.
- [ ] Dispatching consumes capacity until the unit returns and recovers.
- [ ] The end-of-shift review agrees with the simulation's recorded command and event history.

### M3 — Escalation, city consequences, and replayable content

- [ ] **Milestone status:** Not started

**Purpose:** Make the city react to decisions and create memorable, fair outcomes.

**Deliverables**

- [ ] Data-driven deadlines and bounded escalation rules for incidents.
- [ ] One authored major event, such as an ignored genuine threat leading to a blast, with synchronized map marker, skyline effect, road closure, office response, and secondary calls.
- [ ] A director that schedules calls within an explicit difficulty and active-incident budget.
- [ ] More scenario variations, caller histories, and a complete review timeline with clear evidence for grades and penalties.
- [ ] One upgrade cycle that changes the next shift in a visible, understandable way.

**Acceptance checks**

- [ ] Every visible consequence traces to a simulation event with a cause and timestamp.
- [ ] Escalation chains have cooldowns and caps; the director cannot generate unbounded incidents.
- [ ] A prank cannot create a real blast through dialogue or model improvisation.
- [ ] Review comments and any replays are based only on recorded events and consent settings.

### M4 — Cooperative multiplayer vertical slice

- [ ] **Milestone status:** Not started

**Purpose:** Validate that shared decision-making and resource contention work with real players.

**Deliverables**

- [ ] Host and client sessions for two players, followed by a test at the four-player target.
- [ ] Shared calls and responder pools, call ownership and transfer, player presence, reconnect handling, and clear host authority.
- [ ] Reliable network commands for dispatch, decline, transfer, upgrades, and other critical decisions; presentation updates can use suitable lower-priority replication.
- [ ] Lobby setup and a simple way to start and end a cooperative shift.
- [ ] Network diagnostics for command rejection, latency, disconnects, and authoritative event history.

**Acceptance checks**

- [ ] Two players cannot dispatch the same unit at the same time; the host accepts at most one valid allocation.
- [ ] All players converge on the same incident truth, unit status, event timeline, and shift result.
- [ ] Disconnecting a client does not corrupt the host simulation; reconnect behavior is documented and tested.
- [ ] A whole shift can be completed by two players on separate machines.

### M5 — Caller AI and audio, with robust fallbacks

- [ ] **Milestone status:** Not started

**Purpose:** Add flexible conversation while keeping game authority and playability deterministic.

**Deliverables**

- [ ] Text conversation first, with a provider-neutral server-side adapter and a scripted fallback.
- [ ] A structured response format for spoken text, emotion, proposed player-visible action, fact references, and call completion.
- [ ] Strict validation against caller knowledge and allowed actions; rate, token, timeout, cancellation, and cost controls.
- [ ] Optional speech-to-text and text-to-speech behind replaceable adapters, with subtitles and text input available.
- [ ] Audio-device settings, push-to-talk, caller volume controls, and basic latency/failure diagnostics.
- [ ] Secrets loaded only by a trusted server process from local configuration or deployment secrets, never committed or shipped in a client build.

**Acceptance checks**

- [ ] Invalid or unavailable AI output falls back to a scripted or text response without blocking the shift.
- [ ] AI output cannot alter hidden truth, dispatch units, spend money, create incidents, or reveal facts the caller does not know.
- [ ] Requests time out and can be cancelled; the rendering and simulation loops stay responsive.
- [ ] A player can complete the core loop with no microphone and no external AI service.

### M6 — Progression, accessibility, and repeat play

- [ ] **Milestone status:** Not started

**Purpose:** Improve long-term variety and make the game usable by more players.

**Deliverables**

- [ ] Multiple shifts per run, increasing scenario variety, funding, upgrades, and clear reset boundaries between runs and permanent unlocks.
- [ ] Accessibility settings for subtitles, text size, color cues, audio levels, input remapping, and reduced screen shake.
- [ ] Settings and versioned local persistence for progress and preferences.
- [ ] Playtest sessions that measure whether clues are understandable, decisions feel fair, and shift length supports conversation.

**Acceptance checks**

- [ ] Players can identify why a call was scored as it was from the review evidence.
- [ ] Settings persist across restart and can be changed without losing run data.
- [ ] New runs and upgrades add meaningful choices without invalidating basic scenarios.
- [ ] Performance and usability targets are written down and measured on representative hardware.

### M7 — Caller creation and local content packs

- [ ] **Milestone status:** Not started

**Purpose:** Let players create and test caller characters safely before publishing tools are considered.

**Deliverables**

- [ ] Character editor for name, biography, occupation, personality traits, catchphrases, and portrait import/cropping; add drawing tools only if playtests show they are valuable.
- [ ] Mock call preview against a fixed scenario, with traits changing dialogue style but not incident truth.
- [ ] Versioned JSON manifests and local import/export for character packs.
- [ ] Validation for size limits, schema version, media formats, paths, content hashes, and untrusted text.
- [ ] Explicit permissions and deletion controls for any optional recorded voice profile; built-in synthetic voices remain available.

**Acceptance checks**

- [ ] Malformed, oversized, path-traversing, and unsupported content is rejected with a clear message.
- [ ] Custom biographies cannot override game rules, change incident truth, or grant commands.
- [ ] Local packs work without a Steam account or network connection.
- [ ] A character can be previewed and removed without corrupting saved progress.

### M8 — Steam integration and release readiness

- [ ] **Milestone status:** Not started

**Purpose:** Package, distribute, moderate, and support a release candidate.

**Deliverables**

- [ ] Steam lobbies/invites and platform integration after the base network flow is stable.
- [ ] Workshop create, publish, subscribe, update, remove, block, report, and host-selected pack synchronization, if moderation and support capacity are ready.
- [ ] Windows and Linux packaging, clean-install checks, crash logs, onboarding, controller/keyboard review, and final performance pass.
- [ ] Privacy, voice consent, retention/deletion, mod safety, license, and support documentation.
- [ ] Release checklist, known-issues list, and rollback plan.

**Acceptance checks**

- [ ] A clean install can host and join a full shift on each supported platform.
- [ ] User-generated packs remain data-only and are validated before use or synchronization.
- [ ] Voice recording and publication require clear opt-in, revocation, and deletion paths.
- [ ] Store and release materials accurately distinguish implemented features from planned or unsupported features.

## Out of scope for the first playable build

- Voice cloning or publication of cloned voices.
- Steam Workshop and public user-content moderation.
- A large, continuously simulated city or individual civilian simulation.
- Permanent progression trees, many districts, and a large responder roster.
- Physics-driven player interference that can block dispatch interactions.

These can be reconsidered after the first cooperative shift demonstrates that investigation, triage, resource scarcity, and consequences are fun.

## How we will track progress

- [ ] Keep milestone checklists current as work lands; link issues or pull requests from each item when available.
- [ ] Keep scope small enough that the game builds and can be played at the end of every milestone.
- [ ] Record significant dependency and architecture choices, including tradeoffs, in short decision notes.
- [ ] Report limitations honestly. A mock, prototype, or planned integration must not be described as production-ready.

## Local development

Install [Rust through rustup](https://rustup.rs/). The repository selects Rust 1.99.1 (including rustfmt and Clippy); Cargo downloads Bevy 0.20.0 automatically. Commit `Cargo.lock` when updating dependencies.

On Fedora, install native build/window dependencies:

```bash
sudo dnf install gcc-c++ pkgconf-pkg-config libX11-devel libxkbcommon-devel
```

On Ubuntu/Debian:

```bash
sudo apt-get install build-essential pkg-config libx11-dev libxkbcommon-dev
```

On Windows, install Visual Studio Build Tools with the **Desktop development with C++** workload. A working graphics driver is required on both platforms. Linux currently uses X11 or XWayland.

From the repository directory:

```bash
cargo run --locked
```

The M0 build displays a procedural skyline. Press Escape or close the window to exit. To check startup and exit automatically after 60 updates (requires a display and graphics device):

```bash
cargo run --locked -- --smoke-test
```

Development checks:

```bash
cargo fmt --all -- --check
cargo check --locked --workspace --all-targets
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
```

The first compilation downloads and builds the engine and can take several minutes. Logs print to the terminal; set `RUST_LOG=info` for normal startup diagnostics or `RUST_LOG=debug` for more detail. No API credentials or downloaded assets are needed for M0.

See [the architecture decision](docs/architecture.md) for boundaries and pending networking decisions, and [asset provenance](assets/README.md) before adding external content. The two-platform CI workflow checks formatting, compilation, Clippy, and test targets; graphics launch checks currently run locally.

## Run the M1 simulation demo

This terminal demo requires no display, graphics driver, microphone, API key, or engine build:

```bash
cargo run --locked -p wrong-number-sim --bin dispatch-demo
```

It runs a fixed demonstration at seed 42: investigate six callers, correctly refuse three benign reports, dispatch all three responder types, advance 90 seconds, and validate a save/load round-trip. Expected result: **210 points**, three resolved emergencies, three appropriate refusals, and all units recovered. It prints the timestamped event history. This is a scripted rules demonstration; player-controlled office interactions arrive in M2.

Run only the headless rules tests:

```bash
cargo test --locked -p wrong-number-sim
```

See [simulation rules and API](docs/simulation.md) for command examples, scoring, time semantics, and snapshot boundaries.

## M0 verification

- Fedora 44: `cargo build`, formatting, `cargo check --all-targets`, and Clippy passed with the pinned toolchain and lockfile.
- The M0 build initially had no gameplay tests. M1 adds a dedicated simulation test suite.
- The graphical launch check passed on Mesa llvmpipe (software Vulkan), opening the window and exiting normally after 60 updates.
- The NVIDIA hardware driver on the development machine failed with `NVVM compilation failed` / device lost. Hardware rendering and Windows launch remain unverified.
- Where Mesa's software Vulkan driver is installed, this diagnostic fallback can be used on Fedora (it is slower than hardware rendering):

```bash
VK_DRIVER_FILES=/usr/share/vulkan/icd.d/lvp_icd.x86_64.json WGPU_BACKEND=vulkan cargo run --locked -- --smoke-test
```

CI configuration is committed; platform checks remain pending until GitHub reports their results.

## Source brief

The detailed concept and initial requirements are captured in *Wrong Number: Complete Game Design and Technical Specification* (October 2026). This README turns that proposed specification into an implementation sequence; the specification is a design brief, not a record of completed work.
