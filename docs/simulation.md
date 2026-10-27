# M1 simulation

`crates/simulation` is a standalone Rust library with Serde/JSON dependencies and no Bevy dependency. The existing 3D app is not wired to it yet. CI checks and tests all workspace packages.

## Authority and commands

Create a shift with `Simulation::new(seed)`. Only explicit calls to `advance(seconds)` advance the clock. Use `apply(player_id, command)` for typed commands or `apply_json(player_id, json)` for a bounded JSON payload. Valid player IDs are 1–4. Rejected commands leave time, score, units, facts, and event history unchanged.

Commands:

```json
{"AcceptCall":{"incident":1}}
{"Ask":{"incident":1,"fact":"smoke"}}
{"Dispatch":{"unit":3,"incident":1}}
{"Cancel":{"unit":3}}
{"Reroute":{"unit":3,"incident":4}}
{"Decline":{"incident":2}}
```

Accept before investigating, dispatching, or declining. Only the answering player can investigate or decline; a teammate can dispatch. Transfer/ownership UI is future work. A unit must be available and capable of handling that incident. A scenario accepts one active responder in M1. Cancel and reroute work only before arrival. Declining an assigned incident is rejected until the responder is cancelled. A declined genuine emergency keeps running toward its deadline; it does not become benign.

The authority serializes commands in order. This prevents two requests from acquiring the same unit. M1 tests this rule with competing player commands; it does not implement network concurrency or transport.

`call_view(id)` exposes the report, location, state, available questions, and only revealed answers. Questions become available after answering. It excludes hidden truth and scenario category. `incidents()` and `save_json()` are authority/debug access, containing hidden information; never replicate them wholesale to clients or pass them to caller models.

## Clock and responder lifecycle

- Integer seconds; each advance accepts 1–3600 seconds, with a current prototype clock limit of 86,400 seconds.
- Calls begin together at t=0. A paced director is M3 work.
- Seeded scenario locations and ordered maps give repeatable results; splitting an advance into smaller advances does not change the result.
- Travel follows a Manhattan grid at one cell per second, X then Y. It is deliberately simple, with no road graph, traffic, or costs yet.
- Lifecycle: available → assigned → en route → on scene → resolving → returning → recovering → available.
- Assignment starts travel on the next tick. Arrival spends a tick on scene, then starts service. Return and recovery consume real simulation time, including after cancellation.
- A rerouted unit keeps its current grid position. The old incident remains active.
- Deadlines are exclusive: at the deadline the incident expires before any resolution scheduled for that same second. Associated responders return and recover.

## Scoring and review

| Outcome | Points |
| --- | ---: |
| Emergency resolved before deadline | +50 |
| Benign report refused after gathering benign evidence | +20 |
| Benign report refused without clear evidence | +5 |
| Benign report consumes a full responder deployment | -15 |
| Credible emergency expires after danger was revealed | -60 |
| Emergency expires without revealed danger | -5 |
| Unanswered benign report expires | 0 |

The lower penalty for undiscovered danger limits hindsight-only punishment. These are initial balance values, not final tuning. Uninvestigated refusals receive a smaller reward to encourage investigation. No refusal bonus is awarded twice, and a unit returning from an already concluded incident cannot score it again.

`review()` derives a typed result containing the current score, incident outcome counts, available units, and per-incident score explanations from the event history. Events have monotonic sequence numbers and timestamps. City effects and graphics will consume these events in later milestones.

## Saves

`save_json()` writes schema version 1, including the clock, seed, incidents, facts, unit states, score, and event history. `load_json()` rejects unknown versions, oversized input (1 MiB), malformed data, invalid references, duplicate assignments, inconsistent travel/state data, invalid event sequences, and inconsistent outcome/score totals. Deserializing `Simulation` directly also runs invariant checks.

Snapshots belong to the trusted simulation authority. They are not signed, cheat-proof saves or a client import format. Malicious replacement of otherwise coherent scenario facts is outside this local snapshot format's guarantees. Mod importing and network messages require their own boundaries in later milestones.

The automated suite exercises determinism, hidden information, malformed commands, dispatch contention, lifecycle transitions, cancel/reroute, deadlines, scoring, event references, and save/load at every tick of a running response.
