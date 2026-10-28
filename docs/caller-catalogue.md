# Random complaints and in-character conversation

Normal play now uses **300 individually authored complaint scenarios**, stored in `assets/scenarios/complaints.json`. The human-readable index is [the complaint list](design/complaint-catalogue.md). There are 100 police, 100 fire/rescue and 100 medical complaints, covering genuine minor issues, moderate incidents, major emergencies and critical situations. The source records live under `tools/content/`; `python3 tools/content/build_catalogue.py` rebuilds the JSON and index.

## Host-owned randomization

A fresh host session gets a private random seed. Before delivery, the host chooses a complaint from a shuffled deck, a reachable road-access location from the supplied connected pilot district, a caller identity/personality and its fixed genuine/false scenario. No fabricated road connections or arbitrary off-road coordinates are introduced. Adjacent random draws avoid the same access point when alternatives exist. Building interiors and waterfront access are not yet a separate POI/zoning layer; locations are dispatchable road access points.

False-report selection uses a separately shuffled ten-call bag containing exactly one false slot. Thus every complete batch of ten **catalogue-generated complaints** contains one false report, across the entire room rather than per player. Short shifts can end partway through a batch. Consequence-linked follow-up calls remain genuine when the underlying explosion/injury actually occurred; they are not independently randomized into hoaxes. Each shift starts a fresh deck/bag, so a complaint can recur in a later shift.

A true minor complaint is different from a false report. A real noisy rehearsal, routine appointment enquiry or harmless appliance noise may deserve refusal while still being a genuine call. False cases are deliberately fabricated, mistaken video reports or old second-hand rumours; their clarification facts expose the discrepancy when investigated. False reports have no real emergency damage or escalation. The claimed situation can sound serious without changing the actual truth.

Severity determines response windows, service duration and consequence pressure. Only eligible real fire/gas hazards can trigger the existing bounded explosion mechanism; a trapped animal, stalled lift or routine rescue does not become a structural explosion just because it uses a fire unit. Derived medical follow-ups use their own medical scenario and inherit the source location.

Complaint, actual truth, severity, persona and facts remain fixed throughout the conversation. Save/resume retains the sampler's state and generated cases. Joining clients receive public call views and spoken transcripts, not false-report flags, private briefs, future deck order or seeds. Caller names and IDs are stable within a shift so repeat-call counts remain meaningful.

## Prompt contract

The versioned system prompt is `assets/prompts/caller-system.txt`. The host sends only the selected caller's small brief, not the full 300-entry list:

- Caller name, personality and reported complaint.
- The exact road-access address and caller-private knowledge about the selected situation.
- Available observations, verification/source details and location facts.
- The last four dialogue pairs and the player's current question.
- Whether this is the opening or an ordinary conversation turn.

On answering, the host requests an **Opening** immediately, before any player input. The AI writes the first statement in character. An opening cannot complete the call or mark investigation facts as verified. Subsequent **Answer**, **Clarify** and **Goodbye** responses use natural wording rather than copying catalogue sentences. The prompt includes examples for an irritated minor complaint, an unresponsive roommate and a false video report after source verification. It discourages immediate hoax confession, engine classifications and invented details.

Structured output carries spoken text, emotion, an allowed conversational action, known fact references and call completion. The host validates phase, allowed actions, reference IDs, duplicates, output size and basic metadata leakage. The AI has no interface for dispatch, money, world truth, severity, locations or incident creation. Referenced known facts can become case notes; the player makes dispatch decisions.

**Semantic boundary:** natural prose is no longer restricted to verbatim strings. Prompting and representative live evaluations check consistency, but structural validation cannot prove that every paraphrase is semantically faithful. Fixed simulation truth and allowed evidence IDs remain deterministic even if a caller's wording is imperfect. Further automated semantic evaluation and human playtesting remain useful; no perfect hallucination-proof dialogue guarantee is claimed.

## Voice loop

1. Answer a call: the AI generates an opening; local TTS plays it and subtitles appear.
2. Enable the microphone in Audio settings and hold **V** while the caller is ready.
3. Release: the game stops capture and runs the local Whisper adapter.
4. Recognized text automatically goes to the host/provider.
5. The response appears in the transcript and plays through local TTS.

**Review transcript before sending** remains an optional Audio setting; in that mode, edit the result and press Enter. Typed questions also remain available. Microphone access and saving audio still start off; no human microphone was recorded by automated tests. Captured audio stays local. Only recognized text is sent to the caller service/provider.

The supplied adapter uses Faster Whisper CPU/int8 with beam search. This task's launcher now points at the installed `base.en` model. Local speech is imperfect; review mode is available when recognition quality is poor. The TTS backend remains local espeak-ng or a replaceable adapter, with caller volume/output selection and subtitles.

Provider failure, malformed output or exhausted budgets still drops the caller conversation, with no offline caller replacement. Speech-device failures retain subtitles/text; they do not invent another caller. A disconnected human player is never replaced by a bot. The current process request cap counts AI openings as well as subsequent replies; see [caller configuration](callers.md) for limits.

## Verification

```bash
cargo test --locked --workspace
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo build --locked --workspace
python3 tools/net/playtest.py --players 6 --reconnect --dialogue-test --catalogue-test
python3 tools/net/graphics.py --catalogue-test
```

With an explicitly configured local provider and installed speech adapter, the opt-in live check is:

```bash
python3 tools/caller/live_speech_check.py --stt-command /absolute/path/to/local-stt-wrapper --output target/live-caller
```

It runs the real trusted daemon, requests AI openings and replies for a genuine emergency, a genuine minor complaint and a false report, and checks the local synthesized-speech → Whisper → provider → local-TTS chain. It makes paid model requests only when explicitly invoked, writes no credentials into reports, and stops its daemon on exit. Physical microphone quality and listening remain manual checks.
