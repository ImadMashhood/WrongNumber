# Wrong Number development rules

The gameplay reference is `docs/design/gameplay-loop.pdf`; the exact supplied text is `docs/design/gameplay-loop-source.txt`.

- Review the gameplay loop after every milestone and every gameplay-changing commit. Record the specification sections, implementation evidence, tests/playthrough results, remaining gaps and milestone ownership in `docs/reviews/gameplay-loop.md`.
- Fix gaps belonging to the current milestone before advancing. A scripted fallback, prototype or deferred feature must be labelled honestly. Do not claim the complete specification is implemented when only the current milestone passes.
- After M2 passes this review, start M3 automatically; the user has authorized this. No additional confirmation is needed for that transition.
- Explicit user changes override the brief: keep six chairs unnumbered; E pops the player into a seat; mouse look works while seated; start in daytime, cycle through day/night (October 29 user update), and keep simple low-poly graphics.
- Multiplayer authority/replication belongs to M4; caller AI, microphone/STT/TTS and recordings belong to M5. October 29 user override: callers must not hang up automatically. Provider failures keep the line open with a visible retry error; no automatic offline caller replacement. Explicit automated-test fixtures are separate from gameplay.
- Keep the supplied map/graph source intact. Validate paths on actual road edges; do not invent links between disconnected components. Record provisional station placements and any bounded import normalization.
- Keep hardware rendering as the default. Software Vulkan is an explicit diagnostic fallback. Check performance after renderer/UI/map changes.
