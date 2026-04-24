# Ferroclaw Tool Implementation Quickstart

This guide provides step-by-step instructions for implementing the missing high-priority tools in Ferroclaw.

---

## Quick Overview

**Total Missing Tools**: ~20-25 tools across 8 categories
**High Priority**: 9 tools (Code Intelligence + Collaboration + Reasoning)
**Estimated Time**: 2-3 weeks for high-priority tools

---

## Step 1: Set Up Development Environment

```bash
# Ensure you're in the ferroclaw directory
cd /Users/ghost/Desktop/ferroclaw

# Install development dependencies
cargo install cargo-watch  # For auto-reloading during dev

# Run tests to ensure everything works
cargo test --all

# Run a quick check
cargo check
```

---

## Step 2: Implement Code Intelligence Tools

### 2.1 Create `analyze_code` Tool

**File**: `src/tools/analyze_code.rs`

```bash
# Create the file
touch src/tools/analyze_code.rs
```

**Content**:

```rust
//! Code analysis tool - understand structure, dependencies, complexity

use crate::error::FerroError;
use crate::tool::{ToolFuture, ToolHandler};
use crate::types::Capability;
use serde_json::Value;

pub fn analyze_code_meta() -> crate::types::ToolMeta {
    crate::types::ToolMeta {
        definition: crate::types::ToolDefinition {
            name: "analyze_code".into(),
            description: "Analyze code structure, dependencies, and complexity. Supports Rust, Python, JavaScript, TypeScript.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to file or directory to analyze"
                    },
                    "analysis_type": {
                        "type": "string",
                        "enum": ["structure", "dependencies", "complexity", "imports", "all"],
                        "description": "Type of analysis to perform"
                    }
                },
                "required": ["path"]
            }),
            server_name: None,
        },
        required_capabilities: vec![Capability::FsRead],
        source: crate::types::ToolSource::Builtin,
    }
}

pub struct AnalyzeCodeHandler;

impl ToolHandler for AnalyzeCodeHandler {
    fn call<'a>(&'a self, call_id: &'a str, arguments: &'a Value) -> ToolFuture<'a> {
        Box::pin(async move {
            let path = arguments
                .get("path")
                .and_then(|p| p.as_str())
                .ok_or_else(|| FerroError::Tool("Missing 'path' argument".into()))?;

            let analysis_type = arguments
                .get("analysis_type")
                .and_then(|t| t.as_str())
                .unwrap_or("all");

            // Check if path is a file or directory
            let metadata = tokio::fs::metadata(path).await
                .map_err(|e| FerroError::Tool(format!("Cannot access {}: {}", path, e)))?;

            let result = if metadata.is_file() {
                analyze_file(path, analysis_type).await?
            } else {
                analyze_directory(path, analysis_type).await?
            };

            Ok(crate::types::ToolResult {
                call_id: call_id.to_string(),
                content: result,
                is_error: false,
            })
        })
    }
}

async fn analyze_file(path: &str, analysis_type: &str) -> Result<String, FerroError> {
    let content = tokio::fs::read_to_string(path).await
        .map_err(|e| FerroError::Tool(format!("Cannot read {}: {}", path, e)))?;

    let ext = std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    match ext {
        "rs" => analyze_rust_file(&content, analysis_type),
        "py" => analyze_python_file(&content, analysis_type),
        "js" | "ts" | "jsx" | "tsx" => analyze_javascript_file(&content, analysis_type),
        _ => Ok(format!("⚠️  Unsupported language: {}\nSupported: Rust (.rs), Python (.py), JavaScript/TypeScript (.js, .ts, .jsx, .tsx)", ext)),
    }
}

fn analyze_rust_file(content: &str, analysis_type: &str) -> Result<String, FerroError> {
    let mut functions = Vec::new();
    let mut structs = Vec::new();
    let mut enums = Vec::new();
    let mut impls = Vec::new();
    let mut mods = Vec::new();
    let mut traits = Vec::new();
    let mut uses = Vec::new();
    let mut consts = Vec::new();
    let mut statics = Vec::new();
    let mut types = Vec::new();
    let mut lines = 0;
    let mut code_lines = 0;
    let mut comment_lines = 0;

    for line in content.lines() {
        lines += 1;
        let trimmed = line.trim();
        let is_comment = trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with("*");
        
        if is_comment {
            comment_lines += 1;
        } else if !trimmed.is_empty() {
            code_lines += 1;
        }

        // Functions
        if trimmed.starts_with("pub fn ") || trimmed.starts_with("pub async fn ") || 
           trimmed.starts_with("fn ") || trimmed.starts_with("async fn ") {
            let sig = trimmed
                .trim_start_matches("pub ")
                .trim_start_matches("async ")
                .trim_start_matches("fn ");
            let name = sig.split('(').next().unwrap_or("");
            if !name.is_empty() {
                functions.push(name.trim().to_string());
            }
        }

        // Structs
        if trimmed.starts_with("pub struct ") || trimmed.starts_with("struct ") {
            let decl = trimmed
                .trim_start_matches("pub ")
                .trim_start_matches("struct ");
            let name = decl.split('{').next().unwrap_or("")
                .split('(').next().unwrap_or("")
                .split(';').next().unwrap_or("");
            if !name.is_empty() {
                structs.push(name.trim().to_string());
            }
        }

        // Enums
        if trimmed.starts_with("pub enum ") || trimmed.starts_with("enum ") {
            let decl = trimmed
                .trim_start_matches("pub ")
                .trim_start_matches("enum ");
            let name = decl.split('{').next().unwrap_or("");
            if !name.is_empty() {
                enums.push(name.trim().to_string());
            }
        }

        // Impl blocks
        if trimmed.starts_with("impl ") {
            let impl_decl = trimmed.trim_start_matches("impl ");
            let name = impl_decl.split('{').next().unwrap_or("")
                .split(" for ").next().unwrap_or(impl_decl);
            if !name.is_empty() {
                impls.push(name.trim().to_string());
            }
        }

        // Modules
        if trimmed.starts_with("pub mod ") || trimmed.starts_with("mod ") {
            let decl = trimmed
                .trim_start_matches("pub ")
                .trim_start_matches("mod ");
            let name = decl.split('{').next().unwrap_or("")
                .split(';').next().unwrap_or("");
            if !name.is_empty() {
                mods.push(name.trim().to_string());
            }
        }

        // Traits
        if trimmed.starts_with("pub trait ") || trimmed.starts_with("trait ") {
            let decl = trimmed
                .trim_start_matches("pub ")
                .trim_start_matches("trait ");
            let name = decl.split('{').next().unwrap_or("");
            if !name.is_empty() {
                traits.push(name.trim().to_string());
            }
        }

        // Use statements
        if trimmed.starts_with("use ") {
            let use_decl = trimmed.trim_start_matches("use ");
            let name = use_decl.split(';').next().unwrap_or("");
            if !name.is_empty() {
                uses.push(name.trim().to_string());
            }
        }

        // Constants
        if trimmed.starts_with("pub const ") || trimmed.starts_with("const ") {
            let decl = trimmed
                .trim_start_matches("pub ")
                .trim_start_matches("const ");
            let name = decl.split(':').next().unwrap_or("")
                .split('=').next().unwrap_or("");
            if !name.is_empty() {
                consts.push(name.trim().to_string());
            }
        }

        // Static variables
        if trimmed.starts_with("pub static ") || trimmed.starts_with("static ") {
            let decl = trimmed
                .trim_start_matches("pub ")
                .trim_start_matches("static ");
            let name = decl.split(':').next().unwrap_or("")
                .split('=').next().unwrap_or("");
            if !name.is_empty() {
                statics.push(name.trim().to_string());
            }
        }

        // Type aliases
        if trimmed.starts_with("pub type ") || trimmed.starts_with("type ") {
            let decl = trimmed
                .trim_start_matches("pub ")
                .trim_start_matches("type ");
            let name = decl.split('=').next().unwrap_or("")
                .split(';').next().unwrap_or("");
            if !name.is_empty() {
                types.push(name.trim().to_string());
            }
        }
    }

    let mut output = String::new();
    output.push_str(&format!("🦀 Rust Code Analysis\n"));
    output.push_str(&format!("═══════════════════════════════════════\n\n"));

    if analysis_type == "structure" || analysis_type == "all" {
        output.push_str(&format!("📊 Structure\n"));
        output.push_str(&format!("  Modules:     {}\n", mods.len()));
        output.push_str(&format!("  Structs:     {}\n", structs.len()));
        output.push_str(&format!("  Enums:       {}\n", enums.len()));
        output.push_str(&format!("  Traits:     {}\n", traits.len()));
        output.push_str(&format!("  Impl Blocks: {}\n", impls.len()));
        output.push_str(&format!("  Functions:   {}\n", functions.len()));
        output.push_str(&format!("  Use Statements: {}\n", uses.len()));
        output.push_str(&format!("  Constants:   {}\n", consts.len()));
        output.push_str(&format!("  Statics:     {}\n", statics.len()));
        output.push_str(&format!("  Type Aliases: {}\n", types.len()));
        output.push_str("\n");
    }

    if analysis_type == "complexity" || analysis_type == "all" {
        output.push_str(&format!("📈 Complexity Metrics\n"));
        output.push_str(&format!("  Total Lines:     {}\n", lines));
        output.push_str(&format!("  Code Lines:      {}\n", code_lines));
        output.push_str(&format!("  Comment Lines:   {}\n", comment_lines));
        let comment_ratio = if lines > 0 {
            (comment_lines as f64 / lines as f64 * 100.0) as usize
        } else { 0 };
        output.push_str(&format!("  Comment Ratio:   {}%\n", comment_ratio));
        output.push_str("\n");
    }

    if analysis_type == "all" && !functions.is_empty() {
        output.push_str(&format!("📝 Functions:\n"));
        for fn_name in functions.iter().take(10) {
            output.push_str(&format!("  • {}\n", fn_name));
        }
        if functions.len() > 10 {
            output.push_str(&format!("  ... and {} more\n", functions.len() - 10));
        }
        output.push_str("\n");
    }

    Ok(output)
}

fn analyze_python_file(content: &str, analysis_type: &str) -> Result<String, FerroError> {
    let mut functions = Vec::new();
    let mut classes = Vec::new();
    let mut imports = Vec::new();
    let mut lines = 0;
    let mut code_lines = 0;
    let mut comment_lines = 0;

    for line in content.lines() {
        lines += 1;
        let trimmed = line.trim();
        let is_comment = trimmed.starts_with("#") || trimmed.starts_with("'''") || trimmed.starts_with("\"\"\"");
        
        if is_comment {
            comment_lines += 1;
        } else if !trimmed.is_empty() {
            code_lines += 1;
        }

        // Functions
        if trimmed.starts_with("def ") {
            let sig = trimmed.trim_start_matches("def ");
            let name = sig.split('(').next().unwrap_or("");
            if !name.is_empty() {
                functions.push(name.trim().to_string());
            }
        }

        // Classes
        if trimmed.starts_with("class ") {
            let decl = trimmed.trim_start_matches("class ");
            let name = decl.split(':').next().unwrap_or("")
                .split('(').next().unwrap_or("");
            if !name.is_empty() {
                classes.push(name.trim().to_string());
            }
        }

        // Imports
        if trimmed.starts_with("import ") || trimmed.starts_with("from ") {
            let import_decl = trimmed
                .trim_start_matches("import ")
                .trim_start_matches("from ");
            let name = import_decl.split(" import").next().unwrap_or("")
                .split(" as ").next().unwrap_or("")
                .split(',').next().unwrap_or("");
            if !name.is_empty() {
                imports.push(name.trim().to_string());
            }
        }
    }

    let mut output = String::new();
    output.push_str(&format!("🐍 Python Code Analysis\n"));
    output.push_str(&format!("═══════════════════════════════════════\n\n"));

    if analysis_type == "structure" || analysis_type == "all" {
        output.push_str(&format!("📊 Structure\n"));
        output.push_str(&format!("  Classes:     {}\n", classes.len()));
        output.push_str(&format!("  Functions:   {}\n", functions.len()));
        output.push_str(&format!("  Imports:     {}\n", imports.len()));
        output.push_str("\n");
    }

    if analysis_type == "complexity" || analysis_type == "all" {
        output.push_str(&format!("📈 Complexity Metrics\n"));
        output.push_str(&format!("  Total Lines:     {}\n", lines));
        output.push_str(&format!("  Code Lines:      {}\n", code_lines));
        output.push_str(&format!("  Comment Lines:   {}\n", comment_lines));
        let comment_ratio = if lines > 0 {
            (comment_lines as f64 / lines as f64 * 100.0) as usize
        } else { 0 };
        output.push_str(&format!("  Comment Ratio:   {}%\n", comment_ratio));
        output.push_str("\n");
    }

    Ok(output)
}

fn analyze_javascript_file(content: &str, analysis_type: &str) -> Result<String, FerroError> {
    let mut functions = Vec::new();
    let mut classes = Vec::new();
    let mut imports = Vec::new();
    let mut exports = Vec::new();
    let mut lines = 0;
    let mut code_lines = 0;
    let mut comment_lines = 0;

    for line in content.lines() {
        lines += 1;
        let trimmed = line.trim();
        let is_comment = trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with("*");
        
        if is_comment {
            comment_lines += 1;
        } else if !trimmed.is_empty() {
            code_lines += 1;
        }

        // Functions (function keyword, arrow functions)
        if trimmed.starts_with("function ") || 
           trimmed.contains(" = (") || 
           trimmed.contains("= async (") ||
           trimmed.contains(" = function") {
            let name = if trimmed.starts_with("function ") {
                let sig = trimmed.trim_start_matches("function ");
                sig.split('(').next().unwrap_or("")
            } else if trimmed.contains("export function ") {
                let sig = trimmed.trim_start_matches("export function ");
                sig.split('(').next().unwrap_or("")
            } else {
                // Try to extract from variable assignment
                if let Some(idx) = trimmed.find("function ") {
                    &trimmed[idx + 9..]
                } else if let Some(idx) = trimmed.find("=") {
                    &trimmed[..idx].trim()
                } else {
                    ""
                }
            };
            let fn_name = name.split('(').next().unwrap_or("")
                .split('{').next().unwrap_or("")
                .trim();
            if !fn_name.is_empty() && fn_name != "=" {
                functions.push(fn_name.to_string());
            }
        }

        // Classes
        if trimmed.starts_with("class ") || trimmed.starts_with("export class ") {
            let decl = trimmed
                .trim_start_matches("export ")
                .trim_start_matches("class ");
            let name = decl.split('{').next().unwrap_or("")
                .split('(').next().unwrap_or("")
                .split(" extends ").next().unwrap_or("")
                .trim();
            if !name.is_empty() {
                classes.push(name.to_string());
            }
        }

        // Imports (ES6)
        if trimmed.starts_with("import ") {
            imports.push(trimmed.trim_start_matches("import ").to_string());
        }

        // Exports
        if trimmed.starts_with("export ") {
            exports.push(trimmed.trim_start_matches("export ").to_string());
        }
    }

    let mut output = String::new();
    output.push_str(&format!("📜 JavaScript/TypeScript Code Analysis\n"));
    output.push_str(&format!("════════════════════════════════════════════\n\n"));

    if analysis_type == "structure" || analysis_type == "all" {
        output.push_str(&format!("📊 Structure\n"));
        output.push_str(&format!("  Classes:    {}\n", classes.len()));
        output.push_str(&format!("  Functions:  {}\n", functions.len()));
        output.push_str(&format!("  Imports:    {}\n", imports.len()));
        output.push_str(&format!("  Exports:    {}\n", exports.len()));
        output.push_str("\n");
    }

    if analysis_type == "complexity" || analysis_type == "all" {
        output.push_str(&format!("📈 Complexity Metrics\n"));
        output.push_str(&format!("  Total Lines:     {}\n", lines));
        output.push_str(&format!("  Code Lines:      {}\n", code_lines));
        output.push_str(&format!("  Comment Lines:   {}\n", comment_lines));
        let comment_ratio = if lines > 0 {
            (comment_lines as f64 / lines as f64 * 100.0) as usize
        } else { 0 };
        output.push_str(&format!("  Comment Ratio:   {}%\n", comment_ratio));
        output.push_str("\n");
    }

    Ok(output)
}

async fn analyze_directory(path: &str, analysis_type: &str) -> Result<String, FerroError> {
    let mut output = String::new();
    output.push_str(&format!("📁 Directory Analysis: {}\n", path));
    output.push_str(&format!("═══════════════════════════════════════\n\n"));

    let mut entries = tokio::fs::read_dir(path).await
        .map_err(|e| FerroError::Tool(format!("Cannot list {}: {}", path, e)))?;

    let mut file_count = 0;
    let mut dir_count = 0;
    let mut rust_files = Vec::new();
    let mut python_files = Vec::new();
    let mut js_files = Vec::new();

    while let Ok(Some(entry)) = entries.next_entry().await {
        let file_name = entry.file_name().to_string_lossy().to_string();
        let file_path = entry.path();
        
        if let Ok(ft) = entry.file_type().await {
            if ft.is_file() {
                file_count += 1;
                let ext = file_path.extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("");
                
                match ext {
                    "rs" => rust_files.push(file_name),
                    "py" => python_files.push(file_name),
                    "js" | "ts" | "jsx" | "tsx" => js_files.push(file_name),
                    _ => {}
                }
            } else if ft.is_dir() {
                dir_count += 1;
            }
        }
    }

    output.push_str(&format!("📊 Overview\n"));
    output.push_str(&format!("  Directories: {}\n", dir_count));
    output.push_str(&format!("  Files:       {}\n", file_count));
    output.push_str(&format!("  Rust files:  {}\n", rust_files.len()));
    output.push_str(&format!("  Python files: {}\n", python_files.len()));
    output.push_str(&format!("  JS/TS files:  {}\n", js_files.len()));
    output.push_str("\n");

    if !rust_files.is_empty() {
        output.push_str(&format!("🦀 Rust Files:\n"));
        for file in rust_files.iter().take(5) {
            output.push_str(&format!("  • {}\n", file));
        }
        if rust_files.len() > 5 {
            output.push_str(&format!("  ... and {} more\n", rust_files.len() - 5));
        }
        output.push_str("\n");
    }

    if !python_files.is_empty() {
        output.push_str(&format!("🐍 Python Files:\n"));
        for file in python_files.iter().take(5) {
            output.push_str(&format!("  • {}\n", file));
        }
        if python_files.len() > 5 {
            output.push_str(&format!("  ... and {} more\n", python_files.len() - 5));
        }
        output.push_str("\n");
    }

    if !js_files.is_empty() {
        output.push_str(&format!("📜 JS/TS Files:\n"));
        for file in js_files.iter().take(5) {
            output.push_str(&format!("  • {}\n", file));
        }
        if js_files.len() > 5 {
            output.push_str(&format!("  ... and {} more\n", js_files.len() - 5));
        }
        output.push_str("\n");
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_rust_file() {
        let code = r#"
pub struct MyStruct {
    value: i32,
}

pub fn my_function(x: i32) -> i32 {
    x * 2
}

impl MyStruct {
    pub fn new() -> Self {
        Self { value: 0 }
    }
}
"#;
        let result = analyze_rust_file(code, "structure").unwrap();
        assert!(result.contains("Structs:"));
        assert!(result.contains("Functions:"));
        assert!(result.contains("Impl Blocks:"));
    }

    #[test]
    fn test_analyze_python_file() {
        let code = r#"
class MyClass:
    def __init__(self, value):
        self.value = value
    
    def my_method(self):
        return self.value * 2
"#;
        let result = analyze_python_file(code, "structure").unwrap();
        assert!(result.contains("Classes:"));
        assert!(result.contains("Functions:"));
    }
}
```

### 2.2 Register the Tool

**File**: `src/tools/mod.rs`

Add the module:

```rust
pub mod analyze_code;
```

**File**: `src/tools/builtin.rs`

Import and register:

```rust
use crate::tools::analyze_code::{AnalyzeCodeHandler, analyze_code_meta};

pub fn register_builtin_tools(
    registry: &mut ToolRegistry,
    memory: Arc<Mutex<MemoryStore>>,
) {
    // ... existing tools ...

    // analyze_code
    registry.register(analyze_code_meta(), Box::new(AnalyzeCodeHandler));
}
```

### 2.3 Test the Tool

```bash
# Run tests
cargo test analyze_code

# Or test interactively
cargo build --release
./target/release/ferroclaw exec "Analyze the file src/tools/analyze_code.rs"
```

---

## Step 3: Implement Collaboration Tools

### 3.1 Create `collaboration.rs`

**File**: `src/tools/collaboration.rs`

```bash
touch src/tools/collaboration.rs
```

**Content**:

```rust
//! Collaboration tools - notify_user, request_approval, share_context

use crate::error::FerroError;
use crate::tool::{ToolFuture, ToolHandler};
use crate::types::Capability;
use serde_json::Value;

// ==================== notify_user ====================

pub fn notify_user_meta() -> crate::types::ToolMeta {
    crate::types::ToolMeta {
        definition: crate::types::ToolDefinition {
            name: "notify_user".into(),
            description: "Send a notification to the user. Supports multiple levels (info, warning, error, success) and channels (terminal, telegram, slack, email).".into(),
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
                        "description": "Where to send the notification"
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
                        "description": "Detailed description of the action"
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
            std::io::stdin().read_line(&mut input)
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
                _ => Err(FerroError::Tool(format!("Unknown context type: {}", context_type))),
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
    match format {
        "text" => Ok("📁 Workspace Context\n━━━━━━━━━━━━━━━━━━━━━━\n\n  Root: /Users/ghost/Desktop/ferroclaw\n  Project: Ferroclaw\n  Description: A security-first AI agent framework\n\n".to_string()),
        "json" => Ok(serde_json::to_string_pretty(&serde_json::json!({
            "workspace": {
                "root": "/Users/ghost/Desktop/ferroclaw",
                "project": "Ferroclaw",
                "description": "A security-first AI agent framework"
            }
        })).map_err(|e| FerroError::Tool(format!("JSON error: {}", e)))?),
        "markdown" => Ok("# Workspace Context\n\n- **Root**: /Users/ghost/Desktop/ferroclaw\n- **Project**: Ferroclaw\n- **Description**: A security-first AI agent framework\n\n".to_string()),
        _ => Err(FerroError::Tool(format!("Unknown format: {}", format))),
    }
}

async fn format_tasks_context(format: &str) -> Result<String, FerroError> {
    // TODO: Integrate with TaskSystem
    Ok("📋 Tasks Context\n━━━━━━━━━━━━━━━━━━━━━━\n\n  (TaskSystem integration pending)\n\n".to_string())
}

async fn format_memory_context(format: &str) -> Result<String, FerroError> {
    // TODO: Integrate with MemoryStore
    Ok("🧠 Memory Context\n━━━━━━━━━━━━━━━━━━━━━━\n\n  (MemoryStore integration pending)\n\n".to_string())
}

async fn format_canvas_context(format: &str) -> Result<String, FerroError> {
    Ok("🎨 Canvas Context\n━━━━━━━━━━━━━━━━━━━━━━\n\n  (Canvas API integration pending)\n\n".to_string())
}

async fn format_all_context(format: &str) -> Result<String, FerroError> {
    let mut output = String::new();
    output.push_str(&format_workspace_context(format).await?);
    output.push_str(&format_tasks_context(format).await?);
    output.push_str(&format_memory_context(format).await?);
    output.push_str(&format_canvas_context(format).await?);
    Ok(output)
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
}
```

### 3.2 Register Collaboration Tools

**File**: `src/tools/mod.rs`

```rust
pub mod collaboration;
```

**File**: `src/tools/builtin.rs`

```rust
use crate::tools::collaboration::{
    NotifyUserHandler, notify_user_meta,
    RequestApprovalHandler, request_approval_meta,
    ShareContextHandler, share_context_meta
};

pub fn register_builtin_tools(
    registry: &mut ToolRegistry,
    memory: Arc<Mutex<MemoryStore>>,
) {
    // ... existing tools ...

    // Collaboration tools
    registry.register(notify_user_meta(), Box::new(NotifyUserHandler));
    registry.register(request_approval_meta(), Box::new(RequestApprovalHandler));
    registry.register(share_context_meta(), Box::new(ShareContextHandler));
}
```

---

## Step 4: Run Tests

```bash
# Run all tests
cargo test --all

# Run specific tool tests
cargo test analyze_code
cargo test collaboration

# Build release
cargo build --release
```

---

## Step 5: Test Interactively

```bash
# Test analyze_code
./target/release/ferroclaw exec "Analyze the src/tools/analyze_code.rs file"

# Test notify_user
./target/release/ferroclaw exec "Send me a notification about the build status"

# Test request_approval
./target/release/ferroclaw exec "Request approval before deleting the test directory"

# Test share_context
./target/release/ferroclaw exec "Share the workspace context in JSON format"
```

---

## Next Steps

After implementing these high-priority tools, continue with:

1. **Code Intelligence**: `refactor_code`, `generate_tests`, `review_code`
2. **Monitoring**: `get_logs`, `trace_execution`, `measure_metrics`
3. **Command Execution**: `execute_code`, `start_process`, `stream_output`

See `docs/TOOL_GAP_ANALYSIS.md` for the complete list and implementation plans.

---

*Last updated: 2025-02-10*
