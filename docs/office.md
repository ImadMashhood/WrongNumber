# M2 solo office

## Play a shift

Run `cargo run --locked`, or on Linux `bash scripts/run.sh`. You spawn standing in the break room and the 10-minute countdown starts immediately.

- WASD walks; Shift moves faster. Mouse movement looks around while standing or seated; arrow keys are a fallback.
- Aim at a chair and press **E** to sit. Chairs remain unnumbered.
- Aim at your monitor and press **E** to focus dispatch. E on a ringing telephone opens dispatch and answers. Left-click also interacts during room view.
- **Escape** leaves computer focus without standing. In room view, Escape opens the solo pause menu and releases the cursor. Escape again resumes.
- **Space** stands up from seated room view, restoring your previous standing position.
- **E** also targets the wall map, coffee machine, review door and conference table. The circular reticle turns green for usable targets, red for the locked review door.
- **F3** toggles FPS and graphics-adapter information.

Sitting schedules a caller after a seeded random delay of 1–10 seconds. Only one ringing/active conversation is delivered at a time. Undelivered calls and their locations remain hidden. A call stays attached to its original workstation if you stand up; return to that seat to answer it. Leaving computer focus does not stop calls or the timer. The phone rings spatially and flashes a beacon. `--mute` suppresses sound; a missing audio device retains visual/text playability. Microphone/AI conversations remain M5 work.

Answer, ask the scripted question, then assign a responder or decline. Successful dispatch/refusal automatically ends the conversation. Failed dispatch keeps it open. **Send Full Response** atomically reserves one Police, Medical and Fire unit, or reserves none; the UI lists unavailable categories. **End conversation** remains an optional hang-up without a final decision. Questions and refusals require your active conversation; previously investigated open incidents can still receive responders after hang-up. Police, medical and fire teams have distinct capabilities. Assigned/en-route units can be cancelled or rerouted; on-scene, returning and recovering units cannot be reused.

After each conversation ends, another authored caller arrives in 1–15 seconds. Authored scenarios repeat beyond the initial six, within a 96-incident prototype bound. Caller name/identifier, reported street/junction, reported incident type, call duration, notes, status and resource counts appear in dispatch. Scroll individual panels with the mouse wheel.

The timer keeps running throughout room movement and all computer/map screens. Only explicit solo pause stops it. **Finish shift early / skip remaining time** is a playtest convenience: it first closes delivery, then advances the remaining time with normal deadlines and penalties for received calls. At 10 minutes, no more calls are delivered; existing incidents and responder returns finish before the rear door unlocks. Calls never delivered are recorded without a penalty and do not appear as received cases.

Enter the unlocked rear meeting room and aim at the conference table to review the grade, score explanations and funding. Funding is `max(0, 100 + score - floor(actual damage / 1000))`, credited once. The damage assessment also reduces the score used for grading. A shift with no handled calls receives grade F.

Buy at most one upgrade per review, or continue without buying:

- **Improved radios ($50):** responder recovery falls from 15 seconds to 10 seconds next shift.
- **Additional patrol ($90):** adds a fourth response unit (second police team) next shift.

Next shift keeps funding and purchased upgrades for this run, resets the score, and reuses the authored training incident positions. Closing the app ends the run; persistent progression is M6 work. Scenarios repeat from six authored reports at provisional connected road nodes.

## Map and station integration

The supplied coastal image remains unchanged. Headquarters/hospital and call positions are actual road nodes selected in `assets/city/gameplay_config.json`. They lie in the largest supplied connected component (144 nodes); the full import has 2,639 nodes, 1,977 roads and 694 weak components. Other components are not silently joined or dispatched into.

`crates/simulation/src/roads.rs` expands each road polyline into directed edges and uses Dijkstra with estimated road length, speed limit, closed status and configurable responder speed. A multiplier of 1000 means 1x; police default to 1250 (1.25x). The same authority path drives outbound movement, cancellation/rerouting from the current road position, and return travel. Colored route overlays share one cached transparent image. Headquarters pins remain visible when units leave.

The tracer's declared junctions can differ from their polyline endpoints by one pixel per axis. Import explicitly joins only these declared endpoints within that tolerance; larger errors are rejected. No links across disconnected components are generated. Raw graph/image data is preserved. Road scale, names, addresses and headquarters placements remain provisional art/design choices; inspect them before expanding citywide dispatch.

Configurable recovery starts after actual return travel: 15 seconds by default, 10 with radios. Travel and scene handling add to total unavailability. `python3 tools/maps/audit_graph.py` repeats the raw graph audit. A unit with no route is rejected before reservation.

## Verification

`cargo test --locked --workspace` checks simulation rules and the solo shift lifecycle, including automatic countdown/pause, seeded arrival delays, single-call delivery, complete review/purchase/next-shift flow, additional patrol, gated purchases, early finishing, and collision boundaries.

The graphical playthrough tours the main room, arched windows, break room and rear door, then drives the same enabled button observers used by the terminal:

```bash
cargo run --locked -- --playtest --capture-dir /absolute/path/to/screenshots
```

It investigates six calls, handles them correctly, verifies 210 points, enters the review area, buys radios, starts the next shift, and verifies 10-second recovery. It validates that the map asset loaded, saves room-tour, terminal, meeting-room, review, and next-shift screenshots, then exits. The automated window is hidden and ignores physical input so the test does not interfere with work on the desktop. `--smoke-test` remains a shorter 60-frame office startup check.

The Linux launcher uses hardware rendering by default. The old launcher forced Mesa llvmpipe, rendering on the CPU and causing very low FPS. Use `--low-graphics` for 960×600 and no shadows; reserve `--software` for diagnosing a GPU problem.

`cargo run --locked -- --benchmark` renders the same room overview, skips 90 warm-up frames, measures 300 frames, prints average FPS and 95th-percentile frame time, then exits. Normal graphics use 1440×900 and the default VSync presentation mode. This is a reproducible scene sample, not a guarantee for every view, GPU or multiplayer session.

When running the compiled binary directly from a source checkout, set `BEVY_ASSET_ROOT` to the repository directory. `cargo run` already supplies the source asset root. Packaged builds must put `assets/` beside the executable (release packaging is future work).

## M3 first consequence slice

Untreated genuine emergencies accrue abstract monetary damage after an authored delay. Fire starts after 45 seconds ($500/second, cap $250,000), medical after 75 seconds ($250/second, cap $25,000), and police after 60 seconds ($100/second, cap $15,000). These are gameplay balance values. Harmless scenarios produce zero damage. Appropriate responders stop growth on scene; resolved/expired incidents stop accumulating. Declining does not charge damage immediately, so later calls can overlap with an earlier incident's consequences.

The review groups recorded damage by incident and shows first/last damage timestamps, total and funding assessment. No speculative future damage is shown. Major explosions, secondary incidents, road-closure effects and adaptive workload remain unfinished M3 work.

Run `cargo run --locked -- --consequence-test --capture-dir /absolute/path` for the deterministic graphical damage scenario. It declines a credible fire, handles the other cases, verifies $127,500 damage, 100 decision points, 73 funding, and the review/upgrade/next-shift flow. Ordinary `--playtest` still verifies the undamaged 210-point/310-funding flow.
