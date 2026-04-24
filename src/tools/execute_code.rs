//! Code execution tool - run code in multiple languages

use crate::error::FerroError;
use crate::tool::{ToolFuture, ToolHandler};
use crate::types::Capability;
use serde_json::Value;

pub fn execute_code_meta() -> crate::types::ToolMeta {
    crate::types::ToolMeta {
        definition: crate::types::ToolDefinition {
            name: "execute_code".into(),
            description: "Execute code snippets in Python, Node.js, Rust, or other languages. Captures stdout and stderr.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "language": {
                        "type": "string",
                        "enum": ["python", "node", "rust", "bash", "ruby", "php", "go"],
                        "description": "Programming language"
                    },
                    "code": {
                        "type": "string",
                        "description": "Code to execute"
                    },
                    "timeout": {
                        "type": "integer",
                        "description": "Timeout in seconds (default: 30)",
                        "default": 30
                    }
                },
                "required": ["language", "code"]
            }),
            server_name: None,
        },
        required_capabilities: vec![Capability::ProcessExec],
        source: crate::types::ToolSource::Builtin,
    }
}

pub struct ExecuteCodeHandler;

impl ToolHandler for ExecuteCodeHandler {
    fn call<'a>(&'a self, call_id: &'a str, arguments: &'a Value) -> ToolFuture<'a> {
        Box::pin(async move {
            let language = arguments
                .get("language")
                .and_then(|l| l.as_str())
                .ok_or_else(|| FerroError::Tool("Missing 'language' argument".into()))?;

            let code = arguments
                .get("code")
                .and_then(|c| c.as_str())
                .ok_or_else(|| FerroError::Tool("Missing 'code' argument".into()))?;

            let timeout = arguments
                .get("timeout")
                .and_then(|t| t.as_u64())
                .unwrap_or(30);

            let result = match language {
                "python" => execute_python(code, timeout).await,
                "node" => execute_nodejs(code, timeout).await,
                "rust" => execute_rust(code, timeout).await,
                "bash" => execute_bash(code, timeout).await,
                "ruby" => execute_ruby(code, timeout).await,
                "php" => execute_php(code, timeout).await,
                "go" => execute_go(code, timeout).await,
                _ => Err(FerroError::Tool(format!(
                    "Unsupported language: {}. Supported: python, node, rust, bash, ruby, php, go",
                    language
                ))),
            }?;

            Ok(crate::types::ToolResult {
                call_id: call_id.to_string(),
                content: result,
                is_error: false,
            })
        })
    }
}

async fn execute_python(code: &str, timeout: u64) -> Result<String, FerroError> {
    use tokio::process::Command;
    use tokio::time::{Duration, timeout as tokio_timeout};

    let child = Command::new("python3")
        .arg("-c")
        .arg(code)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| FerroError::Tool(format!("Failed to execute Python: {}", e)))?;

    let output = tokio_timeout(Duration::from_secs(timeout), child.wait_with_output())
        .await
        .map_err(|_| FerroError::Tool(format!("Python execution timed out after {}s", timeout)))?
        .map_err(|e| FerroError::Tool(format!("Python execution failed: {}", e)))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    let mut result = String::new();
    result.push_str("🐍 Python Execution Result\n");
    result.push_str("═════════════════════════════════════════\n\n");

    if output.status.success() {
        result.push_str("✅ Exit Status: Success\n\n");
    } else {
        result.push_str(&format!(
            "❌ Exit Status: Failed (code: {})\n\n",
            output.status.code().unwrap_or(-1)
        ));
    }

    if !stdout.is_empty() {
        result.push_str("📤 Output:\n");
        result.push_str(&format!("{}\n\n", stdout));
    }

    if !stderr.is_empty() {
        result.push_str("⚠️  Errors:\n");
        result.push_str(&format!("{}\n\n", stderr));
    }

    Ok(result)
}

async fn execute_nodejs(code: &str, timeout: u64) -> Result<String, FerroError> {
    use tokio::process::Command;
    use tokio::time::{Duration, timeout as tokio_timeout};

    let child = Command::new("node")
        .arg("-e")
        .arg(code)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| FerroError::Tool(format!("Failed to execute Node.js: {}", e)))?;

    let output = tokio_timeout(Duration::from_secs(timeout), child.wait_with_output())
        .await
        .map_err(|_| FerroError::Tool(format!("Node.js execution timed out after {}s", timeout)))?
        .map_err(|e| FerroError::Tool(format!("Node.js execution failed: {}", e)))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    let mut result = String::new();
    result.push_str("📜 Node.js Execution Result\n");
    result.push_str("═════════════════════════════════════════\n\n");

    if output.status.success() {
        result.push_str("✅ Exit Status: Success\n\n");
    } else {
        result.push_str(&format!(
            "❌ Exit Status: Failed (code: {})\n\n",
            output.status.code().unwrap_or(-1)
        ));
    }

    if !stdout.is_empty() {
        result.push_str("📤 Output:\n");
        result.push_str(&format!("{}\n\n", stdout));
    }

    if !stderr.is_empty() {
        result.push_str("⚠️  Errors:\n");
        result.push_str(&format!("{}\n\n", stderr));
    }

    Ok(result)
}

async fn execute_rust(code: &str, timeout: u64) -> Result<String, FerroError> {
    use std::io::Write;
    use tempfile::NamedTempFile;
    use tokio::process::Command;
    use tokio::time::{Duration, timeout as tokio_timeout};

    // Write code to temp file
    let mut temp_file = NamedTempFile::new()
        .map_err(|e| FerroError::Tool(format!("Failed to create temp file: {}", e)))?;

    let rust_code = format!(
        "
fn main() {{
    {}
}}
",
        code
    );

    temp_file
        .write_all(rust_code.as_bytes())
        .map_err(|e| FerroError::Tool(format!("Failed to write temp file: {}", e)))?;

    // Compile and run
    let child = Command::new("rustc")
        .arg(temp_file.path())
        .arg("-o")
        .arg("/tmp/temp_exec")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| FerroError::Tool(format!("Failed to compile Rust: {}", e)))?;

    let compile_output = tokio_timeout(Duration::from_secs(timeout), child.wait_with_output())
        .await
        .map_err(|_| FerroError::Tool(format!("Rust compilation timed out after {}s", timeout)))?
        .map_err(|e| FerroError::Tool(format!("Rust compilation failed: {}", e)))?;

    if !compile_output.status.success() {
        let stderr = String::from_utf8_lossy(&compile_output.stderr);
        return Ok(format!(
            "🦀 Rust Compilation Failed\n\
             ══════════════════════════════════════\n\n\
              ❌ Exit Status: Failed\n\n\
              ⚠️  Errors:\n\
              {}\n",
            stderr
        ));
    }

    // Run the compiled binary
    let child = Command::new("/tmp/temp_exec")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| FerroError::Tool(format!("Failed to execute Rust binary: {}", e)))?;

    let output = tokio_timeout(Duration::from_secs(timeout), child.wait_with_output())
        .await
        .map_err(|_| FerroError::Tool(format!("Rust execution timed out after {}s", timeout)))?
        .map_err(|e| FerroError::Tool(format!("Rust execution failed: {}", e)))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    let mut result = String::new();
    result.push_str("🦀 Rust Execution Result\n");
    result.push_str("═════════════════════════════════════════\n\n");

    if output.status.success() {
        result.push_str("✅ Exit Status: Success\n\n");
    } else {
        result.push_str(&format!(
            "❌ Exit Status: Failed (code: {})\n\n",
            output.status.code().unwrap_or(-1)
        ));
    }

    if !stdout.is_empty() {
        result.push_str("📤 Output:\n");
        result.push_str(&format!("{}\n\n", stdout));
    }

    if !stderr.is_empty() {
        result.push_str("⚠️  Errors:\n");
        result.push_str(&format!("{}\n\n", stderr));
    }

    Ok(result)
}

async fn execute_bash(code: &str, timeout: u64) -> Result<String, FerroError> {
    use tokio::process::Command;
    use tokio::time::{Duration, timeout as tokio_timeout};

    let child = Command::new("bash")
        .arg("-c")
        .arg(code)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| FerroError::Tool(format!("Failed to execute bash: {}", e)))?;

    let output = tokio_timeout(Duration::from_secs(timeout), child.wait_with_output())
        .await
        .map_err(|_| FerroError::Tool(format!("Bash execution timed out after {}s", timeout)))?
        .map_err(|e| FerroError::Tool(format!("Bash execution failed: {}", e)))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    let mut result = String::new();
    result.push_str("💻 Bash Execution Result\n");
    result.push_str("═════════════════════════════════════════\n\n");

    if output.status.success() {
        result.push_str("✅ Exit Status: Success\n\n");
    } else {
        result.push_str(&format!(
            "❌ Exit Status: Failed (code: {})\n\n",
            output.status.code().unwrap_or(-1)
        ));
    }

    if !stdout.is_empty() {
        result.push_str("📤 Output:\n");
        result.push_str(&format!("{}\n\n", stdout));
    }

    if !stderr.is_empty() {
        result.push_str("⚠️  Errors:\n");
        result.push_str(&format!("{}\n\n", stderr));
    }

    Ok(result)
}

async fn execute_ruby(code: &str, timeout: u64) -> Result<String, FerroError> {
    use tokio::process::Command;
    use tokio::time::{Duration, timeout as tokio_timeout};

    let child = Command::new("ruby")
        .arg("-e")
        .arg(code)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| FerroError::Tool(format!("Failed to execute Ruby: {}", e)))?;

    let output = tokio_timeout(Duration::from_secs(timeout), child.wait_with_output())
        .await
        .map_err(|_| FerroError::Tool(format!("Ruby execution timed out after {}s", timeout)))?
        .map_err(|e| FerroError::Tool(format!("Ruby execution failed: {}", e)))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    let mut result = String::new();
    result.push_str("💎 Ruby Execution Result\n");
    result.push_str("═════════════════════════════════════════\n\n");

    if output.status.success() {
        result.push_str("✅ Exit Status: Success\n\n");
    } else {
        result.push_str(&format!(
            "❌ Exit Status: Failed (code: {})\n\n",
            output.status.code().unwrap_or(-1)
        ));
    }

    if !stdout.is_empty() {
        result.push_str("📤 Output:\n");
        result.push_str(&format!("{}\n\n", stdout));
    }

    if !stderr.is_empty() {
        result.push_str("⚠️  Errors:\n");
        result.push_str(&format!("{}\n\n", stderr));
    }

    Ok(result)
}

async fn execute_php(code: &str, timeout: u64) -> Result<String, FerroError> {
    use tokio::process::Command;
    use tokio::time::{Duration, timeout as tokio_timeout};

    let child = Command::new("php")
        .arg("-r")
        .arg(code)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| FerroError::Tool(format!("Failed to execute PHP: {}", e)))?;

    let output = tokio_timeout(Duration::from_secs(timeout), child.wait_with_output())
        .await
        .map_err(|_| FerroError::Tool(format!("PHP execution timed out after {}s", timeout)))?
        .map_err(|e| FerroError::Tool(format!("PHP execution failed: {}", e)))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    let mut result = String::new();
    result.push_str("🐘 PHP Execution Result\n");
    result.push_str("═════════════════════════════════════════\n\n");

    if output.status.success() {
        result.push_str("✅ Exit Status: Success\n\n");
    } else {
        result.push_str(&format!(
            "❌ Exit Status: Failed (code: {})\n\n",
            output.status.code().unwrap_or(-1)
        ));
    }

    if !stdout.is_empty() {
        result.push_str("📤 Output:\n");
        result.push_str(&format!("{}\n\n", stdout));
    }

    if !stderr.is_empty() {
        result.push_str("⚠️  Errors:\n");
        result.push_str(&format!("{}\n\n", stderr));
    }

    Ok(result)
}

async fn execute_go(code: &str, timeout: u64) -> Result<String, FerroError> {
    use std::io::Write;
    use tempfile::NamedTempFile;
    use tokio::process::Command;
    use tokio::time::{Duration, timeout as tokio_timeout};

    // Write code to temp file
    let mut temp_file = NamedTempFile::new()
        .map_err(|e| FerroError::Tool(format!("Failed to create temp file: {}", e)))?;

    let go_code = format!(
        r#"package main

import "fmt"

func main() {{
    {}
}}
"#,
        code
    );

    temp_file
        .write_all(go_code.as_bytes())
        .map_err(|e| FerroError::Tool(format!("Failed to write temp file: {}", e)))?;

    // Compile and run
    let child = Command::new("go")
        .arg("run")
        .arg(temp_file.path())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| FerroError::Tool(format!("Failed to execute Go: {}", e)))?;

    let output = tokio_timeout(Duration::from_secs(timeout), child.wait_with_output())
        .await
        .map_err(|_| FerroError::Tool(format!("Go execution timed out after {}s", timeout)))?
        .map_err(|e| FerroError::Tool(format!("Go execution failed: {}", e)))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    let mut result = String::new();
    result.push_str("🐹 Go Execution Result\n");
    result.push_str("═════════════════════════════════════════\n\n");

    if output.status.success() {
        result.push_str("✅ Exit Status: Success\n\n");
    } else {
        result.push_str(&format!(
            "❌ Exit Status: Failed (code: {})\n\n",
            output.status.code().unwrap_or(-1)
        ));
    }

    if !stdout.is_empty() {
        result.push_str("📤 Output:\n");
        result.push_str(&format!("{}\n\n", stdout));
    }

    if !stderr.is_empty() {
        result.push_str("⚠️  Errors:\n");
        result.push_str(&format!("{}\n\n", stderr));
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_execute_python() {
        let code = "print('Hello, World!')";
        let result = execute_python(code, 5).await.unwrap();
        assert!(result.contains("Python Execution Result"));
        assert!(result.contains("Hello, World!"));
    }

    #[tokio::test]
    async fn test_execute_bash() {
        let code = "echo 'Hello from bash'";
        let result = execute_bash(code, 5).await.unwrap();
        assert!(result.contains("Bash Execution Result"));
        assert!(result.contains("Hello from bash"));
    }
}
