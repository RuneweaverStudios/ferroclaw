//! GlobTool - Pattern-based file/directory search
//!
//! Provides fast pattern matching for files and directories using glob patterns.
//! Supports recursive patterns (**), wildcards (*), and hidden file handling.

use crate::error::{FerroError, Result};
use crate::tool::{ToolFuture, ToolHandler};
use crate::types::{ToolDefinition, ToolMeta, ToolResult, ToolSource};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;

/// Maximum number of results to return
const MAX_RESULTS: usize = 100;

/// Cache for glob search results to speed up repeated searches
#[derive(Debug, Clone)]
struct GlobCache {
    cache: Arc<RwLock<HashMap<String, Vec<PathBuf>>>>,
}

impl GlobCache {
    fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get cached results for a pattern/path combination
    async fn get(&self, key: &str) -> Option<Vec<PathBuf>> {
        let cache = self.cache.read().await;
        cache.get(key).cloned()
    }

    /// Store results in cache
    async fn put(&self, key: String, results: Vec<PathBuf>) {
        let mut cache = self.cache.write().await;
        // Simple cache eviction: keep only last 50 entries
        if cache.len() >= 50 {
            cache.clear();
        }
        cache.insert(key, results);
    }

    /// Generate cache key from pattern and path
    fn cache_key(pattern: &str, path: &Path) -> String {
        format!("{}::{}", pattern, path.display())
    }
}

/// Work item for iterative glob processing
struct GlobWorkItem {
    path: PathBuf,
    segment_idx: usize,
    #[allow(dead_code)]
    in_double_star: bool,
}

/// GlobTool handler for pattern-based file search
pub struct GlobTool {
    cache: GlobCache,
}

impl GlobTool {
    pub fn new() -> Self {
        Self {
            cache: GlobCache::new(),
        }
    }

    /// Execute glob pattern matching
    async fn execute_glob(&self, pattern: &str, base_path: &Path) -> Result<Vec<PathBuf>> {
        // Check cache first
        let cache_key = GlobCache::cache_key(pattern, base_path);
        if let Some(cached) = self.cache.get(&cache_key).await {
            return Ok(cached);
        }

        let mut results = Vec::new();
        let base_path = base_path.to_path_buf();

        // Validate base path exists
        if !fs::try_exists(&base_path).await.unwrap_or(false) {
            return Err(FerroError::Tool(format!(
                "Base path does not exist: {}",
                base_path.display()
            )));
        }

        // Execute the glob search using iterative approach
        self.glob_iterative(&base_path, pattern, &mut results)
            .await?;

        // Sort results
        results.sort();

        // Store in cache
        self.cache.put(cache_key, results.clone()).await;

        Ok(results)
    }

    /// Iterative glob pattern matching (avoids recursive async)
    async fn glob_iterative(
        &self,
        base_path: &Path,
        pattern: &str,
        results: &mut Vec<PathBuf>,
    ) -> Result<()> {
        let segments: Vec<&str> = pattern.split('/').collect();
        let mut work_queue: Vec<GlobWorkItem> = vec![GlobWorkItem {
            path: base_path.to_path_buf(),
            segment_idx: 0,
            in_double_star: false,
        }];

        while let Some(item) = work_queue.pop() {
            // Check if we've exceeded max results
            if results.len() >= MAX_RESULTS {
                break;
            }

            // Process current work item
            if let Err(e) = self
                .process_work_item(&item, &segments, &mut work_queue, results)
                .await
            {
                tracing::debug!("Error processing work item: {}", e);
                continue;
            }
        }

        Ok(())
    }

    /// Process a single work item
    async fn process_work_item(
        &self,
        item: &GlobWorkItem,
        segments: &[&str],
        work_queue: &mut Vec<GlobWorkItem>,
        results: &mut Vec<PathBuf>,
    ) -> Result<()> {
        let GlobWorkItem {
            path,
            segment_idx,
            in_double_star: _,
        } = item;

        // Base case: all segments processed
        if *segment_idx >= segments.len() {
            if fs::try_exists(path).await.unwrap_or(false) {
                results.push(path.clone());
            }
            return Ok(());
        }

        let current_segment = segments[*segment_idx];

        // Handle ** (recursive wildcard)
        if current_segment == "**" {
            // Match current directory - advance to next segment
            work_queue.push(GlobWorkItem {
                path: path.clone(),
                segment_idx: segment_idx + 1,
                in_double_star: true,
            });

            // Recurse into subdirectories - stay on same segment
            let mut entries = match fs::read_dir(path).await {
                Ok(e) => e,
                Err(_) => return Ok(()), // Skip directories we can't read
            };

            while let Ok(Some(entry)) = entries.next_entry().await {
                let entry_path = entry.path();
                let file_type = match entry.file_type().await {
                    Ok(ft) => ft,
                    Err(_) => continue,
                };
                if file_type.is_dir() {
                    work_queue.push(GlobWorkItem {
                        path: entry_path,
                        segment_idx: *segment_idx,
                        in_double_star: true,
                    });
                }
            }
            return Ok(());
        }

        // Handle * (single-level wildcard) and literal matches
        let mut entries = match fs::read_dir(path).await {
            Ok(e) => e,
            Err(_) => return Ok(()),
        };

        while let Ok(Some(entry)) = entries.next_entry().await {
            let entry_path = entry.path();
            let file_name = entry.file_name().to_string_lossy().to_string();

            // Skip hidden files/directories unless pattern starts with '.'
            if file_name.starts_with('.') && !current_segment.starts_with('.') {
                continue;
            }

            // Check if segment matches
            if self.pattern_matches(current_segment, &file_name) {
                // If this is the last segment, add to results
                if *segment_idx == segments.len() - 1 {
                    results.push(entry_path);
                } else {
                    // Continue to next segment
                    work_queue.push(GlobWorkItem {
                        path: entry_path,
                        segment_idx: segment_idx + 1,
                        in_double_star: false,
                    });
                }
            }
        }

        Ok(())
    }

    /// Check if a pattern segment matches a filename
    fn pattern_matches(&self, pattern: &str, filename: &str) -> bool {
        // Simple wildcard matching
        if pattern == "*" {
            return !filename.starts_with('.');
        }
        if pattern.contains('*') {
            return self.wildcard_match(pattern, filename);
        }
        pattern == filename
    }

    /// Convert glob pattern to simple regex (not currently used but kept for reference)
    #[allow(dead_code)]
    fn glob_to_regex(&self, pattern: &str) -> String {
        let mut regex = String::new();
        for c in pattern.chars() {
            match c {
                '*' => regex.push_str(".*"),
                '?' => regex.push('.'),
                '.' => regex.push_str("\\."),
                '+' => regex.push_str("\\+"),
                _ => regex.push(c),
            }
        }
        format!("^{}$", regex)
    }

    /// Simple wildcard matching (supports *)
    fn wildcard_match(&self, pattern: &str, text: &str) -> bool {
        let pattern_chars: Vec<char> = pattern.chars().collect();
        let text_chars: Vec<char> = text.chars().collect();
        let mut p_idx = 0;
        let mut t_idx = 0;
        let mut star_idx = -1i32;
        let mut match_idx = 0i32;

        while t_idx < text_chars.len() {
            if p_idx < pattern_chars.len()
                && (pattern_chars[p_idx] == text_chars[t_idx] || pattern_chars[p_idx] == '?')
            {
                p_idx += 1;
                t_idx += 1;
            } else if p_idx < pattern_chars.len() && pattern_chars[p_idx] == '*' {
                star_idx = p_idx as i32;
                match_idx = t_idx as i32;
                p_idx += 1;
            } else if star_idx != -1 {
                p_idx = (star_idx + 1) as usize;
                match_idx += 1;
                t_idx = match_idx as usize;
            } else {
                return false;
            }
        }

        while p_idx < pattern_chars.len() && pattern_chars[p_idx] == '*' {
            p_idx += 1;
        }

        p_idx == pattern_chars.len()
    }
}

impl Default for GlobTool {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolHandler for GlobTool {
    fn call<'a>(&'a self, call_id: &'a str, arguments: &'a Value) -> ToolFuture<'a> {
        Box::pin(async move {
            let pattern = arguments
                .get("pattern")
                .and_then(|p| p.as_str())
                .ok_or_else(|| FerroError::Tool("Missing 'pattern' argument".into()))?;

            // Get base path (default to current directory)
            let base_path_str = arguments
                .get("path")
                .and_then(|p| p.as_str())
                .unwrap_or(".");
            let base_path = PathBuf::from(base_path_str);

            // Execute glob search
            let mut results = self.execute_glob(pattern, &base_path).await?;

            // Truncate if needed
            let truncated = results.len() > MAX_RESULTS;
            results.truncate(MAX_RESULTS);

            // Convert to relative paths if possible
            let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
            let relative_results: Vec<String> = results
                .iter()
                .map(|p| {
                    p.strip_prefix(&current_dir)
                        .unwrap_or(p)
                        .to_string_lossy()
                        .to_string()
                })
                .collect();

            // Format output
            let content = if relative_results.is_empty() {
                "No files found".to_string()
            } else {
                let mut output = relative_results.join("\n");
                if truncated {
                    output
                        .push_str("\n\n(Results truncated. Use a more specific pattern or path.)");
                }
                output
            };

            Ok(ToolResult {
                call_id: call_id.to_string(),
                content,
                is_error: false,
            })
        })
    }
}

/// Create the GlobTool metadata for registration
pub fn glob_tool_meta() -> ToolMeta {
    ToolMeta {
        definition: ToolDefinition {
            name: "glob".into(),
            description:
                "Find files and directories using glob patterns like **/*.rs or src/**/*.ts".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "pattern": {
                        "type": "string",
                        "description": "The glob pattern to match files against (e.g., **/*.rs, src/**/*.ts)"
                    },
                    "path": {
                        "type": "string",
                        "description": "The directory to search in (default: current directory)"
                    }
                },
                "required": ["pattern"]
            }),
            server_name: None,
        },
        required_capabilities: vec![],
        source: ToolSource::Builtin,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wildcard_match() {
        let tool = GlobTool::new();

        assert!(tool.wildcard_match("*.rs", "test.rs"));
        assert!(tool.wildcard_match("*.rs", "main.rs"));
        assert!(!tool.wildcard_match("*.rs", "test.txt"));
        assert!(tool.wildcard_match("*", "test.rs"));
        assert!(tool.wildcard_match("test*", "test.rs"));
        assert!(tool.wildcard_match("*test", "mytest"));
    }

    #[test]
    fn test_cache_key() {
        let key1 = GlobCache::cache_key("*.rs", Path::new("/src"));
        let key2 = GlobCache::cache_key("*.rs", Path::new("/src"));
        let key3 = GlobCache::cache_key("*.rs", Path::new("/test"));

        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }

    #[tokio::test]
    async fn test_glob_cache() {
        let cache = GlobCache::new();

        // Test put and get
        let results = vec![PathBuf::from("/test/file.rs")];
        cache.put("test_key".to_string(), results.clone()).await;

        let retrieved = cache.get("test_key").await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), results);

        // Test non-existent key
        let missing = cache.get("missing_key").await;
        assert!(missing.is_none());
    }
}
