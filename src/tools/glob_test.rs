//! Tests for GlobTool

use super::*;
use tempfile::TempDir;
use tokio::fs;
use tokio::io::AsyncWriteExt;

/// Helper to create a test directory structure
async fn setup_test_dir() -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    let base = temp_dir.path();

    // Create directory structure:
    // .
    // ├── src/
    // │   ├── main.rs
    // │   ├── utils.rs
    // │   └── tests/
    // │           └── test_utils.rs
    // ├── tests/
    // │     └── integration_test.rs
    // ├── .git/
    // │     └── config
    // ├── README.md
    // └── main.rs

    fs::create_dir_all(base.join("src/tests")).await.unwrap();
    fs::create_dir_all(base.join("tests")).await.unwrap();
    fs::create_dir_all(base.join(".git")).await.unwrap();

    // Create files
    let mut file = fs::File::create(base.join("src/main.rs")).await.unwrap();
    file.write_all(b"fn main() {}").await.unwrap();

    let mut file = fs::File::create(base.join("src/utils.rs")).await.unwrap();
    file.write_all(b"pub fn utils() {}").await.unwrap();

    let mut file = fs::File::create(base.join("src/tests/test_utils.rs")).await.unwrap();
    file.write_all(b"pub fn test_utils() {}").await.unwrap();

    let mut file = fs::File::create(base.join("tests/integration_test.rs")).await.unwrap();
    file.write_all(b"pub fn integration_test() {}").await.unwrap();

    let mut file = fs::File::create(base.join(".git/config")).await.unwrap();
    file.write_all(b"[core]").await.unwrap();

    let mut file = fs::File::create(base.join("README.md")).await.unwrap();
    file.write_all(b"# Test Project").await.unwrap();

    let mut file = fs::File::create(base.join("main.rs")).await.unwrap();
    file.write_all(b"fn main() {}").await.unwrap();

    temp_dir
}

#[tokio::test]
async fn test_basic_pattern() {
    let temp_dir = setup_test_dir().await;
    let tool = GlobTool::new();

    let args = json!({
        "pattern": "*.rs",
        "path": temp_dir.path().to_str()
    });

    let result = tool.call("test-call", &args).await.unwrap();
    assert!(!result.is_error);
    assert!(result.content.contains("main.rs"));
}

#[tokio::test]
async fn test_recursive_pattern() {
    let temp_dir = setup_test_dir().await;
    let tool = GlobTool::new();

    let args = json!({
        "pattern": "**/*.rs",
        "path": temp_dir.path().to_str()
    });

    let result = tool.call("test-call", &args).await.unwrap();
    assert!(!result.is_error);
    assert!(result.content.contains("main.rs"));
    assert!(result.content.contains("src/main.rs"));
    assert!(result.content.contains("src/utils.rs"));
    assert!(result.content.contains("src/tests/test_utils.rs"));
}

#[tokio::test]
async fn test_directory_specific_pattern() {
    let temp_dir = setup_test_dir().await;
    let tool = GlobTool::new();

    let args = json!({
        "pattern": "src/**/*.rs",
        "path": temp_dir.path().to_str()
    });

    let result = tool.call("test-call", &args).await.unwrap();
    assert!(!result.is_error);
    assert!(result.content.contains("src/main.rs"));
    assert!(result.content.contains("src/utils.rs"));
    assert!(result.content.contains("src/tests/test_utils.rs"));
    // Should not contain files in tests/ directory
    assert!(!result.content.contains("tests/integration_test.rs"));
}

#[tokio::test]
async fn test_hidden_files_excluded() {
    let temp_dir = setup_test_dir().await;
    let tool = GlobTool::new();

    let args = json!({
        "pattern": "**/*",
        "path": temp_dir.path().to_str()
    });

    let result = tool.call("test-call", &args).await.unwrap();
    assert!(!result.is_error);
    // .git is a hidden directory, should be excluded by default
    assert!(!result.content.contains(".git"));
}

#[tokio::test]
async fn test_hidden_files_included_with_pattern() {
    let temp_dir = setup_test_dir().await;
    let tool = GlobTool::new();

    let args = json!({
        "pattern": "**/.*",
        "path": temp_dir.path().to_str()
    });

    let result = tool.call("test-call", &args).await.unwrap();
    assert!(!result.is_error);
    // Should find .git when explicitly looking for dotfiles
    assert!(result.content.contains(".git"));
}

#[tokio::test]
async fn test_non_matching_pattern() {
    let temp_dir = setup_test_dir().await;
    let tool = GlobTool::new();

    let args = json!({
        "pattern": "*.nonexistent",
        "path": temp_dir.path().to_str()
    });

    let result = tool.call("test-call", &args).await.unwrap();
    assert!(!result.is_error);
    assert!(result.content.contains("No files found"));
}

#[tokio::test]
async fn test_markdown_files() {
    let temp_dir = setup_test_dir().await;
    let tool = GlobTool::new();

    let args = json!({
        "pattern": "*.md",
        "path": temp_dir.path().to_str()
    });

    let result = tool.call("test-call", &args).await.unwrap();
    assert!(!result.is_error);
    assert!(result.content.contains("README.md"));
}

#[tokio::test]
async fn test_wildcard_in_directory() {
    let temp_dir = setup_test_dir().await;
    let tool = GlobTool::new();

    let args = json!({
        "pattern": "src/*.rs",
        "path": temp_dir.path().to_str()
    });

    let result = tool.call("test-call", &args).await.unwrap();
    assert!(!result.is_error);
    assert!(result.content.contains("src/main.rs"));
    assert!(result.content.contains("src/utils.rs"));
    // Should not contain files in subdirectories
    assert!(!result.content.contains("src/tests/test_utils.rs"));
}

#[tokio::test]
async fn test_cache_functionality() {
    let temp_dir = setup_test_dir().await;
    let tool = GlobTool::new();

    let args = json!({
        "pattern": "*.rs",
        "path": temp_dir.path().to_str()
    });

    // First call should populate cache
    let result1 = tool.call("test-call", &args).await.unwrap();
    assert!(!result1.is_error);

    // Second call should use cache
    let result2 = tool.call("test-call", &args).await.unwrap();
    assert!(!result2.is_error);

    // Results should be identical
    assert_eq!(result1.content, result2.content);
}

#[tokio::test]
async fn test_default_to_current_directory() {
    let temp_dir = setup_test_dir().await;
    let tool = GlobTool::new();

    // Change to temp directory
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let args = json!({
        "pattern": "*.md"
        // No path specified - should use current directory
    });

    let result = tool.call("test-call", &args).await.unwrap();
    assert!(!result.is_error);
    assert!(result.content.contains("README.md"));
}

#[tokio::test]
async fn test_nested_directories() {
    let temp_dir = setup_test_dir().await;
    let tool = GlobTool::new();

    let args = json!({
        "pattern": "**/test_*.rs",
        "path": temp_dir.path().to_str()
    });

    let result = tool.call("test-call", &args).await.unwrap();
    assert!(!result.is_error);
    assert!(result.content.contains("test_utils.rs"));
}

#[tokio::test]
async fn test_error_invalid_path() {
    let tool = GlobTool::new();

    let args = json!({
        "pattern": "*.rs",
        "path": "/nonexistent/path/that/does/not/exist"
    });

    let result = tool.call("test-call", &args).await.unwrap();
    assert!(result.is_error);
    assert!(result.content.contains("does not exist"));
}

#[tokio::test]
async fn test_missing_pattern_argument() {
    let tool = GlobTool::new();

    let args = json!({
        "path": "/some/path"
    });

    let result = tool.call("test-call", &args).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), FerroError::Tool(_)));
}

#[tokio::test]
async fn test_double_star_recursive() {
    let temp_dir = setup_test_dir().await;
    let tool = GlobTool::new();

    // Create deeper nesting
    fs::create_dir_all(temp_dir.path().join("a/b/c")).await.unwrap();
    let mut file = fs::File::create(temp_dir.path().join("a/b/c/deep.rs")).await.unwrap();
    file.write_all(b"// deep file").await.unwrap();

    let args = json!({
        "pattern": "**/*.rs",
        "path": temp_dir.path().to_str()
    });

    let result = tool.call("test-call", &args).await.unwrap();
    assert!(!result.is_error);
    assert!(result.content.contains("deep.rs"));
}

#[tokio::test]
async fn test_pattern_with_extension_wildcard() {
    let temp_dir = setup_test_dir().await;
    let tool = GlobTool::new();

    let args = json!({
        "pattern": "src/*.*",
        "path": temp_dir.path().to_str()
    });

    let result = tool.call("test-call", &args).await.unwrap();
    assert!(!result.is_error);
    assert!(result.content.contains("src/main.rs"));
    assert!(result.content.contains("src/utils.rs"));
}

#[tokio::test]
async fn test_truncation() {
    let temp_dir = TempDir::new().unwrap();
    let tool = GlobTool::new();

    // Create many files to test truncation
    for i in 0..150 {
        let file_path = temp_dir.path().join(format!("file_{}.txt", i));
        let mut file = fs::File::create(&file_path).await.unwrap();
        file.write_all(b"test").await.unwrap();
    }

    let args = json!({
        "pattern": "*.txt",
        "path": temp_dir.path().to_str()
    });

    let result = tool.call("test-call", &args).await.unwrap();
    assert!(!result.is_error);
    // Should have truncation message
    assert!(result.content.contains("truncated"));
}

#[tokio::test]
async fn test_empty_directory() {
    let temp_dir = TempDir::new().unwrap();
    let tool = GlobTool::new();

    let args = json!({
        "pattern": "*.rs",
        "path": temp_dir.path().to_str()
    });

    let result = tool.call("test-call", &args).await.unwrap();
    assert!(!result.is_error);
    assert!(result.content.contains("No files found"));
}

#[tokio::test]
async fn test_multiple_extensions() {
    let temp_dir = setup_test_dir().await;
    let tool = GlobTool::new();

    // Create files with different extensions
    let mut file = fs::File::create(temp_dir.path().join("test.json")).await.unwrap();
    file.write_all(b"{}").await.unwrap();

    let mut file = fs::File::create(temp_dir.path().join("test.toml")).await.unwrap();
    file.write_all(b"").await.unwrap();

    let args = json!({
        "pattern": "*.json",
        "path": temp_dir.path().to_str()
    });

    let result = tool.call("test-call", &args).await.unwrap();
    assert!(!result.is_error);
    assert!(result.content.contains("test.json"));
    assert!(!result.content.contains("test.toml"));
}

#[tokio::test]
async fn test_glob_tool_meta() {
    let meta = glob_tool_meta();

    assert_eq!(meta.definition.name, "glob");
    assert!(meta.definition.description.contains("glob"));
    assert_eq!(meta.definition.input_schema["type"], "object");
    assert_eq!(meta.definition.input_schema["required"], json!(["pattern"]));
    assert!(matches!(meta.source, ToolSource::Builtin));
}
