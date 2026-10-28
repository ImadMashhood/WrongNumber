# M2 solo office

## Play a shift

Run `cargo run --locked`, or on Linux `bash scripts/run.sh`. You spawn standing in the break room and the 10-minute countdown starts immediately.

- WASD walks; Shift moves faster. Mouse movement looks around while standing or seated; arrow keys are a fallback.
- Aim at a chair and press **E** to sit. Chairs remain unnumbered.
- Aim at your monitor or telephone and press **E** to focus the dispatch interface.
- **Escape** leaves computer focus without standing. In room view, Escape opens the solo pause menu and releases the cursor. Escape again resumes.
- **Space** stands up from seated room view, restoring your previous standing position.
- **E** also targets the wall map, coffee machine, review door and conference table. The circular reticle turns green for usable targets, red for the locked review door.
- **F3** toggles FPS and graphics-adapter information.

Sitting schedules a caller after a seeded random delay of 1–10 seconds. Only one ringing/active conversation is delivered at a time. Undelivered calls and their locations remain hidden. A call stays attached to its original workstation if you stand up; return to that seat to answer it. Leaving computer focus does not stop calls or the timer. The phone currently has a visual/text notification; audible spatial ringing and microphone/AI conversation are future audio work.

Answer, ask the scripted question, then assign a responder or decline. **End conversation** frees the line for the next caller while the incident and any dispatched unit continue. Questions and refusals require your active conversation; previously investigated open incidents can still receive responders after hang-up. Police, medical and fire teams have distinct capabilities. Assigned/en-route units can be cancelled or rerouted; on-scene, returning and recovering units cannot be reused.

The timer keeps running throughout room movement and all computer/map screens. Only explicit solo pause stops it. **Finish shift early / skip remaining time** is a playtest convenience: it advances the remaining time with normal deadlines and penalties. At 10 minutes, no more calls are delivered; existing incidents and responder returns finish before the rear door unlocks. Calls never delivered are recorded without a penalty and do not appear as received cases.

Enter the unlocked rear meeting room and aim at the conference table to review the grade, score explanations and funding. Funding is `max(0, 100 + score)`, credited once. A shift with no handled calls receives grade F.

Buy at most one upgrade per review, or continue without buying:

- **Improved radios ($50):** responder recovery falls from 5 seconds to 2 seconds next shift.
- **Additional patrol ($90):** adds a fourth response unit (second police team) next shift.

Next shift keeps funding and purchased upgrades for this run, resets the score, and reuses the authored training incident positions. Closing the app ends the run; persistent progression is M6 work. Scenarios are currently the same six authored reports, at the authored training positions.

## Map and station integration

The M2 screen uses `assets/city/map.png` from the supplied road-map starter. It overlays authored training incidents and distinct police, hospital, and fire station pins, with live responder squares. `src/city.rs` holds those temporary placements. M2 still uses integer-grid X-then-Y travel at one cell per second; roads do not drive the responder paths yet.

The imported graph has **2,639 nodes, 1,977 roads, and 694 disconnected weak components**; its largest component has 144 nodes. Endpoint references are valid, but geometry and connections are not ready for citywide routing. The package explicitly marks its road tracing, synthetic addresses/names, 4 km width, and jurisdiction as placeholders. Station placements are training choices, not confirmed headquarters locations.

Run `python3 tools/maps/audit_graph.py` to repeat the connectivity and field audit after editing the data. The original tracing script and its README are under `tools/maps`; its correction manifest is not automatically applied by that script. The PNG can be replaced at the same path; restart the game to reload it. Next shifts currently reuse these authored training locations and six scenarios; a varied director remains M3 work.

For that integration, agree on map scale and station locations first. A proposed road-data format should contain stable node/road IDs, named edges, road geometry, length in meters, permitted direction, and station/incident connections to reachable nodes. That schema and pathfinding implementation remain future work.

## Verification

`cargo test --locked --workspace` checks simulation rules and the solo shift lifecycle, including automatic countdown/pause, seeded arrival delays, single-call delivery, complete review/purchase/next-shift flow, additional patrol, gated purchases, early finishing, and collision boundaries.

The graphical playthrough tours the main room, arched windows, break room and rear door, then drives the same enabled button observers used by the terminal:

```bash
cargo run --locked -- --playtest --capture-dir /absolute/path/to/screenshots
```

It investigates six calls, handles them correctly, verifies 210 points, enters the review area, buys radios, starts the next shift, and verifies 2-second recovery. It validates that the map asset loaded, saves room-tour, terminal, meeting-room, review, and next-shift screenshots, then exits. The automated window is hidden and ignores physical input so the test does not interfere with work on the desktop. `--smoke-test` remains a shorter 60-frame office startup check.

The Linux launcher uses hardware rendering by default. The old launcher forced Mesa llvmpipe, rendering on the CPU and causing very low FPS. Use `--low-graphics` for 960×600 and no shadows; reserve `--software` for diagnosing a GPU problem.

`cargo run --locked -- --benchmark` renders the same room overview, skips 90 warm-up frames, measures 300 frames, prints average FPS and 95th-percentile frame time, then exits. Normal graphics use 1440×900 and the default VSync presentation mode. This is a reproducible scene sample, not a guarantee for every view, GPU or multiplayer session.

When running the compiled binary directly from a source checkout, set `BEVY_ASSET_ROOT` to the repository directory. `cargo run` already supplies the source asset root. Packaged builds must put `assets/` beside the executable (release packaging is future work).
