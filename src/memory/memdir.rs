//! Memory directory system for file-based persistent memory.
//!
//! Implements Claude Code's memory directory system with:
//! - MEMORY.md entry point with truncation limits
//! - Topic file organization ([topic].md files)
//! - Automatic truncation at 200 lines or 25KB
//! - Formatted memory prompt generation

use crate::config::data_dir;
use crate::error::{FerroError, Result};
use std::fs;
use std::path::PathBuf;

/// Maximum number of lines in MEMORY.md entry point
pub const MAX_ENTRYPOINT_LINES: usize = 200;

/// Maximum bytes in MEMORY.md entry point (~25KB)
pub const MAX_ENTRYPOINT_BYTES: usize = 25_000;

/// Memory directory entry point filename
pub const ENTRYPOINT_NAME: &str = "MEMORY.md";

/// Result of truncating MEMORY.md content
#[derive(Debug, Clone)]
pub struct EntrypointTruncation {
    /// The truncated (or original) content
    pub content: String,
    /// Number of lines in the content
    pub line_count: usize,
    /// Number of bytes in the content
    pub byte_count: usize,
    /// Whether line limit was exceeded
    pub was_line_truncated: bool,
    /// Whether byte limit was exceeded
    pub was_byte_truncated: bool,
}

/// Memory directory system
#[derive(Debug, Clone)]
pub struct Memdir {
    /// Path to the memory directory
    memory_dir: PathBuf,
}

impl Memdir {
    /// Create a new Memdir instance with the default path
    pub fn new() -> Result<Self> {
        let memory_dir = data_dir().join("memory");
        Ok(Self { memory_dir })
    }

    /// Create a new Memdir instance with a custom path
    pub fn with_path(path: PathBuf) -> Self {
        Self { memory_dir: path }
    }

    /// Get the path to the memory directory
    pub fn memory_dir(&self) -> &PathBuf {
        &self.memory_dir
    }

    /// Get the path to MEMORY.md entry point
    pub fn entrypoint_path(&self) -> PathBuf {
        self.memory_dir.join(ENTRYPOINT_NAME)
    }

    /// Ensure the memory directory exists
    pub fn ensure_dir_exists(&self) -> Result<()> {
        fs::create_dir_all(&self.memory_dir)
            .map_err(|e| FerroError::Memory(format!("Failed to create memory directory: {}", e)))?;
        Ok(())
    }

    /// Truncate MEMORY.md content to line AND byte limits
    ///
    /// Line-truncates first (natural boundary), then byte-truncates at the last
    /// newline before the cap so we don't cut mid-line. Appends a warning
    /// message when truncation occurs.
    pub fn truncate_entrypoint(&self, raw: &str) -> EntrypointTruncation {
        let trimmed = raw.trim();
        let content_lines: Vec<&str> = trimmed.lines().collect();
        let line_count = content_lines.len();
        let byte_count = trimmed.len();

        let was_line_truncated = line_count > MAX_ENTRYPOINT_LINES;
        // Check original byte count — long lines are the failure mode the byte cap
        // targets, so post-line-truncation size would understate the warning.
        let was_byte_truncated = byte_count > MAX_ENTRYPOINT_BYTES;

        if !was_line_truncated && !was_byte_truncated {
            return EntrypointTruncation {
                content: trimmed.to_string(),
                line_count,
                byte_count,
                was_line_truncated,
                was_byte_truncated,
            };
        }

        // Apply line truncation first
        let truncated = if was_line_truncated {
            content_lines[..MAX_ENTRYPOINT_LINES].join("\n")
        } else {
            trimmed.to_string()
        };

        // Then apply byte truncation if needed
        let truncated = if truncated.len() > MAX_ENTRYPOINT_BYTES {
            // Find last newline before byte limit
            let cut_at = truncated[..MAX_ENTRYPOINT_BYTES]
                .rfind('\n')
                .unwrap_or(MAX_ENTRYPOINT_BYTES);
            if cut_at > 0 {
                truncated[..cut_at].to_string()
            } else {
                truncated[..MAX_ENTRYPOINT_BYTES].to_string()
            }
        } else {
            truncated
        };

        // Build warning message
        let reason = if was_byte_truncated && !was_line_truncated {
            format!(
                "{} bytes (limit: {} bytes) — index entries are too long",
                byte_count, MAX_ENTRYPOINT_BYTES
            )
        } else if was_line_truncated && !was_byte_truncated {
            format!("{} lines (limit: {})", line_count, MAX_ENTRYPOINT_LINES)
        } else {
            format!("{} lines and {} bytes", line_count, byte_count)
        };

        let warning = format!(
            "\n\n> WARNING: {} is {}. Only part of it was loaded. Keep index entries to one line under ~200 chars; move detail into topic files.",
            ENTRYPOINT_NAME, reason
        );

        EntrypointTruncation {
            content: truncated + &warning,
            line_count,
            byte_count,
            was_line_truncated,
            was_byte_truncated,
        }
    }

    /// Read the MEMORY.md entry point file
    pub fn read_entrypoint(&self) -> Result<String> {
        let path = self.entrypoint_path();
        fs::read_to_string(&path).map_err(|e| {
            FerroError::Memory(format!(
                "Failed to read MEMORY.md at {}: {}",
                path.display(),
                e
            ))
        })
    }

    /// Read and truncate the MEMORY.md entry point
    pub fn read_entrypoint_truncated(&self) -> Result<EntrypointTruncation> {
        let content = self.read_entrypoint()?;
        Ok(self.truncate_entrypoint(&content))
    }

    /// Load a topic file by name (e.g., "user_role" loads user_role.md)
    pub fn load_topic_file(&self, topic_name: &str) -> Result<String> {
        let file_path = self.memory_dir.join(format!("{}.md", topic_name));
        fs::read_to_string(&file_path).map_err(|e| {
            FerroError::Memory(format!(
                "Failed to read topic file {}: {}",
                file_path.display(),
                e
            ))
        })
    }

    /// List all topic files in the memory directory
    pub fn list_topic_files(&self) -> Result<Vec<String>> {
        let entries = fs::read_dir(&self.memory_dir)
            .map_err(|e| FerroError::Memory(format!("Failed to read memory directory: {}", e)))?;

        let mut topics = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|e| {
                FerroError::Memory(format!("Failed to read directory entry: {}", e))
            })?;

            let path = entry.path();
            if path.is_file() {
                let file_name = path.file_name().and_then(|n| n.to_str());
                if let Some(name) = file_name {
                    // Skip MEMORY.md and hidden files
                    if name != ENTRYPOINT_NAME && !name.starts_with('.') {
                        // Remove .md extension
                        if let Some(topic) = name.strip_suffix(".md") {
                            topics.push(topic.to_string());
                        }
                    }
                }
            }
        }

        topics.sort();
        Ok(topics)
    }

    /// Write a topic file
    pub fn write_topic_file(&self, topic_name: &str, content: &str) -> Result<()> {
        self.ensure_dir_exists()?;
        let file_path = self.memory_dir.join(format!("{}.md", topic_name));
        fs::write(&file_path, content).map_err(|e| {
            FerroError::Memory(format!(
                "Failed to write topic file {}: {}",
                file_path.display(),
                e
            ))
        })
    }

    /// Delete a topic file
    pub fn delete_topic_file(&self, topic_name: &str) -> Result<bool> {
        let file_path = self.memory_dir.join(format!("{}.md", topic_name));
        if !file_path.exists() {
            return Ok(false);
        }

        fs::remove_file(&file_path).map_err(|e| {
            FerroError::Memory(format!(
                "Failed to delete topic file {}: {}",
                file_path.display(),
                e
            ))
        })?;
        Ok(true)
    }

    /// Load the memory prompt for inclusion in system context
    ///
    /// This reads MEMORY.md, applies truncation limits, and formats the content
    /// for inclusion in the agent's system prompt.
    pub fn load_memory_prompt(&self) -> Result<String> {
        self.ensure_dir_exists()?;

        let mut lines = vec![
            "# auto memory".to_string(),
            "".to_string(),
            format!(
                "You have a persistent, file-based memory system at: `{}`",
                self.memory_dir.display()
            ),
            "".to_string(),
            "This directory already exists — write to it directly with the Write tool (do not run mkdir or check for its existence).".to_string(),
            "".to_string(),
            "You should build up this memory system over time so that future conversations can have a complete picture of who the user is, how they'd like to collaborate with you, what behaviors to avoid or repeat, and the context behind the work the user gives you.".to_string(),
            "".to_string(),
            "If the user explicitly asks you to remember something, save it immediately. If they ask you to forget something, find and remove the relevant entry.".to_string(),
            "".to_string(),
            "## How to save memories".to_string(),
            "".to_string(),
            "Saving a memory is a two-step process:".to_string(),
            "".to_string(),
            format!(
                "**Step 1** — write the memory to its own file (e.g., `user_role.md`, `feedback_testing.md`).",
            ),
            "".to_string(),
            format!(
                "**Step 2** — add a pointer to that file in `{}`. `{}` is an index, not a memory — each entry should be one line, under ~150 characters: `- [Title](file.md) — one-line hook`. Never write memory content directly into `{}`.",
                ENTRYPOINT_NAME, ENTRYPOINT_NAME, ENTRYPOINT_NAME
            ),
            "".to_string(),
            format!(
                "- `{}` is always loaded into your conversation context — lines after {} will be truncated, so keep the index concise",
                ENTRYPOINT_NAME, MAX_ENTRYPOINT_LINES
            ),
            "- Keep the name, description, and type fields in memory files up-to-date with the content".to_string(),
            "- Organize memory semantically by topic, not chronologically".to_string(),
            "- Update or remove memories that turn out to be wrong or outdated".to_string(),
            "- Do not write duplicate memories. First check if there is an existing memory you can update before writing a new one.".to_string(),
            "".to_string(),
            format!("## {}", ENTRYPOINT_NAME).to_string(),
            "".to_string(),
        ];

        // Try to read and truncate MEMORY.md
        match self.read_entrypoint_truncated() {
            Ok(truncation) => {
                lines.push(truncation.content);
            }
            Err(_) => {
                // File doesn't exist yet
                lines.push(format!(
                    "Your `{}` is currently empty. When you save new memories, they will appear here.",
                    ENTRYPOINT_NAME
                ));
            }
        }

        Ok(lines.join("\n"))
    }

    /// Check if a topic file exists
    pub fn topic_file_exists(&self, topic_name: &str) -> bool {
        let file_path = self.memory_dir.join(format!("{}.md", topic_name));
        file_path.exists()
    }

    /// Get the full path to a topic file
    pub fn topic_file_path(&self, topic_name: &str) -> PathBuf {
        self.memory_dir.join(format!("{}.md", topic_name))
    }
}

impl Default for Memdir {
    fn default() -> Self {
        Self::new().expect("Failed to create default Memdir")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_memdir() -> (Memdir, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let memdir = Memdir::with_path(temp_dir.path().to_path_buf());
        (memdir, temp_dir)
    }

    #[test]
    fn test_truncate_within_limits() {
        let (memdir, _temp) = create_test_memdir();
        let content = "Line 1\nLine 2\nLine 3";
        let result = memdir.truncate_entrypoint(content);

        assert!(!result.was_line_truncated);
        assert!(!result.was_byte_truncated);
        assert_eq!(result.line_count, 3);
        assert_eq!(result.content, content);
    }

    #[test]
    fn test_truncate_line_limit() {
        let (memdir, _temp) = create_test_memdir();
        let mut content = String::new();
        for i in 1..=250 {
            content.push_str(&format!("Line {}\n", i));
        }

        let result = memdir.truncate_entrypoint(&content);

        assert!(result.was_line_truncated);
        assert!(!result.was_byte_truncated);
        assert_eq!(result.line_count, 250);
        assert!(result.content.contains("WARNING"));
        assert!(
            result
                .content
                .contains(&format!("limit: {}", MAX_ENTRYPOINT_LINES))
        );
    }

    #[test]
    fn test_truncate_byte_limit() {
        let (memdir, _temp) = create_test_memdir();
        // Create content with very long lines that exceeds byte limit but not line limit
        let mut content = String::new();
        for i in 1..=150 {
            content.push_str(&format!("Line {}: {}\n", i, "x".repeat(200)));
        }

        let result = memdir.truncate_entrypoint(&content);

        assert!(!result.was_line_truncated);
        assert!(result.was_byte_truncated);
        assert!(result.content.contains("WARNING"));
        assert!(result.content.contains("bytes"));
    }

    #[test]
    fn test_truncate_both_limits() {
        let (memdir, _temp) = create_test_memdir();
        let mut content = String::new();
        for i in 1..=300 {
            content.push_str(&format!("Line {}: {}\n", i, "x".repeat(200)));
        }

        let result = memdir.truncate_entrypoint(&content);

        assert!(result.was_line_truncated);
        assert!(result.was_byte_truncated);
        assert!(result.content.contains("WARNING"));
        assert!(result.content.contains("lines and"));
    }

    #[test]
    fn test_write_and_read_entrypoint() {
        let (memdir, _temp) = create_test_memdir();
        memdir.ensure_dir_exists().unwrap();

        let content = "# Memory\n\n- [Topic](topic.md) — Description";
        fs::write(memdir.entrypoint_path(), content).unwrap();

        let read = memdir.read_entrypoint().unwrap();
        assert_eq!(read, content);
    }

    #[test]
    fn test_topic_file_operations() {
        let (memdir, _temp) = create_test_memdir();

        // Write topic file
        let content = "# User Role\n\nDeveloper";
        memdir.write_topic_file("user_role", content).unwrap();

        // Check exists
        assert!(memdir.topic_file_exists("user_role"));

        // Read topic file
        let read = memdir.load_topic_file("user_role").unwrap();
        assert_eq!(read, content);

        // List topic files
        let topics = memdir.list_topic_files().unwrap();
        assert_eq!(topics, vec!["user_role".to_string()]);

        // Delete topic file
        assert!(memdir.delete_topic_file("user_role").unwrap());
        assert!(!memdir.topic_file_exists("user_role"));

        // Delete again returns false
        assert!(!memdir.delete_topic_file("user_role").unwrap());
    }

    #[test]
    fn test_list_topic_files_excludes_entrypoint() {
        let (memdir, _temp) = create_test_memdir();
        memdir.ensure_dir_exists().unwrap();

        // Create entrypoint
        fs::write(memdir.entrypoint_path(), "# Memory\n").unwrap();

        // Create topic files
        memdir.write_topic_file("topic1", "content1").unwrap();
        memdir.write_topic_file("topic2", "content2").unwrap();

        // List should only include topic files, not MEMORY.md
        let topics = memdir.list_topic_files().unwrap();
        assert_eq!(topics.len(), 2);
        assert!(topics.contains(&"topic1".to_string()));
        assert!(topics.contains(&"topic2".to_string()));
        assert!(!topics.contains(&"MEMORY".to_string()));
    }

    #[test]
    fn test_load_memory_prompt_empty() {
        let (memdir, _temp) = create_test_memdir();

        let prompt = memdir.load_memory_prompt().unwrap();

        assert!(prompt.contains("# auto memory"));
        assert!(prompt.contains("currently empty"));
        assert!(prompt.contains("How to save memories"));
    }

    #[test]
    fn test_load_memory_prompt_with_content() {
        let (memdir, _temp) = create_test_memdir();

        // Write MEMORY.md
        let entrypoint_content = "- [User Role](user_role.md) — Developer working on Ferroclaw\n\
            - [Project](project.md) — Rust-based AI agent framework";
        fs::write(memdir.entrypoint_path(), entrypoint_content).unwrap();

        let prompt = memdir.load_memory_prompt().unwrap();

        assert!(prompt.contains("# auto memory"));
        assert!(prompt.contains("User Role"));
        assert!(prompt.contains("Ferroclaw"));
        assert!(prompt.contains("## MEMORY.md"));
    }

    #[test]
    fn test_load_memory_prompt_with_truncation() {
        let (memdir, _temp) = create_test_memdir();

        // Write MEMORY.md that exceeds line limit
        let mut content = String::new();
        for i in 1..=250 {
            content.push_str(&format!(
                "- [Item {}](item{}.md) — Description {}\n",
                i, i, i
            ));
        }
        fs::write(memdir.entrypoint_path(), content).unwrap();

        let prompt = memdir.load_memory_prompt().unwrap();

        assert!(prompt.contains("WARNING"));
        assert!(prompt.contains(&format!("limit: {}", MAX_ENTRYPOINT_LINES)));
    }
}
