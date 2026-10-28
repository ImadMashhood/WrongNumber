# M5 caller conversation and optional local speech

The office supports typed questions, authored question buttons and optional push-to-talk speech. It remains playable offline with no key, microphone or caller service. Caller dialogue selects from authored knowledge; it cannot rewrite fixed truth, dispatch responders, spend funding or generate events. This is a constrained caller prototype, not unrestricted model-written improvisation.

## Play without an API

Start the game normally. Sit with E, focus your monitor with E and answer a ringing call. Click **Type a question…**, type a short question and press Enter. Examples: “What happened?”, “What can you see?”, “Are they breathing?” and “Where did this happen?” Authored question buttons remain available when interpretation is unclear.

The caller's introduction and replies appear as subtitles and in the shared case transcript. Facts become evidence only after the host validates a reply against that caller's authored knowledge. The display shows whether the answer came from the provider, scripted fallback or an authored question. Cancelling a pending response prevents its later result from revealing facts. Hanging up, transferring, disconnecting or starting another shift invalidates old responses.

Caller speech uses local `espeak-ng` when installed, otherwise subtitles remain available. Linux can install it through the distribution package manager. `--mute` suppresses automatic caller/ring/city audio. Audio settings in the terminal or Escape menu select microphone input, caller output and caller volume. Device choices/volume are session settings; persistence and expanded accessibility settings remain M6.

## Optional OpenRouter / compatible provider

Run the optional **separate trusted server executable** on the host PC. It reads provider credentials; the game binary and joining clients never read them. Nothing needs to be pasted into chat. A live key and explicit model are both required before this process makes provider requests.

Example for Bash; the secret prompt avoids saving the key in shell history:

```bash
read -rsp 'Caller API key: ' CALLER_API_KEY; echo
export CALLER_API_KEY
export CALLER_MODEL='YOUR_CHOSEN_SCHEMA_CAPABLE_MODEL_ID'
cargo run --locked -p wrong-number-caller-server
```

In another terminal, run the game with:

```bash
# Solo
cargo run --locked -- --caller-service 127.0.0.1:47005
# Co-op host
cargo run --locked -- --host 0.0.0.0:47004 --name Imad --caller-service 127.0.0.1:47005
```

Joining clients use the usual `--join` command and do not configure provider access. The service binds only to loopback. `CALLER_ENDPOINT` can select another compatible chat-completions endpoint over HTTPS (or loopback HTTP for a local provider/test). It never follows HTTP redirects. `OPENROUTER_API_KEY` is also recognized by the server. `.env.example` documents configuration, but files are not automatically sourced; supply variables only to the trusted server process.

The provider sees a caller report, personality cue, permitted question/answer facts, at most eight recent dialogue pairs and the current typed question. It does not receive simulation truth, consequences, future calls, responder controls or raw microphone audio. Recognized text sent by a co-op dispatcher reaches the host; a configured cloud provider receives that text and the limited caller context. All dispatchers can see the resulting case transcript.

The provider returns structured `spoken_text`, `emotion`, `proposed_action`, `fact_references` and `call_complete`. An answer must exactly match one permitted authored fact; Clarify/Goodbye use fixed approved phrases. Unsupported actions, fields, unknown facts and invented prose are rejected. A goodbye ends only the conversation. Natural phrasing/paraphrase generation and richer biographies remain a later content expansion; this strict boundary prioritizes consistent evidence.

Provider failures, invalid output, unavailable service or exhausted budgets use the local scripted response. Network calls stay on bounded worker threads. The game permits one pending response per call, six worker requests, one new turn per dispatcher per second and 24 question turns per case. The daemon allows six active requests, 30 requests/minute and `CALLER_MAX_REQUESTS` provider attempts per process (default 100, maximum 1000). It caps input frames/request body and responses, requests at most 256 completion tokens, uses a four-second HTTP timeout and adds OpenRouter maximum prompt/completion pricing of $1 per million tokens. Other compatible endpoints do not receive OpenRouter-specific routing fields. There are no automatic retries. A cancelled in-flight provider request may still be billed; its result is ignored. Account-side spending caps remain appropriate for live keys.

Implementation references: [OpenRouter structured outputs](https://openrouter.ai/docs/guides/features/structured-outputs), [provider routing and maximum prices](https://openrouter.ai/docs/guides/routing/provider-selection). Paid live-provider latency, model compatibility and billing have not been tested without a configured key/model; local HTTP fixtures verify the adapter and failure behavior.

## Local speech recognition

Speech adapters are executable files invoked directly, without a shell:

- `--stt-command PATH`: receives `INPUT.wav OUTPUT.txt`; writes a UTF-8 question of at most 256 bytes.
- `--tts-command PATH`: receives `INPUT.txt OUTPUT.wav`; writes a supported PCM WAV. Without this flag, the game tries `espeak-ng` with fixed arguments.

The supplied `tools/speech/transcribe.py` uses Faster Whisper on CPU with a pre-downloaded local model. Create a Python virtual environment, install `tools/speech/requirements.txt`, download a compatible model and use a small executable wrapper that selects that environment and sets `WRONG_NUMBER_STT_MODEL` to its model folder. The script never downloads a model during gameplay. See [Faster Whisper's installation/model documentation](https://github.com/SYSTRAN/faster-whisper). The model/runtime are optional local dependencies and are not committed.

After launching with your STT adapter, open Audio settings and explicitly **Enable microphone**. With your call open and text editing closed, hold **V** to capture, release to transcribe. Capture stops on lost window focus, leaving terminal focus, disabling the mic or reaching 12 seconds. The recognized text appears for editing; **Enter sends it**. No speech is automatically sent. The shift continues while speech/model workers run. Adapter failures and timeouts retain text/buttons; caller playback cancels when the conversation changes.

On this task's PC, `outputs/run-with-voice.sh` and its local adapter wrapper select the installed CPU model/runtime automatically. Those scripts contain paths, not keys. Physical microphone quality, selected-device playback and human conversation latency still need a listening/play test.

## Local recordings and review

Microphone capture and recording consent start **off on every launch**. Hold-V capture is used transiently for transcription. Speech adapters use a private temporary directory which is removed after normal completion/cancellation. Enabling **Save my microphone and caller audio locally** saves only this player's microphone clips and validated caller speech into ignored `.wrong-number/recordings/`. It never records or relays teammates' microphones. Saved files have private permissions where supported; the folder retains at most 32 clips. There is no cloud audio upload or publication.

The review panel lists local clips for playback alongside shared text excerpts. Disable saving to revoke consent for future clips. **Delete all my saved audio clips** deletes this game's local WAVs and disables saving. Saved clips survive restart until deletion/retention cleanup; text transcripts remain in memory for the current shift and reset for the next day. Automatic highlight selection, mixed full-call recordings and multiplayer voice chat are not claimed by this slice.

## Verification

```bash
cargo test --locked --workspace
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo build --locked --workspace
python3 tools/caller/playtest.py
python3 tools/net/playtest.py --players 6 --reconnect --dialogue-test
cargo run --locked -- --conversation-test --capture-dir target/caller-captures
```

The provider harness launches the real daemon against a local HTTP fixture, verifies structured answers, rejects hallucinated output and exhausts its request budget without any paid request. Unit tests cover authority ownership, cancellation, deadlines, unavailable-service fallback, private-state exclusion and bounded WAV handling. The graphical conversation test activates the real text-entry button and injects keyboard messages through the application's input path. A real local TTS smoke test is opt-in: `cargo test --workspace local_tts_returns_valid_audio -- --ignored` on a PC with `espeak-ng`. A synthesized question was independently transcribed with the installed tiny English CPU model; no human microphone was recorded during testing.
