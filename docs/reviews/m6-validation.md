# M6 validation — October 29, 2026

## Scope and acceptance

Implemented progression/accessibility plus the requested environment/character polish and local Pocket TTS. The reference remains `docs/design/gameplay-loop.pdf` and its exact source text. This is an implementation/automated acceptance report, not a claim that human usability research has been completed.

### Run boundaries

Solo and graphical hosts store version-1 checkpoints in ignored `.wrong-number/progress.json`, separately from `.wrong-number/settings.json`. A new launch resumes at the last shift start or completed review. Live calls, partial recordings, socket identity and in-flight AI requests are not resumed. Reviews keep their simulation evidence, points and damage; funding is already credited once. Closing/reopening a review cannot earn a second grant or career completion. Purchases persist, apply to the next shift, and are host-only in co-op. `Start new run` is available from a completed review and explicitly resets funds/radio/patrol; completed-shift count and a B/A-grade cosmetic commendation remain. Clients never load their local progress into the host's room. Dedicated headless launch/test processes do not currently save campaign checkpoints.

The existing director increases bounded call budget and simultaneous incidents across shifts. Scenarios retain immutable truth, ten-percent false catalogue calls, real road routing and service/recovery constraints. Radios trade funding for shorter recovery; an additional patrol trades more funding for police capacity. Permanent unlocks cannot change evidence or make a base scenario invalid.

### Accessibility

Settings save version, subtitle visibility, 100–130% UI/text scale, high-contrast text, reduced motion, keys, mouse sensitivity/inversion, display and all existing voice/audio controls. Status text accompanies semantic colors. Reduced motion stops cloud drift and avatar walking animation and disables the incident screen flash; no camera shake is implemented. Cars remain moving environmental objects. Scrollable settings, phone transcript and review panels accommodate enlarged text. Text input remains usable without microphone hardware. Caption visibility does not discard transcript/evidence data.

### Environment

Twenty-minute visual day cycle starts at 08:00 and follows the replicated simulation clock across shifts. Lobby time stays fixed; daytime returns after the full cycle. Ambient/sun intensity and sky color change while indoor practical lights remain. Ten pooled cars use two streets; they cannot change dispatch traffic or routes. Seven pooled clouds, bounded distant building volumes, windows/cornices/drainpipes/HVAC and the existing fire escapes create depth. Brick boxes use face-projected world-scale UVs. Procedural textures now include mipmaps and anisotropic filtering to reduce distant mortar shimmer. Shared low-poly avatar meshes have skin/uniform variation, headsets, ID badges, hands/feet, jointed walking arms/legs and a seated pose. Local first-person head/body is hidden; teammates see the model.

## Automated checks

- Workspace unit/integration suite: 134 passed, four existing opt-in tests ignored. Formatting, strict Clippy and build pass.
- Disk checkpoint test covers serialization, atomic write/read, unsupported-version preservation, review credit once, upgrades and career/reset separation. An explicit launch integration test restores a saved review through both actual solo and host startup, then advances to shift two.
- Rendered menu test clicks voice mode/accessibility controls, reloads the settings file, checks invalid join codes, hosts and leaves a room. Test preferences use a disposable path.
- Full rendered shift with actual Pocket TTS exercises generation, WAV decoding, output start/completion, review and next shift, without recording a person. Physical audibility and voice preference still require the player to listen.
- Two rendered co-op processes complete a shared shift/review/upgrade/next shift with matching public digests. Six processes complete reconnect/dialogue testing with matching authoritative results.
- Rendered day/night/street/office/avatar captures inspected. An intersecting background building was caught and corrected. Captures from the exterior overview are a diagnostic camera, not a new walkable outside area.

## Targets and measurement limits

Target this PC at roughly 60 FPS with VSync at 1440×900, steady-state p95 under 25 ms after warm-up. The reference GPU is an NVIDIA RTX 3080; this is not a minimum-spec certification. Recheck on a lower-end CPU/GPU and with six active human conversations before release. Final isolated benchmark: 58.5 FPS average, p95 17.82 ms over 300 samples. Benchmark outputs are also recorded in the gameplay-loop review.

Pocket target: ordinary short caller turns generate within 3 seconds on this CPU; synthesized audio is generated locally without network requests. Short actual fixture replies measured 0.4–0.8 seconds. Two long sample replies measured 2.12–2.58 seconds for approximately 9–10 seconds of audio. The April full-size model measured 3.9–4.7 seconds, so the small English model was selected. Piper's previous 0.47-second benchmark remains faster; Pocket is a voice-engine change, not a proven speed improvement over Piper. LLM/STT time is separate, and WAV delivery currently waits for the complete utterance.

## Remaining human playtest protocol

Run at least two ten-minute shifts with 2–6 people, once at default settings and once with enlarged text/reduced motion. For each resolved/refused call, ask the dispatcher to identify the deciding clue and explain the score from the review. Record misunderstandings, perceived unfair outcomes, interruptions caused by the shift ending, voice delay and dropped calls. Targets: at least 80% of sampled outcomes correctly explained; no outcome without visible review evidence; ordinary turns comfortably finish before shift closure; no discomfort caused by flashes or unreadable phone controls. These are proposed sign-off thresholds, not measured human results. No such human session has been fabricated or marked complete.
