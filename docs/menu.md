# Main menu, controls and direct internet hosting

Normal launch opens **Wrong Number**. The configured AI launcher still supplies the host's local caller service; joining players need no API key. If the launcher cannot start that service, it opens the menu without it so Join still works. Hosting without a service does not fabricate callers or enable an offline fallback.

## Host and join

1. Choose **Host a room** and enter a dispatcher name.
2. Keep TCP port **47000**, or choose another unused port.
3. Enter your router's public IPv4 for internet play. For LAN play use your PC's LAN IPv4. A code made with 127.0.0.1 only works on the host PC.
4. For internet play, forward that TCP port to the host PC's LAN address in the router and allow it through the PC firewall. Use the same internal/external port. This setup is performed by the host; the game does not change router settings.
5. Create the room and use **Copy join code** in the lobby. The creator is dispatcher 1 and controls the shared shift.
6. Friends choose **Join with code**, enter their name, paste the whole WN1 code and Connect. **Join a room** alternatively accepts IP:port (including bracketed IPv6 if the CLI host supports it).
7. Everyone spawns in the break room. Aim at the dispatch doorway and hold Interact (E by default) for 1.5 seconds. The host can start alone; otherwise at least half of connected players must be ready (rounded up: 3 of 6, 3 of 5). A bottom progress bar fills while holding Interact, and the HUD shows the live count. The host validates proximity and the sustained hold; releasing early cancels. The door unlocks when the shift starts. At shift end, after active responders settle, everyone returns to the break room for the next lobby. Review and purchases are temporarily skipped at the user’s request. Leave room returns to the menu; if the creator leaves, clients disconnect. There is no host migration or replacement bot.

Codes include a checksum to catch typos and encode an IP and port. They do not register a public lobby or hide the host address, and are not passwords. NAT, double NAT, carrier-grade NAT, firewall rules and changing WAN addresses can still prevent access. A public address from the ISP or shared VPN is needed when direct forwarding is unavailable. No public relay is deployed. Local socket tests cannot prove the user's router is reachable from another network.

## Settings

- Rebind Forward, Back, Left, Right, Sprint, Interact/Sit, Stand and Push to Talk by clicking the binding and pressing a key. Duplicate assignments swap; Escape cancels and F3 remains reserved. Mouse aims and left-click interacts.
- Mouse sensitivity 0.2–3.0× and invert-Y apply immediately.
- Fullscreen (borderless), VSync and FPS display apply immediately; F3 also toggles the live counter.
- Voice mode cycles **Voice Activation → Push to Talk → Muted**. PTT defaults to V: hold to record, release to send; another press is required after a hard cutoff. Voice activation waits for two continuous seconds below the selected floor. Above-floor speech resets the pause timer.
- Input and caller output devices, caller volume, telephone-ring and ambient volume, speaker test, floor (−90…0 dBFS), maximum listening time (3–12 seconds), and optional transcript review are available. TTS volume affects subsequent playback. Rings use the system output separately.
- Saving clips remains opt-in per launch. This is caller communication, not teammate voice chat.

Settings are saved in `.wrong-number/settings.json` and never committed. Missing/invalid files use defaults. Preferences are local to each player; network snapshots do not override them. Online gameplay continues while settings are open; the microphone stops capturing while the menu has focus.

## Verification

`cargo test --workspace` includes endpoint-code validation, real TCP join through a decoded code, creator authority, host-departure disconnection and preference/binding serialization. Run the rendered menu acceptance check with:

```sh
cargo run --locked -- --menu-test --settings-file target/menu-settings.json --capture-dir target/menu-captures
```

This drives the real button observers through settings, invalid-code validation, room creation, generated code and leaving. It uses an isolated settings file and does not record a microphone or call an AI provider. CLI `--solo`, `--host` and `--join` bypass the front end; existing shift playtests are unchanged.

Calls require a seated dispatcher during an active shift. Opening a monitor while standing does not claim a phone line; the empty-phone panel explains this. The first call arrives 1–10 simulation seconds after seating when the director can deliver it.
