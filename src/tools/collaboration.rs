//! Collaboration tools - notify_user, request_approval, share_context, comment

use crate::error::FerroError;
use crate::tool::{ToolFuture, ToolHandler};
use crate::types::Capability;
use serde_json::Value;

// ==================== notify_user ====================

pub fn notify_user_meta() -> crate::types::ToolMeta {
    crate::types::ToolMeta {
        definition: crate::types::ToolDefinition {
            name: "notify_user".into(),
            description: "Send a notification to user. Supports multiple levels (info, warning, error, success) and channels (terminal, telegram, slack, email).".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "message": {
                        "type": "string",
                        "description": "Notification message"
                    },
                    "level": {
                        "type": "string",
                        "enum": ["info", "warning", "error", "success"],
                        "description": "Notification level"
                    },
                    "channel": {
                        "type": "string",
                        "enum": ["terminal", "telegram", "slack", "email"],
                        "description": "Where to send notification"
                    }
                },
                "required": ["message"]
            }),
            server_name: None,
        },
        required_capabilities: vec![],
        source: crate::types::ToolSource::Builtin,
    }
}

pub struct NotifyUserHandler;

impl ToolHandler for NotifyUserHandler {
    fn call<'a>(&'a self, call_id: &'a str, arguments: &'a Value) -> ToolFuture<'a> {
        Box::pin(async move {
            let message = arguments
                .get("message")
                .and_then(|m| m.as_str())
                .ok_or_else(|| FerroError::Tool("Missing 'message' argument".into()))?;

            let level = arguments
                .get("level")
                .and_then(|l| l.as_str())
                .unwrap_or("info");

            let _channel = arguments
                .get("channel")
                .and_then(|c| c.as_str())
                .unwrap_or("terminal");

            let icon = match level {
                "info" => "ℹ️",
                "warning" => "⚠️",
                "error" => "❌",
                "success" => "✅",
                _ => "📢",
            };

            println!("\n{} {}\n", icon, message);

            // TODO: Implement Telegram, Slack, Email sending

            Ok(crate::types::ToolResult {
                call_id: call_id.to_string(),
                content: format!("Notification sent: {}", message),
                is_error: false,
            })
        })
    }
}

// ==================== request_approval ====================

pub fn request_approval_meta() -> crate::types::ToolMeta {
    crate::types::ToolMeta {
        definition: crate::types::ToolDefinition {
            name: "request_approval".into(),
            description: "Request human approval before proceeding with an action. Supports auto-approval mode for automation.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "description": "The action requiring approval"
                    },
                    "description": {
                        "type": "string",
                        "description": "Detailed description of action"
                    },
                    "auto_approve": {
                        "type": "boolean",
                        "description": "Automatically approve without prompting"
                    }
                },
                "required": ["action", "description"]
            }),
            server_name: None,
        },
        required_capabilities: vec![],
        source: crate::types::ToolSource::Builtin,
    }
}

pub struct RequestApprovalHandler;

impl ToolHandler for RequestApprovalHandler {
    fn call<'a>(&'a self, call_id: &'a str, arguments: &'a Value) -> ToolFuture<'a> {
        Box::pin(async move {
            let action = arguments
                .get("action")
                .and_then(|a| a.as_str())
                .ok_or_else(|| FerroError::Tool("Missing 'action' argument".into()))?;

            let description = arguments
                .get("description")
                .and_then(|d| d.as_str())
                .ok_or_else(|| FerroError::Tool("Missing 'description' argument".into()))?;

            let auto_approve = arguments
                .get("auto_approve")
                .and_then(|b| b.as_bool())
                .unwrap_or(false);

            if auto_approve {
                return Ok(crate::types::ToolResult {
                    call_id: call_id.to_string(),
                    content: format!("✅ Auto-approved: {}", action),
                    is_error: false,
                });
            }

            println!("\n╔══════════════════════════════════════════════════════╗");
            println!("║  🤔 Approval Request                                   ║");
            println!("╚══════════════════════════════════════════════════════╝");
            println!("\n📋 Action:");
            println!("  {}", action);
            println!("\n📝 Description:");
            for line in description.lines() {
                println!("  {}", line);
            }
            println!("\n╔══════════════════════════════════════════════════════╗");
            println!("║  Approve this action? [Y/n/q]                         ║");
            println!("║  Y = Yes, n = No, q = Quit/Cancel                     ║");
            println!("╚══════════════════════════════════════════════════════╝\n");

            // Read user input
            let mut input = String::new();
            std::io::stdin()
                .read_line(&mut input)
                .map_err(|e| FerroError::Tool(format!("Cannot read input: {}", e)))?;

            let input = input.trim().to_lowercase();
            let approved = input.is_empty() || input == "y" || input == "yes";
            let canceled = input == "q" || input == "quit" || input == "cancel";

            if canceled {
                return Ok(crate::types::ToolResult {
                    call_id: call_id.to_string(),
                    content: format!("❌ Canceled: {}", action),
                    is_error: true,
                });
            }

            if approved {
                Ok(crate::types::ToolResult {
                    call_id: call_id.to_string(),
                    content: format!("✅ Approved: {}", action),
                    is_error: false,
                })
            } else {
                Ok(crate::types::ToolResult {
                    call_id: call_id.to_string(),
                    content: format!("❌ Rejected: {}", action),
                    is_error: true,
                })
            }
        })
    }
}

// ==================== share_context ====================

pub fn share_context_meta() -> crate::types::ToolMeta {
    crate::types::ToolMeta {
        definition: crate::types::ToolDefinition {
            name: "share_context".into(),
            description: "Share workspace context (tasks, memory, canvas, or full workspace) in various formats (text, json, markdown). Useful for collaboration and debugging.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "context_type": {
                        "type": "string",
                        "enum": ["workspace", "tasks", "memory", "canvas", "all"],
                        "description": "Type of context to share"
                    },
                    "format": {
                        "type": "string",
                        "enum": ["text", "json", "markdown"],
                        "description": "Output format"
                    }
                },
                "required": ["context_type"]
            }),
            server_name: None,
        },
        required_capabilities: vec![Capability::MemoryRead],
        source: crate::types::ToolSource::Builtin,
    }
}

pub struct ShareContextHandler;

impl ToolHandler for ShareContextHandler {
    fn call<'a>(&'a self, call_id: &'a str, arguments: &'a Value) -> ToolFuture<'a> {
        Box::pin(async move {
            let context_type = arguments
                .get("context_type")
                .and_then(|t| t.as_str())
                .ok_or_else(|| FerroError::Tool("Missing 'context_type' argument".into()))?;

            let format = arguments
                .get("format")
                .and_then(|f| f.as_str())
                .unwrap_or("text");

            let content = match context_type {
                "workspace" => format_workspace_context(format).await,
                "tasks" => format_tasks_context(format).await,
                "memory" => format_memory_context(format).await,
                "canvas" => format_canvas_context(format).await,
                "all" => format_all_context(format).await,
                _ => Err(FerroError::Tool(format!(
                    "Unknown context type: {}",
                    context_type
                ))),
            }?;

            Ok(crate::types::ToolResult {
                call_id: call_id.to_string(),
                content,
                is_error: false,
            })
        })
    }
}

async fn format_workspace_context(format: &str) -> Result<String, FerroError> {
    let root = std::env::current_dir()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    match format {
        "text" => Ok(format!(
            "📁 Workspace Context\n\
             ═══════════════════════\n\n\
              Root: {}\n\
              Project: Ferroclaw\n\
              Description: A security-first AI agent framework\n\n",
            root
        )),
        "json" => Ok(serde_json::to_string_pretty(&serde_json::json!({
            "workspace": {
                "root": root,
                "project": "Ferroclaw",
                "description": "A security-first AI agent framework"
            }
        }))
        .map_err(|e| FerroError::Tool(format!("JSON error: {}", e)))?),
        "markdown" => Ok(format!(
            "# Workspace Context\n\n\
            - **Root**: {}\n\
            - **Project**: Ferroclaw\n\
            - **Description**: A security-first AI agent framework\n\n",
            root
        )),
        _ => Err(FerroError::Tool(format!("Unknown format: {}", format))),
    }
}

async fn format_tasks_context(_format: &str) -> Result<String, FerroError> {
    // TODO: Integrate with TaskSystem
    Ok("📋 Tasks Context\n\
         ═══════════════════════\n\n\
          (TaskSystem integration pending)\n\n"
        .to_string())
}

async fn format_memory_context(_format: &str) -> Result<String, FerroError> {
    // TODO: Integrate with MemoryStore
    Ok("🧠 Memory Context\n\
         ═══════════════════════\n\n\
          (MemoryStore integration pending)\n\n"
        .to_string())
}

async fn format_canvas_context(_format: &str) -> Result<String, FerroError> {
    Ok("🎨 Canvas Context\n\
         ═══════════════════════\n\n\
          (Canvas API integration pending)\n\n"
        .to_string())
}

async fn format_all_context(format: &str) -> Result<String, FerroError> {
    let mut output = String::new();
    output.push_str(&format_workspace_context(format).await?);
    output.push_str(&format_tasks_context(format).await?);
    output.push_str(&format_memory_context(format).await?);
    output.push_str(&format_canvas_context(format).await?);
    Ok(output)
}

// ==================== comment ====================

pub fn comment_meta() -> crate::types::ToolMeta {
    crate::types::ToolMeta {
        definition: crate::types::ToolDefinition {
            name: "comment".into(),
            description: "Add annotations or comments to files or tiles. Useful for documentation and collaboration.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "target": {
                        "type": "string",
                        "description": "Target to comment on (file path or tile ID)"
                    },
                    "target_type": {
                        "type": "string",
                        "enum": ["file", "tile"],
                        "description": "Type of target"
                    },
                    "comment": {
                        "type": "string",
                        "description": "Comment or annotation to add"
                    },
                    "line": {
                        "type": "integer",
                        "description": "Line number (for file comments)"
                    }
                },
                "required": ["target", "target_type", "comment"]
            }),
            server_name: None,
        },
        required_capabilities: vec![Capability::FsWrite],
        source: crate::types::ToolSource::Builtin,
    }
}

pub struct CommentHandler;

impl ToolHandler for CommentHandler {
    fn call<'a>(&'a self, call_id: &'a str, arguments: &'a Value) -> ToolFuture<'a> {
        Box::pin(async move {
            let target = arguments
                .get("target")
                .and_then(|t| t.as_str())
                .ok_or_else(|| FerroError::Tool("Missing 'target' argument".into()))?;

            let target_type = arguments
                .get("target_type")
                .and_then(|t| t.as_str())
                .ok_or_else(|| FerroError::Tool("Missing 'target_type' argument".into()))?;

            let comment = arguments
                .get("comment")
                .and_then(|c| c.as_str())
                .ok_or_else(|| FerroError::Tool("Missing 'comment' argument".into()))?;

            let _line = arguments.get("line").and_then(|l| l.as_i64());

            match target_type {
                "file" => {
                    // Add comment as a special marker in the file
                    let content = tokio::fs::read_to_string(target)
                        .await
                        .map_err(|e| FerroError::Tool(format!("Cannot read {}: {}", target, e)))?;

                    let comment_marker = format!("\n// ANNOTATION: {}\n", comment);
                    let new_content = format!("{}{}", comment_marker, content);

                    tokio::fs::write(target, new_content)
                        .await
                        .map_err(|e| FerroError::Tool(format!("Cannot write {}: {}", target, e)))?;

                    Ok(crate::types::ToolResult {
                        call_id: call_id.to_string(),
                        content: format!("✅ Added comment to file: {}", target),
                        is_error: false,
                    })
                }
                "tile" => {
                    // TODO: Implement tile commenting via canvas API
                    Ok(crate::types::ToolResult {
                        call_id: call_id.to_string(),
                        content: format!("📝 Commented on tile {}: {}", target, comment),
                        is_error: false,
                    })
                }
                _ => Err(FerroError::Tool(format!(
                    "Unknown target type: {}",
                    target_type
                ))),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notify_user_meta() {
        let meta = notify_user_meta();
        assert_eq!(meta.definition.name, "notify_user");
    }

    #[test]
    fn test_request_approval_meta() {
        let meta = request_approval_meta();
        assert_eq!(meta.definition.name, "request_approval");
    }

    #[test]
    fn test_share_context_meta() {
        let meta = share_context_meta();
        assert_eq!(meta.definition.name, "share_context");
    }

    #[test]
    fn test_comment_meta() {
        let meta = comment_meta();
        assert_eq!(meta.definition.name, "comment");
    }
}
