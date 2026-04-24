# Backend Codemap

**Last Updated:** 2026-03-25
**Entry Points:** None detected
**Total Files:** 64
**Total Lines:** 17497

## Entry Points
_No explicit entry points detected_

## Architecture
```
benches/
  └── diet_compression.rs
  └── memory_store.rs
  └── security_audit.rs
src/
  └── cli.rs
  └── config.rs
  └── error.rs
  └── gateway.rs
  └── lib.rs
  └── main.rs
  └── provider.rs
  └── setup.rs
  └── telegram.rs
  └── tool.rs
  └── types.rs
  agent/
    └── context.rs
    └── loop.rs
    └── mod.rs
  channels/
    └── discord.rs
    └── email.rs
    └── homeassistant.rs
    └── mod.rs
    └── router.rs
    └── signal.rs
    └── slack.rs
    └── whatsapp.rs
  mcp/
    └── cache.rs
    └── client.rs
    └── diet.rs
    └── mod.rs
    └── registry.rs
  memory/
    └── mod.rs
    └── store.rs
  providers/
    └── anthropic.rs
    └── mod.rs
    └── openai.rs
    └── openrouter.rs
    └── streaming.rs
    └── zai.rs
  security/
    └── audit.rs
    └── capabilities.rs
    └── mod.rs
  skills/
    └── agentskills.rs
    └── bundled.rs
    └── executor.rs
    └── loader.rs
    └── manifest.rs
    └── mod.rs
  tools/
    └── builtin.rs
    └── mod.rs
  tui/
    └── app.rs
    └── events.rs
    └── mod.rs
    └── ui.rs
tests/
  └── integration_agent.rs
  └── integration_channels.rs
  └── integration_config.rs
  └── integration_diet.rs
  └── integration_memory.rs
  └── integration_providers.rs
  └── integration_security.rs
  └── integration_skill_execution.rs
  └── integration_skills.rs
  └── integration_tui.rs
  └── integration_types.rs
```

## Key Modules
| File | Lines | Purpose |
|------|-------|---------|
| `tests/integration_skill_execution.rs` | 1568 | Integration skill execution module |
| `src/skills/bundled.rs` | 1553 | IP |
| `src/config.rs` | 1045 | Config module |
| `src/setup.rs` | 987 | Setup module |
| `src/telegram.rs` | 635 | Telegram module |
| `src/tools/builtin.rs` | 474 | Builtin module |
| `src/mcp/client.rs` | 449 | Client module |
| `src/mcp/diet.rs` | 449 | Diet module |
| `src/providers/zai.rs` | 381 | Zai module |
| `tests/integration_diet.rs` | 381 | Integration diet module |
| `src/providers/openrouter.rs` | 363 | Openrouter module |
| `src/providers/anthropic.rs` | 361 | Anthropic module |
| `src/types.rs` | 349 | Types module |
| `src/memory/store.rs` | 345 | Store module |
| `src/main.rs` | 336 | Main module |

## Data Flow
1. HTTP Requests → Route Handlers
1. Middleware → Authentication/Validation
1. Controllers → Business Logic
1. Services → Data Access

## External Dependencies
_No external dependencies detected_

## Related Areas
- [frontend.md](./frontend.md) - Frontend modules
- [database.md](./database.md) - Database modules
- [integrations.md](./integrations.md) - Integrations modules
- [workers.md](./workers.md) - Workers modules
