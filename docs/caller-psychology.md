# Caller psychology and investigation

The [user's overhaul brief](design/caller-psychology-source.txt) is preserved verbatim. This implementation extends the 300-complaint catalogue and 50-person cast; it does not replace the road simulation, authority or provider boundary.

## Three separate layers

1. `Simulation` owns the actual incident, consequences, facts, deadlines and scoring. Ten percent of catalogue draws remain false reports; a genuine minor complaint is not a false report.
2. Each private `CaseBrief` now owns an immutable psychology `Profile`. It selects one of six [authored patterns](../assets/scenarios/psychology-patterns.json): truthful, misinformed, deliberately deceptive, withholding, conflicted or truthful-but-manipulative. Goals, fears, tactics, allowed evasions, cover/source fact IDs and disclosure conditions are fixed before dialogue. Traits are deterministic for the recurring caller. Profiles are saved with authority snapshots and omitted from public snapshots.
3. `Dialogue` owns mutable host-only investigation state: trust, stress, defensiveness, cooperation, disclosure permissions, delivered claims, a bounded interaction history and caller memory. The provider receives only permitted facts, relevant private character context and eight recent exchanges. It never receives an undisclosed source answer, full simulation truth or future events.

The initial patterns operate on the catalogue's existing event/source distinction. For example, a deliberately invented report maintains its fixed claim and resists source questions; a mistaken video/message report can acknowledge the source when questioned. Conflicted and withholding callers give urgent observations freely while resisting probing about verification. Manipulative truthful callers press for priority without permission to exaggerate physical facts. Richer authored crimes, medical secrets and witness disputes can be added as content; the current patterns do not invent those events on top of existing incidents.

## Conversation changes state

Natural typed or transcribed speech remains the input. A conservative local intent interpreter handles information, clarification, evidence, contradiction, accusation, reassurance, negotiation, pressure, threats, tricks, irrelevant comments and insults. Fresh reassurance/negotiation produces small capped changes; repeating the same line cannot grind trust. Stress cannot fall below half the caller's baseline anxiety. Accusations increase defensiveness but never prove deception. Disclosure requires the authored trust/cooperation/interaction conditions and a relevant question; not every profile can be persuaded from every starting state.

The same provider request also returns a bounded `behavior` proposal: intent, an exact supporting quote from the current utterance, and an available fact ID or null. Unknown types, quotes absent from the input and unavailable IDs are rejected. A quoted semantic interpretation can make a smaller, capped rapport adjustment for wording the local interpreter missed. It cannot directly unlock a secret, certify evidence, change a dispatch outcome or authorize a hang-up. Changes apply to the next turn. There is no second paid classification request. Existing repetition repair permits at most one rewrite within the original request deadline/budget.

The [actual system prompt](../assets/prompts/caller-system.txt) prioritizes responding to the dispatcher's latest meaning before adding incident information, usually in 10–30 words. It discourages report/address resets and literal jokes based on name transcription mistakes. It supplies goals and stress responses rather than asking every character to be agreeable. Allowed emotions now include anger, defensiveness, embarrassment, relief, panic, cooperation and suspicion. Emotions never count as evidence of a lie.

## Evidence and continuity

Each delivered observation or admission records its incident, turn, dispatcher, time, spoken wording and authored fact IDs. A confrontation is labelled **potential contradiction**. A **verified contradiction** requires both the predefined cover claim and its contradictory source admission to have been disclosed. A model's claim that the player found a contradiction is insufficient. These notes appear alongside the existing incident details, and joining dispatchers receive only the public notes/transcript. No points are awarded for accusations or emotional interpretation. Existing evidence-sensitive refusal and dispatch scoring remains authoritative.

Transfers preserve the same investigation state and transcript. History identifies which dispatcher spoke. A caller's last twelve structured department interactions persist across shifts, including accusations, reassurances, dispatch, decline, transfer and hang-up. Quoted promises are explicitly labelled as what the dispatcher said, not as verified delivery of help. Repeated caller identities receive this bounded memory and a capped initial trust adjustment. Leaving the run resets session memory. There is no invented AI memory or unlimited transcript archive.

Existing linked explosion/medical follow-ups continue to share their actual event lineage. Simultaneous conflicting witness accounts about one incident are not introduced by this patch: that needs a separate call-to-incident model, rather than cloning an incident and accidentally duplicating damage or rewards. The current investigation patterns are a playable foundation for the brief, not a claim that every proposed scenario family and witness system is finished.

## Phone lifecycle and failure

Dispatching help, resolving an incident or reaching its response deadline no longer hangs up an answered phone. The dispatcher can keep talking, then use Hang up. Explicit refusal still concludes that decision. End-of-shift settlement still returns the team to the break-room lobby.

An NPC can end a call only when its authored pattern permits refusal, at least three distinct abusive/threatening inputs have depleted trust/cooperation, and a prior boundary warning has been displayed. The provider must then request a goodbye while the host permission is present. The host delays closure to allow the farewell to play. Ordinary questions, dispatch and a model's unsolicited goodbye cannot trigger this path. Real provider/network failures still disconnect without a scripted replacement; request-budget/rate-limit failures now have specific diagnostics.

Provider wire version is 4; room wire version is 5. All players and the caller daemon must run the matching build. Free-form language is not formally proven fact-consistent: withheld answers are excluded, fact IDs are checked, and unsupported text cannot alter authority, but semantic consistency still requires scenario and live dialogue testing.
