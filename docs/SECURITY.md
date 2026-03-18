# Ferroclaw Security Model

Ferroclaw was designed in direct response to the security failures of existing agent frameworks, particularly OpenClaw's CVE-2026-25253 and its 0.0.0.0 binding default.

---

## Design Principles

1. **Deny by default**: Only 4 of 8 capabilities are granted in the default configuration
2. **No eval/exec of untrusted input**: Tool arguments are validated against JSON schemas, never evaluated
3. **Localhost by default**: Gateway binds 127.0.0.1, refuses 0.0.0.0 without authentication
4. **Tamper-evident audit**: Every tool call is logged in a hash-chained append-only log
5. **No marketplace**: Skills are loaded from local paths, verified by Ed25519 signatures

---

## Capability System

### The 8 Capabilities

| Capability | What It Grants | Risk Level | Default |
|-----------|---------------|------------|---------|
| `fs_read` | Read files, list directories | Low | Enabled |
| `fs_write` | Create, modify, delete files | Medium | **Disabled** |
| `net_outbound` | Make HTTP requests to external services | Low | Enabled |
| `net_listen` | Bind server sockets, accept connections | High | **Disabled** |
| `process_exec` | Execute shell commands via bash | High | **Disabled** |
| `memory_read` | Search and retrieve memories | Low | Enabled |
| `memory_write` | Store and delete memories | Low | Enabled |
| `browser_control` | Automate browser interactions | High | **Disabled** |

### How It Works

1. Each tool declares its required capabilities in `ToolMeta`
2. The session has a `CapabilitySet` derived from config
3. Before every tool execution, `CapabilitySet::check()` runs (15.5 ns)
4. If a required capability is missing, execution is blocked with an actionable error:

```
Error: Tool 'bash' requires capability 'process_exec' which is not granted.
Current capabilities: [fs_read, memory_read, memory_write, net_outbound].
Add 'process_exec' to security.default_capabilities in config.toml to allow.
```

### Configuration

```toml
[security]
# Minimum-privilege default: read-only filesystem + outbound HTTP + memory
default_capabilities = ["fs_read", "net_outbound", "memory_read", "memory_write"]

# Full access (use with caution):
# default_capabilities = ["fs_read", "fs_write", "net_outbound", "net_listen",
#                          "process_exec", "memory_read", "memory_write", "browser_control"]
```

### Comparison with Alternatives

| Framework | Default Permissions |
|-----------|-------------------|
| OpenClaw | Full disk, terminal, browser, network access |
| Hermes Agent | No permission model |
| NanoClaw | No permission model |
| Claude Code | 3 modes: ask (interactive) / auto-accept / plan |
| Codex CLI | OS-level sandbox (all-or-nothing) |
| **Ferroclaw** | **8 independent capability flags, 4 enabled by default** |

---

## Audit Log

### Format

JSON Lines file (`.jsonl`), one entry per tool call:

```json
{
  "timestamp": "2026-03-15T12:00:00Z",
  "tool_name": "read_file",
  "arguments_hash": "a1b2c3...",
  "result_hash": "d4e5f6...",
  "is_error": false,
  "previous_hash": "789abc...",
  "entry_hash": "def012..."
}
```

### Key Properties

| Property | Implementation |
|----------|---------------|
| **Append-only** | File opened with `O_APPEND` |
| **Hash-chained** | Each entry hashes the previous entry's hash |
| **Tamper-evident** | Modification, insertion, or deletion breaks the chain |
| **Privacy-preserving** | Arguments and results are SHA256-hashed, not stored in full |
| **Portable** | Standard JSONL format, readable by any tool |

### Tamper Detection

```bash
# Verify audit log integrity
ferroclaw audit verify

# Output:
# Audit log valid: 1,247 entries verified
# — or —
# AUDIT LOG TAMPERED: chain broken at entry 892
```

Verification performance: 1,000 entries in **2.97 ms**.

### What Tampering Looks Like

| Attack | Detection |
|--------|-----------|
| Modify an entry | Entry hash doesn't match recomputed hash |
| Delete an entry | Next entry's `previous_hash` doesn't match |
| Insert a fake entry | Fake `previous_hash` doesn't match real chain |
| Truncate the log | Missing entries detected by gap in chain |

---

## Gateway Security

### Binding Rules

| Bind Address | Bearer Token | Allowed? |
|-------------|-------------|----------|
| `127.0.0.1` | Not set | Yes (localhost only, safe) |
| `127.0.0.1` | Set | Yes |
| `0.0.0.0` | Set | Yes (warning logged) |
| `0.0.0.0` | **Not set** | **BLOCKED** |

This directly prevents the OpenClaw vulnerability class where the agent is exposed to the network without authentication.

### Authentication

When `bearer_token` or `bearer_token_env` is configured:

```
Authorization: Bearer <token>
```

All endpoints require the token. Unauthenticated requests receive 401.

---

## MCP Tool Security

### Capability Inference

MCP-discovered tools have capabilities inferred from their names:

| Name Pattern | Inferred Capability |
|-------------|-------------------|
| `read_*`, `list_*`, `get_*` | `fs_read` |
| `write_*`, `create_*`, `delete_*` | `fs_write` |
| `exec_*`, `run_*`, `bash_*` | `process_exec` |
| `fetch_*`, `http_*`, `api_*` | `net_outbound` |
| `browser_*`, `navigate_*` | `browser_control` |
| Unknown | `fs_read` (minimum) |

### Process Isolation

Each MCP server runs as a separate child process:
- `stdin`/`stdout` for JSON-RPC communication
- `stderr` is null (suppressed)
- Processes are killed after tool discovery/execution
- 30-second timeout on discovery, 60-second on execution

---

## Skill Verification (Planned)

Infrastructure for Ed25519-signed skill manifests is in place:

```rust
// Dependencies in Cargo.toml
ed25519-dalek = { version = "2", features = ["rand_core"] }
sha2 = "0.10"
```

Planned workflow:
1. Skill author signs manifest with their Ed25519 private key
2. Ferroclaw verifies signature against known public keys before loading
3. Unsigned skills are rejected when `require_skill_signatures = true`

---

## Threat Model

### In Scope

| Threat | Mitigation |
|--------|-----------|
| Malicious MCP tool attempts file write | Blocked by `fs_write` capability (disabled by default) |
| LLM prompt injection causes shell execution | Blocked by `process_exec` capability (disabled by default) |
| Agent exposed to network | Gateway binds 127.0.0.1, requires auth for 0.0.0.0 |
| Audit log tampering | SHA256 hash chain detects any modification |
| Untrusted skill installation | Ed25519 signature verification |
| Large response flooding context | DietMCP auto-redirect (>50KB → temp file) |

### Out of Scope (v0.1.0)

| Threat | Status |
|--------|--------|
| LLM model extraction | Relies on provider-side protections |
| Side-channel attacks on SQLite | Standard SQLite security model |
| Supply-chain attacks on Rust crates | Audited via `cargo audit` |
| Denial of service via token exhaustion | Token budget limits mitigate but don't fully prevent |
