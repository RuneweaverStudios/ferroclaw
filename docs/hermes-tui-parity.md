# Hermes TUI parity plan (ui-tui + tui_gateway)

References:
- https://github.com/NousResearch/hermes-agent/tree/main/ui-tui
- https://github.com/NousResearch/hermes-agent/tree/main/tui_gateway

## Goal
Match Hermes chat runtime behavior:
- stable bounded layout (no terminal growth artifacts)
- deterministic scroll behavior
- real-time event/trace lifecycle
- resilient gateway transport and crash telemetry
- slash/runtime parity where practical in Rust TUI

## Source architecture we are matching
- `ui-tui/src/components/appLayout.tsx`: strict transcript pane + composer pane separation, internal scroll box
- `ui-tui/src/components/messageLine.tsx`: sectioned details (thinking/tools/activity), deterministic rendering
- `ui-tui/src/gatewayClient.ts`: buffered events, startup timeouts, stderr channel, request timeout, crash-safe gateway lifecycle
- `tui_gateway/entry.py`: stdout protocol isolation, signal/crash logging, explicit ready event
- `tui_gateway/ws.py`: same RPC over ws with transport-safe writes
- `tui_gateway/server.py`: long-handler pool dispatch to keep interrupt/approval responsive

## Ferroclaw gap map
1) Render/runtime isolation
- Needed: no shell scrollback bleed, bounded viewport, no edge-cell drift
- Status: in progress; alternate screen + mouse capture restored, edge-cell guard added

2) Transcript model
- Needed: bounded history + stable scroll offsets + no duplicate frame paints
- Status: in progress; added chat entry cap (`MAX_CHAT_ENTRIES=1500`)

3) Event pipeline
- Needed: non-blocking run/event loop parity (`ready`, stream deltas, tool lifecycle, stderr lane)
- Status: partial; callback streaming present, still needs dedicated transport/event channel abstraction

4) Gateway robustness
- Needed: Hermes-style gateway subprocess transport with timeout, stderr telemetry, crash log, signal handling
- Status: missing (currently direct runtime in-process)

5) Interrupt/resume responsiveness
- Needed: long-running work off main dispatcher path (Hermes `_LONG_HANDLERS` style)
- Status: partial

## Execution waves
### Wave A (now)
- [x] Alternate screen + mouse capture for isolated chat runtime
- [x] Remove startup demo tasks
- [x] Cap transcript history to bounded size
- [ ] Validate no duplicate frame growth in repeated tool-heavy runs

### Wave B
- [ ] Introduce Rust-side gateway transport abstraction (stdio lane + stderr lane + ready event)
- [ ] Add startup/request timeout controls
- [ ] Persist crash/exit reason logs for TUI subprocess/runtime

### Wave C
- [ ] Split details sections (thinking/tools/activity) with explicit visibility contract
- [ ] Add stable virtualized transcript row model to preserve smooth scrolling under long output

### Wave D
- [ ] Interrupt/resume hardening: ensure long tool operations do not block control input path
- [ ] Add telemetry surfaces equivalent to Hermes gateway events

## Acceptance criteria
1) Tool-heavy run with thousands of lines does not grow terminal vertically or repaint duplicate frames
2) Mouse + key scroll always acts on transcript pane, never shell scrollback
3) Tool calls stream in real time; no post-hoc dump
4) Interrupt control remains responsive during long tool execution
5) Crash path writes actionable logs with reason and stack context
