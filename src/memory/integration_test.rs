//! Integration tests for MemdirSystem

use ferroclaw::memory::memdir::{Memdir, MAX_ENTRYPOINT_BYTES, MAX_ENTRYPOINT_LINES};
use tempfile::TempDir;

#[test]
fn test_memdir_integration() {
    let temp_dir = TempDir::new().unwrap();
    let memdir = Memdir::with_path(temp_dir.path().to_path_buf());

    // Test basic operations
    memdir.ensure_dir_exists().unwrap();

    // Write a topic file
    memdir.write_topic_file("test", "# Test\n\nContent").unwrap();
    assert!(memdir.topic_file_exists("test"));

    // Load it back
    let content = memdir.load_topic_file("test").unwrap();
    assert!(content.contains("Test"));

    // List topics
    let topics = memdir.list_topic_files().unwrap();
    assert_eq!(topics, vec!["test".to_string()]);

    // Clean up
    assert!(memdir.delete_topic_file("test").unwrap());
    assert!(!memdir.topic_file_exists("test"));
}

#[test]
fn test_memory_prompt_generation() {
    let temp_dir = TempDir::new().unwrap();
    let memdir = Memdir::with_path(temp_dir.path().to_path_buf());

    // Generate prompt for empty memory
    let prompt = memdir.load_memory_prompt().unwrap();
    assert!(prompt.contains("# auto memory"));
    assert!(prompt.contains("currently empty"));

    // Write MEMORY.md
    let entrypoint = "- [Test](test.md) — Test entry";
    std::fs::write(memdir.entrypoint_path(), entrypoint).unwrap();

    // Generate prompt with content
    let prompt = memdir.load_memory_prompt().unwrap();
    assert!(prompt.contains("Test entry"));
}

#[test]
fn test_truncation_limits() {
    let temp_dir = TempDir::new().unwrap();
    let memdir = Memdir::with_path(temp_dir.path().to_path_buf());

    // Test line limit
    let mut long_content = String::new();
    for i in 1..=250 {
        long_content.push_str(&format!("Line {}\n", i));
    }

    let result = memdir.truncate_entrypoint(&long_content);
    assert!(result.was_line_truncated);
    assert_eq!(result.line_count, 250);
    assert!(result.content.contains("WARNING"));

    // Test byte limit
    let mut huge_line = String::new();
    for i in 1..=150 {
        huge_line.push_str(&format!("Line {}: {}\n", i, "x".repeat(200)));
    }

    let result = memdir.truncate_entrypoint(&huge_line);
    assert!(result.was_byte_truncated);
    assert!(result.content.contains("WARNING"));
}
