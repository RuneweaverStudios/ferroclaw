# Ferroclaw ↔ Orca Orchestrator Parity Plan

Status: draft
Owner: Hermes

## Goal
Replicate Orca orchestrator capabilities/behavior in Ferroclaw without regressions in terminal UX.

## Scope
- Slash command UX and skill activation behavior
- Skill discovery behavior/path coverage
- Orchestrator event and tool lifecycle behavior
- Interruption/resume behavior
- Telemetry and run-state visibility
- Canvas bridge and tool contract parity

## Principles + philosophy contract (from Hermes research)
These are non-negotiable and must be encoded as behavior, tests, and acceptance gates.

1) Safety-first execution ordering
- Priority order: safety > immediate user request > grounding/evidence > continuity > speed/verbosity.
- Never trade correctness/safety for a faster-looking response.

2) Interruption/resume is first-class
- Answer interruption immediately.
- Capture checkpoint at interruption.
- Provide explicit resume handoff.
- Resume from checkpoint (not restart).
- Preserve todo/run-state continuity across interruption.

3) Honest uncertainty over fabricated certainty
- If context is missing/ambiguous, retrieve or state uncertainty explicitly.
- Tool/setup/runtime errors are reported as errors, never as successful outcomes.

4) Deterministic observability
- Per-step event stream for model/tool lifecycle must be reconstructable.
- Telemetry fields (tokens/tool calls/timing) must be complete and auditable.

5) Evaluation integrity
- No fixture backfill that inflates runtime telemetry.
- Strict-valid benchmark claims only from reproducible artifacts.

## Critical learnings to carry into implementation
- Queue-on-busy behavior is preferable to hard reject for chat gateways, but with deterministic dequeue semantics.
- Interruption queued-path needs dedicated regression tests (checkpoint capture -> interruption-first response -> resume -> checkpoint clear).
- Memory/recall changes must prove behavior improvements, not just storage changes.
- Conformance and runtime reliability gates come before new feature surface area.

## Current parity snapshot (initial)
- Implemented in Ferroclaw now:
  - Direct slash skill activation (`/<skill>`), `/skills`, `/skills rescan`, `/use`, `/unuse`, `/active-skills`
  - Deep SKILL.md discovery under user/cwd roots including `.claude/skills/<skill>/SKILL.md`
  - Top-of-chat info block with scroll + slash hints and discovered count
  - Dense transcript formatting and framed assistant responses
- Remaining major gaps:
  - Full Orca run-state/event model parity
  - Interruption/resume checkpoint semantics parity
  - Full canvas orchestrator behavior parity (tools + execution lifecycle)
  - Matching telemetry richness for each run/tool call

## Wave 1: UX parity (TUI and slash)
- [x] Remove mouse capture for text selection
- [x] Slash menu open/filter/keyboard behavior
- [x] Direct `/<skill>` command activation
- [x] Top-of-chat info block (not message stream)
- [ ] Exact command edge-case compatibility with Orca parser

Acceptance:
- Typing `/` produces useful suggestions
- Enter executes command; Tab accepts suggestion
- Skill discovery count updates on `/skills rescan`

## Wave 2: Orchestrator runtime parity
- [ ] Define Ferroclaw run-state phases equivalent to Orca
- [ ] Emit per-step events (model call, tool call start/result, errors, completion)
- [ ] Normalize cancellation/interruption behavior
- [ ] Add resume from interrupted run checkpoints

Acceptance:
- Interrupted runs can be resumed with deterministic state restoration
- Tool lifecycle events and ordering match Orca expectations

## Wave 3: Canvas parity
- [skipped] Per user direction: canvas parity is out of scope for this Ferroclaw track.
- [skipped] Build adapter parity for Orca tool manifest and execution protocol
- [skipped] Validate against `/api/canvas/tools` and `/api/canvas/execute`
- [skipped] Ensure workspace, tile, and run synchronization behavior

Acceptance:
- N/A (wave intentionally skipped)

## Wave 4: Telemetry + benchmark parity
- [ ] Add non-null telemetry surfaces for tokens, tools, and per-step timing
- [ ] Harden output formatting and event capture for benchmark adapters
- [ ] Re-run strict-valid parity benchmark against Orca targets

Acceptance:
- Strict-valid benchmark run with complete telemetry fields
- Reproducible report artifacts

## Immediate next step
Implement Wave 2 event/run-state mapping and wire interruption/resume checkpoints first.
