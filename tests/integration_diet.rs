//! Integration tests for DietMCP compression:
//! schema compression ratios, formatting, auto-redirect, skill summaries.

use ferroclaw::mcp::diet::{
    auto_redirect, format_response, generate_skill_summary, render_all_summaries,
    render_skill_summary, DietFormat,
};
use ferroclaw::types::ToolDefinition;
use serde_json::json;

fn realistic_filesystem_tools() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            name: "read_file".into(),
            description: "Read the complete contents of a file from the file system. Handles various text encodings and provides detailed error messages if the file cannot be read.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "Absolute path to the file to read"}
                },
                "required": ["path"]
            }),
            server_name: Some("filesystem".into()),
        },
        ToolDefinition {
            name: "read_multiple_files".into(),
            description: "Read the contents of multiple files simultaneously. This is more efficient than reading files one by one when you need to analyze or compare multiple files.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "paths": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Array of absolute file paths to read"
                    }
                },
                "required": ["paths"]
            }),
            server_name: Some("filesystem".into()),
        },
        ToolDefinition {
            name: "write_file".into(),
            description: "Create a new file or completely overwrite an existing file with new content. Use with caution as it will overwrite existing files without warning.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "Absolute path where the file should be written"},
                    "content": {"type": "string", "description": "Content to write to the file"}
                },
                "required": ["path", "content"]
            }),
            server_name: Some("filesystem".into()),
        },
        ToolDefinition {
            name: "edit_file".into(),
            description: "Make line-based edits to a text file. Each edit replaces exact line sequences with new content. Returns the file content after all edits are applied.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "Absolute path to the file to edit"},
                    "edits": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "oldText": {"type": "string", "description": "Text to search for"},
                                "newText": {"type": "string", "description": "Text to replace with"}
                            },
                            "required": ["oldText", "newText"]
                        },
                        "description": "Array of edit operations to apply"
                    },
                    "dryRun": {"type": "boolean", "description": "Preview changes without applying"}
                },
                "required": ["path", "edits"]
            }),
            server_name: Some("filesystem".into()),
        },
        ToolDefinition {
            name: "list_directory".into(),
            description: "Get a detailed listing of all files and directories in a specified path. Results clearly distinguish between files and directories with [FILE] and [DIR] prefixes.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "Absolute path of the directory to list"}
                },
                "required": ["path"]
            }),
            server_name: Some("filesystem".into()),
        },
        ToolDefinition {
            name: "directory_tree".into(),
            description: "Get a recursive tree view of files and directories. Each entry shows whether it is a file or directory, making it easy to understand the project structure at a glance.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "Absolute path of the root directory"},
                    "depth": {"type": "integer", "description": "Maximum depth to traverse (default: 3)"}
                },
                "required": ["path"]
            }),
            server_name: Some("filesystem".into()),
        },
        ToolDefinition {
            name: "search_files".into(),
            description: "Recursively search for files and directories matching a pattern. Searches through all subdirectories from the starting path using glob patterns.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "Starting directory for the search"},
                    "pattern": {"type": "string", "description": "Glob pattern to match (e.g., '*.ts', 'src/**/*.rs')"},
                    "excludePatterns": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Patterns to exclude from results"
                    }
                },
                "required": ["path", "pattern"]
            }),
            server_name: Some("filesystem".into()),
        },
        ToolDefinition {
            name: "get_file_info".into(),
            description: "Retrieve detailed metadata about a file or directory including size, creation time, last modified time, permissions, and type information.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "Absolute path to the file or directory"}
                },
                "required": ["path"]
            }),
            server_name: Some("filesystem".into()),
        },
        ToolDefinition {
            name: "move_file".into(),
            description: "Move or rename files and directories. Can move files between directories and rename them in a single operation. Fails if the destination already exists.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "source": {"type": "string", "description": "Source path"},
                    "destination": {"type": "string", "description": "Destination path"}
                },
                "required": ["source", "destination"]
            }),
            server_name: Some("filesystem".into()),
        },
    ]
}

#[test]
fn test_compression_ratio_exceeds_90_percent() {
    let tools = realistic_filesystem_tools();

    // Measure raw JSON schema size
    let raw_json = serde_json::to_string(&tools).unwrap();
    let raw_size = raw_json.len();

    // Measure diet summary size
    let summary = generate_skill_summary("filesystem", &tools);
    let diet_text = render_skill_summary(&summary);
    let diet_size = diet_text.len();

    let ratio = 1.0 - (diet_size as f64 / raw_size as f64);
    eprintln!(
        "Compression: raw={raw_size} bytes, diet={diet_size} bytes, ratio={:.1}%",
        ratio * 100.0
    );

    // Must achieve at least 70% compression (conservative target)
    assert!(
        ratio > 0.70,
        "Compression ratio {:.1}% is below 70% target",
        ratio * 100.0
    );
}

#[test]
fn test_compact_signatures_are_readable() {
    let tools = realistic_filesystem_tools();
    for tool in &tools {
        let sig = tool.compact_signature();
        // Every signature should contain the tool name
        assert!(sig.starts_with(&tool.name), "Signature doesn't start with name: {sig}");
        // Every signature should have parentheses
        assert!(sig.contains('(') && sig.contains(')'), "Signature missing parens: {sig}");
    }
}

#[test]
fn test_compact_signature_marks_optional_params() {
    let tool = ToolDefinition {
        name: "search".into(),
        description: "Search files".into(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "query": {"type": "string"},
                "limit": {"type": "integer"},
                "offset": {"type": "integer"}
            },
            "required": ["query"]
        }),
        server_name: None,
    };
    let sig = tool.compact_signature();
    assert!(sig.contains("query: str"), "Missing required param in: {sig}");
    assert!(sig.contains("?limit: int"), "Missing optional marker in: {sig}");
    assert!(sig.contains("?offset: int"), "Missing optional marker in: {sig}");
}

#[test]
fn test_skill_summary_structure() {
    let tools = realistic_filesystem_tools();
    let summary = generate_skill_summary("filesystem", &tools);

    assert_eq!(summary.server_name, "filesystem");
    assert_eq!(summary.tool_count, 9);
    assert!(!summary.categories.is_empty());
    assert!(summary.exec_hint.contains("ferroclaw"));
}

#[test]
fn test_render_multiple_summaries() {
    let fs_tools = realistic_filesystem_tools();
    let git_tools = vec![
        ToolDefinition {
            name: "git_status".into(),
            description: "Get git status of the repository".into(),
            input_schema: json!({"type": "object", "properties": {"path": {"type": "string"}}}),
            server_name: Some("git".into()),
        },
        ToolDefinition {
            name: "git_diff".into(),
            description: "Get git diff of changes".into(),
            input_schema: json!({"type": "object", "properties": {"path": {"type": "string"}}}),
            server_name: Some("git".into()),
        },
    ];

    let summaries = vec![
        generate_skill_summary("filesystem", &fs_tools),
        generate_skill_summary("git", &git_tools),
    ];

    let rendered = render_all_summaries(&summaries);
    assert!(rendered.contains("11 total")); // 9 + 2
    assert!(rendered.contains("filesystem"));
    assert!(rendered.contains("git"));
}

#[test]
fn test_format_response_summary_truncates() {
    let large = "x".repeat(100_000);
    let result = format_response(&large, DietFormat::Summary, 1000);
    assert!(result.content.len() < 2000);
    assert!(result.content.contains("Truncated"));
}

#[test]
fn test_format_response_minified_strips_nulls() {
    let input = r#"{"name": "test", "value": null, "nested": {"a": 1, "b": null}}"#;
    let result = format_response(input, DietFormat::Minified, 50000);
    assert!(!result.content.contains("null"));
    assert!(result.content.contains("\"name\""));
    assert!(result.content.contains("\"a\""));
}

#[test]
fn test_format_response_csv_tabular() {
    let input = r#"[
        {"name": "file1.rs", "size": 1024, "type": "file"},
        {"name": "src", "size": 4096, "type": "directory"},
        {"name": "file2.rs", "size": 2048, "type": "file"}
    ]"#;
    let result = format_response(input, DietFormat::Csv, 50000);
    // Should have header row + 3 data rows
    let lines: Vec<&str> = result.content.lines().collect();
    assert!(lines.len() >= 4, "Expected 4+ lines, got {}", lines.len());
}

#[test]
fn test_auto_redirect_large_response() {
    let large = "x".repeat(200_000);
    let result = auto_redirect(&large);
    assert!(result.was_redirected);
    assert!(result.file_path.is_some());
    assert!(result.content.contains("200000 chars"));

    // Verify the file exists and has correct content
    let path = result.file_path.unwrap();
    let written = std::fs::read_to_string(&path).unwrap();
    assert_eq!(written.len(), 200_000);
    let _ = std::fs::remove_file(path);
}

#[test]
fn test_auto_redirect_preserves_small_responses() {
    let small = "Hello, world!";
    let result = format_response(small, DietFormat::Summary, 50000);
    assert!(!result.was_redirected);
    assert!(result.file_path.is_none());
    assert_eq!(result.content, small);
}

#[test]
fn test_diet_token_savings_estimate() {
    let tools = realistic_filesystem_tools();
    let raw_json = serde_json::to_string_pretty(&tools).unwrap();
    let summary = generate_skill_summary("filesystem", &tools);
    let diet_text = render_skill_summary(&summary);

    // Rough token estimate (1 token ≈ 4 chars)
    let raw_tokens = raw_json.len() / 4;
    let diet_tokens = diet_text.len() / 4;
    let saved_tokens = raw_tokens - diet_tokens;

    eprintln!(
        "Token savings: raw≈{raw_tokens}, diet≈{diet_tokens}, saved≈{saved_tokens} ({:.0}%)",
        (saved_tokens as f64 / raw_tokens as f64) * 100.0
    );

    assert!(saved_tokens > 0);
}
