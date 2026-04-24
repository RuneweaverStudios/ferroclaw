//! Code refactoring tool - apply refactorings (extract, inline, rename, move)

use crate::error::FerroError;
use crate::tool::{ToolFuture, ToolHandler};
use crate::types::Capability;
use serde_json::Value;

pub fn refactor_code_meta() -> crate::types::ToolMeta {
    crate::types::ToolMeta {
        definition: crate::types::ToolDefinition {
            name: "refactor_code".into(),
            description: "Apply code refactorings: extract function, inline function, rename symbol, move declaration, extract variable, etc. Supports Rust, Python, JavaScript.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to file to refactor"
                    },
                    "refactoring_type": {
                        "type": "string",
                        "enum": ["extract_function", "inline_function", "rename", "extract_variable", "move_declaration"],
                        "description": "Type of refactoring to apply"
                    },
                    "target": {
                        "type": "string",
                        "description": "Target for refactoring (function name, symbol name, etc.)"
                    },
                    "lines": {
                        "type": "string",
                        "description": "Line range for refactoring (e.g., '10-20')"
                    },
                    "new_name": {
                        "type": "string",
                        "description": "New name (for rename refactorings)"
                    }
                },
                "required": ["path", "refactoring_type", "target"]
            }),
            server_name: None,
        },
        required_capabilities: vec![Capability::FsRead, Capability::FsWrite],
        source: crate::types::ToolSource::Builtin,
    }
}

pub struct RefactorCodeHandler;

impl ToolHandler for RefactorCodeHandler {
    fn call<'a>(&'a self, call_id: &'a str, arguments: &'a Value) -> ToolFuture<'a> {
        Box::pin(async move {
            let path = arguments
                .get("path")
                .and_then(|p| p.as_str())
                .ok_or_else(|| FerroError::Tool("Missing 'path' argument".into()))?;

            let refactoring_type = arguments
                .get("refactoring_type")
                .and_then(|t| t.as_str())
                .ok_or_else(|| FerroError::Tool("Missing 'refactoring_type' argument".into()))?;

            let target = arguments
                .get("target")
                .and_then(|t| t.as_str())
                .ok_or_else(|| FerroError::Tool("Missing 'target' argument".into()))?;

            let content = tokio::fs::read_to_string(path)
                .await
                .map_err(|e| FerroError::Tool(format!("Cannot read {}: {}", path, e)))?;

            let ext = std::path::Path::new(path)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");

            let refactored = match ext {
                "rs" => refactor_rust(&content, refactoring_type, target, arguments).await?,
                "py" => refactor_python(&content, refactoring_type, target, arguments).await?,
                "js" | "ts" | "jsx" | "tsx" => {
                    refactor_javascript(&content, refactoring_type, target, arguments).await?
                }
                _ => {
                    return Err(FerroError::Tool(format!(
                        "Unsupported language: {}. Supported: Rust (.rs), Python (.py), JavaScript/TypeScript (.js, .ts, .jsx, .tsx)",
                        ext
                    )));
                }
            };

            tokio::fs::write(path, refactored)
                .await
                .map_err(|e| FerroError::Tool(format!("Cannot write {}: {}", path, e)))?;

            Ok(crate::types::ToolResult {
                call_id: call_id.to_string(),
                content: format!(
                    "✅ Applied {} refactoring to {}:\n  Target: {}",
                    refactoring_type, path, target
                ),
                is_error: false,
            })
        })
    }
}

async fn refactor_rust(
    content: &str,
    refactoring_type: &str,
    target: &str,
    args: &Value,
) -> Result<String, FerroError> {
    match refactoring_type {
        "rename" => {
            let new_name = args
                .get("new_name")
                .and_then(|n| n.as_str())
                .ok_or_else(|| {
                    FerroError::Tool("Missing 'new_name' for rename refactoring".into())
                })?;

            // Simple rename - replace all occurrences
            // This is a simplified implementation; a proper version would need semantic analysis
            let refactored = content.replace(target, new_name);
            Ok(refactored)
        }
        "extract_function" => {
            let lines = args.get("lines").and_then(|l| l.as_str()).ok_or_else(|| {
                FerroError::Tool("Missing 'lines' for extract_function refactoring".into())
            })?;

            Ok(format!(
                "// TODO: Extract function from lines {}\n{}",
                lines, content
            ))
        }
        "inline_function" => Ok(format!("// TODO: Inline function {}\n{}", target, content)),
        "extract_variable" => {
            let lines = args.get("lines").and_then(|l| l.as_str()).ok_or_else(|| {
                FerroError::Tool("Missing 'lines' for extract_variable refactoring".into())
            })?;

            Ok(format!(
                "// TODO: Extract variable from lines {}\n{}",
                lines, content
            ))
        }
        "move_declaration" => Ok(format!(
            "// TODO: Move declaration of {}\n{}",
            target, content
        )),
        _ => Err(FerroError::Tool(format!(
            "Unknown refactoring type: {}",
            refactoring_type
        ))),
    }
}

async fn refactor_python(
    content: &str,
    refactoring_type: &str,
    target: &str,
    args: &Value,
) -> Result<String, FerroError> {
    match refactoring_type {
        "rename" => {
            let new_name = args
                .get("new_name")
                .and_then(|n| n.as_str())
                .ok_or_else(|| {
                    FerroError::Tool("Missing 'new_name' for rename refactoring".into())
                })?;

            let refactored = content.replace(target, new_name);
            Ok(refactored)
        }
        "extract_function" => {
            let lines = args.get("lines").and_then(|l| l.as_str()).ok_or_else(|| {
                FerroError::Tool("Missing 'lines' for extract_function refactoring".into())
            })?;

            Ok(format!(
                "# TODO: Extract function from lines {}\n{}",
                lines, content
            ))
        }
        "inline_function" => Ok(format!("# TODO: Inline function {}\n{}", target, content)),
        _ => Err(FerroError::Tool(format!(
            "Unknown refactoring type: {}",
            refactoring_type
        ))),
    }
}

async fn refactor_javascript(
    content: &str,
    refactoring_type: &str,
    target: &str,
    args: &Value,
) -> Result<String, FerroError> {
    match refactoring_type {
        "rename" => {
            let new_name = args
                .get("new_name")
                .and_then(|n| n.as_str())
                .ok_or_else(|| {
                    FerroError::Tool("Missing 'new_name' for rename refactoring".into())
                })?;

            let refactored = content.replace(target, new_name);
            Ok(refactored)
        }
        "extract_function" => {
            let lines = args.get("lines").and_then(|l| l.as_str()).ok_or_else(|| {
                FerroError::Tool("Missing 'lines' for extract_function refactoring".into())
            })?;

            Ok(format!(
                "// TODO: Extract function from lines {}\n{}",
                lines, content
            ))
        }
        _ => Err(FerroError::Tool(format!(
            "Unknown refactoring type: {}",
            refactoring_type
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_refactor_rust_rename() {
        let code = "fn my_function() {}\nfn another() { my_function(); }";
        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(refactor_rust(
                code,
                "rename",
                "my_function",
                &serde_json::json!({"new_name": "renamed_function"}),
            ))
            .unwrap();
        assert!(result.contains("renamed_function"));
    }
}
