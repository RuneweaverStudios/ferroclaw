# Ferroclaw vs. The Field: Competitive Analysis

A systematic comparison of Ferroclaw against five leading AI agent frameworks, backed by primary-source research from official repos, docs, and CVE databases.

---

## Executive Summary

| Metric | Ferroclaw | OpenClaw | Hermes Agent | NanoClaw | Claude Code | Codex CLI |
|--------|-----------|----------|-------------|----------|-------------|-----------|
| **Language** | Rust | TypeScript | Python | TypeScript | TypeScript (Bun) | Rust + TypeScript |
| **Binary/Install** | 5.4 MB single binary | ~1.4 GB (npm + node_modules) | ~93 MB (repo) + 124 pip pkgs | ~15 files + containers | 56.1 MB (npm unpacked) | ~44 MB (binary, compressed) |
| **Runtime Required** | None | Node.js 22+ | Python 3.11+ | Node.js 20+ + Docker | Node.js 18+ | None (Rust binary) |
| **Source Lines** | 7,486 | ~430,000 | ~174,000 | ~3,900 | Proprietary | 40+ Rust crates |
| **Default Bind** | 127.0.0.1 | 0.0.0.0 (canvas host) | N/A | N/A | N/A | N/A |
| **Permission Model** | 8 capability types | None (until post-CVE patches) | 5-layer defense | Container isolation | 5 modes + Seatbelt sandbox | Seatbelt/Bubblewrap + Starlark rules |
| **Audit Log** | Hash-chained | None | None | None | None | None |
| **Providers** | 4 (Anthropic, OpenAI, Zai GLM, OpenRouter) | 15+ | 10+ (OpenRouter, Anthropic, z.ai, Kimi, etc.) | Anthropic-compatible only | 4 (Anthropic, Bedrock, Vertex, Azure) | 3 built-in (OpenAI, Ollama, LM Studio) + custom |
| **MCP Support** | Native + DietMCP | MCP server (via SDK) | Full MCP client (~1,050 lines) | Custom MCP server (stdio) | Native client | MCP server (experimental) |
| **DietMCP Compression** | Native (70-93%) | None | None | None | None | BM25 tool_search for large tool sets |
| **Memory** | SQLite + FTS5 | Markdown files | SQLite + FTS5 | CLAUDE.md hierarchy | File-based (~/.claude/) + auto-memory | SQLite + two-phase memory pipeline |
| **Open Source** | MIT | MIT | MIT | MIT | Proprietary | Apache-2.0 |

---

## Detailed Comparisons

### 1. Ferroclaw vs. OpenClaw

#### Security

OpenClaw has the worst security track record in the agent ecosystem:

| Vulnerability | OpenClaw | Ferroclaw |
|--------------|----------|-----------|
| **CVE-2026-25253** (CVSS 8.8) | One-click RCE: malicious link → WebSocket token exfiltration → `operator.admin` scope → sandbox escape via `tools.exec.host=gateway` → arbitrary code execution. 30,000+ instances observed by Bitsight scans. | Not applicable: no WebSocket control UI, no marketplace, signed skill manifests |
| **Canvas host binding** | `0.0.0.0` by default with **no authentication** (GitHub Issue #5263, classified P1/High, **closed as "Not Planned"**) | `127.0.0.1:8420` — refuses `0.0.0.0` without bearer token |
| **Malicious skills** | ClawHub marketplace: 7-26% malicious depending on scanner (Snyk: 7.1% critical flaws + 76 intentionally malicious; Cisco: 26% with vulnerabilities; bitdoze audit: ~800/~20%). "ClawHavoc" campaign used 3 delivery mechanisms including macOS Atomic Stealer. | Ed25519-signed skill verification, no untrusted registry |
| **Adversarial defense** | 17% baseline defense rate across 47 adversarial scenarios (arXiv:2603.10387). 40+ additional security fixes in 2026.2.x-3.x releases. | Schema-validated tool args, no eval/exec of untrusted input |
| **Audit trail** | None | SHA256 hash-chained append-only log with tamper detection |

#### Architecture

| Aspect | OpenClaw | Ferroclaw |
|--------|----------|-----------|
| Runtime | Node.js 22+, pnpm monorepo | Rust, single static binary |
| Source | ~430,000 lines, 53 config files | 7,486 lines, 32 files |
| Install | **1.4 GB** npm install (largest dep: node-llama-cpp at 752 MB) | 5.4 MB binary |
| Channels | 23+ platform adapters | CLI + HTTP gateway + 7 messaging channels |
| Skills | 3,200+ on ClawHub (7-26% malicious) | 84 bundled + MCP tools + custom TOML + AgentSkills.io |
| MCP | Server-side via `@modelcontextprotocol/sdk` | Client-side + DietMCP compression |

#### Performance

| Operation | OpenClaw | Ferroclaw |
|-----------|----------|-----------|
| Cold start | ~6-8 seconds | <50ms |
| Idle RAM | ~1.2 GB | ~5-15 MB |
| Tool schema processing | Raw JSON (full token cost) | DietMCP compression (70-93% reduction) |
| Capability check | Not performed | 15.5 ns per check |

*OpenClaw performance data from community benchmarks (waelmansour.com Claw ecosystem comparison).*

---

### 2. Ferroclaw vs. Hermes Agent

#### Architecture

| Aspect | Hermes Agent | Ferroclaw |
|--------|-------------|-----------|
| Language | Python 3.11+ | Rust |
| Core agent | `run_agent.py` — 6,159 lines, synchronous loop | `agent/loop.rs` — ~200 lines, async Tokio |
| Total LoC | ~174,000 (442 Python files) | 7,486 (32 Rust files) |
| Dependencies | 124 locked packages (core), 14 optional groups | 269 compile-time, 0 runtime |
| Repo size | ~93 MB | 5.4 MB binary |
| Frontends | TUI CLI, messaging gateway, ACP editor server, batch runner | CLI REPL, HTTP gateway, Telegram |

#### Feature Parity

| Feature | Hermes Agent | Ferroclaw |
|---------|-------------|-----------|
| Providers | 10+ (OpenRouter, Anthropic, OpenAI, z.ai/GLM, Kimi, MiniMax, Codex OAuth) | 4 (Anthropic, OpenAI, Zai GLM, OpenRouter) |
| MCP client | Full implementation (~1,050 lines, stdio + HTTP transports, reconnection) | Native (stdio transport, caching, DietMCP compression) |
| Tool calling | 40+ integrated tools + subagent delegation | 7 built-in + 84 skill tools + MCP tools |
| Memory | SQLite + FTS5 (session DB with schema v4) | SQLite + FTS5 |
| Skills | 70+ bundled/optional across 15+ categories, AgentSkills.io compatible | **84 bundled across 16 categories, AgentSkills.io compatible** + MCP tools + custom TOML skills |
| Messaging | Telegram, Discord, Slack, WhatsApp, Signal, Email, Home Assistant | **Telegram, Discord, Slack, WhatsApp, Signal, Email, Home Assistant** + HTTP gateway |
| Security | 5-layer defense: user auth, dangerous command approval, container isolation, credential filtering, injection scanning + Tirith pre-exec scanner | 8 capability types + hash-chained audit |
| Context compression | Auto at 50% context window (protects first 3 + last 4 turns) | Token budget + sliding window pruning |

#### Key Differentiator

Hermes is a full-featured agent platform with 10+ providers, 40+ tools, and 7 messaging platforms — but it's ~174K lines of Python with a 6,159-line main agent loop. Ferroclaw now matches Hermes on skills (84 bundled across 16 categories with AgentSkills.io compatibility) and messaging (7 channels + HTTP gateway) while maintaining a ~8K line Rust codebase, 5.4 MB binary, and stronger security posture.

---

### 3. Ferroclaw vs. NanoClaw

#### Architecture

| Aspect | NanoClaw | Ferroclaw |
|--------|----------|-----------|
| Design philosophy | Minimal, container-per-conversation | Comprehensive, single-process |
| Codebase | ~3,900 lines, 34.9K tokens (17% of Claude context window) | 7,486 lines |
| Runtime deps | 6 packages (host) + 4 (container agent runner) | 0 runtime deps |
| Process model | SQLite polling loop → Docker/Apple Container per agent session | Single process, in-process tool execution |
| Container startup | 1-2 second overhead per container | N/A (in-process) |
| Concurrency | Max 5 concurrent containers (configurable) | Tokio async (configurable iterations) |
| Providers | Anthropic-compatible only (via `ANTHROPIC_BASE_URL`) | 4 native providers |
| Stars | 20,000+ GitHub stars | New project |

#### Security Comparison

| Aspect | NanoClaw | Ferroclaw |
|--------|----------|-----------|
| **Primary boundary** | OS-level container isolation (Docker/Apple Container) | Application-level capabilities |
| Container user | Unprivileged `node` (uid 1000) | N/A |
| Mount security | Allowlist at `~/.config/nanoclaw/mount-allowlist.json`, symlink resolution, blocked paths (.ssh, .gnupg, .aws, credentials, .env, etc.) | `fs_read`/`fs_write` capability flags |
| Credential exposure | Known limitation: agent can discover API keys via bash inside container | Capabilities can block `process_exec` entirely |
| Network | Unrestricted inside containers | `net_outbound` / `net_listen` separate capabilities |
| Audit | None | Hash-chained append-only log |

#### Key Differentiator

NanoClaw uses Docker containers as its security boundary — strong isolation but adds 1-2s latency per conversation. Ferroclaw provides application-level security at 15.5 ns per check with no container overhead, plus a tamper-evident audit trail NanoClaw lacks.

---

### 4. Ferroclaw vs. Claude Code

#### Architecture

| Aspect | Claude Code | Ferroclaw |
|--------|-------------|-----------|
| Availability | Proprietary (Anthropic subscription) | Open source (MIT) |
| Built with | TypeScript, compiled via Bun, React + Ink TUI | Rust, clap CLI |
| Install size | 56.1 MB (npm unpacked) | 5.4 MB |
| Providers | 4 (Anthropic direct, Amazon Bedrock, Google Vertex AI, Azure Foundry) | 4 (Anthropic, OpenAI, Zai GLM, OpenRouter) |
| MCP support | Full native client | Native client + DietMCP compression |
| Tool set | Read, Write, Edit, MultiEdit, Glob, Grep, Bash, WebFetch, WebSearch, Agent (subagent), TodoRead/TodoWrite, NotebookEdit | 7 built-in + MCP tools |
| Memory | CLAUDE.md files + auto-memory (session-scoped extraction) | SQLite + FTS5 |
| Context | 200K token window, prefix caching (92% reuse rate, 90% cost discount) | Configurable token budget + sliding window |

#### Security Comparison

| Feature | Claude Code | Ferroclaw |
|---------|-------------|-----------|
| Permission modes | 5 (default, acceptEdits, plan, dontAsk, bypassPermissions) | 8 capability types, config-file scoped |
| Sandboxing | Linux bubblewrap + macOS Seatbelt (reduces permission prompts by 84%) | Application-level capability gating |
| Hooks | PreToolUse (can block/modify), PostToolUse, Notification, Stop, SubagentStop | Planned |
| Tool approval | `allowedTools` with glob patterns, `prefix_rule()` | CapabilitySet check per tool call |
| Settings hierarchy | 5 levels: Enterprise MDM → CLI flags → local project → shared project → global user | Single TOML config |
| Audit log | None | Hash-chained, tamper-evident |

#### Benchmark Context

Claude Code on SWE-bench Verified:
- Claude Sonnet 4.5: 72.8-77.2%
- Claude Opus 4.5: **80.9%** (first model to exceed 80%)

Ferroclaw does not yet have SWE-bench scores (it's the framework, not the model — any of its 4 providers can be benchmarked).

#### Key Differentiator

Claude Code is the most polished agentic coding experience with deep IDE integration (VS Code, JetBrains), 92% prefix cache reuse, and 5 permission modes. But it's proprietary and Anthropic-locked. Ferroclaw is open source, supports 4 provider families, and adds DietMCP compression and tamper-evident audit logging that Claude Code lacks.

---

### 5. Ferroclaw vs. Codex CLI

#### Architecture

| Aspect | Codex CLI | Ferroclaw |
|--------|-----------|-----------|
| Core language | Rust (40+ crates in Cargo workspace) | Rust (single crate, 32 files) |
| Binary size | ~44 MB (macOS arm64, compressed) | 5.4 MB |
| Wire API | OpenAI Responses API exclusively | Anthropic Messages API + OpenAI Chat Completions |
| Default model | gpt-5.4 | claude-sonnet-4-20250514 |
| Sub-agents | Multi-agent threads (max 6, configurable to 64 batch workers) | Single agent loop |
| IDE integration | VS Code, Cursor, Windsurf (via JSON-RPC app-server) | None (CLI + API) |
| MCP | Experimental MCP *server* (exposing Codex as a tool) | MCP *client* (consuming external tools) + DietMCP |

#### Sandbox Comparison

| Feature | Codex CLI | Ferroclaw |
|---------|-----------|-----------|
| **macOS** | Seatbelt (`sandbox-exec`) with custom `.sbpl` policy, `(deny default)` baseline, modeled on Chrome's sandbox | Application-level capabilities |
| **Linux** | Bubblewrap + seccomp, `--ro-bind / /`, user/PID namespace isolation, network namespace with managed proxy | Application-level capabilities |
| **Windows** | Native sandbox with elevated/unelevated modes + private desktop isolation | N/A |
| **Network** | Managed proxy with domain allow/deny lists, SOCKS5 support | `net_outbound` / `net_listen` capability flags |
| **Policy engine** | Starlark-based `.rules` files, per-command evaluation, smart auto-approval | Capability set from config, per-tool enforcement |
| **Audit** | None | SHA256 hash-chained log |

#### Memory Comparison

| Feature | Codex CLI | Ferroclaw |
|---------|-----------|-----------|
| In-session | Auto-compaction at token threshold, 20K token max per user message | Token budget + sliding window pruning |
| Cross-session | Two-phase pipeline: rollout extraction → global memory consolidation | SQLite + FTS5 persistent store |
| Project context | AGENTS.md scanning from git root to CWD (32 KiB limit) | System prompt + DietMCP skill summaries |
| Memory ranking | `usage_count` + `last_usage`, stale entry pruning by `max_unused_days` | FTS5 relevance scoring |

#### Key Differentiator

Codex CLI is the most sophisticated sandbox implementation (OS-level on 3 platforms + Starlark policy engine) with a 40-crate Rust workspace. Ferroclaw is dramatically smaller (32 files vs 40+ crates, 5.4 MB vs 44 MB) with application-level security plus an audit trail Codex lacks, native DietMCP compression, and 4 provider families vs Codex's OpenAI focus.

---

## Benchmark Results

All benchmarks run on Apple M-series, Rust 1.94.0, `cargo bench` with Criterion.

### DietMCP Compression

| Tools | Generation Time | Render Time | Compression Ratio |
|-------|----------------|-------------|-------------------|
| 5 | 23 µs | — | ~75% |
| 10 | 45 µs | 2.0 µs | ~78% |
| 25 | 113 µs | — | ~80% |
| 50 | 226 µs | 7.7 µs | ~82% |
| 100 | 451 µs | 14.7 µs | ~85% |

**Compact signature generation**: 2.8 µs per tool

### Response Formatting

| Size | Summary | Minified | CSV |
|------|---------|----------|-----|
| 1 KB | 47 ns | 9.9 µs | 9.0 µs |
| 10 KB | 156 ns | 98 µs | 82 µs |
| 50 KB | 1.5 µs | 492 µs | 416 µs |

### Memory Store (SQLite + FTS5)

| Operation | 50 entries | 200 entries | 1,000 entries |
|-----------|-----------|-------------|---------------|
| Insert (sequential) | 150 µs | 1.7 ms | 9.2 ms |
| FTS5 search | 69 µs | 119 µs | 267 µs |
| List all | 36 µs | 96 µs | 411 µs |
| Save conversation (10 turns) | 43 µs | — | — |
| Retrieve conversation | 16 µs | — | — |

### Security

| Operation | Time |
|-----------|------|
| Capability check (pass) | **15.5 ns** |
| Capability check (fail) | **18.4 ns** |
| Audit write (10 entries) | 459 µs |
| Audit write (100 entries) | 3.4 ms |
| Audit verify (50 entries) | 246 µs |
| Audit verify (200 entries) | 676 µs |
| Audit verify (1,000 entries) | 2.97 ms |

### Cross-Framework Resource Comparison

| Framework | Language | Idle RAM | Cold Start | Install Size |
|-----------|---------|----------|------------|-------------|
| **Ferroclaw** | Rust | ~5-15 MB | <50 ms | 5.4 MB |
| OpenClaw | TypeScript | ~1.2 GB | ~6-8 s | ~1.4 GB |
| Hermes Agent | Python | est. ~200 MB | est. ~3-8 s | ~93 MB repo |
| NanoClaw | TypeScript | ~150 MB | ~3 s + 1-2s per container | Docker + Node.js |
| Claude Code | TypeScript (Bun) | est. ~80-150 MB | est. ~2-3 s | 56.1 MB |
| Codex CLI | Rust | est. ~30-60 MB | est. <1 s | ~44 MB |

*Sources: OpenClaw figures from waelmansour.com ecosystem comparison and GitHub Issue #20464. Claude Code from npm registry. Codex from GitHub Releases. Hermes/NanoClaw from repo analysis. "est." = estimated from architecture analysis.*

---

## Test Coverage

| Test Type | Count | Description |
|-----------|-------|-------------|
| Unit tests | 59 | Per-module tests for all core functionality |
| Integration: config | 13 | Config loading, 4-provider routing, gateway safety |
| Integration: security | 11 | Capability enforcement, audit chain integrity, tamper detection |
| Integration: memory | 12 | CRUD, FTS5 search ranking, conversations, unicode, 100KB content |
| Integration: diet | 11 | Compression ratios, formatting, auto-redirect, signatures |
| Integration: types | 13 | Messages, tool calls, context management, capabilities |
| **Total** | **119** | |

### Benchmark Suites (Criterion)

| Suite | Scenarios |
|-------|-----------|
| `diet_compression` | 5 tool counts × generation + render + signature + 3 formats × 3 sizes |
| `memory_store` | Insert + search + conversation + list × 3 sizes |
| `security_audit` | Capability check + audit write + verify × 3 sizes |

---

## Feature Matrix

| Feature | Ferroclaw | OpenClaw | Hermes | NanoClaw | Claude Code | Codex |
|---------|-----------|----------|--------|----------|-------------|-------|
| Single binary | **Yes** | No | No | No | No | **Yes** |
| Zero runtime deps | **Yes** | No | No | No | No | **Yes** |
| MCP client | **Yes** | Server only | **Yes** (full) | Custom server | **Yes** | Server (experimental) |
| DietMCP compression | **Yes** | No | No | No | No | BM25 tool_search |
| Multi-provider | **4** | 15+ | 10+ | 1 | 4 | 3+ custom |
| Zai GLM support | **Yes** | No | **Yes** | No | No | No |
| OpenRouter support | **Yes** | No | **Yes** | No | No | Via custom provider |
| Capability permissions | **8 types** | Post-CVE patches | 5-layer defense | Container isolation | 5 modes + sandbox | Seatbelt/Bubblewrap + Starlark |
| Tamper-evident audit | **Yes** | No | No | No | No | No |
| Persistent memory | **SQLite+FTS5** | Markdown files | **SQLite+FTS5** | CLAUDE.md | Files + auto-memory | SQLite + two-phase pipeline |
| HTTP gateway | **Yes** | Yes (canvas host, unsafe) | No | No | No | JSON-RPC app-server |
| Telegram bot | **Yes** | Via channel adapter | Via gateway | Via skill | No | No |
| Discord bot | **Yes** | Via channel adapter | Via gateway | No | No | No |
| Slack bot | **Yes** | Via channel adapter | Via gateway | No | No | No |
| WhatsApp | **Yes** | Via channel adapter | Via gateway | No | No | No |
| Signal | **Yes** | No | Via gateway | No | No | No |
| Email | **Yes** | Via channel adapter | Via gateway | No | No | No |
| Home Assistant | **Yes** | Via channel adapter | Via gateway | No | No | No |
| AgentSkills.io | **Yes** | No | **Yes** | No | No | No |
| Bundled skills | **84 (16 categories)** | 3,200+ (ClawHub) | **70+** | None | Built-in only | Built-in only |
| IDE integration | No | macOS/iOS/Android apps | ACP server (VS Code/Zed/JetBrains) | No | VS Code + JetBrains | VS Code + Cursor + Windsurf |
| Subagents | No | No | Subagent delegation | No | Agent/Task tool | Multi-agent threads (6-64) |
| Streaming | SSE-ready | Block-level | Fine-grained tool streaming | Via Claude Agent SDK | Yes | Yes |
| Open source | **MIT** | MIT | MIT | MIT | No | Apache-2.0 |

---

## Deployment Comparison

```
# Ferroclaw: copy one file, zero dependencies
scp ferroclaw server:/usr/local/bin/
ssh server "ferroclaw config init && ferroclaw run"

# OpenClaw: requires Node.js 22 + pnpm, 1.4 GB install
ssh server "npm install -g openclaw"  # 1.4 GB node_modules

# Hermes Agent: requires Python 3.11 + 124 packages
ssh server "pip install hermes-agent"  # + mini-swe-agent submodule

# NanoClaw: requires Node.js 20 + Docker + Claude Code CLI
ssh server "npm install nanoclaw && docker pull ..."

# Claude Code: proprietary, requires subscription
ssh server "npm install -g @anthropic-ai/claude-code"

# Codex CLI: Rust binary, but 44 MB + requires OpenAI auth
ssh server "npm install -g @openai/codex"
```

---

## When to Use What

| Use Case | Recommended | Why |
|----------|-------------|-----|
| Security-critical deployment | **Ferroclaw** | 8 capability types, hash-chained audit, safe-bind gateway |
| Maximum provider flexibility | **Hermes Agent** | 10+ providers, 40+ tools |
| Lightweight prototyping | **NanoClaw** | ~3,900 lines, container-isolated, 6 runtime deps |
| Interactive coding with Claude | **Claude Code** | Best UX, IDE integration, 92% prefix cache, SWE-bench leader |
| OS-level sandbox + code execution | **Codex CLI** | Seatbelt + Bubblewrap + Starlark, 3 OS support |
| Multi-provider production agent | **Ferroclaw** | 4 providers, 5.4 MB binary, DietMCP, audit trail |
| Embedded/IoT/edge deployment | **Ferroclaw** | 5.4 MB, no runtime, ~5 MB RSS |
| Maximum channels + integrations | **OpenClaw** | 23+ channels, 3,200+ skills (audit for malware first) |

---

## Sources

- OpenClaw: [GitHub](https://github.com/openclaw/openclaw), [CVE-2026-25253 (NVD)](https://nvd.nist.gov/vuln/detail/CVE-2026-25253), [Issue #5263](https://github.com/openclaw/openclaw/issues/5263), [Issue #20464](https://github.com/openclaw/openclaw/issues/20464), [arXiv:2603.10387](https://arxiv.org/abs/2603.10387), [Bitsight scan](https://www.bitsight.com/blog/openclaw-ai-security-risks-exposed-instances), [Snyk skills audit](https://snyk.io/blog/openclaw-skills-credential-leaks-research/), [waelmansour.com Claw comparison](https://waelmansour.com/blog/ai-agent-frameworks-the-claw-ecosystem/)
- Hermes Agent: [GitHub](https://github.com/NousResearch/hermes-agent), AGENTS.md, pyproject.toml, v0.2.0 release notes
- NanoClaw: [GitHub](https://github.com/qwibitai/nanoclaw), [SECURITY.md](https://github.com/qwibitai/nanoclaw/blob/main/docs/SECURITY.md), [SPEC.md](https://github.com/qwibitai/nanoclaw/blob/main/docs/SPEC.md), [TechCrunch](https://techcrunch.com/2026/03/13/the-wild-six-weeks-for-nanoclaws-creator-that-led-to-a-deal-with-docker/)
- Claude Code: [Docs](https://code.claude.com/docs/en/overview), [npm](https://www.npmjs.com/package/@anthropic-ai/claude-code), [Anthropic Engineering: Sandboxing](https://www.anthropic.com/engineering/claude-code-sandboxing), [SWE-bench](https://www.anthropic.com/research/swe-bench-sonnet)
- Codex CLI: [GitHub](https://github.com/openai/codex), linux-sandbox README, execpolicy README, memories README, model_provider_info.rs
