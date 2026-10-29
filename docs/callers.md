# M5 caller conversation and optional local speech

The office supports a resource/finance desktop and an incoming phone window with caller portraits, conversation text, dispatch and mute controls. A configured AI service is required for live caller conversation. There is no offline caller replacement. Text input remains available without a microphone. Caller dialogue is generated naturally from a fixed host-selected scenario; it cannot rewrite fixed truth, dispatch responders, spend funding or generate events. The AI writes an in-character opening and replies while the simulation owns the facts. See [the 300-complaint catalogue and prompt contract](caller-catalogue.md).

## Playing and disconnections

Start the game normally. Sit with E, focus your monitor with E and answer a ringing call. Click **Type a question…**, type a short question and press Enter. Examples: “What happened?”, “What can you see?”, “Are they breathing?” and “Where did this happen?” Authored question buttons remain available when interpretation is unclear.

The caller's introduction and replies appear as subtitles and in the shared case transcript. Facts become evidence only after the host validates a reply against that caller's authored knowledge. Replies come from the configured provider. Authored question buttons send their question through that same service; they do not disclose facts offline. Cancelling a pending response prevents its later result from revealing facts. Hanging up, transferring, disconnecting or starting another shift invalidates old responses.

Caller speech uses local `espeak-ng` when installed, otherwise subtitles remain available (audio-device failures do not synthesize a different caller). Linux can install it through the distribution package manager. `--mute` suppresses automatic caller/ring/city audio. Audio settings in the terminal or Escape menu select microphone input, caller output and caller volume. Device choices/volume are session settings; persistence and expanded accessibility settings remain M6.

## Optional OpenRouter / compatible provider

Run the optional **separate trusted server executable** on the host PC. It reads provider credentials; the game binary and joining clients never read them. Nothing needs to be pasted into chat. A live key and explicit model are both required before this process makes provider requests.

Example for Bash; the secret prompt avoids saving the key in shell history:

```bash
read -rsp 'Caller API key: ' CALLER_API_KEY; echo
export CALLER_API_KEY
export CALLER_MODEL='YOUR_CHOSEN_SCHEMA_CAPABLE_MODEL_ID'
cargo run --locked -p wrong-number-caller-server
```

For the default local port, `bash scripts/run-with-ai.sh` builds and starts both the trusted daemon and the game, and stops its daemon when the game exits. The key is loaded only in the daemon child process. This wrapper is for the solo/co-op host, not joining clients. This task's ready-to-use `outputs/run-with-ai.sh` also selects the installed local speech adapter.

Alternatively, start the daemon separately and in another terminal run the game with:

```bash
# Solo
cargo run --locked -- --caller-service 127.0.0.1:47005
# Co-op host
cargo run --locked -- --host 0.0.0.0:47004 --name Imad --caller-service 127.0.0.1:47005
```

Joining clients use the usual `--join` command and do not configure provider access. The service binds only to loopback. `bash scripts/run-caller-server.sh` loads ignored, private `.wrong-number/caller.env` configuration when present. No keys are embedded in the script. `CALLER_ENDPOINT` can select another compatible chat-completions endpoint over HTTPS (or loopback HTTP for a local provider/test). It never follows HTTP redirects. `OPENROUTER_API_KEY` is also recognized by the server. `.env.example` documents configuration, but files are not automatically sourced; supply variables only to the trusted server process.

The provider sees a caller report, personality cue, permitted question/answer facts, at most eight recent dialogue pairs and the current typed question. It does not receive simulation truth, consequences, future calls, responder controls or raw microphone audio. Recognized text sent by a co-op dispatcher reaches the host; a configured cloud provider receives that text and the limited caller context. All dispatchers can see the resulting case transcript.

The provider returns structured `spoken_text`, `emotion`, `proposed_action`, `fact_references` and `call_complete`. Replies use natural prose and must reference only permitted facts. Opening/Answer/Clarify/Goodbye are the only allowed conversational actions. Unsupported actions, fields, invalid fact IDs, oversized replies and basic metadata leakage are rejected. Semantic fidelity of free paraphrases is guided by the prompt and live evaluation; it is not a formal guarantee. A goodbye ends only the conversation. The selected persona and private scenario brief stay fixed across turns.

Provider failures, invalid output, unavailable service or exhausted budgets drop the caller conversation. Its incident, evidence, responders and consequences remain authoritative and continue. No bot or scripted answer replaces it. Network calls stay on bounded worker threads. The game permits one pending response per call, six worker requests, one new turn per dispatcher per second and 24 question turns per case. The daemon allows six active requests, 30 requests/minute and `CALLER_MAX_REQUESTS` provider attempts per process (default 100, maximum 1000). It caps input frames/request body and responses, requests at most 256 completion tokens, uses a 20-second HTTP timeout (22-second service deadline, 23-second authority deadline) and adds configurable OpenRouter maximum token pricing through `CALLER_MAX_INPUT_PRICE` and `CALLER_MAX_OUTPUT_PRICE` (USD per million, default 1 each). Other compatible endpoints do not receive OpenRouter-specific routing fields. Transport/provider failures are not retried. A repeated reply gets at most one live rewrite; both attempts consume the same request/rate budgets and share one 20-second deadline. If the rewrite is still repeated, the caller disconnects without scripted replacement. A cancelled in-flight provider request may still be billed; its result is ignored. Account-side spending caps remain appropriate for live keys.

Implementation references: [OpenRouter structured outputs](https://openrouter.ai/docs/guides/features/structured-outputs), [provider routing and maximum prices](https://openrouter.ai/docs/guides/routing/provider-selection). A live `openai/gpt-4o` request passed through the real daemon in 2.15 seconds on 2026-10-28. Local HTTP fixtures separately verify invalid-response and exhausted-budget disconnections. This is one request, not a latency/billing guarantee.

## Local speech recognition

Speech adapters are executable files invoked directly, without a shell:

- `--stt-command PATH`: receives `INPUT.wav OUTPUT.txt`; writes a UTF-8 question of at most 1024 bytes.
- `--tts-command PATH`: receives `INPUT.txt OUTPUT.wav`; writes a supported PCM WAV. The configured launcher selects local Piper; without an adapter the executable tries `espeak-ng` with fixed arguments. See [neural TTS setup](local-tts.md).

The supplied `tools/speech/transcribe.py` uses Faster Whisper on CPU with a pre-downloaded local model. Create a Python virtual environment, install `tools/speech/requirements.txt`, download a compatible model and use a small executable wrapper that selects that environment and sets `WRONG_NUMBER_STT_MODEL` to its model folder. The script never downloads a model during gameplay. See [Faster Whisper's installation/model documentation](https://github.com/SYSTRAN/faster-whisper). The model/runtime are optional local dependencies and are not committed.

After launching with your STT adapter, answer the phone and speak naturally. Voice activation is on by default during an answered call. Two continuous seconds below the configured noise floor after speech sends the local transcription automatically. The phone has an always-visible mute/unmute button; muting never stops caller playback. At the hard time limit, the entire captured segment is sent to local Whisper even when the gate never detected speech. Empty or failed transcription produces no invented AI input. Mute, hang-up and focus loss still cancel rather than transmit. Capture pauses while the caller speaks, while awaiting an AI reply, while editing text, on lost window focus, or outside the desktop. Each utterance is capped at 12 seconds. Enable **Review transcript before sending** to edit before Enter instead. Headphones help avoid echo; the energy gate may need device-level gain adjustment in a noisy room.

On this task's PC, `outputs/run-wrong-number.sh, outputs/run-with-ai.sh and outputs/run-with-voice.sh` and its local adapter wrapper select the installed CPU model/runtime automatically. Those scripts contain paths, not keys. Physical microphone quality, selected-device playback and human conversation latency still need a listening/play test.

## Local recordings and review

Voice activation starts **on**, but capture only runs during an answered phone call with the desktop focused. The user explicitly requested this default. Saving recordings still starts **off on every launch**. Captured speech is used transiently for local transcription. Speech adapters use a private temporary directory which is removed after normal completion/cancellation. Enabling **Save my microphone and caller audio locally** saves only this player's microphone clips and validated caller speech into ignored `.wrong-number/recordings/`. It never records or relays teammates' microphones. Saved files have private permissions where supported; the folder retains at most 32 clips. There is no cloud audio upload or publication.

The review panel lists local clips for playback alongside shared text excerpts. Disable saving to revoke consent for future clips. **Delete all my saved audio clips** deletes this game's local WAVs and disables saving. Saved clips survive restart until deletion/retention cleanup; text transcripts remain in memory for the current shift and reset for the next day. Automatic highlight selection, mixed full-call recordings and multiplayer voice chat are not claimed by this slice.

## Explicit test fixtures

Deterministic authored replies exist only for unit tests and explicitly invoked automated playtest/bot commands. They never activate because a live service fails, and they are not an offline gameplay option. A disconnected human player is removed from the room and is never replaced by a bot.

## Verification

```bash
cargo test --locked --workspace
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo build --locked --workspace
python3 tools/caller/playtest.py
python3 tools/net/playtest.py --players 6 --reconnect --dialogue-test
cargo run --locked -- --conversation-test --capture-dir target/caller-captures
```

The provider harness launches the real daemon against a local HTTP fixture, verifies structured answers, rejects unsupported fact references and exhausts its request budget without any paid request. Unit tests cover authority ownership, cancellation, deadlines, unavailable-service disconnection, private-state exclusion and bounded WAV handling. The graphical conversation test activates the real text-entry button and injects keyboard messages through the application's input path. A real local TTS smoke test is opt-in: `cargo test --workspace local_tts_returns_valid_audio -- --ignored` on a PC with `espeak-ng`. A synthesized question was independently transcribed with the installed base English CPU model; no human microphone was recorded during testing.

The normal task launcher now starts the trusted caller service and local STT adapter together. A plain game binary still requires an explicit service address; it cannot invent a caller when disconnected. Acknowledgements with no fact references keep the line open and award no evidence. The phone disappears on hang-up, refusal, completed dispatch or disconnection; resources continue on the desktop. Portraits are original procedural low-poly illustrations, consistent by caller ID.

## Listening controls and output diagnostics

The phone and Sound app show separate microphone and caller waveforms. These are RMS envelopes of actual microphone frames and decoded caller PCM, synchronized to playback position; no simulated activity is shown. The listening readout shows elapsed time and the configured limit. Sound settings changes the maximum utterance from 3–12 seconds (default 12) and the noise floor from -90 to 0 dBFS (default -48). Raise the floor to reject room noise; lower it if quiet speech is missed. Audio below the floor counts toward the two-second pause. Changing either control restarts the current capture segment. Settings currently last for the game session.

Sound settings includes **Test speakers**, **Replay last caller reply**, output-device selection and volume. Replay uses the actual most recent reply without calling the model again. Persistent output status distinguishes speech generation, actual playback, completion and device failures. The AI launcher writes private, overwritten-per-launch diagnostics to .wrong-number/caller.log and .wrong-number/game.log; these contain status/device details, not credentials or spoken transcripts.

The current machine's system default routes to the Focusrite Scarlett Solo. An in-game audible conversation playthrough and playback of a previously generated live-AI WAV both completed through the real game audio path; the game streams were unmuted at the OS mixer. This does not establish that the user's physical headphones were connected to that output. Select the appropriate output in Sound and use Test speakers if nothing is heard.

The computer can be opened while standing: aim at a nearby monitor and press E. The desktop offers Resources and Sound apps, a taskbar, a minimizable resource window and the existing city map as wallpaper. Taking a seat is still required to receive/handle a dispatcher line, preserving multiplayer workstation ownership.

## Recurring cast and repetition control

The game selects from 50 stable authored caller identities: 40 original adults and ten clearly fictional parodies. Their biographies, speaking style and stress reactions live in assets/scenarios/callers.json; docs/design/caller-cast.md is the readable cast bible. Profiles are assigned independently of incident truth/severity, including consequence follow-ups. Each has an original stylized portrait. Only the selected profile is sent to the AI.

Caller-service protocol 4 includes hashes of every earlier reply in that conversation. The latest eight exchanges also travel as actual user/assistant chat turns. The host and service reject normalized exact repeats; the service also checks high bigram overlap against recent replies to catch cosmetic rewrites. One bounded paid rewrite is permitted for repetition, never a canned fallback. New wording must preserve known facts; asking the same factual question cannot guarantee new information. Richer reactions come from character, context and emotional response, not fabricated incident evidence.

The desktop clock, finances, resource ETA and speech status update existing text entities. Other UI changes are staged for a layout frame before replacing the visible tree, preventing blank flashes during clock/capture updates. The selected character, hidden facts and request budgets remain host-owned.


## Main-menu voice preferences

Settings is available before joining and from the in-game room menu. Voice Activation remains the initial default; Push to Talk and Muted are explicit alternatives. Push to Talk uses V by default (rebindable), records only while the answered call is focused and the caller is not speaking, and submits on release. The silence gate does not prematurely end a held PTT turn. Both modes enforce the same configured maximum recording time (3–12 seconds); after a hard cutoff, PTT requires a new press for a new turn. Losing focus, entering menus, muting, leaving or hanging up cancels capture rather than submitting it.

Preferences live in ignored `.wrong-number/settings.json`: voice mode, caller volume, ringtone volume, named input/output devices, noise floor, recording limit, transcript-review preference and control/display settings. Device names resolve after enumeration, falling back to the system default if missing. Saving audio consent is deliberately session-only. The menu does not provide teammate voice chat. TTS uses the selected caller output; telephone rings and ambient loops use Bevy's system audio output with separate volumes.

## Psychology and continuity

The provider now uses contract version 4, including bounded behavioral proposals. See [psychology/investigation](caller-psychology.md) for private state, locked disclosures, public provenance, transfer continuity and session memory. Dispatch and incident deadlines no longer automatically end an answered call. Piper is now the configured faster local speech engine; see [speech setup](local-tts.md).
