# Ferroclaw Architecture

## System Overview

```
                            ┌──────────────────────────────────────────┐
                            │              ferroclaw binary            │
                            │                 (5.4 MB)                 │
                            ├──────────────────────────────────────────┤
                            │                                          │
┌─────────┐  ┌──────────┐  │  ┌──────────┐    ┌───────────────────┐  │
│  CLI    ├──┤ Gateway  ├──┼─▶│  Agent   │───▶│   LLM Provider    │  │
│  REPL   │  │ REST API │  │  │  Loop    │    │ (Anthropic/OpenAI │  │
└─────────┘  └──────────┘  │  │  (ReAct) │    │  /Zai/OpenRouter) │  │
                            │  └────┬─────┘    └───────────────────┘  │
┌──────────┐               │       │                                   │
│ Telegram ├───────────────┤       ▼                                   │
│   Bot    │               │  ┌────────────┐   ┌───────────────────┐  │
└──────────┘               │  │   Tool     │──▶│   MCP Client      │  │
                            │  │  Registry  │   │ + DietMCP Layer   │  │
                            │  └────┬───────┘   └──────┬────────────┘  │
                            │       │                   │              │
                            │       ▼                   ▼              │
                            │  ┌─────────┐    ┌─────────────────┐     │
                            │  │Security │    │  MCP Servers     │     │
                            │  │Caps+Audit│   │  (stdio/SSE)    │     │
                            │  └─────────┘    └─────────────────┘     │
                            │       │                                  │
                            │       ▼                                  │
                            │  ┌──────────┐                            │
                            │  │ Memory   │                            │
                            │  │SQLite+FTS│                            │
                            │  └──────────┘                            │
                            └──────────────────────────────────────────┘
```

## Module Map

### Entry Points (`src/main.rs`, `src/cli.rs`)

The CLI uses `clap` derive macros for type-safe argument parsing:

```
ferroclaw run              → Interactive REPL
ferroclaw exec <prompt>    → One-shot execution
ferroclaw mcp list         → List MCP servers + tools
ferroclaw mcp diet         → Show DietMCP skill summaries
ferroclaw config init      → Generate config file
ferroclaw serve            → Start gateway + Telegram
ferroclaw audit verify     → Verify audit log integrity
```

### Agent Loop (`src/agent/`)

The core ReAct loop:

```
1. Assemble context
   ├── System prompt
   ├── DietMCP skill summaries (compressed)
   ├── Conversation history
   └── Memory context

2. Call LLM provider
   └── Returns: text | tool_use blocks

3. If tool_use:
   ├── Check capabilities (15 ns)
   ├── Route to handler
   │   ├── Built-in tool → direct execution
   │   └── MCP tool → MCP client → server
   ├── Format response (DietMCP)
   ├── Append tool_result
   └── Loop to step 2

4. If text response:
   └── Return to user
```

**Budget enforcement**: Token usage is tracked per-turn. The context manager prunes old messages (sliding window) when approaching the budget limit, preserving system messages and recent turns.

### Provider Layer (`src/providers/`)

Four providers, unified behind the `LlmProvider` trait:

| Provider | Module | Model Routing |
|----------|--------|---------------|
| Anthropic | `anthropic.rs` | `claude-*` prefixed models |
| Zai GLM | `zai.rs` | `glm-*` prefixed models |
| OpenRouter | `openrouter.rs` | `provider/model` format (contains `/`) |
| OpenAI | `openai.rs` | Fallback for all other models |

Routing priority: Zai → OpenRouter → Anthropic → OpenAI.

Each provider:
- Formats messages to the API's expected shape
- Parses tool_use/function_call blocks from responses
- Extracts token usage for budget tracking
- Returns a normalized `ProviderResponse`

### MCP + DietMCP (`src/mcp/`)

**MCP Client** (`client.rs`):
- Spawns MCP servers as child processes (stdio transport)
- Sends JSON-RPC: `initialize` → `notifications/initialized` → `tools/list`
- Executes tool calls via `tools/call`
- 30s discovery timeout, 60s execution timeout

**DietMCP** (`diet.rs`) — the core differentiator:

```
Raw JSON Schema (9 tools, filesystem)     →  ~4,200 bytes
DietMCP Compact Summary                   →  ~800 bytes
Compression: ~81%
```

How it works:
1. **Categorize** tools by keyword (file, search, git, network, etc.)
2. **Compact signatures**: `read_file(path: str)` instead of full JSON schema
3. **Truncate descriptions** to 80 chars
4. **Format responses**: Summary (truncate), Minified (strip nulls), CSV (tabular)
5. **Auto-redirect**: Responses >50KB → temp file, return pointer

**Schema Cache** (`cache.rs`):
- SHA256-keyed by server name + config fingerprint
- Configurable TTL per server
- Atomic writes (write tmp → rename)
- File-based, survives restarts

### Security (`src/security/`)

**Capabilities** (`capabilities.rs`):

8 capability types form a permission matrix:

| Capability | Grants | Default |
|-----------|--------|---------|
| `fs_read` | Read files, list directories | Yes |
| `fs_write` | Write/delete files | **No** |
| `net_outbound` | HTTP requests | Yes |
| `net_listen` | Bind server sockets | **No** |
| `process_exec` | Execute shell commands | **No** |
| `memory_read` | Search/get memories | Yes |
| `memory_write` | Store/delete memories | Yes |
| `browser_control` | Browser automation | **No** |

Check time: **15.5 ns** (pass), **18.4 ns** (fail).

**Audit Log** (`audit.rs`):

```
Entry N:
  timestamp: 2026-03-15T12:00:00Z
  tool_name: "read_file"
  arguments_hash: SHA256(args)     ← args never stored in full
  result_hash: SHA256(result)      ← results never stored in full
  previous_hash: Entry(N-1).hash  ← chain link
  entry_hash: SHA256(all above)   ← tamper detection
```

Verification walks the chain: if any entry is modified, inserted, or deleted, the hash chain breaks. Verification of 1,000 entries: **2.97 ms**.

### Memory (`src/memory/`)

SQLite with FTS5 full-text search:

```sql
-- Main table
CREATE TABLE memories (
    id INTEGER PRIMARY KEY,
    key TEXT UNIQUE,
    content TEXT,
    created_at TEXT,
    updated_at TEXT
);

-- FTS5 virtual table (auto-synced via triggers)
CREATE VIRTUAL TABLE memories_fts USING fts5(key, content);

-- Conversation history
CREATE TABLE conversations (
    id INTEGER PRIMARY KEY,
    session_id TEXT,
    role TEXT,
    content TEXT,
    timestamp TEXT
);
```

Performance:
- Insert: 70 µs per entry
- FTS5 search: 69-267 µs depending on corpus size
- Conversation save: 43 µs per turn

### Built-in Tools (`src/tools/`)

7 tools ship with every Ferroclaw binary:

| Tool | Capability Required | Description |
|------|-------------------|-------------|
| `read_file` | `fs_read` | Read file contents |
| `write_file` | `fs_write` | Write content to file |
| `list_directory` | `fs_read` | List directory entries |
| `bash` | `process_exec` | Execute shell commands |
| `web_fetch` | `net_outbound` | HTTP GET with size limits |
| `memory_search` | `memory_read` | FTS5 full-text search |
| `memory_store` | `memory_write` | Store key-value memories |

### Gateway (`src/gateway.rs`)

HTTP API that **refuses to bind 0.0.0.0 without authentication**:

```rust
if self.bind_addr == "0.0.0.0" && self.bearer_token.is_none() {
    return Err("Refusing to bind to 0.0.0.0 without bearer_token");
}
```

This directly addresses OpenClaw's CVE-2026-25253 attack vector.

### Channels (`src/channels/`)

Abstraction trait for multi-platform messaging:

```rust
pub trait Channel: Send + Sync {
    fn name(&self) -> &str;
}
```

Current implementations:
- CLI REPL (interactive)
- HTTP Gateway (REST API)
- Telegram Bot (long-polling, allowlist-gated)

## Data Flow

```
User Input
    │
    ▼
CLI / Gateway / Telegram
    │
    ▼
Agent Loop
    │
    ├─── Context Manager (token budget check)
    │
    ├─── System Prompt + DietMCP Summaries
    │
    ▼
LLM Provider (Anthropic / OpenAI / Zai / OpenRouter)
    │
    ├─── Text Response → return to user
    │
    └─── Tool Call(s)
         │
         ├── Capability Check (15 ns)
         │   └── Denied? → error message, no execution
         │
         ├── Built-in Tool → execute directly
         │   └── Audit Log (hash-chained entry)
         │
         └── MCP Tool → MCP Client → server process
             ├── DietMCP format response
             └── Audit Log (hash-chained entry)
                 │
                 ▼
         Append tool_result → loop back to LLM
```

## Configuration

Single TOML file at `~/.config/ferroclaw/config.toml`:

```toml
[agent]
default_model = "claude-sonnet-4-20250514"  # or "glm-5" or "openai/gpt-4o"
max_iterations = 30
token_budget = 200000

[providers.anthropic]
api_key_env = "ANTHROPIC_API_KEY"

[providers.zai]
api_key_env = "ZAI_API_KEY"

[providers.openrouter]
api_key_env = "OPENROUTER_API_KEY"

[security]
default_capabilities = ["fs_read", "net_outbound", "memory_read", "memory_write"]

[mcp_servers.filesystem]
command = "npx"
args = ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"]
```

## Build

```bash
cargo build --release    # 5.4 MB binary, LTO + strip
cargo test --all         # 119 tests
cargo bench              # Criterion benchmarks with HTML reports
```
