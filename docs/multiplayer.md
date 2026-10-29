# M4 direct-connect co-op

The playable prototype supports one host and up to five joining dispatchers. Normal launch opens the main menu, with solo or multiplayer choices. M3 and M4 use the same simulation rules, road routes, consequence ledger and review calculations. Joining clients need no provider key; the solo/hosting process needs the configured AI caller service for live conversations.

## Host and join

Use the same repository commit on each PC and install the README's native dependencies. From the repository:

```bash
# Host: listen on LAN interfaces, TCP port 47004
cargo run --locked -- --host 0.0.0.0:47004 --name Imad
# Client: use the host PC's actual LAN IPv4 address
cargo run --locked -- --join 192.168.1.50:47004 --name Partner
```

Linux can use `bash scripts/run.sh` with the same trailing arguments to select hardware Vulkan automatically. To test two windows on one PC, host on `127.0.0.1:47004` and join that address from another terminal. Use different names for different local clients.

The host's firewall must allow inbound TCP on the chosen port. The join address is the host PC's address, not `0.0.0.0`. Direct LAN and port-forwarded internet connections are supported; internet relay, Steam invitations and automatic NAT traversal remain future work. Use trusted friends or a private VPN; the prototype transport does not encrypt traffic or authenticate an account.

Click **Return to office / take a seat** or press Escape to leave the lobby view. All players begin standing in the break room. Walk to an available chair, aim and press E; aim at the monitor and press E again. Before entering dispatch, hold E at the break-room door for 1.5 seconds. The creator/host can start immediately, or half of connected players must ready up. The ten-minute clock starts when the door unlocks. Chairs have no number labels. Mouse look works while seated, Escape leaves monitor focus, and Space stands from free look.

Each seated dispatcher has one line, with independent ringing, conversation and selected case. Other telephones have spatial rings and blinking beacons. Facts and incident notes are shared. Use the transfer button while talking to move the conversation, including its evidence, to a connected, seated teammate whose line is idle. Everyone draws from the same responder pool. A conflicting or stale order is rejected rather than silently changing an assignment. Explicit rerouting remains available.

Online menus keep the shared clock running. The host alone starts/ends shifts, spends team funding and advances the next day. At closing, outstanding incidents and real return/recovery routes finish; players walk to the rear review room. The next day returns everyone to the break room and lobby. Solo still starts immediately and has a real pause.

## Disconnect and reconnect

A lost client releases its chair. Its unanswered ringing call becomes eligible for another line without resetting the deadline. An answered conversation ends, while its case, evidence, dispatched responders and consequences continue. Joining during a shift is allowed.

A successful join saves an opaque room token in ignored `.wrong-number/<address-name>.json`. The token reclaims the same dispatcher ID and command sequence after a disconnect; the returning player stands in the break room and takes an available workstation. Click **Reconnect** in the disconnected lobby or rerun the same command. A disconnected slot can be taken by a new player, which invalidates its old token. The token never authorizes host controls.

For a new host session at the same address, delete that client's old resume file if it reports “Resume token not found in this room”, or use a fresh explicit `--resume path/to/new-session.json`. Tokens apply to a live host room; persistent rooms and host migration are not implemented. Closing the host disconnects the group; clients cannot elect a replacement host. Resume files contain credentials and should not be shared or committed.

## Diagnostics and automated checks

The room panel shows connected players, workstation presence, round-trip latency, revision and rejected-command count. Recorded simulation events are the shared decision/city timeline. F3 shows rendering performance. Rejections during contention are expected; they do not allocate a second unit.

```bash
cargo test --locked --workspace
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo build --locked
python3 tools/net/playtest.py --players 2 --output target/network-two
python3 tools/net/playtest.py --players 6 --reconnect --output target/network-six
```

The harness starts actual OS processes over TCP, moves dispatchers through validated room geometry, seats them, completes an accelerated 600-second shift, returns every player to the break room and reaches the next lobby. Six-player reconnect testing terminates a client during the live shift and restarts it with its saved identity. It compares all public simulation/phase/funding/upgrade digests, IDs, peak room size and completion state, then deletes test credentials. It never creates graphics/audio devices. CI runs both process checks on Linux and Windows.

`python3 tools/net/graphics.py` is the local GPU verification: two real Bevy windows render into captures, verify another dispatcher avatar, hold ready at the break-room door, complete a shared shift, return to the next break-room lobby. It writes captures/reports into `target/network-graphics` (or `--output <folder>`); it requires a desktop display and graphics device. Solo `--playtest`, `--consequence-test`, `--escalation-test` and `--benchmark` remain available.

## Architecture

`Room` is the sole authority. It binds each authenticated connection to a server-assigned player ID; requests cannot select another actor. It owns the clock, scenario truth, hidden answers, call schedules, chair claims, players, responder commands, roads, consequences, results and purchases. Per-player `Desk` state keeps monitor focus and line ownership separate. Workload scales with connected players while respecting active/call/follow-up caps.

The M4 spike uses standard-library TCP/JSON rather than adding a Bevy-version-dependent networking plugin. Small six-player rooms and reliable, infrequent dispatch decisions justify this simple baseline. `crates/net` handles partial frames/writes on worker threads, bounded queues, message-size/rate limits, disconnects and timeouts. Commands carry shift IDs and monotonically ordered sequence numbers, with cached acknowledgements preventing duplicate side effects. Responder orders include the observed assignment so competing stale Send clicks cannot become reroutes. Chairs, recipient availability, movement and host privileges are checked centrally.

Snapshots contain only public incident views, revealed evidence, unit states, roads, events and shared results. Pending caller content, fixed truth and unrevealed answers remain host-side. The host UI and clients reconstruct a read-only presentation mirror that rejects local commands, clock advancement and authority saves. Snapshot digests are checked after reconstruction. Replication runs at four updates per second plus authority changes; validated poses are sent at ten per second and avatars interpolate. TCP head-of-line blocking is an accepted prototype tradeoff; pose datagrams, deltas and bandwidth tuning can follow measured need.

## Acceptance evidence and remaining manual check

- [x] Complete two-process and six-process shifts; all players agree on the next-lobby public digest.
- [x] Six-process disconnect/reconnect mid-shift preserves identity and reaches the same result.
- [x] Two rendered processes pass lobby-button, avatar, shared-shift and next-day checks.
- [x] Unit tests cover seat contention, simultaneous Send requests, duplicate/old-shift commands, host privileges, line transfer, lost ringing calls, private-state exclusion, read-only mirrors, explosions/closures and invalid movement.
- [x] Offline graphical shift, consequences, explosion/closure and FPS regression checks.
- [ ] Two physical PCs on the same LAN: host/join, take separate chairs, answer simultaneous calls, transfer, race one unit, disconnect/reconnect, finish a ten-minute shift and compare review/funding. Record OS, GPU, commit and latency.

The last check needs another physical PC and a human co-op session. Local multi-process success does not claim that hardware/network compatibility check has happened. Voice, microphones, natural AI callers and recordings remain M5.


## Main menu and internet endpoint codes

Normal launch opens the front end. Host creates the authoritative room on the creator's PC, binding all IPv4 interfaces on the chosen TCP port (default 47000). The host supplies the public IPv4 reachable by friends, or a LAN IPv4 for local play. The lobby shows and copies a versioned WN1 endpoint code; Join with Code validates the checksum and connects asynchronously using the same TCP protocol as direct Join. The code is an address, not a secret or NAT traversal mechanism. Client names are limited to 24 characters. The host may start with a 1.5-second door hold, or at least half of connected players can ready to start; there is no migration or bot takeover if the host leaves.

Internet hosting needs TCP port forwarding and a firewall rule on the host. Match external and internal port numbers. A private LAN address only works within the same LAN/VPN; loopback only works on the same PC. Carrier-grade NAT generally requires an ISP public address or a shared VPN. No relay, automatic router reconfiguration, public matchmaking or encrypted public-server service is included. This remains the existing trusted-friends transport; do not treat an address code as authentication. Real two-network reachability cannot be established by loopback acceptance tests.

Connection attempts run off the render thread; Back cancels a pending result. Failed or disconnected sessions offer reconnect/leave controls. Leaving drops the transport and returns to the main menu. Host and clients retain the existing authoritative simulation, six-player cap, reconnect token checks and no-offline-replacement behavior.

Room protocol is now **6**; all peers must run the same build. Ready holds are validated by host wall-clock time, recent heartbeats and door proximity. Disconnects remove votes and recalculate the connected-player threshold. Shift completion resets poses/seats and clears readiness, returning everyone to the break room with the review open. The host can buy one upgrade and prepare the next lobby, or explicitly start a new run. M6 checkpoints belong to the graphical host; clients never restore local campaign data over shared state.
