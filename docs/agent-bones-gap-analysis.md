# Agent-bones → Ferroclaw improvement analysis

Date: 2026-04-24
Input repo: `/Users/ghost/Desktop/agent-bones`
Compared against: `/Users/ghost/Desktop/Projects/ferroclaw`

## Executive summary
Yes — ferroclaw can be materially improved using patterns from agent-bones, especially for your active p3 wave (event pipeline, lifecycle, interruption/resume, telemetry).

Highest-impact gaps:
1. No first-class deterministic StopContract in ferroclaw loop returns.
2. No explicit run-level invariant checks equivalent to agent-bones I1–I5.
3. Event taxonomy is present, but run terminal outcomes are not normalized into a machine-readable stop object.
4. Gateway returns bounded fallback text on errors/timeouts but does not expose structured stop reason/metrics contract.
5. Missing conformance case harness focused on runtime invariants.

## What agent-bones does well (reusable)
- Explicit stop contract type + reason enum + metrics:
  - `core-agent/src/types.ts` lines 66-80
  - `core-agent/src/policy.ts` lines 7-21
- Deterministic bounded halt reasons:
  - `core-agent/src/loop.ts` lines 38-49, 125-151, 163-171
- Empty-final-after-tools guard:
  - `core-agent/src/loop.ts` lines 98-111
- Retry classifier split retryable vs non-retryable:
  - `core-agent/src/errors.ts` lines 1-14
- Conformance cases encoded as data:
  - `core-agent/evals/tasks.conformance.json`

## Ferroclaw current state (relevant evidence)
- Loop has rich streaming `AgentEvent` model:
  - `src/agent/loop.rs` lines 34-71
- Loop terminates with raw `Result<String>` / errors, not structured stop object:
  - `src/agent/loop.rs` lines 138-145, 191-210, 293-377
- Provider responses carry optional provider stop reason only:
  - `src/types.rs` lines 248-252
- Gateway summarizes tokens/tool count but returns text-centric response body; no canonical stop contract field:
  - `src/gateway.rs` lines 183-197, 199-215, 217-233

## Concrete improvements to apply in ferroclaw

### A) Add canonical `RunStopContract` + `RunOutcome`
Add to `src/types.rs`:
- `RunStopReason` enum (assistant_final, budget_iterations, budget_tokens, budget_wall_clock, budget_tools_iteration, budget_tools_total, error_non_retryable, error_retry_exhausted, error_empty_final_after_tools, interrupted)
- `RunStopContract { reason, iterations, tool_calls_total, elapsed_ms, notes? }`
- `RunOutcome { text, stop, usage?, events? }`

Then change loop API from:
- `run_with_callback(...) -> Result<String>`
To:
- `run_with_callback(...) -> Result<RunOutcome>`

Impact: deterministic terminal semantics everywhere (TUI, CLI, gateway, harness).

### B) Enforce explicit invariants in loop
Implement invariant checks near loop boundaries:
- I1: user message exists in provider input
- I2: every model-emitted tool call produces either tool result or explicit tool-error result
- I3: tool totals/iteration caps
- I4: no empty final text after at least one tool batch
- I5: every halt path emits stop contract

This directly maps to agent-bones SPEC invariants.

### C) Normalize retry outcomes as stop reasons
Current behavior mixes errors and fallback text paths.
Unify to stop reasons:
- retry exhausted -> `error_retry_exhausted`
- non-retryable provider error -> `error_non_retryable`

### D) Gateway response should include stop contract
For `/v1/responses`, include stop metadata in response payload (or an extension block), derived from `RunOutcome.stop`.
This gives harness/runtime parity and removes ambiguity around pass/fail validity.

### E) Add conformance test file + runner assertions
Mirror agent-bones approach with ferroclaw-specific cases:
- final-non-empty-after-tools
- user-message-required
- non-retryable-4xx-no-replay
- budgeted-stop-contract
- interrupt-emits-stopped-reason

Wire into CI tests so regressions fail fast.

### F) Telemetry lane parity for p3
You already have event streaming (`AgentEvent`); next step is to persist per-turn lifecycle envelope:
- run_id
- iteration events
- tool start/result timestamps
- terminal stop contract

This closes the p3 telemetry-surface objective cleanly.

## Recommended implementation order (p3-aligned)
1. Introduce `RunStopContract` types and adapt loop return type.
2. Patch TUI/CLI/gateway callers to consume `RunOutcome`.
3. Add empty-final-after-tools and deterministic halt coverage.
4. Expose stop contract in gateway responses.
5. Add conformance tests + CI gate.

## Why this matters for your current pain points
- Eliminates ambiguous terminal states ("ready" vs actually failed).
- Makes real-time traces and final state consistent.
- Gives hard validity criteria so setup/error outputs never count as success.
- Improves interruption/resume accounting with explicit stop reason provenance.
