# Gameplay-loop review log

Reference: [saved specification](../design/gameplay-loop.pdf), with the [unaltered source](../design/gameplay-loop-source.txt). The supplied document ends mid-sentence in section 11.2. Explicit later user directions override its numbered-chair and smooth-seating suggestions.

Review each milestone and each gameplay-changing commit. Record the section, observed behavior, evidence and any gap before deciding to advance. An in-scope failure blocks the next milestone; a future milestone feature remains unchecked and named. Reviews use the running game and authority tests, not documentation claims alone.

## M2 re-review - 2026-10-27

Initial verdict: **did not match the updated M2 baseline**. The previous M2 claim covered the original solo milestone but omitted road routes, automatic call completion, Full Response, the specified recovery range, full call information and ongoing caller delivery. Fix M2 before starting M3.

| Specification | M2 requirement and implementation evidence | Disposition |
| --- | --- | --- |
| 2.1-2.2 | Standing break-room spawn; 600-second solo countdown continues during movement and computer focus. Explicit solo pause remains available. | Test and rendered-playthrough gate |
| 2.3, 3.1-3.5 | Six unnumbered chairs, mouse look, ray/range/occlusion targeting, circular reticle, separate seating/computer focus, stand/release. Pop-in seating follows the user's override. | Test and rendered-playthrough gate |
| 4.1-4.4 | Shared city board, permanent headquarters/hospital markers, visible incident/unit pins, Dijkstra paths using road distance, road speed, direction, closure and unit speed. Units follow route polylines outbound and home. | Road tests and rendered-playthrough gate |
| 4.2, 4.4 | Provisional pilot locations are real nodes in the supplied largest connected district. Declared road/junction endpoints normalize at most one pixel per axis; no disconnected components are joined. The rest of the draft graph remains unapproved for dispatch. | Explicit prototype boundary, not citywide completion |
| 5.1-5.2 | First call delay 1-10 seconds; one live solo line; ringing telephone spatial audio, a physical flashing beacon and HUD/terminal notification. Missing audio devices and `--mute` retain visual/text playability. | State tests and rendered-playthrough gate; device listening remains manual |
| 6.4 | Fictional caller name/identifier, reported street/junction, investigation state, call duration, evidence notes, call state and available resources. Hidden truth stays in the simulation. | UI and authority gate |
| 7.1-7.3, 9.4-9.5 | Individual responders or atomic Police/Fire/Medical Full Response; unavailable categories shown; no silent partial allocation; successful dispatch/refusal ends the conversation. Failed dispatch keeps it open. | Authority and session tests |
| 9.2-9.3 | Travel, handling, real return travel, then configurable recovery (15 seconds; radio upgrade 10 seconds). | Road/lifecycle and next-shift tests |
| 10.1 | Subsequent calls after configurable 1-15 seconds; seeded authored scenarios repeat while dispatch remains open, with a 96-incident prototype bound. No new delivery after 600 seconds. | Session tests |
| 1, 7.3 | Recorded outcomes, grade/funding, review room and upgrade/next-shift cycle. Early-finish shortcut closes delivery before advancing outstanding responses. | Full graphical shift gate |
| 8, 10.3, 11 | Monetary damage, delayed escalation, adaptive workload and major city audio/visual events. | M3, not claimed by M2 |
| 2.2-2.3, 4.4, 10.2 | Network-synchronized timer, exclusive shared chair claims, simultaneous conversations, replicated resources and unit positions. | M4, not claimed by solo IDs 1-6 |
| 5.3, 6.1-6.3 | Natural AI/microphone conversations, STT/TTS and caller audio/recordings. | M5; scripted question fallback is M2 |
| 3.2, 11.1 | Full accessibility settings and extensive ambient city soundscape. | Configurable reticle is M2; expanded settings M6, ambience M3 presentation |

Final verdict: **PASS for the corrected solo M2 baseline**. The full game specification is still a multi-milestone target; the table above names its remaining owners.

Evidence: 46 automated tests pass (17 office/session, 23 simulation rules, 6 road/full-response). Formatting and strict Clippy pass. The NVIDIA graphical playthrough targets a chair and monitor, clicks Full Response, handles six calls for 210 points, displays live road paths, opens the review, purchases radios and verifies 10-second recovery next shift. Captures were inspected for readable caller details, availability counts and route alignment. Repeat-call and failed/atomic dispatch behavior have dedicated tests. A 300-frame normal-graphics benchmark on the RTX 3080 averages 58.3 FPS at 1440x900 (p95 18.43 ms), following 90 warm-up frames. Spatial sound is wired through Bevy; physical-device listening remains a manual check, with mute/no-device fallbacks.

The 14-page PDF was rendered and inspected. Extracted text contains every supplied source line; the preserved source is byte-for-byte identical to the attachment. M3 is now authorized to start automatically. Its initial work is the deterministic monetary consequence ledger, not a claim that all M3 escalation/effects are finished.


## M3 initial consequence slice - 2026-10-27

Verdict: **PASS for this slice; M3 remains IN PROGRESS**. M2 was reviewed and committed first (`c7f5b35`), then this work began automatically under the user's standing instruction.

| Specification | Implemented and reviewed | Remaining work |
| --- | --- | --- |
| 6.2, 8.1-8.3 | Fixed truth drives delayed per-second monetary damage; caps bound losses, harmless reports stay at zero, pending calls are unaffected, and suitable on-scene responders stop growth. | Data-driven severity variants, city conditions and escalation chains. |
| 8.2, 8.4 | `DamageApplied` records incident, cause, amount, cumulative total and timestamp; snapshots validate the ledger. Live dispatch and review show actual recorded totals. | Major city audio/visual feedback, closures and secondary calls. |
| 1, 7.3, 8.1 | Review groups each incident's damage interval and explains its funding/grade assessment. | Broader balance and replay variety. |
| 10.3 | The M2 seeded repeating caller loop retains its explicit 96-incident bound. | Adaptive severity/volume budgets by players, resources, city state and remaining time. |

Verification: 51 automated tests pass (18 session/office, 23 original rules, 6 road/full-response and 4 consequence tests); strict Clippy and formatting pass. The rendered consequence scenario records $127,500 from a declined credible fire, 100 decision points, a 127-point damage assessment, grade F and 73 funding, then purchases radios and starts the next shift with 23 funding. The ordinary 210-point/310-funding graphical shift remains a regression check. Damage/review screenshots were inspected for readable evidence and arithmetic. The post-change benchmark averages 58.4 FPS at 1440x900 on the RTX 3080 (p95 17.91 ms, 300 measured frames after 90 warm-up frames). The next M3 work is authored escalation/secondary-call presentation and workload budgets; no M4 transition is approved by this slice alone.


## M3 workload and authored escalation - 2026-10-27

Verdict: **PASS for this slice; M3 remains IN PROGRESS**. Continue M3; do not advance to M4 yet.

| Specification | Reviewed behavior and evidence | Remaining work |
| --- | --- | --- |
| 6.2, 8.2-8.3 | Validated JSON fixes capability deadlines and consequence profiles before calls. An untreated parentless real fire can emit one structure-explosion event after 120 seconds; benign/fire prank truth never qualifies. Appropriate on-scene fire response prevents it. | More authored severity/scenario variations. |
| 8.4, 11.2 | The same timestamped event drives an urgent red map pin, an office flash and original spatial boom with distance attenuation. Fast-forwarded/closing history does not replay effects; mute/reduced-effects flags are available. Rendered captures show the office and urgent source marker. | Skyline destruction, runtime road closures and broader city sound/effects. Physical-device audio listening remains manual. |
| 8.4, 10.1-10.3 | A single fixed-truth medical follow-up links to its source, stays hidden until delivery and waits for line/capacity before taking priority over ordinary calls. Children cannot escalate. Two-event global cap, 90-second cooldown and matching reserved follow-up slots prevent unbounded chains. | More secondary-call types and multiplayer integration. |
| 10.3 | Solo workload defaults to four active incidents and 32 calls, scaling by shift to six active/64 calls. Delay bounds respond to active incidents, busy responders and late-shift time; seeded template weights consider availability/difficulty while retaining scarcity. The terminal displays the active budget. | Player-count scaling belongs to M4; broader balance still needs human playtesting. |
| 1, 8 | Review records blast time, source and linked call, with the existing damage ledger/funding arithmetic. Normal shift/review/radio/next-shift regression still passes. | Complete replayable caller histories and more content. |

Verification: 60 automated tests pass (21 office/session/effect, 23 original rules, 6 road/full-response, 4 consequence, 6 escalation/director), with formatting and strict Clippy. New tests cover rule validation/atomicity, configured deadlines in queued and immediately ringing starts, deterministic escalation/save replay, source-child validation, caps/cooldown, prank prevention, director budgets, deferred line delivery and follow-up priority.

All three NVIDIA graphical paths pass: ordinary shift (210 points/310 funding), consequence shift ($137,500 damage including the $10,000 burst, 100 points/63 funding, then radios and 13 funding next shift), and live escalation/flash/map/snapshot. The escalation path also passes on software Vulkan with reduced graphics. Screenshots were inspected for readable source/follow-up evidence. The final normal-graphics benchmark averages 58.4 FPS at 1440x900 on the RTX 3080 (p95 18.73 ms, 300 measured frames after 90 warm-up frames). M3 remains open for the remaining scope listed above.


## M3 completion gate - 2026-10-27

Verdict: **PASS for the M3 solo vertical slice; proceed to M4 as requested**.

- Specification 8.4/11: a recorded blast produces low-poly exterior scorch/fire/smoke, office flash, spatial boom, urgent map marker and a causally linked medical call.
- Specification 4.4/8.4: the authority chooses a nearby non-bridge road, rejects closures that could trap a unit or disconnect the district, preserves positions while recalculating live routes, displays the closure and reopens it after 120 seconds. Source graph data remains intact. Dedicated tests cover atomic rejection, rerouting, connectivity, snapshot restoration and reopening.
- Specification 6/10: authored repeat variations preserve truth/capability and stable caller identity; the workstation shows caller history. Decision, dispatch, arrival, outcome, closure and reopening entries come from recorded events; repeated damage is grouped in its ledger.
- Verification: 62 tests, formatting and strict Clippy pass. Normal, consequence and live escalation graphical paths pass. Facade damage and closed-road captures were inspected. RTX 3080 benchmark: 58.5 FPS, p95 17.71 ms at 1440x900, 300 measured frames after 90 warm-up frames.
- Scope: the styled exterior uses the adjacent urban facades, not a newly simulated 3D city. Live replication, shared seats and simultaneous lines belong to M4; voice/AI and recordings remain M5.

## M4 co-op implementation gate — 2026-10-28

Verdict: **PASS for the implemented local co-op vertical slice. Two-physical-PC LAN acceptance remains unchecked.** M3's completion gate passed before beginning M4. This review does not authorize or claim M5 voice/AI work.

| Specification | Reviewed behavior and implementation evidence | Disposition |
| --- | --- | --- |
| 1, 2.1–2.3 | Up to six players join one room, stand in the break room, claim exclusive unnumbered chairs and start a host-controlled 600-second shift. Subsequent shifts reset poses/desks into the next lobby. | Room tests; 2/6-process complete shifts; two rendered processes |
| 3.1–3.5, user overrides | E pop-in seating, mouse aim and seated look remain; remote low-poly avatars show standing/seated presence; online Escape menus do not pause authority. A clear lobby Return to office button restores movement/focus. | Geometry/seat/pause tests and rendered avatar/lobby capture |
| 4.4, 8.2–8.4 | One host owns timer, roads, unit routes, resource status, score, damage, explosions and temporary safe closures. Public mirrors reject command execution, clock advancement and authority saves. The rendered wall map, facade effects and review consume replicated events. | Public projection test includes blast, damage and road closure; cross-process public digest comparisons |
| 5.1–5.2, 6.4 | Independent per-player line scheduling, workstation beacons/spatial rings, caller notes and case selection. Pending caller content, truth and unrevealed answers are absent from the public protocol. | Independent-line/ownership/private-projection tests; simultaneous client process play |
| 7.1–7.3, 9.4–9.5 | Shared responder pool; explicit Send/Cancel/Reroute requests include observed assignment. Host rejects stale allocations, duplicate side effects, old-shift commands and unauthorized purchases. Full Response stays atomic. Idle seated teammates can receive an owned conversation and existing evidence. | Race, duplicate, host-role, transfer and simulation tests |
| 9.2–9.3, 10.1–10.3 | Real outbound/home routes and recovery remain authoritative. Workload scales by connected players while preserving capped active incidents, total calls and reserved follow-ups. Disconnect releases chair; unanswered ringing calls are reclaimable without changing deadlines; answered calls close while cases/responses persist. Same-room reconnect restores ID/sequence. | Orphan-line/movement tests and six-process kill/restart playtest |
| 1, 7.3 | Shared review, grade, funding and host-only team purchases; next shift returns everyone to the same lobby and public state. | Complete headless and rendered review/upgrade/next-day paths |
| 5.3, 6.1–6.3 | Microphone conversation, natural AI callers, STT/TTS, proximity voice and recordings. | Remain future work; M4 uses authored questions/text and telephone audio |

Verification: **73 tests pass**, including six room-authority tests, two public-projection tests and two actual TCP transport tests. Formatting, `cargo check --locked --workspace --all-targets`, and strict Clippy pass. The real-process harness passes a two-player shift and a six-player shift with a client terminated/restarted mid-shift; all peers converge on the same next-lobby public simulation/phase/funding/upgrade digest. Two rendered Bevy processes pass the actual lobby Start button, shared-mesh avatar presence, full shift, review and next-lobby checks; lobby, avatar, host review and next-day screenshots were inspected.

Offline ordinary, monetary-consequence and live blast/closure graphical regressions all pass. The normal-graphics RTX 3080 room benchmark remains **58.5 FPS**, p95 **17.72 ms** at 1440×900, 300 measured frames after 90 warm-up frames. This measures the solo fixed-camera scene, not six concurrent rendered windows or a remote-PC session. Spatial sound is implemented; physical-device listening remains manual.

Boundary: direct trusted-LAN TCP/JSON, versioned protocol, bounded queues/frame/rate limits and worker-thread socket I/O. No Steam relay/invites, internet NAT traversal, host migration or persistent rooms are claimed. Local separate OS processes do not substitute for a human test on two physical machines. The exact LAN checklist is retained unchecked in the README and `docs/multiplayer.md`; Linux/Windows CI now includes headless two-player and six-player reconnect process checks. Current local evidence is Linux/NVIDIA; new remote CI outcomes must be checked after push.

The repeated six-player stress run caught a boundary error in existing routing: expiry/escalation can run after the authority clock advances but before that tick updates a moving unit's stored position. Choosing the road at the newer time could select the far side of a junction and make a legitimate return path fail. Return and closure replanning now choose the source road at the timestamp that produced the stored position. A dedicated sub-second-junction expiry regression verifies the responder returns safely and its snapshot validates. This repair belongs to the M4 integration gate and does not change the supplied graph or introduce fallback links.
