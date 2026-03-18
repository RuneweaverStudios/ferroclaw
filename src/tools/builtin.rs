//! Built-in tools: filesystem, bash, web_fetch, memory.

use crate::error::FerroError;
use crate::memory::MemoryStore;
use crate::tool::{ToolFuture, ToolHandler, ToolRegistry};
use crate::types::{Capability, ToolDefinition, ToolMeta, ToolResult, ToolSource};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Register all built-in tools into the registry.
pub fn register_builtin_tools(
    registry: &mut ToolRegistry,
    memory: Arc<Mutex<MemoryStore>>,
) {
    // read_file
    registry.register(
        ToolMeta {
            definition: ToolDefinition {
                name: "read_file".into(),
                description: "Read the contents of a file at the given path".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Absolute path to the file to read"
                        }
                    },
                    "required": ["path"]
                }),
                server_name: None,
            },
            required_capabilities: vec![Capability::FsRead],
            source: ToolSource::Builtin,
        },
        Box::new(ReadFileHandler),
    );

    // write_file
    registry.register(
        ToolMeta {
            definition: ToolDefinition {
                name: "write_file".into(),
                description: "Write content to a file at the given path".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Absolute path to the file to write"
                        },
                        "content": {
                            "type": "string",
                            "description": "Content to write to the file"
                        }
                    },
                    "required": ["path", "content"]
                }),
                server_name: None,
            },
            required_capabilities: vec![Capability::FsWrite],
            source: ToolSource::Builtin,
        },
        Box::new(WriteFileHandler),
    );

    // list_directory
    registry.register(
        ToolMeta {
            definition: ToolDefinition {
                name: "list_directory".into(),
                description: "List contents of a directory".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Absolute path to the directory to list"
                        }
                    },
                    "required": ["path"]
                }),
                server_name: None,
            },
            required_capabilities: vec![Capability::FsRead],
            source: ToolSource::Builtin,
        },
        Box::new(ListDirectoryHandler),
    );

    // bash
    registry.register(
        ToolMeta {
            definition: ToolDefinition {
                name: "bash".into(),
                description: "Execute a bash command and return stdout/stderr".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "command": {
                            "type": "string",
                            "description": "The bash command to execute"
                        }
                    },
                    "required": ["command"]
                }),
                server_name: None,
            },
            required_capabilities: vec![Capability::ProcessExec],
            source: ToolSource::Builtin,
        },
        Box::new(BashHandler),
    );

    // web_fetch
    registry.register(
        ToolMeta {
            definition: ToolDefinition {
                name: "web_fetch".into(),
                description: "Fetch content from a URL via HTTP GET".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "url": {
                            "type": "string",
                            "description": "The URL to fetch"
                        }
                    },
                    "required": ["url"]
                }),
                server_name: None,
            },
            required_capabilities: vec![Capability::NetOutbound],
            source: ToolSource::Builtin,
        },
        Box::new(WebFetchHandler),
    );

    // memory_search
    registry.register(
        ToolMeta {
            definition: ToolDefinition {
                name: "memory_search".into(),
                description: "Search stored memories using full-text search".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Search query"
                        }
                    },
                    "required": ["query"]
                }),
                server_name: None,
            },
            required_capabilities: vec![Capability::MemoryRead],
            source: ToolSource::Builtin,
        },
        Box::new(MemorySearchHandler {
            store: Arc::clone(&memory),
        }),
    );

    // memory_store
    registry.register(
        ToolMeta {
            definition: ToolDefinition {
                name: "memory_store".into(),
                description: "Store a key-value pair in persistent memory".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "key": {
                            "type": "string",
                            "description": "Memory key (e.g. 'user_preference_theme')"
                        },
                        "content": {
                            "type": "string",
                            "description": "Content to remember"
                        }
                    },
                    "required": ["key", "content"]
                }),
                server_name: None,
            },
            required_capabilities: vec![Capability::MemoryWrite],
            source: ToolSource::Builtin,
        },
        Box::new(MemoryStoreHandler {
            store: Arc::clone(&memory),
        }),
    );
}

// --- Tool Handlers ---

struct ReadFileHandler;
impl ToolHandler for ReadFileHandler {
    fn call<'a>(&'a self, call_id: &'a str, arguments: &'a Value) -> ToolFuture<'a> {
        Box::pin(async move {
            let path = arguments
                .get("path")
                .and_then(|p| p.as_str())
                .ok_or_else(|| FerroError::Tool("Missing 'path' argument".into()))?;

            match tokio::fs::read_to_string(path).await {
                Ok(content) => Ok(ToolResult {
                    call_id: call_id.to_string(),
                    content,
                    is_error: false,
                }),
                Err(e) => Ok(ToolResult {
                    call_id: call_id.to_string(),
                    content: format!("Error reading {path}: {e}"),
                    is_error: true,
                }),
            }
        })
    }
}

struct WriteFileHandler;
impl ToolHandler for WriteFileHandler {
    fn call<'a>(&'a self, call_id: &'a str, arguments: &'a Value) -> ToolFuture<'a> {
        Box::pin(async move {
            let path = arguments
                .get("path")
                .and_then(|p| p.as_str())
                .ok_or_else(|| FerroError::Tool("Missing 'path' argument".into()))?;
            let content = arguments
                .get("content")
                .and_then(|c| c.as_str())
                .ok_or_else(|| FerroError::Tool("Missing 'content' argument".into()))?;

            match tokio::fs::write(path, content).await {
                Ok(_) => Ok(ToolResult {
                    call_id: call_id.to_string(),
                    content: format!("Successfully wrote {} bytes to {path}", content.len()),
                    is_error: false,
                }),
                Err(e) => Ok(ToolResult {
                    call_id: call_id.to_string(),
                    content: format!("Error writing {path}: {e}"),
                    is_error: true,
                }),
            }
        })
    }
}

struct ListDirectoryHandler;
impl ToolHandler for ListDirectoryHandler {
    fn call<'a>(&'a self, call_id: &'a str, arguments: &'a Value) -> ToolFuture<'a> {
        Box::pin(async move {
            let path = arguments
                .get("path")
                .and_then(|p| p.as_str())
                .ok_or_else(|| FerroError::Tool("Missing 'path' argument".into()))?;

            match tokio::fs::read_dir(path).await {
                Ok(mut entries) => {
                    let mut items = Vec::new();
                    while let Ok(Some(entry)) = entries.next_entry().await {
                        let name = entry.file_name().to_string_lossy().to_string();
                        let file_type = entry.file_type().await.ok();
                        let suffix = if file_type.as_ref().is_some_and(|ft| ft.is_dir()) {
                            "/"
                        } else {
                            ""
                        };
                        items.push(format!("{name}{suffix}"));
                    }
                    items.sort();
                    Ok(ToolResult {
                        call_id: call_id.to_string(),
                        content: items.join("\n"),
                        is_error: false,
                    })
                }
                Err(e) => Ok(ToolResult {
                    call_id: call_id.to_string(),
                    content: format!("Error listing {path}: {e}"),
                    is_error: true,
                }),
            }
        })
    }
}

struct BashHandler;
impl ToolHandler for BashHandler {
    fn call<'a>(&'a self, call_id: &'a str, arguments: &'a Value) -> ToolFuture<'a> {
        Box::pin(async move {
            let command = arguments
                .get("command")
                .and_then(|c| c.as_str())
                .ok_or_else(|| FerroError::Tool("Missing 'command' argument".into()))?;

            let output = tokio::process::Command::new("bash")
                .arg("-c")
                .arg(command)
                .output()
                .await
                .map_err(|e| FerroError::Tool(format!("Failed to execute command: {e}")))?;

            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            let content = if output.status.success() {
                stdout.to_string()
            } else {
                format!(
                    "Exit code: {}\nStdout: {stdout}\nStderr: {stderr}",
                    output.status.code().unwrap_or(-1)
                )
            };

            Ok(ToolResult {
                call_id: call_id.to_string(),
                content,
                is_error: !output.status.success(),
            })
        })
    }
}

struct WebFetchHandler;
impl ToolHandler for WebFetchHandler {
    fn call<'a>(&'a self, call_id: &'a str, arguments: &'a Value) -> ToolFuture<'a> {
        Box::pin(async move {
            let url = arguments
                .get("url")
                .and_then(|u| u.as_str())
                .ok_or_else(|| FerroError::Tool("Missing 'url' argument".into()))?;

            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .map_err(|e| FerroError::Tool(format!("HTTP client error: {e}")))?;

            match client.get(url).send().await {
                Ok(resp) => {
                    let status = resp.status();
                    let body = resp
                        .text()
                        .await
                        .unwrap_or_else(|e| format!("Failed to read body: {e}"));

                    // Limit response size
                    let truncated = if body.len() > 50_000 {
                        format!("{}...\n[Truncated: {} total chars]", &body[..50_000], body.len())
                    } else {
                        body
                    };

                    Ok(ToolResult {
                        call_id: call_id.to_string(),
                        content: format!("[{status}]\n{truncated}"),
                        is_error: !status.is_success(),
                    })
                }
                Err(e) => Ok(ToolResult {
                    call_id: call_id.to_string(),
                    content: format!("Fetch error: {e}"),
                    is_error: true,
                }),
            }
        })
    }
}

struct MemorySearchHandler {
    store: Arc<Mutex<MemoryStore>>,
}
impl ToolHandler for MemorySearchHandler {
    fn call<'a>(&'a self, call_id: &'a str, arguments: &'a Value) -> ToolFuture<'a> {
        Box::pin(async move {
            let query = arguments
                .get("query")
                .and_then(|q| q.as_str())
                .ok_or_else(|| FerroError::Tool("Missing 'query' argument".into()))?;

            let store = self.store.lock().await;
            match store.search(query, 10) {
                Ok(memories) => {
                    if memories.is_empty() {
                        Ok(ToolResult {
                            call_id: call_id.to_string(),
                            content: "No memories found.".into(),
                            is_error: false,
                        })
                    } else {
                        let results: Vec<String> = memories
                            .iter()
                            .map(|m| format!("[{}] {}", m.key, m.content))
                            .collect();
                        Ok(ToolResult {
                            call_id: call_id.to_string(),
                            content: results.join("\n"),
                            is_error: false,
                        })
                    }
                }
                Err(e) => Ok(ToolResult {
                    call_id: call_id.to_string(),
                    content: format!("Memory search error: {e}"),
                    is_error: true,
                }),
            }
        })
    }
}

struct MemoryStoreHandler {
    store: Arc<Mutex<MemoryStore>>,
}
impl ToolHandler for MemoryStoreHandler {
    fn call<'a>(&'a self, call_id: &'a str, arguments: &'a Value) -> ToolFuture<'a> {
        Box::pin(async move {
            let key = arguments
                .get("key")
                .and_then(|k| k.as_str())
                .ok_or_else(|| FerroError::Tool("Missing 'key' argument".into()))?;
            let content = arguments
                .get("content")
                .and_then(|c| c.as_str())
                .ok_or_else(|| FerroError::Tool("Missing 'content' argument".into()))?;

            let store = self.store.lock().await;
            match store.insert(key, content) {
                Ok(_) => Ok(ToolResult {
                    call_id: call_id.to_string(),
                    content: format!("Stored memory '{key}'"),
                    is_error: false,
                }),
                Err(e) => Ok(ToolResult {
                    call_id: call_id.to_string(),
                    content: format!("Memory store error: {e}"),
                    is_error: true,
                }),
            }
        })
    }
}
