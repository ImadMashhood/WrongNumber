# M2 solo office

## Play a shift

Run `cargo run --locked` from the repository. The app starts in the office in preparation mode; timers wait until you begin the shift.

- WASD: walk. Hold Shift to move faster.
- Mouse movement looks around automatically while walking; arrow keys remain a fallback. The cursor releases at terminals, reviews, pause menus, or when the window loses focus.
- E: interact with the nearby dispatch desk, wall map, or review table.
- Escape: leave a terminal; in the office it opens the pause menu. Escape again resumes. Closing the window or the pause menu's Quit button exits.

Walk up behind any dispatch chair and press E to sit at that workstation and use its computer. Leaving the console restores your standing position. Then select **Begin shift**. Select a numbered call, answer it, ask the available question, and choose a responder or decline. Revealed answers stay in the case notes. All controls go through the M1 authority command validation.

Police, medical, and fire teams have distinct capabilities. A unit already assigned/en route can be cancelled when its call is selected, or rerouted when another answered call is selected. A busy on-scene, returning, or recovering unit cannot be reused. The map and roster display its status and ETA.

Time continues while using the terminal and walking around. The pause menu freezes it. A 3x clock option helps playtesting. **Advance to shift end** runs all remaining deadlines and recovery time using the same simulation rules; it is a deliberate shortcut with normal missed-emergency penalties.

When all incidents have concluded and responders have recovered, or the 330-second maximum shift window expires, return to the office. Enter the rear meeting room after its secured door unlocks. Press E to see recorded outcomes, score explanations, and funding. Funding is `max(0, 100 + shift score)` and is credited once per shift.

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

`cargo test --locked --workspace` checks simulation rules and the solo shift lifecycle, including preparation/pause, complete review/purchase/next-shift flow, additional patrol, gated purchases, early finishing, and collision boundaries.

The graphical playthrough tours the main room, arched windows, break room and rear door, then drives the same enabled button observers used by the terminal:

```bash
cargo run --locked -- --playtest --capture-dir /absolute/path/to/screenshots
```

It investigates six calls, handles them correctly, verifies 210 points, enters the review area, buys radios, starts the next shift, and verifies 2-second recovery. It validates that the map asset loaded, saves room-tour, terminal, meeting-room, review, and next-shift screenshots, then exits. The automated window is hidden and ignores physical input so the test does not interfere with work on the desktop. `--smoke-test` remains a shorter 60-frame office startup check.

On the development machine, use the Mesa Vulkan fallback documented in the README to bypass the NVIDIA initialization failure. Software rendering is intended for validation; performance on a working hardware driver must still be checked.

When running the compiled binary directly from a source checkout, set `BEVY_ASSET_ROOT` to the repository directory. `cargo run` already supplies the source asset root. Packaged builds must put `assets/` beside the executable (release packaging is future work).
