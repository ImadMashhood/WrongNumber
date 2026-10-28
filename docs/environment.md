# Dispatch-center environment refinement

The room uses the supplied dispatch-center image and description as a design reference, with the user's requested low-poly detail level. The focus is readable shapes and layout: brick arches, ordinary municipal-office equipment, warm fixtures, close urban facades, and usable supporting rooms.

## Layout and use

- Main dispatch hall: approximately 18 × 14 meters, with a 4.5-meter ceiling.
- Six unnumbered chairs/workstations face one large front city board. Each station has a primary map monitor, secondary status monitor, keyboard, mouse, telephone/handset, headset, task lamp, mug, drawers and papers.
- Four tall arched left windows and two front windows show nearby brick facades, fire escapes, window air conditioners, and selective rooftop equipment. The exterior is lit for daytime, with a blue sky background and no open skyline.
- The right-hand break room is physically accessible through the glass partition. It contains a refrigerator, microwave, coffee machine, cabinets, sink, table/chairs, noticeboard and trash can.
- The secured rear door leads to a separate meeting room with a six-seat table, presentation screen, and projector prop. Its visual state and collision gate derive from the same solo shift phase. It remains locked through live dispatch and response closure and unlocks for the review.

Aim at a dispatch chair and press **E** to pop into its seated camera position. Seated mouse look stays active. Aim at the monitor and press **E** again for computer focus; Escape returns to seated view and Space restores the previous standing pose. Chairs are unnumbered. The wall map opens a read-only overview.

Mouse movement controls the view automatically while walking or seated in the focused window. Escape opens pause or leaves a terminal; UI screens release the mouse cursor. Returning to walking recaptures it and discards the first mouse delta to avoid a view jump. Focus loss releases capture. Arrow keys remain a fallback.

## Implementation and visual budget

`src/environment/` separates shared geometry/material helpers, dispatch hall, break room, meeting room and nearby facades. Most props reuse a cube mesh; round silhouettes use ten-sided meshes. Brick and tile materials use small repeating procedural patterns instead of a mesh for every brick. Only a few local lights cast shadows.

`src/wall_map.rs` draws a live shared map/status board into one render target. The front board and desk monitors reuse it with different UV crops. The simulation still owns incident truth, response commands, unit state and review outcomes; the new room does not introduce another simulation.

Road routing continues to use the explicit training grid while the imported graph is corrected. The room has six physical seats, but network ownership, six connected characters, proximity voice, dynamic environmental events, and a day/night cycle are separate future work. The current outdoor lighting is a fixed daytime setup.

## Verification

The workspace tests cover all six chair approaches, entering the break room, furniture bounds, locked/unlocked review-room access, seated camera restoration, cursor capture gating, and the full shift/review/upgrade logic.

The `--playtest --capture-dir ...` path additionally captures a tour of the dispatch hall, windows, break room, rear door, and unlocked meeting room, then completes the live six-call terminal flow, purchases radios and verifies the next shift. It renders in a hidden window so desktop input cannot disturb the test. Screenshot inspection checks geometry, lighting and readability.

Hardware Vulkan now works on the development RTX 3080. The launcher defaults to hardware, F3 shows frame rate, and `--benchmark` measures the room. The shared board caches its rendered texture between updates; HUD and hover components update only when their values change. Reduced graphics disable shadow maps and use a smaller window.
