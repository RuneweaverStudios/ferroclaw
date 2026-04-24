# FileEditTool - Exact String Replacement

## Overview

FileEditTool provides precise, safe file editing through exact string replacement. Unlike traditional text editors that modify files based on line numbers or patterns, FileEditTool requires the exact text to replace, ensuring only intended changes are made.

**Key Features:**
- Exact string matching (no regex, no patterns)
- Uniqueness validation (must match exactly once)
- Atomic write operations (no corruption on failure)
- Multi-line replacement support
- Clear error messages

## How It Works

### Validation Pipeline

1. **Read**: Load the entire file content into memory
2. **Existence Check**: Verify `old_string` exists in the file
3. **Uniqueness Check**: Ensure `old_string` appears exactly once
4. **Replacement**: Replace first occurrence with `new_string`
5. **Atomic Write**: Use temporary file + atomic rename for safe persistence

### Safety Guarantees

- **No silent failures**: Every error is reported with context
- **No partial writes**: Atomic operations prevent corruption
- **No accidental multi-replacement**: Uniqueness check required
- **No data loss**: Original file unchanged until write succeeds

## Usage

### JSON API

```json
{
  "name": "file_edit",
  "arguments": {
    "file_path": "/path/to/file.rs",
    "old_string": "let x = 1;",
    "new_string": "let x = 42;"
  }
}
```

### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `file_path` | string | Yes | Absolute path to the file |
| `old_string` | string | Yes | Exact text to replace (must be unique) |
| `new_string` | string | Yes | Replacement text |

### Response

**Success:**
```json
{
  "call_id": "call-123",
  "content": "Successfully replaced 'let x = 1;' with 'let x = 42;' in '/path/to/file.rs'",
  "is_error": false
}
```

**Error - String not found:**
```json
{
  "call_id": "call-124",
  "content": "Error: The string 'let x = 1;' was not found in the file '/path/to/file.rs'",
  "is_error": true
}
```

**Error - Not unique:**
```json
{
  "call_id": "call-125",
  "content": "Error: The string 'let x = 1;' appears 3 times in '/path/to/file.rs'. For safety, the string to replace must be unique. Please provide more context to make the string unique.",
  "is_error": true
}
```

## Examples

### Single-Line Replacement

**File: `config.toml`**
```toml
debug = false
port = 8080
```

**Edit:**
```json
{
  "file_path": "/home/user/project/config.toml",
  "old_string": "debug = false",
  "new_string": "debug = true"
}
```

**Result:**
```toml
debug = true
port = 8080
```

### Multi-Line Replacement

**File: `src/main.rs`**
```rust
fn main() {
    println!("Hello");
    println!("World");
}
```

**Edit:**
```json
{
  "file_path": "/home/user/project/src/main.rs",
  "old_string": "    println!(\"Hello\");\n    println!(\"World\");",
  "new_string": "    println!(\"Hello, World!\");"
}
```

**Result:**
```rust
fn main() {
    println!("Hello, World!");
}
```

### Adding Context for Uniqueness

When `old_string` appears multiple times, include more surrounding context:

**File: `user.rs`**
```rust
fn process_user(user: &User) {
    let name = user.name;
    let email = user.email;
}

fn display_user(user: &User) {
    let name = user.name;
    let email = user.email;
}
```

**Bad Edit (fails - not unique):**
```json
{
  "file_path": "/home/user/project/src/user.rs",
  "old_string": "    let name = user.name;\n    let email = user.email;",
  "new_string": "    let User { name, email } = user;"
}
```

**Good Edit (includes function context):**
```json
{
  "file_path": "/home/user/project/src/user.rs",
  "old_string": "fn process_user(user: &User) {\n    let name = user.name;\n    let email = user.email;\n}",
  "new_string": "fn process_user(user: &User) {\n    let User { name, email } = user;\n}"
}
```

## Error Handling

### Common Errors

| Error | Cause | Solution |
|-------|-------|----------|
| `Missing 'file_path' argument` | Required parameter missing | Add `file_path` to arguments |
| `Missing 'old_string' argument` | Required parameter missing | Add `old_string` to arguments |
| `Missing 'new_string' argument` | Required parameter missing | Add `new_string` to arguments |
| `Failed to read file` | File doesn't exist or permissions issue | Check file path and permissions |
| `String not found` | `old_string` doesn't match file content | Verify exact text (including whitespace) |
| `Appears N times` | `old_string` is not unique | Add more context to make it unique |
| `Failed to save file` | Disk full or permissions issue | Check disk space and write permissions |

### Error Recovery

FileEditTool never modifies the original file on error:
- Read failures → No changes made
- Validation failures → No changes made
- Write failures → Temporary file cleaned up, original untouched

## Best Practices

### 1. Provide Sufficient Context

```rust
// Bad - too generic
"return Ok(())"

// Good - includes function signature
"fn validate_input(input: &str) -> Result<()> {\n    return Ok(());\n}"
```

### 2. Preserve Whitespace

```rust
// Include exact indentation
"    if condition {\n        do_something();\n    }"
```

### 3. Use Multi-Line for Complex Changes

```json
{
  "old_string": "struct User {\n    name: String,\n    email: String,\n}",
  "new_string": "struct User {\n    name: String,\n    email: String,\n    age: u32,\n}"
}
```

### 4. Test Before Editing

For complex edits, verify the string exists first:

```bash
# Check if string exists
grep -n "exact string here" file.rs

# Then edit with confidence
```

## Implementation Details

### Atomic Write Algorithm

```rust
1. Create temp file in same directory as target
2. Write new content to temp file
3. fsync temp file (ensure data written to disk)
4. Atomic rename temp → target (atomic on POSIX)
5. Cleanup temp file on failure
```

**Why atomic rename matters:**
- No partial writes visible to other processes
- No corruption if system crashes during write
- No need for backup files
- Instantaneous switch to new content

### Performance Considerations

- **Small files** (< 1MB): Entire file loaded into memory
- **Large files** (> 1MB): Consider alternative tools (split, sed)
- **Network drives**: Atomic operations still safe
- **Symbolic links**: Resolved to real path automatically

## Troubleshooting

### Issue: "String not found" but I can see it

**Cause**: Whitespace or encoding mismatch

**Solution**:
```bash
# Check exact bytes
xxd file.txt | grep -C 3 "your string"

# Look for hidden characters
cat -A file.txt | grep "your string"
```

### Issue: "Appears N times" but I only see one

**Cause**: String appears in comments, tests, or generated code

**Solution**: Include more context to disambiguate:

```rust
// Instead of
"fn process() {"

// Use
"pub fn process_data(input: Data) -> Result<Output> {\n    fn process() {"
```

### Issue: Edit fails on binary file

**Cause**: File contains null bytes or invalid UTF-8

**Solution**: FileEditTool only supports text files. Use hex editor for binary files.

## See Also

- **ReadTool**: Read file contents before editing
- **GrepTool**: Search for patterns before editing
- **WriteTool**: Create new files or overwrite completely
- **CommitTool**: Commit file changes to git
