# Ferroclaw Benchmarks

All benchmarks run with [Criterion.rs](https://github.com/bheisler/criterion.rs) on Apple Silicon (M-series), Rust 1.94.0.

Run benchmarks: `cargo bench`
View HTML reports: `open target/criterion/report/index.html`

---

## DietMCP Compression

The core value proposition: compress MCP tool schemas to save LLM context tokens.

### Skill Summary Generation

Time to categorize tools, generate compact signatures, and build skill summaries:

| Tools | Time | Throughput |
|-------|------|------------|
| 5 | 23.1 µs | 217K tools/sec |
| 10 | 45.4 µs | 220K tools/sec |
| 25 | 113.4 µs | 220K tools/sec |
| 50 | 226.0 µs | 221K tools/sec |
| 100 | 451.2 µs | 222K tools/sec |

Linear scaling — no hidden quadratic behavior.

### Render to Text

Time to serialize a skill summary into LLM-ready text:

| Tools | Render Time |
|-------|-------------|
| 10 | 2.0 µs |
| 50 | 7.7 µs |
| 100 | 14.7 µs |

### Compact Signature

Single tool signature generation (e.g., `search(query: str, ?limit: int)`):

**2.82 µs** per signature

### Compression Ratio

Measured against a realistic 9-tool filesystem MCP server schema:

```
Raw JSON schema:  ~4,200 bytes (~1,050 tokens)
DietMCP summary:  ~800 bytes   (~200 tokens)
Compression:      ~81%
Token savings:    ~850 tokens per server
```

With 5 MCP servers (typical setup): **~4,250 tokens saved per request**.

### Response Formatting

Time to format tool responses before adding to conversation:

| Response Size | Summary | Minified | CSV |
|--------------|---------|----------|-----|
| 1 KB | 48 ns | 9.9 µs | 9.0 µs |
| 10 KB | 156 ns | 98 µs | 82 µs |
| 50 KB | 1.5 µs | 492 µs | 416 µs |

Summary format is near-instant for small responses. Minified and CSV do JSON parsing + restructuring but remain sub-millisecond even for 50KB responses.

---

## Memory Store (SQLite + FTS5)

### Insert Throughput

| Entries | Total Time | Per Entry |
|---------|-----------|-----------|
| 10 | 150 µs | 15.0 µs |
| 100 | 1.7 ms | 17.0 µs |
| 500 | 9.2 ms | 18.4 µs |

Consistent per-entry cost. Dominated by SQLite WAL write + FTS5 index update.

### Full-Text Search (FTS5)

| Corpus Size | Search Time |
|-------------|-------------|
| 50 entries | 69 µs |
| 200 entries | 119 µs |
| 1,000 entries | 267 µs |

Sub-millisecond search even at 1,000 entries. FTS5's inverted index provides O(log n) lookup.

### Conversation Persistence

| Operation | Time |
|-----------|------|
| Save 10 turns | 43 µs |
| Save 50 turns | 173 µs |
| Save 200 turns | 671 µs |
| Retrieve 10 turns | 16 µs |
| Retrieve 50 turns | 28 µs |
| Retrieve 200 turns | 70 µs |

Retrieval is ~4x faster than saving (no write I/O).

### List All

| Entries | Time |
|---------|------|
| 50 | 36 µs |
| 200 | 96 µs |
| 1,000 | 411 µs |

---

## Security

### Capability Check

The most performance-critical security operation — runs on every tool call:

| Outcome | Time |
|---------|------|
| Pass (capabilities match) | **15.5 ns** |
| Fail (capability missing) | **18.4 ns** |

Negligible overhead. A HashSet lookup.

### Audit Log

#### Write Throughput

| Entries | Total Time | Per Entry |
|---------|-----------|-----------|
| 10 | 459 µs | 45.9 µs |
| 100 | 3.4 ms | 34.0 µs |
| 500 | 16.1 ms | 32.2 µs |

Per-entry cost decreases with batch size (file handle amortization).

#### Verification

| Entries | Verify Time | Per Entry |
|---------|------------|-----------|
| 50 | 246 µs | 4.9 µs |
| 200 | 676 µs | 3.4 µs |
| 1,000 | 2.97 ms | 3.0 µs |

Full hash-chain verification of 1,000 entries in under 3ms.

---

## Binary Size

| Build | Size |
|-------|------|
| Debug | ~45 MB |
| Release | **5.4 MB** |
| Release (no LTO) | ~8.2 MB |

Release profile: `lto = true`, `codegen-units = 1`, `strip = true`.

---

## Comparative Performance Estimates

These are estimated comparisons based on architecture analysis (not head-to-head benchmarks):

| Operation | Ferroclaw | Node.js Agent* | Python Agent* |
|-----------|-----------|---------------|--------------|
| Cold start | <50 ms | 2-5 s | 3-8 s |
| Memory baseline | 5-15 MB | 80-150 MB | 100-200 MB |
| Capability check | 15 ns | ~1 µs (JS object lookup) | ~5 µs (dict lookup) |
| Schema compression | 226 µs (50 tools) | N/A | ~50 ms (Python dietmcp) |
| FTS5 search (200 entries) | 119 µs | N/A | ~5-20 ms (whoosh/etc.) |

*Estimates based on typical runtime overhead for these languages/frameworks.

---

## Running Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench --bench diet_compression
cargo bench --bench memory_store
cargo bench --bench security_audit

# Generate HTML reports
cargo bench
open target/criterion/report/index.html

# Compare against baseline
cargo bench -- --save-baseline v0.1.0
# ... make changes ...
cargo bench -- --baseline v0.1.0
```
