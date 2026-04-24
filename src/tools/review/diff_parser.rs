//! Diff parsing utilities for code review
//!
//! Parses unified diff format into structured hunks for analysis.

use crate::error::{FerroError, Result};
use regex_lite::Regex;
use std::path::Path;

/// Represents a single line in a diff
#[derive(Debug, Clone, PartialEq)]
pub struct DiffLine {
    pub line_type: DiffLineType,
    pub content: String,
    pub line_number: Option<usize>,
}

/// Type of diff line
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DiffLineType {
    Context,    // Unchanged line (space prefix)
    Added,      // Added line (+ prefix)
    Deleted,    // Deleted line (- prefix)
    Range,      // Hunk header (@@ prefix)
    Header,     // File header (---/+++ prefix)
}

/// Represents a hunk in a diff
#[derive(Debug, Clone)]
pub struct DiffHunk {
    pub file_path: String,
    pub old_start: usize,
    pub old_count: usize,
    pub new_start: usize,
    pub new_count: usize,
    pub lines: Vec<DiffLine>,
}

impl DiffHunk {
    /// Get only added lines from this hunk
    pub fn added_lines(&self) -> Vec<&DiffLine> {
        self.lines.iter()
            .filter(|line| line.line_type == DiffLineType::Added)
            .collect()
    }

    /// Get only deleted lines from this hunk
    pub fn deleted_lines(&self) -> Vec<&DiffLine> {
        self.lines.iter()
            .filter(|line| line.line_type == DiffLineType::Deleted)
            .collect()
    }

    /// Get the actual code content (without diff markers)
    pub fn code_content(&self) -> String {
        self.lines.iter()
            .filter(|line| line.line_type != DiffLineType::Range && line.line_type != DiffLineType::Header)
            .map(|line| line.content.as_str())
            .collect::<Vec<&str>>()
            .join("\n")
    }
}

/// Parser for unified diff format
pub struct DiffParser;

impl DiffParser {
    /// Parse diff text into structured hunks
    pub fn parse(diff_text: &str) -> Result<Vec<DiffHunk>> {
        let mut hunks = Vec::new();
        let lines: Vec<&str> = diff_text.lines().collect();
        let mut i = 0;

        // Regex patterns
        let old_file_re = Regex::new(r"^---\s+(?P<path>.+)").unwrap();
        let new_file_re = Regex::new(r"^\+\+\+\s+(?P<path>.+)").unwrap();
        let hunk_re = Regex::new(r"^@@\s+\-(?P<old_start>\d+)(?:,(?P<old_count>\d+))?\s+\+(?P<new_start>\d+)(?:,(?P<new_count>\d+))?\s+@@").unwrap();

        let mut current_file = String::new();
        let mut current_hunk: Option<DiffHunk> = None;

        while i < lines.len() {
            let line = lines[i];

            // Check for old file header
            if let Some(caps) = old_file_re.captures(line) {
                if let Some(hunk) = current_hunk.take() {
                    hunks.push(hunk);
                }
                current_file = caps.name("path")
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_else(|| "unknown".to_string());
                i += 1;
                continue;
            }

            // Check for new file header
            if let Some(caps) = new_file_re.captures(line) {
                current_file = caps.name("path")
                    .map(|m| {
                        let path = m.as_str();
                        // Remove a/ prefix if present
                        path.strip_prefix("a/").unwrap_or(path).to_string()
                    })
                    .unwrap_or_else(|| current_file.clone());
                i += 1;
                continue;
            }

            // Check for hunk header
            if let Some(caps) = hunk_re.captures(line) {
                if let Some(hunk) = current_hunk.take() {
                    hunks.push(hunk);
                }

                let old_start: usize = caps.name("old_start")
                    .and_then(|m| m.as_str().parse().ok())
                    .unwrap_or(1);
                let old_count: usize = caps.name("old_count")
                    .and_then(|m| m.as_str().parse().ok())
                    .unwrap_or(1);
                let new_start: usize = caps.name("new_start")
                    .and_then(|m| m.as_str().parse().ok())
                    .unwrap_or(1);
                let new_count: usize = caps.name("new_count")
                    .and_then(|m| m.as_str().parse().ok())
                    .unwrap_or(1);

                current_hunk = Some(DiffHunk {
                    file_path: current_file.clone(),
                    old_start,
                    old_count,
                    new_start,
                    new_count,
                    lines: Vec::new(),
                });

                // Add the range line itself
                if let Some(ref mut hunk) = current_hunk {
                    hunk.lines.push(DiffLine {
                        line_type: DiffLineType::Range,
                        content: line.to_string(),
                        line_number: None,
                    });
                }

                i += 1;
                continue;
            }

            // Process diff lines
            if let Some(ref mut hunk) = current_hunk {
                if let Some(prefix) = line.chars().next() {
                    let line_type = match prefix {
                        ' ' => DiffLineType::Context,
                        '+' => DiffLineType::Added,
                        '-' => DiffLineType::Deleted,
                        '\\' => DiffLineType::Context, // Handle "\ No newline at end of file"
                        _ => DiffLineType::Context,
                    };

                    let content = if prefix == ' ' || prefix == '+' || prefix == '-' {
                        line[1..].to_string()
                    } else {
                        line.to_string()
                    };

                    // Calculate line number
                    let line_number = if line_type == DiffLineType::Added {
                        Some(hunk.new_start + hunk.added_lines().len())
                    } else if line_type == DiffLineType::Deleted {
                        Some(hunk.old_start + hunk.deleted_lines().len())
                    } else {
                        None
                    };

                    hunk.lines.push(DiffLine {
                        line_type,
                        content,
                        line_number,
                    });
                }
            }

            i += 1;
        }

        // Don't forget the last hunk
        if let Some(hunk) = current_hunk {
            hunks.push(hunk);
        }

        Ok(hunks)
    }

    /// Filter hunks by file pattern (glob-style)
    pub fn filter_by_pattern(hunks: &[DiffHunk], pattern: &str) -> Vec<DiffHunk> {
        // Convert glob pattern to regex
        let regex_pattern = pattern
            .replace("**", ".*")
            .replace("*", "[^/]*")
            .replace("?", ".");

        if let Ok(re) = Regex::new(&regex_pattern) {
            hunks.iter()
                .filter(|hunk| re.is_match(&hunk.file_path))
                .cloned()
                .collect()
        } else {
            // If regex fails, return empty
            Vec::new()
        }
    }

    /// Filter hunks by file extension
    pub fn filter_by_extension(hunks: &[DiffHunk], extension: &str) -> Vec<DiffHunk> {
        let ext = extension.trim_start_matches('.');
        hunks.iter()
            .filter(|hunk| {
                Path::new(&hunk.file_path)
                    .extension()
                    .map(|e| e.to_string_lossy() == ext)
                    .unwrap_or(false)
            })
            .cloned()
            .collect()
    }

    /// Get statistics about hunks
    pub fn stats(hunks: &[DiffHunk]) -> DiffStats {
        let files = std::collections::HashSet::<String>::from_iter(
            hunks.iter().map(|h| h.file_path.clone())
        );

        let insertions = hunks.iter()
            .map(|h| h.added_lines().len())
            .sum();

        let deletions = hunks.iter()
            .map(|h| h.deleted_lines().len())
            .sum();

        DiffStats {
            files_changed: files.len(),
            insertions,
            deletions,
        }
    }
}

/// Statistics about a diff
#[derive(Debug, Clone)]
pub struct DiffStats {
    pub files_changed: usize,
    pub insertions: usize,
    pub deletions: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_diff() {
        let diff = r#"---
+++
@@ -1,3 +1,4 @@
 line 1
-old line 2
+new line 2
 line 3
+new line 4"#;

        let hunks = DiffParser::parse(diff).unwrap();
        assert_eq!(hunks.len(), 1);

        let hunk = &hunks[0];
        assert_eq!(hunk.old_start, 1);
        assert_eq!(hunk.new_start, 1);
        assert_eq!(hunk.added_lines().len(), 2);
        assert_eq!(hunk.deleted_lines().len(), 1);
    }

    #[test]
    fn test_parse_multi_file_diff() {
        let diff = r#"---
+++
@@ -1,1 +1,1 @@
-old
+new
---
+++
@@ -1,1 +1,1 @@
-old2
+new2"#;

        let hunks = DiffParser::parse(diff).unwrap();
        assert_eq!(hunks.len(), 2);
    }

    #[test]
    fn test_filter_by_extension() {
        let diff = r#"---
+++a/file1.rs
@@ -1,1 +1,1 @@
-old
+new
---
+++b/file2.js
@@ -1,1 +1,1 @@
-old2
+new2"#;

        let hunks = DiffParser::parse(diff).unwrap();
        let rust_hunks = DiffParser::filter_by_extension(&hunks, "rs");
        assert_eq!(rust_hunks.len(), 1);
        assert!(rust_hunks[0].file_path.contains("file1.rs"));
    }

    #[test]
    fn test_diff_stats() {
        let diff = r#"---
+++
@@ -1,3 +1,4 @@
 old
+new1
+new2
-old2"#;

        let hunks = DiffParser::parse(diff).unwrap();
        let stats = DiffParser::stats(&hunks);
        assert_eq!(stats.insertions, 2);
        assert_eq!(stats.deletions, 1);
        assert_eq!(stats.files_changed, 1);
    }

    #[test]
    fn test_code_content() {
        let diff = r#"---
+++
@@ -1,3 +1,3 @@
 fn test() {
-    old();
+    new();
 }"#;

        let hunks = DiffParser::parse(diff).unwrap();
        let hunk = &hunks[0];
        let content = hunk.code_content();
        assert!(content.contains("new()"));
        assert!(!content.contains("old()"));
    }
}
