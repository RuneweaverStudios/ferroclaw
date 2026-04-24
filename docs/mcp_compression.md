# MCP Schema Compression

## Overview

The MCP schema compression module reduces token usage by **70-93%** when sending tool schemas to LLMs, while preserving essential semantic information.

## Why Compression?

MCP servers return verbose JSON Schema that wastes tokens:

```json
{
  "type": "object",
  "$schema": "http://json-schema.org/draft-07/schema#",
  "$id": "https://example.com/schemas/tool.json",
  "title": "File Reader Tool",
  "description": "Read the complete contents of a file from the filesystem...",
  "properties": {
    "path": {
      "type": "string",
      "title": "File Path",
      "description": "Absolute path to the file",
      "examples": ["/home/user/file.txt"],
      "minLength": 1,
      "maxLength": 1000
    }
  }
}
```

**After compression** (80% reduction):

```json
{
  "type": "object",
  "description": "Read the complete contents of a file...",
  "properties": {
    "path": {"type": "string"}
  },
  "required": ["path"]
}
```

## Compression Strategies

1. **Remove metadata** - Strip `$schema`, `$id`, `title`
2. **Remove examples** - LLMs don't need example values
3. **Remove defaults** - Optional metadata
4. **Remove validation** - Strip `minLength`, `maxLength`, `pattern`, `format`
5. **Truncate descriptions** - Summarize to 80 characters
6. **Remove property descriptions** - Keep only tool-level description
7. **Collapse oneOf/anyOf** - Convert to union types like `"string | integer"`
8. **Flatten nested objects** - Inline simple nested schemas

## Usage

### Basic Compression

```rust
use ferroclaw::mcp::compression::{compress_schema, compress_tools};

// Compress a single schema
let compressed = compress_schema(&schema);
println!("Reduced by {}%", compressed.metrics.reduction_percent());

// Compress multiple tools
let (compressed_tools, metrics) = compress_tools(&tools);
```

### Integration with MCP Client

```rust
use ferroclaw::mcp::client::McpClient;

// Compression is enabled by default
let client = McpClient::new(servers, 50000);

// Disable compression if needed
let client = McpClient::new(servers, 50000)
    .with_compression(false);
```

### Custom Configuration

```rust
use ferroclaw::mcp::compression::{compress_schema_with_config, CompressionConfig};

let config = CompressionConfig {
    max_description_len: 40,      // Shorter descriptions
    remove_examples: true,
    remove_defaults: true,
    collapse_oneof: true,
    flatten_nested: true,
    remove_metadata: true,
    remove_validation: true,
    simplify_single_props: true,
    remove_property_descriptions: true,
};

let compressed = compress_schema_with_config(&schema, config);
```

## Benchmarks

### Real-World MCP Server (5 tools)

```
Original: 5,317 chars (~1,329 tokens)
Compressed: 1,068 chars (~267 tokens)
Reduction: 79.9%
Tokens saved: ~1,062
```

### Per-Tool Breakdown

| Tool | Original | Compressed | Reduction |
|------|----------|------------|-----------|
| read_file | 739 | 165 | 77.7% |
| search_files | 1,138 | 216 | 81.0% |
| git_commit | 867 | 187 | 78.4% |
| db_query | 1,413 | 194 | 86.3% |
| http_request | 1,160 | 306 | 73.6% |

## Compression Analysis

```rust
use ferroclaw::mcp::compression::SchemaAnalyzer;

let analyzer = SchemaAnalyzer::analyze(&schema);

println!("Total fields: {}", analyzer.total_fields);
println!("Optional fields: {}", analyzer.optional_fields);
println!("Examples: {}", analyzer.examples_count);
println!("Potential reduction: {:.1}%", analyzer.estimate_reduction());
```

## Examples

Run the compression demo:

```bash
cargo run --example compression_demo
cargo run --example compression_benchmark
```

## Testing

```bash
# Run compression tests
cargo test compression --lib

# Run specific test
cargo test test_compress_meets_target --lib
```

## Implementation Details

### Compression Pipeline

1. **Parse** - Deserialize JSON schema into `serde_json::Value`
2. **Transform** - Apply compression strategies recursively
3. **Validate** - Ensure essential structure preserved
4. **Measure** - Calculate compression metrics

### What Gets Preserved

- ✓ Type information (`string`, `integer`, `array`, etc.)
- ✓ Required fields
- ✓ Property structure
- ✓ Tool-level description (truncated)
- ✓ Enum values
- ✓ Object structure

### What Gets Removed

- ✗ Metadata (`$schema`, `$id`, `title`)
- ✗ Examples
- ✗ Defaults
- ✗ Validation constraints (`minLength`, `pattern`, etc.)
- ✗ Property descriptions
- ✗ Redundant oneOf/anyOf alternatives

## Design Principles

1. **Semantic preservation** - LLM must understand how to use the tool
2. **Backward compatibility** - Compression is transparent to MCP protocol
3. **Configurable** - Can adjust aggressiveness per use case
4. **Measurable** - Track compression ratios and token savings

## Future Enhancements

- [ ] Machine learning to identify which fields LLMs actually use
- [ ] Differential compression based on LLM model
- [ ] Compression of enum arrays to ranges
- [ ] Semantic merging of similar properties
- [ ] Compression cache for repeated schemas

## References

- [DietMCP](https://github.com/jasonaibraham/dietmcp) - Original inspiration
- [JSON Schema](https://json-schema.org/) - Schema specification
- [MCP Protocol](https://modelcontextprotocol.io/) - MCP standard
