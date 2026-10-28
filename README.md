# Wrong Number

**A 1–6 player cooperative emergency-dispatch comedy game built with Rust and Bevy.**

Players work a shift in a busy dispatch office, investigate calls from unreliable callers, and decide when to send scarce response teams. The city remembers those choices: incidents resolve, responders stay occupied, new calls can emerge, and the team reviews its performance before buying upgrades.

The central challenge is sound triage. A refusal can be the right call, while a credible emergency left unanswered can become much worse. Humor comes from callers, player arguments, limited resources, and consequences the game can show and explain.

This repository is the project home. Solo and direct-connect co-op prototypes are playable; delayed consequences, explosions and safe road closures affect the city. M5 adds constrained caller conversation, optional local speech and a separate trusted provider service. Character creation and Steam Workshop support remain future work.

## Project status

**M3/M4 implemented; M5 random-complaint and natural caller conversation prototype implemented and locally verified.** Play a solo dispatch shift in the office, investigate AI callers, assign responders on the supplied coastal map, review outcomes, and buy an upgrade for the next shift. Responders follow speed-weighted Dijkstra routes on the supplied graph inside a connected pilot district; the remaining disconnected map still needs correction. Live caller conversation requires the configured AI service; provider failure drops the conversation. OpenRouter-compatible dialogue and local STT/TTS use bounded adapters. See [caller and speech setup](docs/callers.md). M4 has passed real multi-process and rendered host/client shifts; the two-physical-PC LAN check remains pending. See [multiplayer setup and verification](docs/multiplayer.md). Milestone checkboxes distinguish delivered work from remaining acceptance checks. Scope and technical choices may change as prototypes and playtests provide evidence.

## Design pillars

1. **Triage is the game.** Players investigate incomplete reports and choose whether a response is justified.
2. **The simulation owns the truth.** Caller dialogue can be mistaken or deceptive, but it cannot change an incident's established facts.
3. **Consequences are visible and traceable.** Important simulation events appear in the map, the city, and the shift review.
4. **Coordination matters.** Players share calls and limited response units; simultaneous actions must resolve consistently.
5. **Disconnections stay real.** Lost players leave the room; failed AI connections end the caller conversation without bot or scripted substitution.

## Proposed technical direction

- **Language and engine:** Rust and Bevy for the Windows/Linux PC game.
- **Simulation:** typed, data-driven rules for incident truth, deadlines, unit availability, travel, escalation, scoring, and upgrades. Keep game rules testable without opening a rendered scene.
- **Networking:** server-authoritative shared state over a bounded, versioned TCP/JSON protocol in `crates/net`; reliable commands and public snapshots use background socket workers. See [the M4 decision](docs/multiplayer.md#architecture).
- **Presentation:** a first-person dispatch office, interactive call terminals, a readable city map, and a stylized skyline. Simulate only the city details that affect decisions; present other activity selectively.
- **AI and voice:** optional adapters behind a server-side boundary. The game client must not contain paid-provider credentials. Validate all generated output against known scenario facts and allowed actions.
- **Content:** versioned data files for scenarios and, later, user-created caller packs. Treat imported text and media as untrusted data; never load executable content from a pack.

These are starting choices, not locked dependency versions. Before implementation, record the selected Rust toolchain, Bevy version, networking approach, and supported platform build commands in this README or an architecture decision record.

## Development milestones

Milestones are ordered by dependency. Each ends with a reviewable build or artifact, a short playtest or verification pass, and a decision to continue, revise, or reduce scope. No milestone should be described as complete until its acceptance checks pass.

### M0 — Repository and build foundation

- [x] **Milestone status:** Implemented — Linux launch and Linux/Windows CI verified

**Purpose:** Make changes repeatable and produce a minimal app on target platforms.

**Deliverables**

- [x] Cargo workspace or package structure chosen for the prototype, with a minimal Bevy application that opens and exits cleanly.
- [x] Formatting, linting, logging, error reporting, and a documented development setup.
- [x] Continuous integration that checks formatting, compiles the project, and runs fast automated checks on Linux; add a Windows build check as soon as practical.
- [x] A first architecture note naming crate boundaries, data serialization approach, and the initial networking experiment.
- [x] Placeholder asset policy and licensing notes for any third-party assets.

**Acceptance checks**

- [x] A new contributor can follow the README to install the Rust toolchain and run the app.
- [x] CI gives a clear pass/fail result for the documented checks.
- [x] The selected Bevy and Rust versions build together in CI; incompatible versions are not guessed or left implicit.

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

- [x] **Milestone status:** Implemented and locally verified

**Purpose:** Turn the simulation into a complete, understandable game loop.

**Deliverables**

- [x] A compact navigable office with a player viewpoint, one working desk, call terminal, city map, windows, and a review area.
- [x] Interactions for answering, asking scripted questions, reading clues, dispatching or declining, and checking unit availability.
- [x] Three response types—police, ambulance, and fire—with distinct capabilities and visible busy/available states.
- [x] A compact city with several districts, locations, simple travel times, incident markers, and a stylized skyline.
- [x] A ten-minute shift starting in the break room: delayed incoming calls, incident outcomes, performance grade/review, and one upgrade choice.
- [x] Clear text and visual feedback for action results, incident status, and why score changed.
- [x] Mouse look while standing or seated; E targets a chair, then the monitor; Escape leaves focus without standing, Space stands.
- [x] Circular aim reticle with interaction prompts and locked-door feedback.
- [x] Seeded 1–10 second initial / 1–15 second subsequent call delays, one active conversation per occupied solo workstation, and explicit hang-up.
- [x] Automatic conversation completion after dispatch/refusal; atomic Full Response; caller details/duration, category availability counts, spatial phone ringing and a visual beacon.
- [x] Actual outbound/return road paths with configurable unit speed and 15-second recovery (radio upgrade: 10 seconds).
- [x] Hardware rendering by default, explicit reduced-graphics/software options, F3 frame display, and a reproducible benchmark.

**Acceptance checks**

- [x] A new player can finish a shift without using developer tools or reading source code.
- [x] At least one real and one fake call require different sound decisions.
- [x] Dispatching consumes capacity until the unit returns and recovers.
- [x] The end-of-shift review agrees with the simulation's recorded command and event history.

### M3 — Escalation, city consequences, and replayable content

- [x] **Milestone status:** Implemented — reviewed solo escalation, safe closures, caller variety and timeline

**Purpose:** Make the city react to decisions and create memorable, fair outcomes.

**Delivered so far**

- [x] Delayed, bounded monetary damage driven by fixed scenario truth; benign reports cannot generate damage.
- [x] Appropriate responders stop damage growth on scene; damage events replay and survive save/load.
- [x] Per-incident damage timeline, funding assessment and grade impact in the review.
- [x] Data-driven capability deadlines and bounded escalation rules in `assets/scenarios/m3_rules.json`.
- [x] A solo workload director with active-incident/call budgets, shift difficulty, response availability and late-shift pacing.
- [x] An untreated real fire can explode once, mark the map urgent, trigger a brief room flash/distant boom and queue one linked medical follow-up. At most two blasts per shift with a 90-second cooldown; follow-ups cannot chain.
- [x] Radio/patrol upgrade cycles visibly affect the next shift and remain covered by graphical regression checks.

**Completion deliverables**

- [x] Blast facade damage/smoke and safe timed road closures, with current-position rerouting and reopening; networking synchronization is M4.
- [x] Two additional authored variations per caller template, stable caller history, and a recorded decision/city timeline.

**Acceptance checks**

- [x] Every currently visible consequence traces to a simulation event with a cause and timestamp.
- [x] Escalation chains have cooldowns and caps; the director cannot generate unbounded incidents.
- [x] A prank cannot create a real blast; only fixed emergency/fire truth can trigger the authored event.
- [x] Current review comments use recorded events only. Voice recordings/replays remain M5 work.

### M4 — Cooperative multiplayer vertical slice

- [x] **Milestone status:** Implemented — local 2/6-process shifts and reconnect verified; physical LAN acceptance pending

**Purpose:** Validate that shared decision-making and resource contention work with real players.

**Deliverables**

- [x] Host and client sessions for two players, followed by a test at the six-player target.
- [x] Shared calls and responder pools, call ownership and transfer, player presence, reconnect handling, and clear host authority.
- [x] Reliable network commands for dispatch, decline, transfer, upgrades, and other critical decisions; presentation updates can use suitable lower-priority replication.
- [x] Lobby setup and a simple way to start and end a cooperative shift.
- [x] Network diagnostics for command rejection, latency, disconnects, and authoritative event history.

**Acceptance checks**

- [x] Two players cannot dispatch the same unit at the same time; the host accepts at most one valid allocation.
- [x] All players converge on the same incident truth, unit status, event timeline, and shift result.
- [x] Disconnecting a client does not corrupt the host simulation; reconnect behavior is documented and tested.
- [ ] A whole shift can be completed by two players on separate physical machines (manual LAN check pending; separate OS processes and two rendered clients pass locally).

### M5 — Caller AI and audio, with explicit disconnect handling

- [x] **Milestone status:** Implemented natural conversation prototype — 300 complaints, seeded 10% false-report batches, live openings and local speech verified; microphone listening check pending

**Purpose:** Add flexible conversation while keeping game authority and playability deterministic.

**Deliverables**

- [x] Text conversation first, with a provider-neutral server-side adapter and explicit call disconnection on failure.
- [x] A structured response format for spoken text, emotion, proposed player-visible action, fact references, and call completion.
- [x] Strict validation against caller knowledge and allowed actions; rate, token, timeout, cancellation, and cost controls.
- [x] Optional speech-to-text and text-to-speech behind replaceable adapters, with subtitles and text input available.
- [x] Audio-device settings, push-to-talk, caller volume controls, and basic latency/failure diagnostics.
- [x] Secrets loaded only by a trusted server process from local configuration or deployment secrets, never committed or shipped in a client build.

**Acceptance checks**

- [x] Invalid or unavailable AI output drops the caller conversation without blocking the shift.
- [x] AI output cannot alter truth, dispatch units, spend money or create incidents; evidence IDs are restricted to caller knowledge. Natural-prose consistency uses prompts/live evaluation rather than a formal semantic guarantee.
- [x] Requests time out and can be cancelled; the rendering and simulation loops stay responsive.
- [x] Typed input works without a microphone; live caller conversation requires the AI service (updated user requirement).

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
sudo dnf install gcc-c++ pkgconf-pkg-config alsa-lib-devel libX11-devel libxkbcommon-devel libxkbcommon-x11-devel
```

On Ubuntu/Debian:

```bash
sudo apt-get install build-essential pkg-config libasound2-dev libx11-dev libxkbcommon-dev libxkbcommon-x11-dev
```

On Windows, install Visual Studio Build Tools with the **Desktop development with C++** workload. A working graphics driver is required on both platforms. Linux currently uses X11 or XWayland.

From the repository directory:

```bash
cargo run --locked
```

In offline solo, the shift starts immediately in the break room. Walk behind a dispatch chair, aim at it and press **E** to sit. Look at your monitor and press **E** again to focus dispatch. Calls arrive after 1–10 seconds. **Escape** leaves computer focus while you stay seated; **Space** stands up. Use **F3** for FPS. See [the office guide](docs/office.md) for the complete flow.

On Linux, `bash scripts/run.sh` uses normal hardware rendering and selects the NVIDIA Vulkan driver when available. `bash scripts/run.sh --low-graphics` reduces resolution and disables shadows. `--software` explicitly selects Mesa CPU rendering with reduced graphics; it is a slow diagnostic fallback.

To check startup and exit automatically after 60 updates (requires a display and graphics device):

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

The first compilation downloads and builds the engine and can take several minutes. Logs print to the terminal; set `RUST_LOG=info` for normal startup diagnostics or `RUST_LOG=debug` for more detail. Provider credentials belong only in the trusted caller server; clients do not need them.

See [the architecture decision](docs/architecture.md) for boundaries and networking decisions, and [asset provenance](assets/README.md) before adding external content. The two-platform CI workflow checks formatting, compilation, Clippy, and test targets; Linux CI also runs the graphical shift under Xvfb with software Vulkan.

## Caller conversation and optional voice (M5)

Normal play draws from [300 complaints](docs/design/complaint-catalogue.md), with random reachable addresses, stable caller personas and minor-to-critical severity. One in each ten catalogue-generated complaints is false; genuine minor complaints remain distinct. [Scenario generation and prompt design](docs/caller-catalogue.md) documents the exact policy and evidence.

Answer a call, click **Type a question...**, type and press **Enter**. Caller replies, subtitles and evidence use the same authority rules in solo/co-op. Authored question buttons remain available. Live dialogue requires the configured service; it never falls back to scripted replies. The AI writes an opening statement and natural replies from the selected fixed scenario; the host controls truth and outcomes.

Optional caller speech uses local `espeak-ng` or a replaceable TTS adapter. With a configured local STT adapter, answer the phone and speak naturally. Voice activation sends a local transcription after a short pause; the phone has a mute/unmute button. Optional transcript review lets you edit before pressing Enter. Voice activation is **on during answered calls**; saved audio remains **off by default**. Caller volume/device controls and opt-in local clip replay/deletion are included.

For OpenRouter or another compatible provider, start `cargo run --locked -p wrong-number-caller-server` with a locally configured key and explicit model, then pass `--caller-service 127.0.0.1:47005` to the solo/co-op host. Clients need no key. See [setup, limits, privacy and verification](docs/callers.md); a live GPT-4o request is verified; physical microphone/listening quality still needs playtesting.

## Run co-op (M4)

The host starts a lobby and owns the shared shift. Up to five clients join the host's LAN address; all players need the same commit.

```bash
# Host
cargo run --locked -- --host 0.0.0.0:47004 --name Imad
# Another PC: replace this example with the host's LAN IPv4 address
cargo run --locked -- --join 192.168.1.50:47004 --name Partner
```

On Linux, use `bash scripts/run.sh` followed by the same arguments for hardware-driver selection. Click **Return to office / take a seat**, use E at a free chair and monitor, then let the host click **Start shared shift** in the lobby. Escape → Room lobby opens room controls. Online menus keep the clock running. Shared responders are allocated atomically; calls can transfer to a connected, seated, idle teammate. Only the host ends shifts, buys team upgrades and starts the next day.

See [the multiplayer guide](docs/multiplayer.md) for reconnects, testing and the remaining two-PC acceptance checklist. Direct LAN connectivity is implemented; Steam invitations, relay/NAT traversal and voice remain future milestones.

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
- The original M0 NVIDIA launch failed with `NVVM compilation failed` / device lost. The current M2 build now launches and completes a shift on the RTX 3080 using Vulkan. Windows graphical launch remains unverified.
- Where Mesa's software Vulkan driver is installed, this diagnostic fallback can be used on Fedora (it is slower than hardware rendering):

```bash
VK_DRIVER_FILES=/usr/share/vulkan/icd.d/lvp_icd.x86_64.json WGPU_BACKEND=vulkan cargo run --locked -- --smoke-test
```

The M0/M1 and preceding M2 GitHub CI runs passed on Linux and Windows. M2 adds a Linux software-Vulkan graphical playthrough to CI; results for this revision remain pending until GitHub completes it.

## M2 verification and imported map

- 62 local automated tests pass: 24 simulation rules tests, 7 road/full-response tests, 4 consequence tests, 6 escalation/director tests and 21 office/session/effect tests.
- Formatting, workspace compilation, and Clippy pass. Both hardware/normal-graphics and software/reduced-graphics graphical playthroughs pass locally.
- The graphical playthrough on NVIDIA Vulkan verifies targeted seating and monitor focus, six sequential delayed calls, 210 points, the ten-minute shift boundary, review, a radio purchase, and shorter recovery in the next shift. Screenshots are inspected for room geometry and UI readability.
- The supplied graph has 2,639 nodes, 1,977 roads and 694 disconnected weak components. The playable pilot uses real nodes/roads in its 144-node largest component. No links are invented between components. Road/junction endpoint offsets of at most one pixel per axis are normalized at import. Run `python3 tools/maps/audit_graph.py` after graph edits.
- The 4 km scale, synthetic names and provisional headquarters remain design choices to review. Pilot node placements, recovery, initial/subsequent delay bounds, responder speeds and reticle colors live in `assets/city/gameplay_config.json`. Incident deadlines, damage/escalation rules and workload budgets live in `assets/scenarios/m3_rules.json`. Run upgrades last until the app closes.

See [the office guide](docs/office.md) for controls, map integration boundaries, and the graphical test command.

## Dispatch-center environment pass

The office now follows the supplied environment reference: a deeper brick industrial room, tall arched windows looking onto close neighboring buildings in daylight, six unnumbered workstations with simple recognizable equipment, a large live city board, a glass-fronted break room, and a separate rear meeting room. Shared low-poly meshes and small repeating brick/floor textures keep the detail restrained.

Mouse look works while walking and seated. Aim at a chair and press E to sit, then aim at the monitor and press E to use dispatch. Escape returns to seated free-look; Space stands. The rear meeting-room door stays locked during the shift and opens when review becomes available. The wall map and desktop monitors share a live render target driven by the existing simulation.

The graphical verification tours the rebuilt room and completes the existing shift/review/upgrade flow. M4 adds simple networked dispatcher avatars; proximity voice remains future work. Road routes now drive the pilot district. The launcher no longer forces CPU rendering. The same 1440×900 room benchmark measured 58.5 FPS on the RTX 3080 versus 3.0 FPS on CPU llvmpipe (300 measured frames after 90 warm-up frames, normal graphics, VSync). This is a single-machine scene measurement. See [the environment guide](docs/environment.md).

## Source brief

The current [gameplay specification PDF](docs/design/gameplay-loop.pdf) and its [exact supplied source](docs/design/gameplay-loop-source.txt) are checked into this repo. [The review log](docs/reviews/gameplay-loop.md) records the M2 mismatch and corrections, and must be updated after each milestone and gameplay-changing commit (see `AGENTS.md`). Explicit user directions override the brief: unnumbered chairs, pop-in E seating, daylight, low-poly graphics.

The detailed concept and initial requirements are captured in *Wrong Number: Complete Game Design and Technical Specification* (October 2026). This README turns that proposed specification into an implementation sequence; the specification is a design brief, not a record of completed work.

### Desktop and phone

The computer displays resource availability, cash, projected shift income and damage deductions. Incoming calls open a separate phone window with a caller portrait and conversation. Answer to connect, speak naturally or type, dispatch from the phone, and use its mute or hang-up controls. Hanging up removes the window. Launch through scripts/run-with-ai.sh with your local STT adapter; the task’s normal output launcher already supplies it. Provider replies have a 20-second HTTP budget so the old four-second cutoff no longer disconnects slow openings.
